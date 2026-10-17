//! Clip exports in the background (implementation plan §8 and §9)
//!
//! An export job writes its clips one after another on its own thread with
//! the core's restart export, which only publishes a file once it validated
//! and never replaces one. Each clip's file name is chosen just before its
//! export, so clips of one job never take the same name. Cancelling skips
//! the clips not yet started, as a clip under way runs to its end. A job
//! dropped with its demo view is cancelled the same way and not waited for

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};

use eframe::egui;
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::slice::ExportError;
use lambda_slicer_core::slice::naming::{clip_file_name, free_path, playdemo};
use lambda_slicer_core::slice::restart::run_restart;
use lambda_slicer_core::timeline::time::DemoTime;

/// A clip by its requested range, which is what its export depends on
pub type ClipKey = (DemoTime, DemoTime);

/// A finished export
#[derive(Clone, Debug, PartialEq)]
pub struct Exported {
    pub path: PathBuf,
    /// The console command that plays it, when it is inside a mod folder
    pub playdemo: Option<String>,
    /// Kept snapshots whose decoded state matched the source
    pub compared: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportState {
    Waiting,
    Running,
    Done(Exported),
    Failed(Message),
    Cancelled,
}

enum Event {
    Started(ClipKey),
    Finished(ClipKey, Result<Exported, Message>),
    Cancelled(ClipKey),
}

pub struct ExportJob {
    cancel: Arc<AtomicBool>,
    rx: Receiver<Event>,
    /// Clips not yet reported finished
    open: usize,
}

impl ExportJob {
    /// Export `clips` of `source` into `folder`
    pub fn start(
        source: PathBuf,
        folder: PathBuf,
        clips: Vec<ClipKey>,
        ctx: egui::Context,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let open = clips.len();
        let thread_cancel = cancel.clone();
        std::thread::Builder::new()
            .name("clip export".into())
            .spawn(move || {
                for (start, end) in clips {
                    let key = (start, end);
                    if thread_cancel.load(Ordering::Relaxed) {
                        let _ = tx.send(Event::Cancelled(key));
                        ctx.request_repaint();
                        continue;
                    }
                    let _ = tx.send(Event::Started(key));
                    ctx.request_repaint();
                    let dest = free_path(&folder, &clip_file_name(&source, start, end));
                    // A panic fails this clip, not the job
                    let run = || run_restart(&source, &dest, start, Some(end), Limits::default());
                    let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
                        Ok(Ok(r)) => Ok(Exported {
                            playdemo: playdemo(&r.published.path),
                            path: r.published.path,
                            compared: r.validation.compared_snapshots,
                        }),
                        Ok(Err(e)) => Err(e.message()),
                        Err(_) => Err(ExportError::Unsupported(
                            "the export stopped unexpectedly".into(),
                        )
                        .message()),
                    };
                    let _ = tx.send(Event::Finished(key, result));
                    ctx.request_repaint();
                }
            })
            .expect("spawn clip export");
        Self { cancel, rx, open }
    }

    /// Skip the clips not yet started
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn cancelling(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// Apply what happened since the last call. Returns true once every
    /// clip is accounted for
    pub fn poll(&mut self, states: &mut std::collections::HashMap<ClipKey, ExportState>) -> bool {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Started(key) => {
                    states.insert(key, ExportState::Running);
                }
                Event::Finished(key, result) => {
                    self.open -= 1;
                    let state = match result {
                        Ok(e) => ExportState::Done(e),
                        Err(m) => ExportState::Failed(m),
                    };
                    states.insert(key, state);
                }
                Event::Cancelled(key) => {
                    self.open -= 1;
                    states.insert(key, ExportState::Cancelled);
                }
            }
        }
        self.open == 0
    }
}

impl Drop for ExportJob {
    fn drop(&mut self) {
        self.cancel();
    }
}
