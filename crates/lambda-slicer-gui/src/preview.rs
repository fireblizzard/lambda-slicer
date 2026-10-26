//! Moments of the demo for the seek bar's preview, played off the window's
//! thread
//!
//! A thread keeps a playback of its own that shares the worker's
//! checkpoints, so a moment is reached from the checkpoint before it, in a
//! few milliseconds. Only the newest moment asked for is played to, and
//! one a newer ask replaces stops between groups

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use lambda_slicer_playback::GameFiles;
use lambda_slicer_playback::PresentedFrame;
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;
use lambda_slicer_playback::worker::Loaded;

/// A moment asked for, numbered in turn
struct Ask {
    number: u64,
    demo: Loaded,
    time: SourceTime,
}

/// The frame at a moment asked for, with the ask's number
pub struct Answer {
    pub number: u64,
    pub frame: Box<PresentedFrame>,
}

pub struct Previews {
    asks: Option<Sender<Ask>>,
    answers: Receiver<Answer>,
    /// The number of the newest ask, which the thread stops for
    newest: Arc<AtomicU64>,
    closing: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Previews {
    /// Start the thread, its playbacks reading the game's files. `notify`
    /// is called after every answer, to wake the window
    pub fn spawn(files: GameFiles, notify: impl Fn() + Send + 'static) -> Self {
        let (ask_tx, ask_rx) = mpsc::channel();
        let (answer_tx, answer_rx) = mpsc::channel();
        let newest = Arc::new(AtomicU64::new(0));
        let closing = Arc::new(AtomicBool::new(false));
        let thread = {
            let (newest, closing) = (newest.clone(), closing.clone());
            std::thread::Builder::new()
                .name("preview".into())
                .spawn(move || run(files, ask_rx, answer_tx, newest, closing, notify))
                .expect("spawn preview thread")
        };

        Self {
            asks: Some(ask_tx),
            answers: answer_rx,
            newest,
            closing,
            thread: Some(thread),
        }
    }

    /// Ask for the frame of `demo` at `time`, numbered `number`, which
    /// should be higher than any asked before
    pub fn ask(&self, number: u64, demo: Loaded, time: SourceTime) {
        let Some(asks) = &self.asks else {
            return;
        };
        self.newest.store(number, Ordering::SeqCst);
        let _ = asks.send(Ask { number, demo, time });
    }

    /// Answers waiting, oldest first
    pub fn answers(&self) -> Vec<Answer> {
        self.answers.try_iter().collect()
    }
}

impl Drop for Previews {
    fn drop(&mut self) {
        // A seek under way stops between groups, and the closed channel
        // ends the thread's loop
        self.closing.store(true, Ordering::SeqCst);
        self.asks = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The thread's playback, for one load generation. None when the demo
/// could not be opened
type Own = Option<(u64, Option<Playback>)>;

fn run(
    files: GameFiles,
    asks: Receiver<Ask>,
    answers: Sender<Answer>,
    newest: Arc<AtomicU64>,
    closing: Arc<AtomicBool>,
    notify: impl Fn(),
) {
    let mut own: Own = None;
    while let Ok(first) = asks.recv() {
        // Only the newest ask waiting is answered
        let ask = asks.try_iter().last().unwrap_or(first);

        let wanted = ask.number;
        let replaced = {
            let (newest, closing) = (newest.clone(), closing.clone());
            move || closing.load(Ordering::SeqCst) || newest.load(Ordering::SeqCst) != wanted
        };
        let Some(playback) = playback_for(&mut own, &ask.demo, &files) else {
            continue;
        };
        playback.set_interrupt(replaced.clone());
        if playback.advance_to(ask.time).is_err() || replaced() {
            continue;
        }

        let frame = playback.present(Stamp::default(), ask.time, CameraMode::Recorder, false);
        let answer = Answer {
            number: ask.number,
            frame: Box::new(frame),
        };
        if answers.send(answer).is_err() {
            return;
        }
        notify();
    }
}

/// The thread's playback of `demo`, opened again on a new generation
fn playback_for<'a>(
    own: &'a mut Own,
    demo: &Loaded,
    files: &GameFiles,
) -> Option<&'a mut Playback> {
    let stale = own
        .as_ref()
        .is_none_or(|(generation, _)| *generation != demo.generation);
    if stale {
        let opened = Playback::open_with(&demo.demo, demo.limits, demo.checkpoints.clone())
            .inspect_err(|e| eprintln!("preview: {e:?}"))
            .ok()
            .map(|mut playback| {
                playback.set_game_files(files);
                playback
            });
        *own = Some((demo.generation, opened));
    }
    own.as_mut()?.1.as_mut()
}
