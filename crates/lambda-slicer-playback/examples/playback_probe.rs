//! Plays real demos through and checks a few moments both ways
//!
//! `playback_probe DEMO...`
//!
//! For each demo, the time to play every group sequentially and what the
//! last frame holds, then for moments at a quarter, half and three quarters
//! of the demo, whether reaching them directly and reaching them back from
//! the end give the same state

use std::path::Path;
use std::time::Instant;

use lambda_slicer_core::container::Limits;
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

fn digest(p: &mut Playback, at: SourceTime) -> String {
    let f = p.present(Stamp::default(), at, CameraMode::Recorder, false);
    format!(
        "{:?}",
        (
            f.position,
            &f.entities,
            &f.sounds,
            f.recorder,
            f.viewmodel,
            &f.stopped
        )
    )
}

fn probe(path: &Path) -> Result<bool, String> {
    let open = || Playback::open(path, Limits::default()).map_err(|m| m.text);

    let started = Instant::now();
    let mut p = open()?;
    p.advance_to(SourceTime(f64::MAX)).map_err(|m| m.text)?;
    let seconds = started.elapsed().as_secs_f64();
    let end = p.position().map_or(SourceTime::ZERO, |at| at.time);
    let last = p.present(Stamp::default(), end, CameraMode::Recorder, false);
    println!(
        "{}: {} groups, {} sessions, {:.1} s of playback in {seconds:.2} s, {} entities and {} sounds at the end, map {:?}, stopped {:?}",
        path.display(),
        last.position.map_or(0, |at| at.group + 1),
        last.position.map_or(0, |at| at.segment),
        end.0,
        last.entities.len(),
        last.sounds.len(),
        last.map,
        last.stopped.map(|m| m.text),
    );

    let mut same = true;
    for fraction in [0.25, 0.5, 0.75] {
        let at = SourceTime(end.0 * fraction);
        let mut direct = open()?;
        direct.advance_to(at).map_err(|m| m.text)?;
        let expected = digest(&mut direct, at);

        let mut back = open()?;
        back.advance_to(SourceTime(f64::MAX)).map_err(|m| m.text)?;
        back.advance_to(at).map_err(|m| m.text)?;
        let matches = digest(&mut back, at) == expected;
        println!(
            "  at {:.1} s: {} entities, {} sounds, direct and back from the end {}",
            at.0,
            direct.scene().entities.len(),
            direct.audio().channels().len(),
            if matches { "agree" } else { "DIFFER" },
        );
        same &= matches;
    }
    Ok(same)
}

fn main() {
    let mut ok = true;
    for path in std::env::args().skip(1) {
        match probe(Path::new(&path)) {
            Ok(same) => ok &= same,
            Err(e) => {
                println!("{path}: {e}");
                ok = false;
            }
        }
    }
    std::process::exit(if ok { 0 } else { 1 });
}
