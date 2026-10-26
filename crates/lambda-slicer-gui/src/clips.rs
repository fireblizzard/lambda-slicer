//! Clips marked in demos, kept between runs as JSON beside the settings
//!
//! A clip is a named, requested range `[start, end)` and the part of the
//! demo it is in. The demo's clock can start over after a map change, so two
//! parts can share times, and the part says which one the clip cuts. The
//! range an export really cuts snaps to the recording's frames and is worked
//! out when shown, never stored. Clips are keyed by the demo's path as the
//! library shows it.
//!
//! A clips file that cannot be used is never written over. One that reads
//! but is not valid is kept under another name and the clips start empty,
//! and one that cannot be read at all stops saving for the session

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lambda_slicer_core::timeline::time::DemoTime;
use serde::{Deserialize, Serialize};

use crate::export::ClipKey;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clip {
    pub name: String,
    pub start: DemoTime,
    pub end: DemoTime,
    /// The session it is in, the decoder's count of them. Clips kept before
    /// parts were stored have none, and cut the earliest part with their
    /// times
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<u32>,
}

impl Clip {
    /// What its export depends on, its range and its part
    pub fn key(&self) -> ClipKey {
        (self.start, self.end, self.part)
    }

    /// The requested length, zero when the end is not after the start
    pub fn length(&self) -> DemoTime {
        DemoTime::from_micros((self.end.micros() - self.start.micros()).max(0))
    }
}

/// Why the clips file could not be used
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadProblem {
    /// Not valid clips. It was renamed to `moved`, so saving starts afresh
    Moved { moved: PathBuf, detail: String },
    /// Not readable, and so never saved over this session
    Unreadable { detail: String },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipStore {
    pub demos: BTreeMap<String, Vec<Clip>>,
}

/// The demo's path as text, or for a path that is not valid Unicode its own
/// bytes in hex, so two such paths never share their clips
fn key(demo: &Path) -> String {
    match demo.to_str() {
        Some(text) => text.to_string(),
        None => {
            let bytes = lambda_slicer_core::library::cache::native_bytes(demo);
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            format!("native:{hex}")
        }
    }
}

impl ClipStore {
    /// The store at `path`, empty when it is missing. A file that cannot
    /// be used gives an empty store and the reason, and is left where it
    /// is or renamed, never lost
    pub fn load(path: &Path) -> (Self, Option<LoadProblem>) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Self::default(), None),
            Err(e) => {
                let detail = e.to_string();
                return (Self::default(), Some(LoadProblem::Unreadable { detail }));
            }
        };
        match serde_json::from_slice(&bytes) {
            Ok(store) => (store, None),
            Err(e) => {
                let detail = e.to_string();
                let problem = match keep_aside(path, &bytes) {
                    Some(moved) => LoadProblem::Moved { moved, detail },
                    None => LoadProblem::Unreadable { detail },
                };
                (Self::default(), Some(problem))
            }
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        crate::settings::write_json(path, &serde_json::to_vec_pretty(self)?)
    }

    pub fn get(&self, demo: &Path) -> Vec<Clip> {
        self.demos.get(&key(demo)).cloned().unwrap_or_default()
    }

    /// Replace a demo's clips, forgetting the demo when it has none
    pub fn set(&mut self, demo: &Path, clips: Vec<Clip>) {
        if clips.is_empty() {
            self.demos.remove(&key(demo));
        } else {
            self.demos.insert(key(demo), clips);
        }
    }
}

/// Copy an unusable clips file's `bytes` to a new file beside it, never
/// replacing one, then remove it. The copy's name, when that worked
fn keep_aside(path: &Path, bytes: &[u8]) -> Option<PathBuf> {
    use std::io::Write;
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (moved, mut file) = (1..100).find_map(|n| {
        let suffix = match n {
            1 => format!("json.unreadable-{seconds}"),
            n => format!("json.unreadable-{seconds}-{n}"),
        };
        let candidate = path.with_extension(suffix);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
            .ok()?;
        Some((candidate, file))
    })?;
    if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
        drop(file);
        let _ = std::fs::remove_file(&moved);
        return None;
    }
    std::fs::remove_file(path).ok()?;
    Some(moved)
}

/// The first of `name(1)`, `name(2)` and so on that no clip uses yet
pub fn next_name(clips: &[Clip], name: impl Fn(i64) -> String) -> String {
    (1..)
        .map(name)
        .find(|n| clips.iter().all(|c| &c.name != n))
        .expect("an unused name")
}

