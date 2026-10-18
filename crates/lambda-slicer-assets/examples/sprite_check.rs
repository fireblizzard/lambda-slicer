//! Reads every sprite in the given folders, and AG's sprites named as
//! models, reporting those that fail
//!
//! `sprite_check FOLDER...`

use std::path::Path;

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::sprite;

fn main() {
    let (mut read, mut failed) = (0, 0);
    let mut formats = std::collections::BTreeMap::new();
    for folder in std::env::args().skip(1) {
        let Ok(entries) = std::fs::read_dir(Path::new(&folder)) else {
            eprintln!("{folder}: not a folder");
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            if !bytes.starts_with(b"IDSP") {
                continue;
            }
            match sprite::parse(&bytes, Limits::default()) {
                Ok(s) => {
                    read += 1;
                    *formats
                        .entry(format!("{:?} {:?}", s.orientation, s.format))
                        .or_insert(0) += 1;
                }
                Err(e) => {
                    failed += 1;
                    println!("{}: {e}", path.display());
                }
            }
        }
    }
    println!("{read} read, {failed} failed");
    for (kind, count) in formats {
        println!("  {count:4} {kind}");
    }
}
