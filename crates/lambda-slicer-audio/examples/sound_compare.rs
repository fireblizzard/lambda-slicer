//! Compares a recording of a demo's sound with the viewer's mix of it
//!
//! `sound_compare INSTALL DEMO RECORDING.wav START SECONDS`
//!
//! The demo's sound from `START` for `SECONDS` is mixed as `mix_probe`
//! mixes it, at the recording's rate. Both are taken to one channel and to
//! their loudness in 10 ms windows. The mix is slid along the recording to
//! where the two agree best, which says where `START` falls in the
//! recording. Then the sounds starting in each, a window much louder than
//! the 50 ms before it, are paired within 60 ms of each other, and the pairs
//! say how far apart the same sound starts and how much louder the
//! recording is. The recording may be the game's own, taken from the sound
//! card while the game plays the demo, or the viewer's sound tap
//!
//! A WAV of 16-bit or 32-bit float samples, any number of channels, is read

use std::path::Path;
use std::sync::Arc;

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_audio::mixer::{Listener, Output, follow, mix};
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

/// Seconds in each loudness window
const WINDOW: f64 = 0.01;
/// Windows a sound's start is compared with
const BEFORE: usize = 5;
/// How far apart, in seconds, the same sound's starts may be
const PAIRED: f64 = 0.06;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, demo, recording, start, seconds] = args.as_slice() else {
        eprintln!("usage: sound_compare INSTALL DEMO RECORDING.wav START SECONDS");
        std::process::exit(2);
    };
    let (start, seconds): (f64, f64) = (start.parse().unwrap(), seconds.parse().unwrap());
    let (rate, recorded) = read_wav(Path::new(recording)).expect("recording");
    let mixed = mix_demo(install, demo, rate, start, seconds);

    let per_window = (f64::from(rate) * WINDOW) as usize;
    let a = loudness(&recorded, per_window);
    let b = loudness(&mixed, per_window);
    let Some((lag, agreement)) = best_lag(&a, &b) else {
        eprintln!("the recording is shorter than the stretch mixed");
        std::process::exit(1);
    };
    println!(
        "{start} s of the demo falls {:.3} s into the recording, loudness agreeing {agreement:.3}",
        lag as f64 * WINDOW
    );

    let in_recording = starts(&a[lag..lag + b.len()]);
    let in_mix = starts(&b);
    let mut gaps = Vec::new();
    let mut ratios = Vec::new();
    for &m in &in_mix {
        let near = in_recording
            .iter()
            .copied()
            .min_by_key(|&r| (r as i64 - m as i64).abs());
        if let Some(r) = near
            && (r as f64 - m as f64).abs() * WINDOW <= PAIRED
        {
            gaps.push((r as f64 - m as f64) * WINDOW * 1000.);
            let level = |e: &[f32], i: usize| {
                e[i..(i + 10).min(e.len())]
                    .iter()
                    .copied()
                    .fold(0f32, f32::max)
            };
            let mixed_level = level(&b, m);
            if mixed_level > 0. {
                ratios.push(f64::from(level(&a[lag..], r) / mixed_level));
            }
        }
    }
    println!(
        "sounds starting: {} in the mix, {} in the recording, {} paired",
        in_mix.len(),
        in_recording.len(),
        gaps.len()
    );
    if !gaps.is_empty() {
        let (median, low, high) = spread(&mut gaps);
        println!(
            "the recording's start after the mix's: median {median:.0} ms, from {low:.0} to {high:.0} ms (5th to 95th percentile)"
        );
        let (median, low, high) = spread(&mut ratios);
        println!(
            "the recording's loudness over the mix's: median {median:.2}, from {low:.2} to {high:.2}"
        );
    }
}

/// The median and the 5th and 95th percentiles
fn spread(values: &mut [f64]) -> (f64, f64, f64) {
    values.sort_by(f64::total_cmp);
    let at = |q: f64| values[((values.len() - 1) as f64 * q).round() as usize];
    (at(0.5), at(0.05), at(0.95))
}

