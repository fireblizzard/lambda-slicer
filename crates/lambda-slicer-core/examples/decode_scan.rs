//! Development probe: decode every `.dem` under the given roots (read-only)
//! and summarize network decode status and failure causes.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lambda_slicer_core::state::{DecodeOptions, DecodeStatus, decode_demo};

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

/// Error text with numbers removed, for grouping.
fn category(e: &str) -> String {
    let tail = e.rsplit(": ").next().unwrap_or(e);
    let msg = e.split(" at payload byte ").next().unwrap_or("");
    let op = msg.rsplit(' ').next().unwrap_or("");
    let t: String = tail
        .chars()
        .map(|c| if c.is_ascii_digit() { '#' } else { c })
        .collect();
    format!("{op}: {t}")
}

fn main() {
    let mut files = Vec::new();
    for root in std::env::args().skip(1) {
        walk(Path::new(&root), &mut files);
    }
    let start = std::time::Instant::now();
    let mut status = BTreeMap::<String, u64>::new();
    let mut cats = BTreeMap::<String, (u64, Vec<String>)>::new();
    let (mut msgs, mut frames, mut dist, mut inferred, mut dup) = (0u64, 0u64, 0u32, 0u64, 0u64);
    let mut opcodes = BTreeMap::<String, u64>::new();
    for p in &files {
        let opts = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };
        let (r, _) = decode_demo(p, &opts, |_, _, _| {});
        *status.entry(format!("{:?}", r.status)).or_default() += 1;
        msgs += r.stats.messages;
        frames += r.stats.network_frames;
        dist = dist.max(r.stats.max_delta_distance);
        if r.stats.sequence_offset_changes > 0 || r.stats.empty_delta_bases > 0 {
            inferred += 1;
        }
        dup += r.stats.duplicate_sequences;
        for (k, v) in &r.stats.by_opcode {
            *opcodes.entry(k.clone()).or_default() += v;
        }
        if r.status != DecodeStatus::Complete {
            let e = r.error.unwrap_or_default();
            let c = cats.entry(category(&e)).or_default();
            c.0 += 1;
            if c.1.len() < 4 {
                c.1.push(format!("{}: {e}", p.display()));
            }
        }
    }
    println!(
        "{} files, {:.1}s, {frames} network frames, {msgs} messages",
        files.len(),
        start.elapsed().as_secs_f64()
    );
    println!("status {status:?}");
    println!(
        "max delta distance {dist}, files with inferred delta bases {inferred}, duplicate sequences {dup}"
    );
    println!("opcodes {opcodes:?}");
    for (k, (n, ex)) in &cats {
        println!("{n:6}  {k}");
        for e in ex {
            println!("          {e}");
        }
    }
}
