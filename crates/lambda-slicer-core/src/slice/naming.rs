//! File names for exported clips and the console command that plays them
//!
//! A clip of `[1:36, 4:48)` from `source.dem` is suggested as
//! `source__01m36s000-04m48s000.dem`, or after its own name as
//! `source__name.dem` or `name.dem`. Names keep only ASCII letters, digits,
//! `-`, `_` and `.`, since the game's console splits a command at spaces and
//! reads paths in the system code page. No name ends in a dot, and a clip's
//! own name standing alone is kept off the reserved Windows device names. A
//! free name is only a suggestion, as publication itself refuses to replace
//! any file

use std::path::{Path, PathBuf};

use crate::timeline::time::DemoTime;

/// Longest part of a name taken from the source's own name
const STEM_CHARS: usize = 64;

/// `01m36s000`, or `1h02m03s004` from an hour on
fn stamp(t: DemoTime) -> String {
    let ms = t.micros().max(0) / 1000;
    let (h, m, s, ms) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000);
    if h > 0 {
        format!("{h}h{m:02}m{s:02}s{ms:03}")
    } else {
        format!("{m:02}m{s:02}s{ms:03}")
    }
}

/// Windows device names, which no file may have before its first dot
const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// A name part with every character the console or a file system could
/// trip over replaced by `_`, runs of them merged, `demo` when nothing is
/// left
pub fn sanitize(text: &str) -> String {
    kept(text).unwrap_or_else(|| "demo".into())
}

/// What [`sanitize`] keeps of a name part, none when nothing is left
fn kept(text: &str) -> Option<String> {
    let mut out = String::new();
    for c in text.chars() {
        let c = if c.is_ascii_alphanumeric() || matches!(c, '-' | '.') {
            c
        } else {
            '_'
        };
        if !(c == '_' && out.ends_with('_')) {
            out.push(c);
        }
    }
    let out = out.trim_matches(|c| c == '_' || c == '.');
    let out: String = out.chars().take(STEM_CHARS).collect();
    let out = out.trim_end_matches(['_', '.']);
    (!out.is_empty()).then(|| out.to_string())
}

/// The suggested file name for `[start, end)` of `source`
pub fn clip_file_name(source: &Path, start: DemoTime, end: DemoTime) -> String {
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    format!("{}__{}-{}.dem", sanitize(&stem), stamp(start), stamp(end))
}

/// The file name for a clip called `name` of `source`, after the source's
/// name when `prefixed`. A name with nothing the console can keep is named
/// by its times instead
pub fn named_clip_file_name(
    source: &Path,
    name: &str,
    (start, end): (DemoTime, DemoTime),
    prefixed: bool,
) -> String {
    let Some(name) = kept(name) else {
        return clip_file_name(source, start, end);
    };

    if prefixed {
        let stem = source
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        return format!("{}__{name}.dem", sanitize(&stem));
    }

    // A device name before the first dot is kept off with a `_` after it
    let device = name.split('.').next().unwrap_or_default();
    if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(device)) {
        let rest = &name[device.len()..];
        return format!("{device}_{rest}.dem");
    }
    format!("{name}.dem")
}

/// `name` in `dir`, or with `_2`, `_3` and so on before the extension when
/// a file already has that name
pub fn free_path(dir: &Path, name: &str) -> PathBuf {
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s, format!(".{e}")),
        None => (name, String::new()),
    };
    (1..)
        .map(|n| match n {
            1 => dir.join(name),
            n => dir.join(format!("{stem}_{n}{ext}")),
        })
        .find(|p| p.symlink_metadata().is_err())
        .expect("a free name")
}

/// The mod folder holding `path`, the nearest folder above it with a
/// `liblist.gam`
pub fn mod_dir(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|d| d.join("liblist.gam").is_file())
        .map(Path::to_path_buf)
}