/// The demo's sound from `start` for `seconds`, as one channel
fn mix_demo(install: &str, demo: &str, rate: u32, start: f64, seconds: f64) -> Vec<f32> {
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
    let output = Output {
        rate,
        speed: 1.,
        volume: 1.,
    };
    let block = (rate / 100).max(1) as usize;
    let blocks = (seconds * f64::from(rate) / block as f64) as usize;
    let sound = |name: &_| library.sound(name);
    let mut out = Vec::with_capacity(blocks * block);
    for k in 0..blocks {
        let at = SourceTime(start + (k * block) as f64 / f64::from(rate));
        let to = SourceTime(at.0 + block as f64 / f64::from(rate));
        playback.advance_to(to).expect("playback");
        let listener = Listener::of(&playback);
        let channels = follow(
            playback.audio().channels(),
            playback.scene(),
            listener.entity,
        );
        let samples = mix(&channels, &sound, &listener, at, block, &output);
        out.extend(samples.iter().map(|[l, r]| (l + r) / 2.));
    }
    out
}

/// The loudness of each window, its root mean square
fn loudness(samples: &[f32], per_window: usize) -> Vec<f32> {
    samples
        .chunks(per_window.max(1))
        .map(|w| (w.iter().map(|v| v * v).sum::<f32>() / w.len() as f32).sqrt())
        .collect()
}

/// Where along `long` the shorter `short` agrees best, and how well, as the
/// correlation of their loudness on a log scale
fn best_lag(long: &[f32], short: &[f32]) -> Option<(usize, f64)> {
    let log = |e: &[f32]| -> Vec<f64> { e.iter().map(|&v| f64::from(v.max(1e-5)).ln()).collect() };
    let (long, short) = (log(long), log(short));
    let n = short.len();
    let slots = long.len().checked_sub(n)? + 1;
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let ms = mean(&short);
    let ss: f64 = short.iter().map(|v| (v - ms) * (v - ms)).sum();
    (0..slots)
        .map(|lag| {
            let part = &long[lag..lag + n];
            let ml = mean(part);
            let sl: f64 = part.iter().map(|v| (v - ml) * (v - ml)).sum();
            let cross: f64 = part
                .iter()
                .zip(&short)
                .map(|(a, b)| (a - ml) * (b - ms))
                .sum();
            (lag, cross / (sl * ss).sqrt().max(1e-12))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// The windows where a sound starts, four times louder than the loudest of
/// the windows before it and loud enough to hear
fn starts(envelope: &[f32]) -> Vec<usize> {
    let peak = envelope.iter().copied().fold(0f32, f32::max);
    let floor = peak * 0.02;
    (BEFORE..envelope.len())
        .filter(|&i| {
            let before = envelope[i - BEFORE..i].iter().copied().fold(0f32, f32::max);
            envelope[i] > floor && envelope[i] > before * 4. && envelope[i - 1] <= before * 4.
        })
        .collect()
}

/// A WAV file's rate and its samples as one channel, from 16-bit or 32-bit
/// float samples
fn read_wav(path: &Path) -> Result<(u32, Vec<f32>), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.get(0..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err("not a WAV file".into());
    }
    let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
    let u32_at =
        |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
    let (mut format, mut data) = (None, None);
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let size = u32_at(at + 4) as usize;
        let body = at + 8..(at + 8 + size).min(bytes.len());
        match &bytes[at..at + 4] {
            b"fmt " => {
                format = Some((
                    u16_at(at + 8),
                    u16_at(at + 10),
                    u32_at(at + 12),
                    u16_at(at + 22),
                ))
            }
            b"data" => data = Some(body),
            _ => {}
        }
        at += 8 + size + size % 2;
    }
    let (kind, channels, rate, bits) = format.ok_or("no format chunk")?;
    let data = &bytes[data.ok_or("no data chunk")?];
    let channels = usize::from(channels.max(1));
    let samples: Vec<f32> = match (kind, bits) {
        (1, 16) | (0xfffe, 16) => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| f32::from(i16::from_le_bytes(*b)) / 32768.)
            .collect(),
        (3, 32) | (0xfffe, 32) => data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect(),
        _ => return Err(format!("{bits}-bit samples of format {kind} are not read")),
    };
    let mono = samples
        .chunks_exact(channels)
        .map(|f| f.iter().sum::<f32>() / channels as f32)
        .collect();
    Ok((rate, mono))
}
