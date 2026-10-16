//! Library scans in the background (implementation plan §7.1)
//!
//! A scan runs the core scanner on its own thread and sends its events over
//! a channel. The window drains a bounded number of events per frame, so a
//! burst of finished files never stalls drawing, and is woken by each event.
//! Cancelling keeps every finished file

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::JoinHandle;

use eframe::egui;
use lambda_slicer_core::library::scan::{
    Outcome, ScanControl, ScanEvent, ScanOptions, ScanProgress, scan,
};
use lambda_slicer_core::message::{Coded, Message};

/// Events handled per frame at most
const EVENTS_PER_FRAME: usize = 500;

/// Problem files kept for display, the most recent ones
const PROBLEMS_KEPT: usize = 200;

pub enum JobEvent {
    Progress(ScanProgress),
    /// A file that was stored, so the library list may change
    Stored,
    /// A file or directory that could not be handled
    Problem {
        path: PathBuf,
        message: Problem,
    },
    Finished(Result<ScanProgress, Message>),
}

/// Why a file or directory could not be handled
#[derive(Clone, Debug)]
pub enum Problem {
    Message(Message),
    /// Stored, but the decode stopped early or failed
    Status(String),
    Changed,
}

pub struct ScanJob {
    control: Arc<ScanControl>,
    rx: Receiver<JobEvent>,
    thread: Option<JoinHandle<()>>,
    pub progress: ScanProgress,
    pub finished: Option<Result<ScanProgress, Message>>,
    /// Most recent problems, newest last
    pub problems: Vec<(PathBuf, Problem)>,
}

impl ScanJob {
    pub fn start(options: ScanOptions, ctx: egui::Context) -> Self {
        let control = ScanControl::new();
        let (tx, rx) = mpsc::channel();
        let thread_control = control.clone();
        let thread = std::thread::Builder::new()
            .name("library scan".into())
            .spawn(move || {
                let send = |e: JobEvent| {
                    let _ = tx.send(e);
                    ctx.request_repaint();
                };
                let result = scan(&options, &thread_control, |event| match event {
                    ScanEvent::Progress(p) => send(JobEvent::Progress(p.clone())),
                    ScanEvent::DiscoveryError { path, error } => send(JobEvent::Problem {
                        path: path.to_path_buf(),
                        message: Problem::Message(
                            Message::new("io-other", error.to_string()).arg("detail", error),
                        ),
                    }),
                    ScanEvent::File { path, outcome } => match outcome {
                        Outcome::Analyzed { status } => {
                            send(JobEvent::Stored);
                            if status != "complete" {
                                send(JobEvent::Problem {
                                    path: path.to_path_buf(),
                                    message: Problem::Status(status.clone()),
                                });
                            }
                        }
                        Outcome::Unreadable { error } | Outcome::CacheFailed { error } => {
                            send(JobEvent::Problem {
                                path: path.to_path_buf(),
                                message: Problem::Message(error.clone()),
                            })
                        }
                        Outcome::SourceChanged => send(JobEvent::Problem {
                            path: path.to_path_buf(),
                            message: Problem::Changed,
                        }),
                        Outcome::CacheHit | Outcome::Duplicate { .. } | Outcome::Cancelled => {}
                    },
                });
                send(JobEvent::Finished(result.map_err(|e| e.message())));
            })
            .expect("spawn scan thread");

        Self {
            control,
            rx,
            thread: Some(thread),
            progress: ScanProgress::default(),
            finished: None,
            problems: Vec::new(),
        }
    }

    pub fn cancel(&self) {
        self.control.cancel();
    }

    pub fn running(&self) -> bool {
        self.finished.is_none()
    }

    pub fn cancelling(&self) -> bool {
        self.running() && self.control.is_cancelled()
    }

    /// Analyse `path` before anything else queued
    pub fn prioritize(&self, path: PathBuf) {
        self.control.prioritize(path);
    }

    /// Apply up to a frame's worth of events. True when stored files may
    /// have changed the library
    pub fn drain(&mut self) -> bool {
        let mut changed = false;
        for _ in 0..EVENTS_PER_FRAME {
            match self.rx.try_recv() {
                Ok(JobEvent::Progress(p)) => self.progress = p,
                Ok(JobEvent::Stored) => changed = true,
                Ok(JobEvent::Problem { path, message }) => {
                    self.problems.push((path, message));
                    if self.problems.len() > PROBLEMS_KEPT {
                        self.problems.remove(0);
                    }
                }
                Ok(JobEvent::Finished(result)) => {
                    if let Ok(p) = &result {
                        self.progress = p.clone();
                    }
                    self.finished = Some(result);
                    changed = true;
                    if let Some(t) = self.thread.take() {
                        let _ = t.join();
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        changed
    }
}

impl Drop for ScanJob {
    /// Closing the window stops the scan and waits for it, so the cache is
    /// left between transactions
    fn drop(&mut self) {
        self.control.cancel();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
