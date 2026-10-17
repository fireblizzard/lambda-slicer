//! Playback off the window's thread
//!
//! The worker answers requests in order but skips any that a newer request
//! already replaced, so dragging the playhead never queues up work. A load
//! is never skipped. Its demo is opened before the newest request of its
//! generation is answered. Replies go back with the stamp of the request
//! they answer, for [`Controller::accept`](crate::controller::Controller::accept)
//! to judge

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use lambda_slicer_core::container::Limits;

use crate::controller::{Reply, Request, Target};
use crate::playback::Playback;

pub struct Worker {
    requests: Option<Sender<Request>>,
    replies: Receiver<Reply>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Start the worker. `notify` is called after every reply, to wake the
    /// window
    pub fn spawn(limits: Limits, notify: impl Fn() + Send + 'static) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<Request>();
        let (reply_tx, reply_rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("playback".into())
            .spawn(move || run(limits, request_rx, reply_tx, notify))
            .expect("spawn playback worker");

        Self {
            requests: Some(request_tx),
            replies: reply_rx,
            thread: Some(thread),
        }
    }

    pub fn send(&self, request: Request) {
        if let Some(tx) = &self.requests {
            let _ = tx.send(request);
        }
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
        // Closing the request channel ends the worker's loop
        self.requests = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(limits: Limits, requests: Receiver<Request>, replies: Sender<Reply>, notify: impl Fn()) {
    let mut playback: Option<(u64, Playback)> = None;

    while let Ok(first) = requests.recv() {
        let mut batch = vec![first];
        batch.extend(requests.try_iter());
        let Some(newest) = batch.pop() else {
            continue;
        };

        let load = std::iter::once(&newest)
            .chain(batch.iter().rev())
            .filter(|r| r.stamp.generation == newest.stamp.generation)
            .find_map(|r| r.load.clone());
        if let Some(path) = load {
            playback = None;
            match Playback::open(&path, limits) {
                Ok(p) => playback = Some((newest.stamp.generation, p)),
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
