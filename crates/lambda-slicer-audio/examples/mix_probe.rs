//! Mixes a stretch of a demo's sound to a WAV file, heard from the
//! recorder's view
//!
//! `mix_probe INSTALL DEMO START SECONDS OUT.wav [RATE]`
//!
//! Playback moves in blocks of 10 ms, the sounds each block holds mixed at
//! 48 kHz or at `RATE`. The level, the channels heard, the muzzle flashes,
//! the temporary entities and decals made and the sounds that could not be
//! read are reported

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_audio::mixer::{Listener, Output, follow, mix, write_wav};
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_playback::audio::SoundName;
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, demo, start, seconds, out, rest @ ..] = args.as_slice() else {
        eprintln!("usage: mix_probe INSTALL DEMO START SECONDS OUT.wav [RATE]");
        std::process::exit(2);
    };
    let (start, seconds): (f64, f64) = (start.parse().unwrap(), seconds.parse().unwrap());
    let rate: u32 = rest.first().map_or(48_000, |r| r.parse().unwrap());
    let block = (rate / 100) as usize;

    let mut file = std::fs::File::open(demo).expect("demo");
    let len = file.metadata().expect("demo").len();
    let game = read_header(&mut file, len).expect("header").game_dir();
    let search = SearchPath::new(
        install.as_ref(),
        &game,
        &SearchOptions::default(),
        AssetLimits::default(),
    )
    .expect("search path");
    let library = Arc::new(Library::new(search, AssetLimits::default()));

    let mut playback = Playback::open(Path::new(demo), Limits::default()).expect("demo");
    playback.set_game_files(&library.game_files());

    // The master volume, below full scale as the game's usually is
    let output = Output {
        rate,
        speed: 1.,
        volume: 0.7,
    };
    let started = std::time::Instant::now();
    let mut samples = Vec::new();
    let mut heard = BTreeSet::new();
    let mut missing = BTreeSet::new();
    let mut flashes = BTreeSet::new();
    let mut temp_seen = std::collections::HashSet::new();
    let mut temps: std::collections::BTreeMap<String, u32> = Default::default();
    let mut decal_seen = std::collections::HashSet::new();
    let mut decals: std::collections::BTreeMap<u16, u32> = Default::default();
    let blocks = (seconds * f64::from(rate) / block as f64) as usize;
    for b in 0..blocks {
        let at = SourceTime(start + (b * block) as f64 / f64::from(rate));
        // As the device does, playback moves to the end of the block so a
        // sound starting within it is heard from its start
        let to = SourceTime(at.0 + block as f64 / f64::from(rate));
        playback.advance_to(to).expect("playback");
        for f in playback.animations().flashes() {
            flashes.insert((f.time.0.to_bits(), format!("{:?}", f.holder)));
        }
        for d in playback.decals().list().iter() {
            if decal_seen.insert(d.id) {
                *decals.entry(d.texture).or_insert(0) += 1;
            }
        }
        for e in playback.tempents().entities() {
            if temp_seen.insert(e.id) {
                let model = e.model.as_deref().unwrap_or("none, a spark shower");
                *temps.entry(model.to_owned()).or_insert(0) += 1;
            }
        }
        let channels = playback.audio().channels();
        for c in channels {
            match (&c.sound, library.sound(&c.sound)) {
                (SoundName::File(name), Some(_)) => heard.insert(name.to_string()),
                (other, None) => missing.insert(format!("{other:?}")),
                _ => false,
            };
        }
        let sound = |name: &SoundName| library.sound(name);
        let listener = Listener::of(&playback);
        let channels = follow(channels, playback.scene(), listener.entity);
        samples.extend(mix(&channels, &sound, &listener, at, block, &output));
    }

    let peak = samples.iter().flatten().fold(0f32, |m, v| m.max(v.abs()));
    let rms = (samples.iter().flatten().map(|v| v * v).sum::<f32>()
        / (samples.len() * 2).max(1) as f32)
        .sqrt();
    let clipped = samples.iter().flatten().filter(|v| v.abs() >= 1.).count();
    write_wav(Path::new(out), rate, &samples).expect("write");
    println!(
        "{seconds} s from {start} s mixed in {:?}: peak {peak:.3}, rms {rms:.4}, {clipped} clipped samples",
        started.elapsed()
    );
    println!(
        "{} sounds heard: {:?}",
        heard.len(),
        heard.iter().collect::<Vec<_>>()
    );
    let viewmodel = flashes.iter().filter(|(_, h)| h == "Viewmodel").count();
    println!(
        "{} muzzle flashes, {viewmodel} of them the viewmodel's",
        flashes.len()
    );
    println!("temporary entities made: {temps:?}");
    println!("decals made, by their place in decals.wad: {decals:?}");
    println!(
        "{} without samples: {:?}",
        missing.len(),
        missing.iter().take(8).collect::<Vec<_>>()
    );
}
