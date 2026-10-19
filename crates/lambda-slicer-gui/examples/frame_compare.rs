//! Compares the game's frames of a clip with the viewer's
//! (docs/viewer-validation.md, "Reference captures")
//!
//! `frame_compare GAME_FRAMES VIEWER_FOLDER [--frames FIRST-LAST | --seconds S]
//! [--sheet SHEET.png]`
//!
//! `GAME_FRAMES` is the name `startmovie` wrote the game's frames under,
//! such as `.../ag/ls12d_kill`, a 24-bit BMP for every tenth of a second
//! numbered from 1. `VIEWER_FOLDER` holds the viewer's frames of the same
//! clip as `--viewer-capture` writes them, `<seconds>.png`, every 0.05 s.
//!
//! The demo's frames among the game's, which loading screens and the menu
//! surround, are the frames numbered `FIRST` to `LAST`, or `S` seconds from
//! the first frame that differs from the one before, the demo's first after
//! the loading screen. Without either they are guessed as the longest run of
//! frames that differ from the one before, still for 3 s at most, which a
//! longer pause in the demo would cut short. It says which frames it took
//! and how many of them found a viewer frame to pair with.
//!
//! Both are cut to the 4:3 scene in their middle, as the game draws 4:3
//! between black bars, and made into grey 160 by 120 thumbnails, with AG's
//! speedometer under the crosshair left out. The offset between the game's
//! clock and the clip's is the one within 2 s, in steps of 0.05 s, that
//! makes the pairs differ least. It prints that offset, how much the pairs
//! differ on average (the mean absolute difference of grey levels out of
//! 255), the same for frames a second apart, and the pairs that differ
//! most. With `--sheet` it writes every tenth pair side by side, the game's
//! on the left

use std::path::{Path, PathBuf};

use image::RgbImage;
use image::imageops::{FilterType, resize};

const THUMB_WIDTH: usize = 160;
const THUMB_HEIGHT: usize = 120;
/// The game's frames a second
const GAME_RATE: f64 = 10.;
/// How much a frame differs from the one before when the picture moved
const MOVED: f32 = 0.5;
/// Frames a guessed run may stay still for, 3 s
const STILL: usize = 30;
/// Frames of movement, and then of stillness, after which a guess takes
/// the game's frames to be past the demo, 10 s still
const PLAYED: usize = 20;
const DONE: usize = 100;
/// How far apart in seconds two frames may be to make a pair
const PAIRED: f64 = 0.03;
/// The pictures on the contact sheet
const SHEET_WIDTH: u32 = 320;
const SHEET_HEIGHT: u32 = 240;

/// A frame as compared, grey levels row by row
type Thumb = Vec<f32>;

/// A 24-bit uncompressed BMP, as `startmovie` writes them
fn read_bmp(path: &Path) -> Option<RgbImage> {
    let bytes = std::fs::read(path).ok()?;
    let field = |at: usize| -> Option<u32> {
        let le = bytes.get(at..at + 4)?;
        Some(u32::from_le_bytes([le[0], le[1], le[2], le[3]]))
    };
    if bytes.get(..2) != Some(b"BM") {
        return None;
    }

    let data = field(10)? as usize;
    let width = field(18)?;
    let height = field(22)? as i32;
    let bits = u16::from_le_bytes([*bytes.get(28)?, *bytes.get(29)?]);
    if bits != 24 || field(30)? != 0 {
        return None;
    }

    // Rows are padded to four bytes, and stored bottom up unless the
    // height is negative
    let rows = height.unsigned_abs();
    let stride = (width as usize * 3).div_ceil(4) * 4;
    let mut image = RgbImage::new(width, rows);
    for y in 0..rows {
        let stored = if height > 0 { rows - 1 - y } else { y };
        let start = data + stored as usize * stride;
        let row = bytes.get(start..start + width as usize * 3)?;
        let (pixels, _) = row.as_chunks::<3>();
        for (x, [b, g, r]) in pixels.iter().enumerate() {
            image.put_pixel(x as u32, y, image::Rgb([*r, *g, *b]));
        }
    }

    Some(image)
}

/// The 4:3 scene in the middle of `frame`
fn scene(frame: &RgbImage) -> RgbImage {
    let (width, height) = frame.dimensions();
    let scene_width = (height * 4 / 3).min(width);
    let scene_height = (width * 3 / 4).min(height);
    let x = (width - scene_width) / 2;
    let y = (height - scene_height) / 2;
    image::imageops::crop_imm(frame, x, y, scene_width, scene_height).to_image()
}

