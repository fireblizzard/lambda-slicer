//! Seeks through checkpoints on real demos, timed and checked against one
//! sequential pass (docs/viewer-profile.md, "Checkpoints")
//!
//! `seek_probe [--spacing SECONDS] [--budget MIB] DEMO...`
//!
//! For each demo, the time to build checkpoints for all of it and the memory
//! they take, the time of a seek with none yet, then 60 seeks to moments
//! chosen at random, in random order, each timed and compared with the
//! state sequential playback has at that moment

use std::path::Path;
use std::time::Instant;

use lambda_slicer_core::container::Limits;
use lambda_slicer_playback::checkpoint::{BUDGET, Checkpoints, SPACING};
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

const SEEKS: usize = 60;

fn digest(p: &mut Playback, at: SourceTime) -> String {
    let d = p.decoder();
    let decoder = format!(
        "{:?}",
        (
            d.time_bits,
            d.segment,
            d.stats.messages,
            d.latest_snapshot().map(|s| s.sequence)
        )
    );
    let f = p.present(Stamp::default(), at, CameraMode::Recorder, false);
    format!(
        "{decoder} {:?}",
        (
            f.position,
            f.scene_revision,
            &f.entities,
            f.recorder,
            f.viewmodel,
            &f.sounds,
            f.audio_revision,
            &f.stopped
        )
    )
}

/// Private bytes of this process
#[cfg(windows)]
fn private_bytes() -> Option<u64> {
    #[repr(C)]
    #[derive(Default)]
    struct Counters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged_pool: usize,
        quota_paged_pool: usize,
        quota_peak_non_paged_pool: usize,
        quota_non_paged_pool: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(process: isize, counters: *mut Counters, cb: u32) -> i32;
    }

    let mut counters = Counters {
        cb: size_of::<Counters>() as u32,
        ..Default::default()
    };
    // SAFETY The pseudo handle needs no closing, and the counters are a
    // PROCESS_MEMORY_COUNTERS of the size given
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    (ok != 0).then_some(counters.pagefile_usage as u64)
}

#[cfg(not(windows))]
fn private_bytes() -> Option<u64> {
    None
}

fn mib(bytes: Option<u64>) -> String {
    bytes.map_or("n/a".into(), |b| format!("{:.0} MiB", b as f64 / 1048576.))
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn probe(path: &Path, spacing: f64, budget: u64) -> Result<bool, String> {
    let limits = Limits::default();
    let text = |m: lambda_slicer_core::message::Message| m.text;

    // Building every checkpoint, as the worker does while idle
    let before = private_bytes();
    let store = Checkpoints::shared(spacing, budget);
    let started = Instant::now();
    let mut indexer = Playback::open_with(path, limits, store.clone()).map_err(text)?;
    while !indexer.index(1000) {}
    let index_seconds = started.elapsed().as_secs_f64();
    let end = indexer.position().map_or(0., |p| p.time.0);
    drop(indexer);
    let after = private_bytes();
    let (count, estimate, final_spacing) = {
        let s = store.lock().unwrap();
        (s.len(), s.bytes(), s.spacing())
    };
    let added = before.zip(after).map(|(b, a)| a.saturating_sub(b));
    println!(
        "{}: {end:.0} s of playback, {count} checkpoints {final_spacing} s apart built in {index_seconds:.1} s, {} measured and {} estimated for them",
        path.display(),
        mib(added),
        mib(Some(estimate)),
    );

    // Moments at random, and the state sequential playback has at each
    let mut seed = 0x9e37_79b9_u32;
    let mut random = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };
    let mut targets: Vec<f64> = (0..SEEKS)
        .map(|_| f64::from(random() % 1_000_000) / 1_000_000. * end)
        .collect();
    targets.sort_by(f64::total_cmp);
    let mut sequential =
        Playback::open_with(path, limits, Checkpoints::shared(f64::INFINITY, u64::MAX))
            .map_err(text)?;
    let mut expected = Vec::new();
    for &t in &targets {
        sequential.advance_to(SourceTime(t)).map_err(text)?;
        expected.push(digest(&mut sequential, SourceTime(t)));
    }

    // A seek before any checkpoint exists replays from the start
    let mut cold = Playback::open(path, limits).map_err(text)?;
    let started = Instant::now();
    cold.advance_to(SourceTime(end * 0.75)).map_err(text)?;
    let cold_ms = started.elapsed().as_secs_f64() * 1000.;

    let mut p = Playback::open_with(path, limits, store).map_err(text)?;
    let mut order: Vec<usize> = (0..SEEKS).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, random() as usize % (i + 1));
    }
    let mut times = Vec::new();
    let mut mismatches = 0;
    for i in order {
        let at = SourceTime(targets[i]);
        let started = Instant::now();
        p.advance_to(at).map_err(text)?;
        let state = digest(&mut p, at);
        times.push(started.elapsed().as_secs_f64() * 1000.);
        if state != expected[i] {
            mismatches += 1;
            println!("  differs at {:.3} s", at.0);
        }
    }
    times.sort_by(f64::total_cmp);
    println!(
        "  cold seek to 3/4 {cold_ms:.0} ms. {SEEKS} indexed seeks p50 {:.1}, p95 {:.1}, max {:.1} ms, {mismatches} differing from sequential playback",
        percentile(&times, 0.5),
        percentile(&times, 0.95),
        times[times.len() - 1],
    );
    Ok(mismatches == 0)
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let (mut spacing, mut budget) = (SPACING, BUDGET);
    loop {
        match args.peek().map(String::as_str) {
            Some("--spacing") => {
                args.next();
                spacing = args.next().and_then(|s| s.parse().ok()).unwrap_or(SPACING);
            }
            Some("--budget") => {
                args.next();
                budget = args
                    .next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .map_or(BUDGET, |m| m * 1024 * 1024);
            }
            _ => break,
        }
    }
    let mut ok = true;
    for path in args {
        match probe(Path::new(&path), spacing, budget) {
            Ok(same) => ok &= same,
            Err(e) => {
                println!("{path}: {e}");
                ok = false;
            }
        }
    }
    std::process::exit(if ok { 0 } else { 1 });
}
