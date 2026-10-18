//! Reads every WAV under the given folders, reporting those that fail and
//! how many loop
//!
//! `wav_check FOLDER...`

use std::path::{Path, PathBuf};

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::wav;

fn files(folder: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
        {
            out.push(path);
        }
    }
}

fn main() {
    let mut paths = Vec::new();
    for folder in std::env::args().skip(1) {
        files(Path::new(&folder), &mut paths);
    }
    let (mut read, mut looping, mut failed) = (0, 0, 0);
    let mut formats = std::collections::BTreeMap::new();
    for path in &paths {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        match wav::parse(&bytes, Limits::default()) {
            Ok(s) => {
                read += 1;
                looping += usize::from(s.loop_start.is_some());
                *formats
                    .entry(format!("{} Hz, {} channels", s.rate, s.channels))
                    .or_insert(0) += 1;
            }
            Err(e) => {
                failed += 1;
                if failed <= 10 {
                    println!("{}: {e}", path.display());
                }
            }
        }
    }
    println!("{read} read, {looping} looping, {failed} failed");
    for (kind, count) in formats {
        println!("  {count:5} {kind}");
    }
}
