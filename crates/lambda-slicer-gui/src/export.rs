//! Clip exports in the background
//!
//! An export job writes its clips one after another on its own thread with
//! the core's restart export, which only publishes a file once it validated
//! and never replaces one. Each clip's file is named after its times, or
//! after its own name when the settings ask, and the number that keeps it
//! apart from a file already there is chosen just before its export, so
//! clips of one job never take the same name. Cancelling stops the clip
//! under way within a moment, leaving no file, and skips the clips not yet
//! started. A job dropped with its demo view is cancelled the same way and
//! not waited for.
//!
//! A clip refused for where it starts, such as an effect under way there,
//! then has starts a little earlier and later tried, and the nearest that
//! works is offered with the refusal.
//!
//! Each clip's range and file name are taken when the job starts, so editing
//! a clip later never makes its result look like an export of the new range

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};

use eframe::egui;
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::slice::ExportError;
use lambda_slicer_core::slice::marks::{Mark, MarkTexture};
use lambda_slicer_core::slice::naming::{
    clip_file_name, free_path, named_clip_file_name, playdemo,
};
use lambda_slicer_core::slice::restart::{nearest_start, run_restart_marked};
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_playback::GameFiles;
use lambda_slicer_playback::decals::DecalName;
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;
use lambda_slicer_playback::worker::Loaded;

use crate::clips::Clip;

/// The demo as the viewer loaded it and the game's files, which a clip's
/// export plays to the clip's start to find every decal there
#[derive(Clone)]
pub struct DecalSource {
    pub loaded: Loaded,
    pub files: GameFiles,
}

/// Every decal `playback` has on the map, as a clip places them again. One
/// on a brush entity is where the brush stands now, and one on an entity
/// gone from the map is left out, as are the map's own
fn marks(playback: &Playback) -> Vec<Mark> {
    let entities = &playback.scene().entities;
    let mut out = Vec::new();
    for decal in playback.decals().list().iter() {
        if decal.map {
            continue;
        }
        let origin = if decal.entity == 0 {
            decal.origin
        } else {
            let Some(e) = entities.iter().find(|e| e.id.number == decal.entity) else {
                continue;
            };
            [0, 1, 2].map(|k| decal.origin[k] + e.origin[k])
        };
        let texture = match &decal.name {
            DecalName::Number(number) => MarkTexture::Number(*number),
            DecalName::Named(name) => MarkTexture::Named {
                name: name.to_string(),
                place: decal.texture,
            },
        };
        out.push(Mark {
            texture,
            entity: decal.entity,
            origin,
        });
    }
    out
}

/// A clip by its requested range and its part, which is what its export
/// depends on
pub type ClipKey = (DemoTime, DemoTime, Option<u32>);

/// A clip to export, by the demo view's identity for it, and the range and
/// file name it had when the export was asked for
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportItem {
    pub id: u64,
    pub range: ClipKey,
    pub file_name: String,
}

/// How exported clips are named
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileNaming {
    /// After the clip's own name rather than its times
    pub clip_name: bool,
    /// The demo's name first, when the clip's name is used
    pub demo_first: bool,
}

impl Default for FileNaming {
    fn default() -> Self {
        Self {
            clip_name: true,
            demo_first: true,
        }
    }
}

impl FileNaming {
    /// The file name `clip` of the demo at `source` is exported under,
    /// before any number that keeps it apart from a file already there
    pub fn file_name(&self, source: &Path, clip: &Clip) -> String {
        if self.clip_name {
            let range = (clip.start, clip.end);
            return named_clip_file_name(source, &clip.name, range, self.demo_first);
        }
        clip_file_name(source, clip.start, clip.end)
    }
}

/// A finished export
#[derive(Clone, Debug, PartialEq)]
pub struct Exported {
    pub path: PathBuf,
    /// The console command that plays it, when it is inside a mod folder
    pub playdemo: Option<String>,
    /// Kept snapshots whose decoded state matched the source
    pub compared: u64,
}

/// Why an export failed, and for a clip refused for where it starts the
/// nearest start that works, if one was found
#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    pub message: Message,
    pub nearest: Option<DemoTime>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportState {
    Waiting,
    Running,
    /// Refused for where it starts, and looking for a start that works
    Searching,
    Done(Exported),
    Failed(Failure),
    Cancelled,
    /// Left out of an export of every clip, as it cannot be exported
    Skipped(Message),
}

enum Event {
    Started(u64),
    Searching(u64),
    Finished(u64, Result<Exported, Failure>),
    Cancelled(u64),
}

pub struct ExportJob {
    cancel: Arc<AtomicBool>,
    rx: Receiver<Event>,
    /// Clips not yet reported finished
    open: usize,
}

