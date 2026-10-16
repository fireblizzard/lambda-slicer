//! Library scan jobs (implementation plan §7.1)
//!
//! A discovery thread streams paths into a bounded queue, workers check the
//! cache by metadata and analyse what is missing or stale, and the calling
//! thread is the only cache writer. Files asked for with
//! [`ScanControl::prioritize`] jump the queue. Cancelling stops discovery and
//! every worker at the next frame, keeps what was finished, and stores
//! nothing for files cut short. A file that fails, even by panicking, gets
//! its own result and the scan goes on

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufReader;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

use super::cache::{Cache, CacheError, Entry, HeaderInfo, analysis_version};
use super::discover::{self, Found};
use super::fingerprint::{file_identity, file_metadata, hash_file, quick_fingerprint};
use crate::analysis::{AnalysisReport, analyze_demo_with};
use crate::container::{DemoReader, Limits};
use crate::message::{Coded, Message, io_message};
use crate::state::DecodeStatus;

/// Discovered paths waiting for a worker, beyond which discovery waits
const QUEUE_BOUND: usize = 1024;

/// Shortest time between progress events
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub roots: Vec<PathBuf>,
    pub cache: PathBuf,
    /// Parallel analyses. One suits a single spinning disk
    pub workers: usize,
    /// Analyse even files with a current cache entry
    pub force: bool,
    /// Also hash every analysed file in full
    pub full_hash: bool,
    pub limits: Limits,
}

#[derive(Default)]
struct Queue {
    priority: VecDeque<PathBuf>,
    normal: VecDeque<PathBuf>,
    discovery_done: bool,
}

/// Shared between a running scan and whoever controls it
#[derive(Default)]
pub struct ScanControl {
    cancel: Arc<AtomicBool>,
    queue: Mutex<Queue>,
    changed: Condvar,
}

impl ScanControl {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Stop discovery and every analysis at the next frame
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.changed.notify_all();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// Analyse `path` before anything discovered, such as a file the user
    /// opened. A current cache entry still counts as done
    pub fn prioritize(&self, path: PathBuf) {
        self.queue.lock().unwrap().priority.push_back(path);
        self.changed.notify_all();
    }

    /// Queue a discovered path, waiting while the queue is full. False when
    /// cancelled
    fn push(&self, path: PathBuf) -> bool {
        let mut q = self.queue.lock().unwrap();
        while q.normal.len() >= QUEUE_BOUND && !self.is_cancelled() {
            q = self.changed.wait(q).unwrap();
        }
        q.normal.push_back(path);
        self.changed.notify_all();

        !self.is_cancelled()
    }

    fn finish_discovery(&self) {
        self.queue.lock().unwrap().discovery_done = true;
        self.changed.notify_all();
    }

    /// The next path, or none once discovery is done and the queue empty,
    /// or when cancelled
    fn pop(&self) -> Option<PathBuf> {
        let mut q = self.queue.lock().unwrap();
        loop {
            if self.is_cancelled() {
                return None;
            }
            if let Some(p) = q.priority.pop_front().or_else(|| q.normal.pop_front()) {
                self.changed.notify_all();
                return Some(p);
            }
            if q.discovery_done {
                return None;
            }
            q = self.changed.wait(q).unwrap();
        }
    }
}

/// What happened to one file
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum Outcome {
    /// The cache entry is current
    CacheHit,
    /// Analysed and stored, with the cache status
    Analyzed { status: String },
    /// The same file as an earlier path, by volume identity
    Duplicate { of: String },
    /// The file could not be read, nothing stored
    Unreadable { error: Message },
    /// The file changed while it was analysed, nothing stored
    SourceChanged,
    /// Stopped by cancellation, nothing stored
    Cancelled,
    /// Analysed, but the result could not be stored
    CacheFailed { error: Message },
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ScanProgress {
    pub discovered: u64,
    pub completed: u64,
    pub cache_hits: u64,
    pub analyzed: u64,
    /// Of those analysed, how many did not decode completely
    pub incomplete: u64,
    pub duplicates: u64,
    pub unreadable: u64,
    pub changed: u64,
    pub cache_failures: u64,
    pub directory_errors: u64,
    pub skipped_links: u64,
    /// Bytes read by hashing and analysis
    pub bytes_read: u64,
    /// Files being analysed now
    pub current: Vec<String>,
    pub elapsed_seconds: f64,
    pub discovery_done: bool,
    pub cancelled: bool,
}

impl ScanProgress {
    /// Discovered files not yet completed
    pub fn pending(&self) -> u64 {
        self.discovered.saturating_sub(self.completed)
    }
}

pub enum ScanEvent<'a> {
    File {
        path: &'a Path,
        outcome: &'a Outcome,
    },
    /// A directory or entry discovery could not read
    DiscoveryError {
        path: &'a Path,
        error: &'a str,
    },
    Progress(&'a ScanProgress),
}

/// One analysed file, owned so it can cross to the writer
struct Analysed {
    path: PathBuf,
    size: u64,
    mtime_ns: Option<i64>,
    identity: Option<String>,
    quick_hash: Option<String>,
    full_hash: Option<String>,
    header: Option<HeaderInfo>,
    report: Option<AnalysisReport>,
    error: Option<Message>,
}

enum Update {
    Discovered,
    Found(Found),
    Started(usize, PathBuf),
    Done {
        worker: usize,
        path: PathBuf,
        outcome: Outcome,
        analysed: Option<Box<Analysed>>,
        bytes: u64,
    },
}

fn clamp_ns(ns: Option<i128>) -> Option<i64> {
    ns.map(|n| n.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
}

fn header_info(path: &Path, limits: Limits) -> Option<HeaderInfo> {
    let r = DemoReader::new(BufReader::new(File::open(path).ok()?), limits).ok()?;

    Some(HeaderInfo {
        map: r.header.map_name(),
        game_dir: r.header.game_dir(),
        demo_protocol: r.header.demo_protocol,
        network_protocol: r.header.network_protocol,
        duration: r.directory.iter().map(|e| e.track_time()).sum(),
    })
}

fn panic_text(p: &(dyn std::any::Any + Send)) -> String {
    p.downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| p.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into())
}

struct Worker<'a> {
    id: usize,
    options: &'a ScanOptions,
    control: &'a ScanControl,
    cache: Cache,
    version: String,
    /// File identity to the first path seen with it
    seen: &'a Mutex<HashMap<String, PathBuf>>,
    tx: Sender<Update>,
}