/// A time typed as seconds, minutes and seconds or `HH:MM:SS.mmm`, with a
/// point, a comma or the Arabic decimal separator before the fraction
pub fn parse_time(text: &str) -> Option<DemoTime> {
    let cleaned: String = text
        .trim()
        .chars()
        .map(|c| if c == ',' || c == '\u{66b}' { '.' } else { c })
        .collect();
    cleaned.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(name: &str, start: &str, end: &str) -> Clip {
        Clip {
            name: name.into(),
            start: start.parse().unwrap(),
            end: end.parse().unwrap(),
            part: None,
        }
    }

    #[test]
    fn a_clip_keeps_its_part_and_one_kept_without_has_none() {
        let mut later = clip("Later", "10", "20");
        later.part = Some(2);
        let text = serde_json::to_string(&later).unwrap();
        assert_eq!(serde_json::from_str::<Clip>(&text).unwrap(), later);

        // A clip with no part is kept as clips were before parts
        let earlier = clip("Earlier", "10", "20");
        let text = serde_json::to_string(&earlier).unwrap();
        assert!(!text.contains("part"), "{text}");
        assert_eq!(serde_json::from_str::<Clip>(&text).unwrap(), earlier);
    }

    #[test]
    fn clips_round_trip_per_demo() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("clips.json");
        assert_eq!(ClipStore::load(&path), (ClipStore::default(), None));

        let demo = Path::new("F:/demos/a.dem");
        let mut store = ClipStore::default();
        store.set(demo, vec![clip("Clip 1", "1:36", "4:48")]);
        store.save_to(&path).unwrap();
        let (loaded, problem) = ClipStore::load(&path);
        assert_eq!(problem, None);
        assert_eq!(loaded.get(demo), store.get(demo));
        assert_eq!(loaded.get(Path::new("F:/demos/b.dem")), Vec::new());

        store.set(demo, Vec::new());
        assert!(store.demos.is_empty());

        // A file that is not clips is kept under another name, never lost
        std::fs::write(&path, b"not json").unwrap();
        let (loaded, problem) = ClipStore::load(&path);
        assert_eq!(loaded, ClipStore::default());
        let Some(LoadProblem::Moved { moved, .. }) = problem else {
            panic!("{problem:?}");
        };
        assert_eq!(std::fs::read(&moved).unwrap(), b"not json");
        assert!(!path.exists());

        // A second one in the same second gets a name of its own
        std::fs::write(&path, b"still not json").unwrap();
        let (_, problem) = ClipStore::load(&path);
        let Some(LoadProblem::Moved { moved: second, .. }) = problem else {
            panic!("{problem:?}");
        };
        assert_ne!(second, moved);
        assert_eq!(std::fs::read(&moved).unwrap(), b"not json");
        assert_eq!(std::fs::read(&second).unwrap(), b"still not json");
    }

    #[test]
    fn paths_that_are_not_unicode_keep_their_own_clips() {
        // Two names differing only where they are not valid Unicode
        #[cfg(windows)]
        let (a, b) = {
            use std::os::windows::ffi::OsStringExt;
            let name = |unit: u16| std::ffi::OsString::from_wide(&[0x61, unit, 0x2e, 0x64]);
            (PathBuf::from(name(0xd800)), PathBuf::from(name(0xd801)))
        };
        #[cfg(not(windows))]
        let (a, b) = {
            use std::os::unix::ffi::OsStrExt;
            let name = |byte: u8| std::ffi::OsStr::from_bytes(&[b'a', byte, b'.', b'd']).to_owned();
            (PathBuf::from(name(0xfe)), PathBuf::from(name(0xff)))
        };
        assert_eq!(a.to_string_lossy(), b.to_string_lossy());
        assert_ne!(key(&a), key(&b));

        let mut store = ClipStore::default();
        store.set(&a, vec![clip("A", "0", "1")]);
        assert_eq!(store.get(&b), Vec::new());
        assert_eq!(store.get(&a).len(), 1);

        let native = lambda_slicer_core::library::cache::native_bytes(&a);
        assert_eq!(
            lambda_slicer_core::library::cache::path_from_native(&native),
            Some(a)
        );
    }

    #[test]
    fn names_and_lengths() {
        let clips = [clip("Clip 1", "0", "1"), clip("Clip 3", "0", "1")];
        assert_eq!(next_name(&clips, |n| format!("Clip {n}")), "Clip 2");
        assert_eq!(
            clip("x", "1:36", "4:48").length().to_string(),
            "00:03:12.000"
        );
        assert_eq!(clip("x", "10", "5").length(), DemoTime::ZERO);
    }

    #[test]
    fn times_take_any_decimal_separator() {
        let t = |s: &str| parse_time(s).map(|t| t.to_string());
        assert_eq!(t("96").as_deref(), Some("00:01:36.000"));
        assert_eq!(t(" 1:36 ").as_deref(), Some("00:01:36.000"));
        assert_eq!(t("00:01:36,250").as_deref(), Some("00:01:36.250"));
        assert_eq!(t("96\u{66b}5").as_deref(), Some("00:01:36.500"));
        assert_eq!(t("1:60"), None);
        assert_eq!(t(""), None);
    }
}