impl ExportJob {
    /// Export `clips` of `source` into `folder`, with every decal at each
    /// clip's start that `decals` finds, or the server's own without it
    pub fn start(
        source: PathBuf,
        folder: PathBuf,
        clips: Vec<ExportItem>,
        decals: Option<DecalSource>,
        ctx: egui::Context,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let open = clips.len();
        let thread_cancel = cancel.clone();
        std::thread::Builder::new()
            .name("clip export".into())
            .spawn(move || {
                // One playback for every clip, opened when first needed and
                // stopping when the job is cancelled
                let playback: RefCell<Option<Playback>> = RefCell::new(None);
                let open = |source: &DecalSource| {
                    let loaded = &source.loaded;
                    let checkpoints = loaded.checkpoints.clone();
                    let mut playback =
                        Playback::open_with(&loaded.demo, loaded.limits, checkpoints).ok()?;
                    playback.set_game_files(&source.files);
                    let cancel = thread_cancel.clone();
                    playback.set_interrupt(move || cancel.load(Ordering::Relaxed));
                    Some(playback)
                };
                let marks_at = |time: f64| {
                    let source = decals.as_ref()?;
                    let mut slot = playback.borrow_mut();
                    if slot.is_none() {
                        *slot = open(source);
                    }
                    let playback = slot.as_mut()?;
                    playback.advance_to(SourceTime(time)).ok()?;
                    Some(marks(playback))
                };

                for item in clips {
                    let (start, end, part) = item.range;
                    if thread_cancel.load(Ordering::Relaxed) {
                        let _ = tx.send(Event::Cancelled(item.id));
                        ctx.request_repaint();
                        continue;
                    }
                    let _ = tx.send(Event::Started(item.id));
                    ctx.request_repaint();
                    let dest = free_path(&folder, &item.file_name);
                    // A panic fails this clip, not the job
                    let limits = Limits::default();
                    let stop = &*thread_cancel;
                    let run = || {
                        let marks_at: &dyn Fn(f64) -> Option<Vec<Mark>> = &marks_at;
                        let dest = dest.as_path();
                        run_restart_marked(
                            &source,
                            dest,
                            start,
                            Some(end),
                            part,
                            limits,
                            stop,
                            Some(marks_at),
                        )
                    };
                    let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
                        Ok(Err(ExportError::Stopped)) => {
                            let _ = tx.send(Event::Cancelled(item.id));
                            ctx.request_repaint();
                            continue;
                        }
                        Ok(Ok(r)) => Ok(Exported {
                            playdemo: playdemo(&r.published.path),
                            path: r.published.path,
                            compared: r.validation.compared_snapshots,
                        }),
                        Ok(Err(e)) if e.depends_on_start() => {
                            let _ = tx.send(Event::Searching(item.id));
                            ctx.request_repaint();
                            let scratch = std::env::temp_dir();
                            let search =
                                || nearest_start(&source, &scratch, start, end, part, limits, stop);
                            let nearest = match std::panic::catch_unwind(
                                std::panic::AssertUnwindSafe(search),
                            ) {
                                Ok(Err(ExportError::Stopped)) => {
                                    let _ = tx.send(Event::Cancelled(item.id));
                                    ctx.request_repaint();
                                    continue;
                                }
                                Ok(Ok(nearest)) => nearest,
                                _ => None,
                            };
                            Err(Failure {
                                message: e.message(),
                                nearest,
                            })
                        }
                        Ok(Err(e)) => Err(Failure {
                            message: e.message(),
                            nearest: None,
                        }),
                        Err(_) => Err(Failure {
                            message: ExportError::Unsupported(
                                "the export stopped unexpectedly".into(),
                            )
                            .message(),
                            nearest: None,
                        }),
                    };
                    let _ = tx.send(Event::Finished(item.id, result));
                    ctx.request_repaint();
                }
            })
            .expect("spawn clip export");
        Self { cancel, rx, open }
    }

    /// Stop the clip under way and skip those not yet started
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn cancelling(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// Hand `update` each clip's new state since the last call, by its
    /// identity. Returns true once every clip is accounted for
    pub fn poll(&mut self, mut update: impl FnMut(u64, ExportState)) -> bool {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Started(id) => update(id, ExportState::Running),
                Event::Searching(id) => update(id, ExportState::Searching),
                Event::Finished(id, result) => {
                    self.open -= 1;
                    let state = match result {
                        Ok(e) => ExportState::Done(e),
                        Err(f) => ExportState::Failed(f),
                    };
                    update(id, state);
                }
                Event::Cancelled(id) => {
                    self.open -= 1;
                    update(id, ExportState::Cancelled);
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