impl Worker<'_> {
    fn run(self) {
        while let Some(path) = self.control.pop() {
            let _ = self.tx.send(Update::Started(self.id, path.clone()));
            let (outcome, analysed, bytes) = self.one(&path);
            let _ = self.tx.send(Update::Done {
                worker: self.id,
                path,
                outcome,
                analysed,
                bytes,
            });
        }
    }

    /// Record `identity` for `path`, returning the earlier path if another
    /// already has it
    fn claim(&self, identity: &str, path: &Path) -> Option<PathBuf> {
        let mut seen = self.seen.lock().unwrap();
        match seen.get(identity) {
            Some(first) if first != path => Some(first.clone()),
            Some(_) => None,
            None => {
                seen.insert(identity.to_string(), path.to_path_buf());
                None
            }
        }
    }

    fn one(&self, path: &Path) -> (Outcome, Option<Box<Analysed>>, u64) {
        let unreadable = |e: std::io::Error| {
            (
                Outcome::Unreadable {
                    error: io_message(&e),
                },
                None,
                0,
            )
        };
        let (size, mtime) = match file_metadata(path) {
            Ok((size, mtime)) => (size, clamp_ns(mtime)),
            Err(e) => return unreadable(e),
        };

        if !self.options.force
            && let Ok(Some(c)) = self.cache.lookup(path)
            && c.size == size
            && c.mtime_ns == mtime
            && c.version == self.version
        {
            if let Some(of) = c.file_identity.as_deref().and_then(|i| self.claim(i, path)) {
                return (
                    Outcome::Duplicate {
                        of: of.display().to_string(),
                    },
                    None,
                    0,
                );
            }
            return (Outcome::CacheHit, None, 0);
        }

        let identity = match file_identity(path) {
            Ok(i) => i,
            Err(e) => return unreadable(e),
        };
        if let Some(of) = self.claim(&identity, path) {
            return (
                Outcome::Duplicate {
                    of: of.display().to_string(),
                },
                None,
                0,
            );
        }

        let quick = match quick_fingerprint(path) {
            Ok(q) => q,
            Err(e) => return unreadable(e),
        };
        let mut bytes = quick.size.min(128 * 1024);
        let full_hash = if self.options.full_hash {
            match hash_file(path) {
                Ok(h) => {
                    bytes += size;
                    Some(h)
                }
                Err(e) => return unreadable(e),
            }
        } else {
            None
        };
        let header = header_info(path, self.options.limits);

        let cancel = Some(self.control.cancel.clone());
        let result = catch_unwind(AssertUnwindSafe(|| {
            analyze_demo_with(path, self.options.limits, cancel)
        }));
        bytes += size;
        let (report, error) = match result {
            Ok(r) if r.decode == DecodeStatus::Cancelled => {
                return (Outcome::Cancelled, None, bytes);
            }
            Ok(r) => (Some(r), None),
            Err(p) => {
                let detail = panic_text(p.as_ref());
                let m = Message::new("analysis-panicked", format!("analysis panicked: {detail}"))
                    .arg("detail", detail);
                (None, Some(m))
            }
        };

        match file_metadata(path) {
            Ok((s, m)) if s == size && clamp_ns(m) == mtime => {}
            _ => return (Outcome::SourceChanged, None, bytes),
        }

        let analysed = Analysed {
            path: path.to_path_buf(),
            size,
            mtime_ns: mtime,
            identity: Some(identity),
            quick_hash: Some(quick.quick_hash),
            full_hash,
            header,
            report,
            error,
        };
        // The writer fills in the stored status
        (
            Outcome::Analyzed {
                status: String::new(),
            },
            Some(Box::new(analysed)),
            bytes,
        )
    }
}