/// Whether a thumbnail's pixel is under AG's speedometer, which the game
/// draws below the crosshair whatever `hud_draw` says
fn under_speedometer(x: usize, y: usize) -> bool {
    let (fx, fy) = (
        x as f32 / THUMB_WIDTH as f32,
        y as f32 / THUMB_HEIGHT as f32,
    );
    (0.44..0.56).contains(&fx) && (0.47..0.56).contains(&fy)
}

/// The scene's grey levels averaged over boxes, as the thumbnail's pixels
fn thumbnail(scene: &RgbImage) -> Thumb {
    let (width, height) = (scene.width() as usize, scene.height() as usize);
    let mut sums = vec![0f32; THUMB_WIDTH * THUMB_HEIGHT];
    let mut counts = vec![0u32; THUMB_WIDTH * THUMB_HEIGHT];
    for (x, y, pixel) in scene.enumerate_pixels() {
        let [r, g, b] = pixel.0.map(f32::from);
        let tx = x as usize * THUMB_WIDTH / width;
        let ty = y as usize * THUMB_HEIGHT / height;
        let at = ty * THUMB_WIDTH + tx;
        sums[at] += 0.299 * r + 0.587 * g + 0.114 * b;
        counts[at] += 1;
    }

    sums.iter()
        .zip(&counts)
        .map(|(sum, count)| sum / (*count).max(1) as f32)
        .collect()
}

/// The mean absolute difference of two thumbnails' grey levels, the
/// speedometer left out
fn difference(a: &Thumb, b: &Thumb) -> f32 {
    let mut total = 0.;
    let mut counted = 0;
    for y in 0..THUMB_HEIGHT {
        for x in 0..THUMB_WIDTH {
            if under_speedometer(x, y) {
                continue;
            }
            let at = y * THUMB_WIDTH + x;
            total += (a[at] - b[at]).abs();
            counted += 1;
        }
    }

    total / counted as f32
}

/// A frame of one side, and its file to read again for the contact sheet
struct Frame {
    thumb: Thumb,
    path: PathBuf,
}

/// A game's BMP or a viewer's PNG
fn read_frame(path: &Path) -> Option<RgbImage> {
    if path.extension().is_some_and(|e| e == "bmp") {
        return read_bmp(path);
    }
    image::open(path).ok().map(|image| image.to_rgb8())
}

/// Which of the game's frames are the demo's
#[derive(Clone, Copy, Debug)]
enum Interval {
    /// The frames numbered from the first to the last, both included
    Frames(usize, usize),
    /// This many seconds from the first frame that differs from the one
    /// before
    Seconds(f64),
    /// The longest run of frames that differ from the one before
    Guessed,
}

/// The index of the first frame that differs from the one before
fn first_change(frames: &[Frame]) -> Option<usize> {
    (1..frames.len()).find(|&i| difference(&frames[i].thumb, &frames[i - 1].thumb) > MOVED)
}

/// The game's frames under `prefix`, in order, from `prefix00001.bmp` on,
/// read as far as `interval` needs
fn game_frames(prefix: &Path, interval: Interval) -> Vec<Frame> {
    let mut frames: Vec<Frame> = Vec::new();
    let mut moved = 0;
    let mut still = 0;
    let mut first = None;
    for number in 1.. {
        let mut name = prefix.as_os_str().to_owned();
        name.push(format!("{number:05}.bmp"));
        let path = PathBuf::from(name);
        let Some(image) = read_bmp(&path) else {
            break;
        };
        let thumb = thumbnail(&scene(&image));

        let changed = frames
            .last()
            .is_some_and(|last| difference(&thumb, &last.thumb) > MOVED);
        if changed {
            moved += 1;
            still = 0;
            first.get_or_insert(frames.len());
        } else if !frames.is_empty() {
            still += 1;
        }
        frames.push(Frame { thumb, path });

        let enough = match interval {
            Interval::Frames(_, last) => frames.len() >= last,
            Interval::Seconds(seconds) => {
                let wanted = (seconds * GAME_RATE) as usize;
                first.is_some_and(|f| frames.len() >= f + wanted)
            }
            Interval::Guessed => moved >= PLAYED && still >= DONE,
        };
        if enough {
            break;
        }
    }
    frames
}

