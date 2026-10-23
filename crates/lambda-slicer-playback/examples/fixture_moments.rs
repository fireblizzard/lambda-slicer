//! Finds moments in a demo for the viewer's fixture matrix
//!
//! `fixture_moments DEMO [STEP]`
//!
//! Playback goes through the demo `STEP` seconds at a time (0.25 unless
//! given) and notes when each case is under way, as listed here
//!
//! - a gauss charging, its spin sound (`ambience/pulsemachine.wav`) playing on a
//!   player, as maps also play it as an ambient
//! - the Egon's beams drawn
//! - a client event waiting for its delay
//! - a reload, a sound with `reload` or `clip` in its name starting
//! - a brush entity moving
//! - a sprite drawn in a render mode other than normal
//! - an ambient loop, a sound on a static channel
//! - a monster dying, a sound with `die` or `death` in its name starting on
//!   an entity that is not a player
//!
//! For each it prints how many steps had it and up to five times, at least
//! two seconds apart

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use lambda_slicer_core::container::Limits;
use lambda_slicer_playback::audio::SoundName;
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(demo) = args.first() else {
        eprintln!("usage: fixture_moments DEMO [STEP]");
        std::process::exit(2);
    };
    let step: f64 = args.get(1).map_or(0.25, |s| s.parse().unwrap());
    let mut p = Playback::open(Path::new(demo), Limits::default()).expect("demo");

    let mut found: BTreeMap<&str, (usize, Vec<f64>)> = BTreeMap::new();
    let mut note = |kind: &'static str, at: f64| {
        let (count, times) = found.entry(kind).or_default();
        *count += 1;
        if times.len() < 5 && times.last().is_none_or(|last| at - last >= 2.) {
            times.push(at);
        }
    };
    let mut brushes: HashMap<u16, [f32; 3]> = HashMap::new();
    let mut at = 0.;
    loop {
        let time = SourceTime(at);
        p.advance_to(time).expect("playback");
        let frame = p.present(Stamp::default(), time, CameraMode::Recorder, false);
        let starting = |c: &&lambda_slicer_playback::audio::Channel| {
            c.started.0 > at - step && c.started.0 <= at
        };
        let name = |s: &SoundName| match s {
            SoundName::File(f) => f.to_ascii_lowercase(),
            _ => String::new(),
        };
        let channels = p.audio().channels();
        let players: Vec<u16> = frame.players.iter().map(|p| p.entity).collect();
        // The spin on a player, as maps also play the sound as an ambient
        if channels.iter().any(|c| {
            name(&c.sound) == "ambience/pulsemachine.wav"
                && !c.is_static()
                && players.contains(&c.entity)
        }) {
            note("gauss charging", at);
        }
        if frame.beams.egon() {
            note("egon beams", at);
        }
        if !p.events().pending().is_empty() {
            note("delayed event waiting", at);
        }
        if channels.iter().filter(starting).any(|c| {
            let n = name(&c.sound);
            n.contains("reload") || n.contains("clip")
        }) {
            note("reload", at);
        }
        if channels.iter().any(|c| c.is_static()) {
            note("ambient loop", at);
        }
        if channels.iter().filter(starting).any(|c| {
            let n = name(&c.sound);
            (n.contains("die") || n.contains("death"))
                && c.entity > 0
                && !players.contains(&c.entity)
        }) {
            note("monster dying", at);
        }
        for e in frame.entities.iter() {
            let model = e.model.as_deref().unwrap_or_default();
            if model.starts_with('*') {
                let before = brushes.insert(e.id.number, e.origin);
                if before.is_some_and(|o| o != e.origin) {
                    note("brush entity moving", at);
                }
            }
            if !e.beam && model.ends_with(".spr") && (1..=5).contains(&e.render_mode) {
                note("translucent sprite", at);
            }
        }

        if frame.end {
            break;
        }
        at += step;
    }

    println!("{demo}: {at:.0} s");
    for (kind, (count, times)) in &found {
        let times: Vec<String> = times.iter().map(|t| format!("{t:.2}")).collect();
        println!("  {kind}: {count} steps, at {}", times.join(", "));
    }
}
