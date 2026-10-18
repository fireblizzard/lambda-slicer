//! Mixes a stretch of a demo's sound to a WAV file, heard from the
//! recorder's view
//!
//! `mix_probe INSTALL DEMO START SECONDS OUT.wav`
//!
//! Playback moves in blocks of 10 ms, the sounds each block holds mixed at
//! 48 kHz. The level, the channels heard and the sounds that could not be
//! read are reported

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_audio::mixer::{Listener, Output, mix};
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_playback::audio::SoundName;
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

const RATE: u32 = 48_000;
const BLOCK: usize = 480;

/// The game's right for these angles
fn right(angles: [f32; 3]) -> [f32; 3] {
    let [p, y, r] = angles.map(f32::to_radians);
    let (sp, cp) = p.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sr, cr) = r.sin_cos();
    [-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp]
}

fn write_wav(path: &Path, samples: &[[f32; 2]]) -> std::io::Result<()> {
    let data: Vec<u8> = samples
        .iter()
        .flat_map(|s| s.map(|v| (v * 32767.) as i16))
        .flat_map(i16::to_le_bytes)
        .collect();
    let mut out = b"RIFF".to_vec();
    out.extend((36 + data.len() as u32).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(RATE.to_le_bytes());
    out.extend((RATE * 4).to_le_bytes());
    out.extend(4u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    std::fs::write(path, out)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, demo, start, seconds, out, ..] = args.as_slice() else {
        eprintln!("usage: mix_probe INSTALL DEMO START SECONDS OUT.wav");
        std::process::exit(2);
    };
    let (start, seconds): (f64, f64) = (start.parse().unwrap(), seconds.parse().unwrap());

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
    let lengths = library.lengths();
    playback.set_sound_lengths(move |name| lengths(name));

    // The master volume, below full scale as the game's usually is
    let output = Output {
        rate: RATE,
        speed: 1.,
        volume: 0.7,
    };
    let started = std::time::Instant::now();
    let mut samples = Vec::new();
    let mut heard = BTreeSet::new();
    let mut missing = BTreeSet::new();
    let blocks = (seconds * f64::from(RATE) / BLOCK as f64) as usize;
    for b in 0..blocks {
        let at = SourceTime(start + (b * BLOCK) as f64 / f64::from(RATE));
        playback.advance_to(at).expect("playback");
        let frame = playback.present(Stamp::default(), at, CameraMode::Recorder, true);
        let listener = Listener {
            origin: frame.recorder.origin,
            right: right(frame.recorder.angles),
            entity: frame.recorder.view_entity.unwrap_or(frame.local_entity),
        };
        for c in &frame.sounds {
            match (&c.sound, library.sound(&c.sound)) {
                (SoundName::File(name), Some(_)) => heard.insert(name.to_string()),
                (other, None) => missing.insert(format!("{other:?}")),
                _ => false,
            };
        }
        let sound = |name: &SoundName| library.sound(name);
        samples.extend(mix(&frame.sounds, &sound, &listener, at, BLOCK, &output));
    }

    let peak = samples.iter().flatten().fold(0f32, |m, v| m.max(v.abs()));
    let rms = (samples.iter().flatten().map(|v| v * v).sum::<f32>()
        / (samples.len() * 2).max(1) as f32)
        .sqrt();
    let clipped = samples.iter().flatten().filter(|v| v.abs() >= 1.).count();
    write_wav(Path::new(out), &samples).expect("write");
    println!(
        "{seconds} s from {start} s mixed in {:?}: peak {peak:.3}, rms {rms:.4}, {clipped} clipped samples",
        started.elapsed()
    );
    println!(
        "{} sounds heard: {:?}",
        heard.len(),
        heard.iter().take(12).collect::<Vec<_>>()
    );
    println!(
        "{} without samples: {:?}",
        missing.len(),
        missing.iter().take(8).collect::<Vec<_>>()
    );
}