/// Run a scan to the end or until cancelled, calling `on_event` from this
/// thread for every file, discovery error, and at most every 100 ms with
/// progress. Errors only when the cache cannot be opened
pub fn scan(
    options: &ScanOptions,
    control: &Arc<ScanControl>,
    mut on_event: impl FnMut(ScanEvent),
) -> Result<ScanProgress, CacheError> {
    let started = Instant::now();
    let mut writer = Cache::open(&options.cache)?;
    let mut readers = Vec::new();
    for _ in 0..options.workers.max(1) {
        readers.push(Cache::open(&options.cache)?);
    }

    let version = analysis_version();
    let seen = Mutex::new(HashMap::new());
    let mut progress = ScanProgress::default();
    let mut current: Vec<Option<PathBuf>> = vec![None; readers.len()];
    let mut last_progress = Instant::now() - PROGRESS_INTERVAL;

    std::thread::scope(|s| {
        let (tx, rx) = mpsc::channel::<Update>();

        let discovery_tx = tx.clone();
        s.spawn(move || {
            let (roots, errors) = discover::roots(&options.roots);
            for e in errors {
                let _ = discovery_tx.send(Update::Found(e));
            }
            discover::walk(&roots, |f| match f {
                Found::Demo(p) => {
                    let _ = discovery_tx.send(Update::Discovered);
                    control.push(p)
                }
                other => {
                    let _ = discovery_tx.send(Update::Found(other));
                    !control.is_cancelled()
                }
            });
            control.finish_discovery();
        });

        for (id, cache) in readers.into_iter().enumerate() {
            let worker = Worker {
                id,
                options,
                control,
                cache,
                version: version.clone(),
                seen: &seen,
                tx: tx.clone(),
            };
            s.spawn(move || worker.run());
        }
        drop(tx);

        loop {
            let message = rx.recv_timeout(PROGRESS_INTERVAL);
            match message {
                Ok(Update::Discovered) => progress.discovered += 1,
                Ok(Update::Found(Found::Error { path, error })) => {
                    progress.directory_errors += 1;
                    on_event(ScanEvent::DiscoveryError {
                        path: &path,
                        error: &error,
                    });
                }
                Ok(Update::Found(Found::Skipped(_))) => progress.skipped_links += 1,
                Ok(Update::Found(Found::Demo(_))) => {}
                Ok(Update::Started(worker, path)) => current[worker] = Some(path),
                Ok(Update::Done {
                    worker,
                    path,
                    mut outcome,
                    analysed,
                    bytes,
                }) => {
                    current[worker] = None;
                    progress.bytes_read += bytes;
                    if let Some(a) = analysed {
                        let entry = Entry {
                            path: &a.path,
                            size: a.size,
                            mtime_ns: a.mtime_ns,
                            file_identity: a.identity.clone(),
                            quick_hash: a.quick_hash.clone(),
                            full_hash: a.full_hash.clone(),
                            header: a.header.clone(),
                            report: a.report.as_ref(),
                            error: a.error.clone(),
                        };
                        outcome = match writer.store(&entry) {
                            Ok(_) => Outcome::Analyzed {
                                status: entry.status().to_string(),
                            },
                            Err(e) => Outcome::CacheFailed { error: e.message() },
                        };
                    }
                    match &outcome {
                        Outcome::CacheHit => progress.cache_hits += 1,
                        Outcome::Analyzed { status } => {
                            progress.analyzed += 1;
                            if status != "complete" {
                                progress.incomplete += 1;
                            }
                        }
                        Outcome::Duplicate { .. } => progress.duplicates += 1,
                        Outcome::Unreadable { .. } => progress.unreadable += 1,
                        Outcome::SourceChanged => progress.changed += 1,
                        Outcome::CacheFailed { .. } => progress.cache_failures += 1,
                        Outcome::Cancelled => {}
                    }
                    if outcome != Outcome::Cancelled {
                        progress.completed += 1;
                    }
                    on_event(ScanEvent::File {
                        path: &path,
                        outcome: &outcome,
                    });
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }

            if last_progress.elapsed() >= PROGRESS_INTERVAL {
                last_progress = Instant::now();
                progress.current = current
                    .iter()
                    .flatten()
                    .map(|p| p.display().to_string())
                    .collect();
                progress.elapsed_seconds = started.elapsed().as_secs_f64();
                progress.discovery_done = control.queue.lock().unwrap().discovery_done;
                on_event(ScanEvent::Progress(&progress));
            }
        }
    });

    progress.current.clear();
    progress.elapsed_seconds = started.elapsed().as_secs_f64();
    progress.discovery_done = control.queue.lock().unwrap().discovery_done;
    progress.cancelled = control.is_cancelled();

    Ok(progress)
}
