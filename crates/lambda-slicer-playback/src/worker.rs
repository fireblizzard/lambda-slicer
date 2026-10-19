//! Playback off the window's thread
//!
//! The worker answers requests in order but skips any that a newer request
//! already replaced, so dragging the playhead never queues up work. A load
//! is never skipped. Its demo is opened before the newest request of its
//! generation is answered. Replies go back with the stamp of the request
//! they answer, for [`Controller::accept`](crate::controller::Controller::accept)
//! to judge.
//!
//! A seek, step or load sent while another is being answered stops that
//! one between groups, unanswered, as does dropping the worker. Requests
//! of the same stamp, as playing sends them, do not, so playback slower
//! than the clock still shows frames.
//!
//! While no request waits, a second playback of the same demo plays ahead
//! in short slices to add checkpoints, so later seeks start nearby. A new
//! request waits for one slice at most

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use lambda_slicer_core::container::Limits;

use crate::GameFiles;
use crate::controller::{Reply, Request, Stamp, Target};
use crate::playback::Playback;

pub struct Worker {
    requests: Option<Sender<Counted>>,
    replies: Receiver<Reply>,
    thread: Option<JoinHandle<()>>,
    loaded: Arc<Mutex<Option<Loaded>>>,
    /// Counts each request with a stamp other than the one before, and
    /// the worker's drop
    changes: Arc<AtomicU64>,
    last: Mutex<Option<Stamp>>,
}

/// The demo the worker has open, for another playback of it to share its
/// checkpoints
#[derive(Clone)]
pub struct Loaded {
    pub generation: u64,
    pub demo: Arc<Path>,
    pub checkpoints: crate::checkpoint::Shared,
    pub limits: Limits,
}

impl Worker {
    /// Start the worker. `notify` is called after every reply, to wake the
    /// window
    pub fn spawn(limits: Limits, notify: impl Fn() + Send + 'static) -> Self {
        Self::spawn_with(limits, None, notify)
    }

    /// Start the worker, its playbacks taking each sound's length and each
    /// model's sequences from the game's files
    pub fn spawn_with(
        limits: Limits,
        files: Option<GameFiles>,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<Counted>();
        let loaded = Arc::new(Mutex::new(None));
        let thread_loaded = loaded.clone();
        let changes = Arc::new(AtomicU64::new(0));
        let thread_changes = changes.clone();
        let (reply_tx, reply_rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("playback".into())
            .spawn(move || {
                let worker = Thread {
                    requests: request_rx,
                    replies: reply_tx,
                    loaded: thread_loaded,
                    changes: thread_changes,
                };
                run(limits, files, worker, notify)
            })
            .expect("spawn playback worker");

        Self {
            requests: Some(request_tx),
            replies: reply_rx,
            thread: Some(thread),
            loaded,
            changes,
            last: Mutex::new(None),
        }
    }

    pub fn send(&self, request: Request) {
        let Some(tx) = &self.requests else {
            return;
        };
        // Sent with the count it makes, so the worker stops only for a
        // change after the request it answers
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        if *last != Some(request.stamp) {
            *last = Some(request.stamp);
            self.changes.fetch_add(1, Ordering::SeqCst);
        }
        let count = self.changes.load(Ordering::SeqCst);
        let _ = tx.send((count, request));
    }

    /// The demo open now and its checkpoints
    pub fn loaded(&self) -> Option<Loaded> {
        self.loaded
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Replies waiting, oldest first
    pub fn replies(&self) -> Vec<Reply> {
        self.replies.try_iter().collect()
    }

    /// Wait for the next reply, for tests and command-line tools
    pub fn wait(&self) -> Option<Reply> {
        self.replies.recv().ok()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Closing the request channel ends the worker's loop, once a seek
        // under way has stopped
        self.changes.fetch_add(1, Ordering::SeqCst);
        self.requests = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Groups the indexer plays between looks at the request channel, a few
/// tens of milliseconds
const INDEX_SLICE: usize = 1000;

/// A request and the count of changes once it was sent
type Counted = (u64, Request);

/// The worker thread's ends of what it shares with the [`Worker`]
struct Thread {
    requests: Receiver<Counted>,
    replies: Sender<Reply>,
    loaded: Arc<Mutex<Option<Loaded>>>,
    changes: Arc<AtomicU64>,
}

fn run(limits: Limits, files: Option<GameFiles>, worker: Thread, notify: impl Fn()) {
    let Thread {
        requests,
        replies,
        loaded,
        changes,
    } = worker;
    // The count of changes the request being answered was sent with
    let taken = Arc::new(AtomicU64::new(0));
    let replaced = {
        let (changes, taken) = (changes.clone(), taken.clone());
        move || changes.load(Ordering::SeqCst) != taken.load(Ordering::SeqCst)
    };
    let open = |path: &std::path::Path, store: Option<crate::checkpoint::Shared>| {
        let mut p = match store {
            Some(store) => Playback::open_with(path, limits, store),
            None => Playback::open(path, limits),
        }?;
        if let Some(files) = &files {
            p.set_game_files(files);
        }
        p.set_interrupt(replaced.clone());
        Ok::<_, lambda_slicer_core::message::Message>(p)
    };
    let mut playback: Option<(u64, Playback)> = None;
    let mut indexer: Option<Playback> = None;

    loop {
        let first = match &mut indexer {
            Some(i) => match requests.try_recv() {
                Ok(request) => request,
                Err(TryRecvError::Empty) => {
                    if i.index(INDEX_SLICE) {
                        indexer = None;
                    }
                    continue;
                }
                Err(TryRecvError::Disconnected) => return,
            },
            None => match requests.recv() {
                Ok(request) => request,
                Err(_) => return,
            },
        };
        let mut batch = vec![first];
        batch.extend(requests.try_iter());
        let Some((count, newest)) = batch.pop() else {
            continue;
        };
        taken.store(count, Ordering::SeqCst);
        let batch: Vec<Request> = batch.into_iter().map(|(_, r)| r).collect();

        let load = std::iter::once(&newest)
            .chain(batch.iter().rev())
            .filter(|r| r.stamp.generation == newest.stamp.generation)
            .find_map(|r| r.load.clone());
        if let Some(path) = load {
            playback = None;
            indexer = None;
            match open(&path, None) {
                Ok(p) => {
                    indexer = open(&path, Some(p.checkpoints().clone())).ok();
                    *loaded.lock().unwrap_or_else(|e| e.into_inner()) = Some(Loaded {
                        generation: newest.stamp.generation,
                        demo: Arc::from(path.as_path()),
                        checkpoints: p.checkpoints().clone(),
                        limits,
                    });
                    playback = Some((newest.stamp.generation, p));
                }
                Err(message) => {
                    let _ = replies.send(Reply::Failed {
                        stamp: newest.stamp,
                        message,
                    });
                    notify();
                    continue;
                }
            }
        }

        let Some((generation, p)) = &mut playback else {
            continue;
        };
        if *generation != newest.stamp.generation {
            continue;
        }

        let reached = match newest.target {
            Target::Time(time) => p.advance_to(time).map(|_| time),
            Target::Group(group) => p
                .seek_group(group)
                .map(|_| p.position().map_or(Default::default(), |at| at.time)),
        };
        // A newer seek or load waits, and is answered instead
        if replaced() {
            continue;
        }
        let reply = match reached {
            Ok(at) => Reply::Frame(Box::new(p.present(
                newest.stamp,
                at,
                newest.camera,
                newest.audible,
            ))),
            Err(message) => Reply::Failed {
                stamp: newest.stamp,
                message,
            },
        };
        if replies.send(reply).is_err() {
            return;
        }
        notify();
    }
}