/// The demo's frames among the game's, as a range of indices
fn demo_frames(frames: &[Frame], interval: Interval) -> std::ops::Range<usize> {
    match interval {
        Interval::Frames(first, last) => (first - 1).min(frames.len())..last.min(frames.len()),
        Interval::Seconds(seconds) => {
            let Some(first) = first_change(frames) else {
                return 0..0;
            };
            let end = first + (seconds * GAME_RATE) as usize;
            first..end.min(frames.len())
        }
        Interval::Guessed => longest_run(frames),
    }
}

/// The viewer's frames in `folder` with their times, in time order
fn viewer_frames(folder: &Path) -> Vec<(f64, Frame)> {
    let mut frames = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return frames;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "png") {
            continue;
        }
        let Some(time) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.parse::<f64>().ok())
        else {
            continue;
        };
        let Some(image) = read_frame(&path) else {
            continue;
        };
        let thumb = thumbnail(&scene(&image));
        frames.push((time, Frame { thumb, path }));
    }
    frames.sort_by(|a, b| a.0.total_cmp(&b.0));
    frames
}

/// The longest run of frames that differ from the one before, as a range
/// of indices. A run may stay still for [`STILL`] frames, as loading
/// screens and the menu stay still longer
fn longest_run(frames: &[Frame]) -> std::ops::Range<usize> {
    let mut best = 0..0;
    let mut run: Option<std::ops::Range<usize>> = None;
    for i in 1..frames.len() {
        let moving = difference(&frames[i].thumb, &frames[i - 1].thumb) > MOVED;
        if !moving {
            continue;
        }

        // A run begins with the first frame that differs from the one
        // before, and goes on through still frames for a while
        let current = match &run {
            Some(r) if i - r.end <= STILL => r.start..i + 1,
            _ => i..i + 1,
        };
        if current.len() > best.len() {
            best = current.clone();
        }
        run = Some(current);
    }
    best
}

/// The viewer's frame nearest `time`, when one is close enough to pair
fn nearest(viewer: &[(f64, Frame)], time: f64) -> Option<&Frame> {
    // The frames on either side of `time`
    let after = viewer.partition_point(|(t, _)| *t < time);
    let before = after.checked_sub(1).and_then(|i| viewer.get(i));
    let closer = match (before, viewer.get(after)) {
        (Some(before), Some(after)) if time - before.0 <= after.0 - time => before,
        (_, Some(after)) => after,
        (Some(before), None) => before,
        (None, None) => return None,
    };

    if (closer.0 - time).abs() > PAIRED {
        return None;
    }
    Some(&closer.1)
}

/// Each game frame's clip time with `offset`, its pair's difference, and
/// the difference from the viewer's frame a second later
fn pairs(game: &[Frame], viewer: &[(f64, Frame)], offset: f64) -> Vec<(f64, f32, Option<f32>)> {
    let mut out = Vec::new();
    for (i, g) in game.iter().enumerate() {
        let time = i as f64 / GAME_RATE + offset;
        let Some(v) = nearest(viewer, time) else {
            continue;
        };
        let apart = nearest(viewer, time + 1.).map(|later| difference(&g.thumb, &later.thumb));
        out.push((time, difference(&g.thumb, &v.thumb), apart));
    }
    out
}

fn mean(values: impl Iterator<Item = f32>) -> f32 {
    let mut sum = 0.;
    let mut count = 0;
    for value in values {
        sum += value;
        count += 1;
    }
    sum / count.max(1) as f32
}

/// Every tenth pair side by side, the game's frame on the left
fn write_sheet(path: &Path, game: &[Frame], viewer: &[(f64, Frame)], offset: f64) {
    let mut chosen = Vec::new();
    for (i, g) in game.iter().enumerate().step_by(10) {
        let time = i as f64 / GAME_RATE + offset;
        if let Some(v) = nearest(viewer, time) {
            chosen.push((g, v));
        }
    }

    let small = |frame: &Frame| {
        let image = read_frame(&frame.path).unwrap_or_default();
        resize(
            &scene(&image),
            SHEET_WIDTH,
            SHEET_HEIGHT,
            FilterType::Triangle,
        )
    };
    let rows = chosen.len().max(1) as u32;
    let mut sheet = RgbImage::new(SHEET_WIDTH * 2, SHEET_HEIGHT * rows);
    for (row, (g, v)) in chosen.iter().enumerate() {
        let y = i64::from(row as u32 * SHEET_HEIGHT);
        image::imageops::replace(&mut sheet, &small(g), 0, y);
        image::imageops::replace(&mut sheet, &small(v), i64::from(SHEET_WIDTH), y);
    }
    if let Err(e) = sheet.save(path) {
        eprintln!("{}: {e}", path.display());
    }
}

