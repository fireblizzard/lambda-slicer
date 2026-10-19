//! Seeks through real demos with the game's files and checks that sound
//! and effects come back as sequential playback has them (implementation
//! plan §13.5, M10)
//!
//! `effects_seek INSTALL DEMO...`
//!
//! For each demo, checkpoints are built for all of it as the worker builds
//! them, then playback seeks to moments chosen at random, in random order.
//! At each the sound channels with their pitches, the queued events, the
//! animations' client events, the temporary entities, decals, beams,
//! trails and tracers are compared with what one sequential pass has at
//! the same moment. It says how long building the checkpoints took and
//! each seek's time at the median and 95th percentile

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_playback::GameFiles;
use lambda_slicer_playback::checkpoint::{BUDGET, Checkpoints, SPACING};
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

const SEEKS: usize = 100;

/// Everything sound and the effects hold at `at`, as text
fn digest(p: &mut Playback, at: SourceTime) -> String {
    let channels = format!("{:?}", p.audio().channels());
    let events = format!("{:?}", p.events());
    let flashes = format!("{:?}", p.animations().flashes());
    let f = p.present(Stamp::default(), at, CameraMode::Recorder, false);
    format!(
        "{channels} {events} {flashes} {:?}",
        (
            f.position,
            f.viewmodel,
            &f.temp_entities,
            &f.decals,
            f.decal_revision,
            &f.beams,
        )
    )
}

fn check(install: &Path, path: &Path) -> Result<bool, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let game = read_header(&mut file, len)
        .map_err(|e| e.to_string())?
        .game_dir();
    let search = SearchPath::new(
        install,
        &game,
        &SearchOptions::default(),
        AssetLimits::default(),
    )
    .map_err(|e| e.to_string())?;
    let files: GameFiles = Arc::new(Library::new(search, AssetLimits::default())).game_files();
    let limits = Limits::default();
    let open = |store| {
        let mut p = Playback::open_with(path, limits, store).map_err(|m| m.text)?;
        p.set_game_files(&files);
        Ok::<_, String>(p)
    };

    let store = Checkpoints::shared(SPACING, BUDGET);
    let mut indexer = open(store.clone())?;
    let indexing = Instant::now();
    while !indexer.index(1000) {}
    let indexed = indexing.elapsed().as_secs_f64();
    let end = indexer.position().map_or(0., |p| p.time.0);
    drop(indexer);

    let mut seed = 0x2545_f491_u32;
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
    let mut sequential = open(Checkpoints::shared(f64::INFINITY, u64::MAX))?;
    let mut expected = Vec::new();
    for &t in &targets {
        sequential.advance_to(SourceTime(t)).map_err(|m| m.text)?;
        expected.push(digest(&mut sequential, SourceTime(t)));
    }

    let mut p = open(store.clone())?;
    let mut order: Vec<usize> = (0..SEEKS).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, random() as usize % (i + 1));
    }
    let mut differing = Vec::new();
    let mut times = Vec::new();
    for i in order {
        let at = SourceTime(targets[i]);
        let seeking = Instant::now();
        p.advance_to(at).map_err(|m| m.text)?;
        times.push(seeking.elapsed().as_secs_f64() * 1000.);
        if digest(&mut p, at) != expected[i] {
            differing.push(at.0);
        }
    }
    differing.sort_by(f64::total_cmp);

    times.sort_by(f64::total_cmp);
    let percentile = |q: f64| times[((times.len() - 1) as f64 * q).round() as usize];
    let spacing = store.lock().map_or(0., |s| s.spacing());
    println!(
        "{}: {end:.0} s, checkpoints {spacing:.0} s apart built in {indexed:.1} s, {SEEKS} seeks p50 {:.1}, p95 {:.1}, max {:.1} ms, {} differing from sequential playback{}",
        path.display(),
        percentile(0.5),
        percentile(0.95),
        times[times.len() - 1],
        differing.len(),
        if differing.is_empty() {
            String::new()
        } else {
            format!(", at {differing:.3?}")
        }
    );
    Ok(differing.is_empty())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, demos @ ..] = args.as_slice() else {
        eprintln!("usage: effects_seek INSTALL DEMO...");
        std::process::exit(2);
    };
    let mut all = true;
    for demo in demos {
        match check(Path::new(install), Path::new(demo)) {
            Ok(same) => all &= same,
            Err(e) => {
                eprintln!("{demo}: {e}");
                all = false;
            }
        }
    }
    std::process::exit(if all { 0 } else { 1 });
}
