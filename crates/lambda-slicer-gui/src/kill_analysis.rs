//! Telling which Gauss kills of the open demo went through a wall, on a
//! thread of its own once the analysis is in and the game's files are
//! known
//!
//! The events and the movement refinements never wait for it. Its verdicts
//! arrive a kill at a time, each raising a revision the shown events are
//! worked out again for. Another demo or other game files drop the job and
//! any reply it had yet to give

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use eframe::egui;
use lambda_slicer_core::message::Message;
use lambda_slicer_playback::MapHulls;
use lambda_slicer_playback::kill_features::{self, GaussKill, Progress, WallGauss};

use crate::timeline::{EventKind, KillKey, Model};

/// How far telling the kills through a wall has got
#[derive(Clone, Debug, PartialEq)]
pub enum WallStatus {
    /// The game's files are needed to trace the map
    NeedsFiles,
    Checking,
    Done,
    Failed(Message),
}

/// What the job's thread sends back
enum Reply {
    Verdict(KillKey, WallGauss, Progress),
    Finished,
    Failed(Message),
}

/// The job telling a demo's kills apart, stopped when dropped
struct WallJob {
    cancel: Arc<AtomicBool>,
    rx: Receiver<Reply>,
    thread: Option<JoinHandle<()>>,
}

impl WallJob {
    fn start(
        path: PathBuf,
        hulls: MapHulls,
        kills: Vec<(KillKey, GaussKill)>,
        ctx: egui::Context,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let thread_cancel = cancel.clone();

        let thread = std::thread::Builder::new()
            .name("wallgauss".into())
            .spawn(move || {
                let (keys, kills): (Vec<KillKey>, Vec<GaussKill>) = kills.into_iter().unzip();
                let verdict = |index: usize, wall: WallGauss, progress: Progress| {
                    let _ = tx.send(Reply::Verdict(keys[index], wall, progress));
                    ctx.request_repaint();
                };
                let result = kill_features::classify(&path, hulls, &kills, &thread_cancel, verdict);

                let reply = match result {
                    Ok(()) => Reply::Finished,
                    Err(message) => Reply::Failed(message),
                };
                let _ = tx.send(reply);
                ctx.request_repaint();
            })
            .expect("spawn wallgauss");

        Self {
            cancel,
            rx,
            thread: Some(thread),
        }
    }
}

impl Drop for WallJob {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The open demo's kills through a wall, as far as they are told
pub struct Walls {
    job: Option<WallJob>,
    /// The installation the verdicts are for, none without the game's
    /// files, and nothing until set up
    set_up_for: Option<Option<PathBuf>>,
    verdicts: HashMap<KillKey, WallGauss>,
    revision: u64,
    progress: Progress,
    status: WallStatus,
}

impl Default for Walls {
    fn default() -> Self {
        Self {
            job: None,
            set_up_for: None,
            verdicts: HashMap::new(),
            revision: 0,
            progress: Progress::default(),
            status: WallStatus::NeedsFiles,
        }
    }
}

impl Walls {
    /// The verdict on one kill, none while it is not told yet
    pub fn get(&self, key: &KillKey) -> Option<&WallGauss> {
        self.verdicts.get(key)
    }

    /// Raised whenever a verdict arrives or the verdicts start over
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn status(&self) -> &WallStatus {
        &self.status
    }

    pub fn progress(&self) -> Progress {
        self.progress
    }

    /// Tell the kills of `model` apart with the game's files from `install`,
    /// whose maps `hulls` gives, or say the files are needed. Starts over
    /// when the installation changes and does nothing otherwise
    pub fn update(
        &mut self,
        path: &Path,
        model: &Model,
        install: Option<&Path>,
        hulls: impl FnOnce() -> Option<MapHulls>,
        ctx: &egui::Context,
    ) {
        let install = install.map(Path::to_path_buf);
        if self.set_up_for.as_ref() == Some(&install) {
            return;
        }

        // Verdicts from other files, or a job still giving them, go
        self.job = None;
        self.verdicts.clear();
        self.revision += 1;
        self.progress = Progress::default();
        self.set_up_for = Some(install.clone());

        let Some(hulls) = install.and_then(|_| hulls()) else {
            self.status = WallStatus::NeedsFiles;
            return;
        };

        let mut kills = Vec::new();
        for event in &model.events {
            if let EventKind::Kill { features, .. } = &event.kind
                && let Some(kill) = &features.gauss_kill
            {
                kills.push((features.key, kill.clone()));
            }
        }
        if kills.is_empty() {
            self.status = WallStatus::Done;
            return;
        }

        self.progress.total = kills.len();
        self.status = WallStatus::Checking;
        self.job = Some(WallJob::start(
            path.to_path_buf(),
            hulls,
            kills,
            ctx.clone(),
        ));
    }

    /// A verdict as if the job had sent it, for tests
    #[cfg(test)]
    pub fn insert(&mut self, key: KillKey, wall: WallGauss) {
        self.verdicts.insert(key, wall);
        self.revision += 1;
    }

    /// Take in the verdicts that arrived
    pub fn poll(&mut self) {
        let Some(job) = &self.job else {
            return;
        };

        let mut finished = false;
        while let Ok(reply) = job.rx.try_recv() {
            match reply {
                Reply::Verdict(key, wall, progress) => {
                    self.verdicts.insert(key, wall);
                    self.progress = progress;
                    self.revision += 1;
                }
                Reply::Finished => {
                    self.status = WallStatus::Done;
                    finished = true;
                }
                Reply::Failed(message) => {
                    self.status = WallStatus::Failed(message);
                    finished = true;
                }
            }
        }

        if finished {
            self.job = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use lambda_slicer_playback::kill_features::Doubt;

    use super::*;

    #[test]
    fn other_game_files_start_the_verdicts_over() {
        let ctx = egui::Context::default();
        let model = Model::default();
        let mut walls = Walls::default();

        // Without the game's files nothing is told, and it says so
        walls.update(Path::new("a.dem"), &model, None, || None, &ctx);
        assert_eq!(walls.status(), &WallStatus::NeedsFiles);
        let key = KillKey {
            segment: 0,
            ordinal: 3,
        };
        walls.insert(key, WallGauss::Unknown(Doubt::NoMap));
        let before = walls.revision();

        // The same files again change nothing
        walls.update(Path::new("a.dem"), &model, None, || None, &ctx);
        assert_eq!(walls.revision(), before);
        assert!(walls.get(&key).is_some());

        // Other files drop what was told with the old ones
        let install = Path::new("C:/games/Half-Life");
        walls.update(Path::new("a.dem"), &model, Some(install), || None, &ctx);
        assert!(walls.revision() > before);
        assert_eq!(walls.get(&key), None);
    }
}