/// The command line's frames, `FIRST-LAST`
fn parse_frames(text: &str) -> Option<Interval> {
    let (first, last) = text.split_once('-')?;
    let (first, last) = (first.parse().ok()?, last.parse().ok()?);
    if first < 1 || last < first {
        return None;
    }
    Some(Interval::Frames(first, last))
}

/// The command line's seconds, more than none
fn parse_seconds(text: &str) -> Option<Interval> {
    let seconds: f64 = text.parse().ok()?;
    if seconds <= 0. {
        return None;
    }
    Some(Interval::Seconds(seconds))
}

fn main() {
    let usage = "usage: frame_compare GAME_FRAMES VIEWER_FOLDER [--frames FIRST-LAST | --seconds S] [--sheet SHEET.png]";
    let mut args = std::env::args().skip(1);
    let (Some(game_prefix), Some(viewer_folder)) = (args.next(), args.next()) else {
        eprintln!("{usage}");
        std::process::exit(2);
    };
    let mut interval = Interval::Guessed;
    let mut sheet = None;
    while let Some(flag) = args.next() {
        let value = args.next().unwrap_or_default();
        match flag.as_str() {
            "--frames" if let Some(given) = parse_frames(&value) => interval = given,
            "--seconds" if let Some(given) = parse_seconds(&value) => interval = given,
            "--sheet" if !value.is_empty() => sheet = Some(PathBuf::from(value)),
            _ => {
                eprintln!("{usage}");
                std::process::exit(2);
            }
        }
    }

    let all = game_frames(Path::new(&game_prefix), interval);
    let run = demo_frames(&all, interval);
    let game = &all[run.clone()];
    let viewer = viewer_frames(Path::new(&viewer_folder));
    if game.is_empty() || viewer.is_empty() {
        eprintln!(
            "{} of the game's frames taken as the demo, of {} read, and {} viewer frames",
            game.len(),
            all.len(),
            viewer.len()
        );
        std::process::exit(1);
    }
    let how = match interval {
        Interval::Frames(..) => "as given".to_owned(),
        Interval::Seconds(seconds) => format!("{seconds} s from the first change"),
        Interval::Guessed => "guessed from the movement".to_owned(),
    };
    println!(
        "game frames {} to {} of {} read, {how}, and {} viewer frames from {:.2} to {:.2} s",
        run.start + 1,
        run.end,
        all.len(),
        viewer.len(),
        viewer[0].0,
        viewer[viewer.len() - 1].0
    );

    // The offset that pairs at least half the game's frames and makes the
    // pairs differ least
    let mut best: Option<(f64, f32)> = None;
    for step in -40..=40 {
        let offset = f64::from(step) * 0.05;
        let found = pairs(game, &viewer, offset);
        if found.len() * 2 < game.len() {
            continue;
        }
        let score = mean(found.iter().map(|(_, d, _)| *d));
        if best.is_none_or(|(_, b)| score < b) {
            best = Some((offset, score));
        }
    }
    let Some((offset, score)) = best else {
        eprintln!("no offset within 2 s pairs half the game's frames");
        std::process::exit(1);
    };

    let found = pairs(game, &viewer, offset);
    let apart = mean(found.iter().filter_map(|(_, _, a)| *a));
    println!(
        "offset {offset:+.2} s, {} of {} game frames paired, differing by {score:.1} on average, against {apart:.1} for frames a second apart",
        found.len(),
        game.len()
    );
    let unpaired = game.len() - found.len();
    if unpaired > 0 {
        println!(
            "{unpaired} of the demo's frames found no viewer frame within {PAIRED} s, their clip times before or after the viewer's capture"
        );
    }
    let mut worst = found.clone();
    worst.sort_by(|a, b| b.1.total_cmp(&a.1));
    let listed: Vec<String> = worst
        .iter()
        .take(5)
        .map(|(t, d, _)| format!("{t:.2} s {d:.1}"))
        .collect();
    println!("differing most at {}", listed.join(", "));

    if let Some(path) = sheet {
        write_sheet(&path, game, &viewer, offset);
    }
}
