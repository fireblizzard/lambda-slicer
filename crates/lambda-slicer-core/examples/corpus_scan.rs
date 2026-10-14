//! Development probe: inspect every `.dem` under the given roots (read-only)
//! and summarize container status and findings.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lambda_slicer_core::container::Limits;
use lambda_slicer_core::validate::structural::{Severity, inspect};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        let p = e.path();
        if ft.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("dem")) {
            out.push(p);
        }
    }
}

fn main() {
    let mut files = Vec::new();
    for root in std::env::args().skip(1) {
        walk(Path::new(&root), &mut files);
    }
    let start = std::time::Instant::now();
    let (mut bytes, mut status) = (0u64, BTreeMap::<String, u64>::new());
    let mut codes = BTreeMap::<(String, &'static str), u64>::new();
    let mut profiles = BTreeMap::<(i32, i32, String), u64>::new();
    let mut examples = BTreeMap::<&'static str, Vec<String>>::new();
    for p in &files {
        let r = inspect(p, Limits::default());
        bytes += r.size;
        *status.entry(format!("{:?}", r.status)).or_default() += 1;
        if let Some(h) = &r.header {
            *profiles
                .entry((h.demo_protocol, h.network_protocol, h.game_dir.clone()))
                .or_default() += 1;
        }
        for f in &r.findings {
            *codes
                .entry((format!("{:?}", f.severity), f.code))
                .or_default() += 1;
            if f.severity >= Severity::Warning {
                let ex = examples.entry(f.code).or_default();
                if ex.len() < 3 {
                    ex.push(format!("{}: {}", p.display(), f.message));
                }
            }
        }
    }
    println!(
        "{} files, {} bytes, {:.1}s",
        files.len(),
        bytes,
        start.elapsed().as_secs_f64()
    );
    println!("status {status:?}");
    println!("profiles {profiles:?}");
    for (k, v) in &codes {
        println!("  {k:?}: {v}");
    }
    for (k, v) in &examples {
        println!("examples {k}:");
        for e in v {
            println!("    {e}");
        }
    }
}