/// The command that plays a demo within `mod_dir`, its path relative to the
/// mod folder without `.dem`. `None` when the path cannot be typed into the
/// console as it is
pub fn playdemo_in(mod_dir: &Path, demo: &Path) -> Option<String> {
    let relative = demo.strip_prefix(mod_dir).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_str().map(str::to_string))
        .collect::<Option<_>>()?;
    let mut path = parts.join("/");
    if let Some(stem) = path.strip_suffix(".dem") {
        path = stem.to_string();
    }
    let plain = |c: char| c.is_ascii_graphic() && c != '"' && c != ';';
    if path.is_empty() || !path.chars().all(|c| plain(c) || c == ' ') {
        return None;
    }
    Some(if path.contains(' ') {
        format!("playdemo \"{path}\"")
    } else {
        format!("playdemo {path}")
    })
}

/// [`playdemo_in`] for the mod folder holding `demo`, if any
pub fn playdemo(demo: &Path) -> Option<String> {
    playdemo_in(&mod_dir(demo)?, demo)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DemoTime {
        s.parse().unwrap()
    }

    #[test]
    fn names_follow_the_plan() {
        let name = clip_file_name(Path::new("F:/d/source.dem"), at("1:36"), at("4:48"));
        assert_eq!(name, "source__01m36s000-04m48s000.dem");
        let name = clip_file_name(
            Path::new("our match 3:1 «final».dem"),
            at("1:02:03.004"),
            at("1:02:13.5"),
        );
        assert_eq!(name, "our_match_3_1_final__1h02m03s004-1h02m13s500.dem");
        assert_eq!(sanitize("...__"), "demo");
        assert_eq!(sanitize("CON"), "CON");
        assert_eq!(sanitize(&"x".repeat(100)).len(), STEM_CHARS);
    }

    #[test]
    fn a_clip_named_by_its_own_name_keeps_what_the_console_can_read() {
        let source = Path::new("F:/d/20230910_crossfire.dem");
        let range = (at("1:36"), at("4:48"));
        let named = |name: &str, prefixed| named_clip_file_name(source, name, range, prefixed);

        assert_eq!(
            named("Triple gauss", true),
            "20230910_crossfire__Triple_gauss.dem"
        );
        assert_eq!(named("Triple gauss!", false), "Triple_gauss.dem");

        // Nothing the console can read is named by the times
        assert_eq!(
            named("三杀", false),
            "20230910_crossfire__01m36s000-04m48s000.dem"
        );

        // Windows keeps device names for itself
        assert_eq!(named("con", false), "con_.dem");
        assert_eq!(named("LPT1.final", false), "LPT1_.final.dem");
        assert_eq!(named("con", true), "20230910_crossfire__con.dem");
    }

    #[test]
    fn free_paths_skip_taken_names() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(free_path(dir.path(), "a.dem"), dir.path().join("a.dem"));
        std::fs::write(dir.path().join("a.dem"), b"").unwrap();
        std::fs::write(dir.path().join("a_2.dem"), b"").unwrap();
        assert_eq!(free_path(dir.path(), "a.dem"), dir.path().join("a_3.dem"));
    }

    #[test]
    fn playdemo_takes_the_path_inside_the_mod_folder() {
        let dir = tempfile::tempdir().unwrap();
        let ag = dir.path().join("Half-Life").join("ag");
        std::fs::create_dir_all(ag.join("clips")).unwrap();
        std::fs::write(ag.join("liblist.gam"), b"game \"AG\"").unwrap();

        let demo = ag.join("clips").join("a__01m36s000-04m48s000.dem");
        assert_eq!(mod_dir(&demo).as_deref(), Some(ag.as_path()));
        assert_eq!(
            playdemo(&demo).as_deref(),
            Some("playdemo clips/a__01m36s000-04m48s000")
        );
        assert_eq!(
            playdemo(&ag.join("my clips").join("a.dem")).as_deref(),
            Some("playdemo \"my clips/a\"")
        );
        assert_eq!(playdemo(&ag.join("клипы").join("a.dem")), None);
        assert_eq!(playdemo(&dir.path().join("a.dem")), None);
    }
}
