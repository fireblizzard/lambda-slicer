//! Reaches each moment of the viewer's fixture matrix every way playback
//! can and checks that they agree (implementation plan §13.9, and
//! docs/viewer-validation.md, "Equivalence")
//!
//! `fixture_equivalence INSTALL MANIFEST [CASE...]`
//!
//! Each moment of the manifest's fixtures, or of the cases named, is
//! reached
//!
//! - by sequential playback with no checkpoints, the reference
//! - by playing the three seconds before it in display-sized steps
//! - by a direct seek, from checkpoints covering the whole demo
//! - by a seek back from 30 seconds later
//! - by seeks from a checkpoint kept a second before it, and from one kept
//!   while a client event waits for its delay, when one waits in the two
//!   seconds before it
//! - by three drags of the playhead through the worker, as the viewer
//!   drags, each ending on the moment
//! - by a clip round the moment looped through the controller three
//!   times, at three rates and with a pause
//!
//! Each way's whole playback state, the frame it presents and half a
//! second of sound mixed from there are compared with the reference's. A
//! drag gives only the worker's frame, and the clip's starts give no
//! sound. Then a slice round the moment is exported and the same moment in
//! it is checked the same way, against sequential playback of the slice

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_audio::mixer::{Listener, Output, follow, mix};
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_core::fixtures::load_manifest;
use lambda_slicer_core::slice::restart::run_restart;
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_playback::checkpoint::{BUDGET, Checkpoints, SPACING, Shared};
use lambda_slicer_playback::controller::{
    CameraMode, Command, Controller, LoopRange, Reply, Request, Stamp, Target,
};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::{Rate, SourceTime};
use lambda_slicer_playback::worker::Worker;
use lambda_slicer_playback::{GameFiles, PresentedFrame};

/// The mix's sample rate, and the samples mixed at a time
const RATE: u32 = 48_000;
const BLOCK: usize = 480;
/// Seconds of sound compared
const LISTEN: f64 = 0.5;
/// A display frame
const FRAME: f64 = 1. / 60.;

/// A demo and the game's files it plays with
struct Demo {
    path: PathBuf,
    files: GameFiles,
    library: Arc<Library>,
}

impl Demo {
    fn open(install: &Path, path: &Path) -> Result<Self, String> {
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
        let library = Arc::new(Library::new(search, AssetLimits::default()));
        Ok(Self {
            path: path.to_owned(),
            files: library.game_files(),
            library,
        })
    }

    /// The same game's files for another demo, a slice of this one
    fn other(&self, path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            files: self.files.clone(),
            library: self.library.clone(),
        }
    }

    fn playback(&self, store: Shared) -> Result<Playback, String> {
        let mut p =
            Playback::open_with(&self.path, Limits::default(), store).map_err(|m| m.text)?;
        p.set_game_files(&self.files);
        Ok(p)
    }

    /// Playback with no checkpoints, the reference
    fn sequential(&self) -> Result<Playback, String> {
        self.playback(Checkpoints::shared(f64::INFINITY, u64::MAX))
    }

    /// Checkpoints for the whole demo, as the worker builds them, and
    /// where it ends. A demo that stops decoding before its end is refused,
    /// as every way of reaching a moment would agree on the same broken
    /// state
    fn index(&self) -> Result<(Shared, f64), String> {
        let store = Checkpoints::shared(SPACING, BUDGET);
        let mut indexer = self.playback(store.clone())?;
        while !indexer.index(1000) {}

        let end = indexer.position().map_or(0., |p| p.time.0);
        if let Some(stopped) = indexer.stopped() {
            return Err(format!("playback stopped at {end:.3} s, {}", stopped.text));
        }
        Ok((store, end))
    }

    /// The reference at `at`, sound included
    fn reference(&self, at: SourceTime) -> Result<Measured, String> {
        let mut p = self.sequential()?;
        p.advance_to(at).map_err(|m| m.text)?;
        let (state, frame) = observe(&mut p, at);
        let sound = self.listen(&mut p, at)?;
        Ok(Measured {
            state,
            frame,
            sound,
        })
    }

    /// `LISTEN` seconds of sound from `at`, as `mix_probe` mixes it,
    /// playing on as it goes
    fn listen(&self, p: &mut Playback, at: SourceTime) -> Result<Vec<[f32; 2]>, String> {
        let output = Output {
            rate: RATE,
            speed: 1.,
            volume: 1.,
        };
        let sound = |name: &_| self.library.sound(name);
        let blocks = (LISTEN * f64::from(RATE)) as usize / BLOCK;
        let mut out = Vec::with_capacity(blocks * BLOCK);
        for k in 0..blocks {
            let from = SourceTime(at.0 + (k * BLOCK) as f64 / f64::from(RATE));
            let to = SourceTime(from.0 + BLOCK as f64 / f64::from(RATE));
            p.advance_to(to).map_err(|m| m.text)?;
            let channels = follow(p.audio().channels(), p.scene());
            let listener = Listener::of(p);
            out.extend(mix(&channels, &sound, &listener, from, BLOCK, &output));
        }
        Ok(out)
    }
}

/// What one way of reaching a moment holds there
struct Measured {
    state: String,
    frame: String,
    sound: Vec<[f32; 2]>,
}

/// The frame presented at `at`, without how far checkpoints reach, which
/// differs between stores
fn present(p: &mut Playback, stamp: Stamp, at: SourceTime) -> PresentedFrame {
    let mut frame = p.present(stamp, at, CameraMode::Recorder, false);
    frame.indexed = (SourceTime::ZERO, false);
    frame
}

/// The whole playback state and the frame presented at `at`, as text
fn observe(p: &mut Playback, at: SourceTime) -> (String, String) {
    let state = p.fingerprint();
    let frame = format!("{:?}", present(p, Stamp::default(), at));
    (state, frame)
}

/// Where two texts first differ, with what comes just before, which names
/// the field
fn difference(a: &str, b: &str) -> Option<String> {
    // The first byte that differs, or where the shorter text ends
    let at = match a.bytes().zip(b.bytes()).position(|(x, y)| x != y) {
        Some(at) => at,
        None if a.len() != b.len() => a.len().min(b.len()),
        None => return None,
    };

    // A byte position moved back to the start of its character
    let boundary = |s: &str, i: usize| {
        let mut i = i.min(s.len());
        while !s.is_char_boundary(i) {
            i -= 1;
        }
        i
    };
    let piece = |s: &str| {
        let from = boundary(s, at.saturating_sub(160));
        let split = boundary(s, at);
        let to = boundary(s, at + 80);
        format!("…{}⟨{}⟩", &s[from..split], &s[split..to])
    };

    Some(format!(
        "\n      got  {}\n      want {}",
        piece(a),
        piece(b)
    ))
}

/// What differs between `got` and `want`, nothing when they agree
fn compare(got: &Measured, want: &Measured) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(d) = difference(&got.state, &want.state) {
        out.push(format!("state{d}"));
    }
    if let Some(d) = difference(&got.frame, &want.frame) {
        out.push(format!("frame{d}"));
    }
    let differing: Vec<usize> = (0..got.sound.len().max(want.sound.len()))
        .filter(|&i| got.sound.get(i) != want.sound.get(i))
        .collect();
    if let Some(first) = differing.first() {
        out.push(format!(
            "sound, {} of {} samples, from {:.4} s on",
            differing.len(),
            want.sound.len(),
            *first as f64 / f64::from(RATE)
        ));
    }
    out
}

/// The outcome of one way, printed
fn report(way: &str, differences: &[String]) -> bool {
    if differences.is_empty() {
        println!("    {way}: same");
    } else {
        println!("    {way}: differs in {}", differences.join(", "));
    }
    differences.is_empty()
}

/// Play the three seconds before `at` in display frames
fn played(demo: &Demo, at: SourceTime) -> Result<Measured, String> {
    let mut p = demo.sequential()?;
    let mut t = (at.0 - 3.).max(0.);
    while t < at.0 {
        p.advance_to(SourceTime(t)).map_err(|m| m.text)?;
        t += FRAME;
    }
    p.advance_to(at).map_err(|m| m.text)?;
    let (state, frame) = observe(&mut p, at);
    let sound = demo.listen(&mut p, at)?;
    Ok(Measured {
        state,
        frame,
        sound,
    })
}

/// Seek to `at`, from `before` first when given
fn sought(
    demo: &Demo,
    store: &Shared,
    before: Option<SourceTime>,
    at: SourceTime,
) -> Result<Measured, String> {
    let mut p = demo.playback(store.clone())?;
    if let Some(before) = before {
        p.advance_to(before).map_err(|m| m.text)?;
    }
    p.advance_to(at).map_err(|m| m.text)?;
    let (state, frame) = observe(&mut p, at);
    let sound = demo.listen(&mut p, at)?;
    Ok(Measured {
        state,
        frame,
        sound,
    })
}

/// Seek to `at` from a checkpoint kept after `group`, on a store with no
/// other checkpoint after the demo's first group
fn from_checkpoint(demo: &Demo, group: u64, at: SourceTime) -> Result<Measured, String> {
    let store = Checkpoints::shared(f64::INFINITY, u64::MAX);
    let mut keeper = demo.playback(store.clone())?;
    if !keeper.keep_checkpoint_at(group).map_err(|m| m.text)? {
        return Err(format!("no group {group}"));
    }
    sought(demo, &store, None, at)
}

/// The groups to keep checkpoints after, the first a second or less
/// before `at`, and the first in the two seconds before it where a client
/// event waits for its delay, when there is one
fn restore_points(demo: &Demo, at: SourceTime) -> Result<(u64, Option<u64>), String> {
    let mut p = demo.sequential()?;
    p.advance_to(SourceTime(at.0 - 2.)).map_err(|m| m.text)?;
    let mut g = p.position().map_or(0, |p| p.group);
    let (mut second, mut waiting) = (None, None);
    while second.is_none() || waiting.is_none() {
        p.seek_group(g + 1).map_err(|m| m.text)?;
        let Some(position) = p.position().filter(|q| q.group == g + 1 && q.time < at) else {
            break;
        };
        g = position.group;
        if second.is_none() && position.time.0 >= at.0 - 1. {
            second = Some(g);
        }
        if waiting.is_none() && !p.events().pending().is_empty() {
            waiting = Some(g);
        }
    }
    Ok((second.unwrap_or(g), waiting))
}

/// Drag the playhead through the worker three times, each from 20 seconds
/// after `at` to 20 before and back to it, a frame apart, and compare each
/// drag's last frame with sequential playback's
fn dragged(demo: &Demo, at: SourceTime, end: f64) -> Result<Vec<String>, String> {
    let worker = Worker::spawn_with(Limits::default(), Some(demo.files.clone()), || {});
    let mut c = Controller::default();
    let start = Instant::now();
    let wall = |start: Instant| start.elapsed().as_secs_f64();
    let load = c
        .command(Command::Load(demo.path.clone()), wall(start))
        .ok_or("no load")?;
    worker.send(load);
    let reply = worker.wait().ok_or("no reply to the load")?;
    c.accept(&reply, wall(start));

    let mut expected = demo.sequential()?;
    expected.advance_to(at).map_err(|m| m.text)?;
    let mut out = Vec::new();
    for drag in 0..3 {
        let path = (0..=60).map(|i| {
            let swing = if i <= 30 {
                20. - 40. * f64::from(i) / 30.
            } else {
                -20. + 20. * f64::from(i - 30) / 30.
            };
            SourceTime((at.0 + swing).clamp(0., end))
        });
        // The reply to the newest request, which may come while dragging
        let mut newest = None;
        let take = |c: &mut Controller, newest: &mut Option<Reply>, reply: Reply| {
            c.accept(&reply, wall(start));
            if reply.stamp() == c.stamp() {
                *newest = Some(reply);
            }
        };
        for t in path.chain([at]) {
            if let Some(request) = c.command(Command::Scrub(t), wall(start)) {
                worker.send(request);
            }
            std::thread::sleep(Duration::from_secs_f64(FRAME));
            for reply in worker.replies() {
                take(&mut c, &mut newest, reply);
            }
        }
        c.command(Command::EndScrub { cancel: false }, wall(start));
        while newest.as_ref().is_none_or(|r| r.stamp() != c.stamp()) {
            let reply = worker.wait().ok_or("the worker stopped")?;
            take(&mut c, &mut newest, reply);
        }
        let frame = match newest.take() {
            Some(Reply::Frame(frame)) => frame,
            Some(Reply::Failed { message, .. }) => return Err(message.text),
            None => unreachable!(),
        };
        let mut frame = *frame;
        frame.indexed = (SourceTime::ZERO, false);
        let want = present(&mut expected, frame.stamp, at);
        if let Some(d) = difference(&format!("{frame:?}"), &format!("{want:?}")) {
            out.push(format!("drag {} frame{d}", drag + 1));
        }
    }
    Ok(out)
}

/// Loop a clip from two seconds before `at` to one after through the
/// controller, a display frame at a time at normal speed, then double,
/// then half with a pause, comparing the state at each start and at `at`
/// in the third round
fn looped(demo: &Demo, store: &Shared, at: SourceTime) -> Result<Vec<String>, String> {
    let range = LoopRange {
        start: SourceTime((at.0 - 2.).max(0.)),
        end: SourceTime(at.0 + 1.),
    };
    let mut p = demo.playback(store.clone())?;
    let apply = |p: &mut Playback, request: &Request| match request.target {
        Target::Time(t) => p.advance_to(t).map_err(|m| m.text),
        Target::Group(g) => p.seek_group(g).map_err(|m| m.text),
    };
    let mut c = Controller::default();
    c.command(Command::Load(demo.path.clone()), 0.);
    c.command(Command::SetLoop(Some(range)), 0.);
    let seek = c
        .command(Command::SeekTo(range.start), 0.)
        .ok_or("no seek")?;
    apply(&mut p, &seek)?;
    if let Some(request) = c.command(Command::Play, 0.) {
        apply(&mut p, &request)?;
    }

    let mut out = Vec::new();
    let mut round = 1;
    let mut revision = c.stamp().revision;
    let mut paused = false;
    let mut wall = 0.;
    // Long enough for three rounds at the slowest rate
    while wall < 60. {
        wall += FRAME;
        if round == 3 && !paused && c.time(wall) > SourceTime(range.start.0 + 0.5) {
            c.command(Command::Pause, wall);
            wall += 0.5;
            paused = true;
            if let Some(request) = c.command(Command::Play, wall) {
                apply(&mut p, &request)?;
            }
        }
        let Some(request) = c.tick(wall) else {
            continue;
        };
        apply(&mut p, &request)?;
        let Target::Time(t) = request.target else {
            continue;
        };
        if request.stamp.revision != revision {
            revision = request.stamp.revision;
            round += 1;
            let rate = [Rate::Normal, Rate::Double, Rate::Half][(round - 1).min(2)];
            c.command(Command::SetRate(rate), wall);
            let (state, frame) = observe(&mut p, t);
            let want = demo.reference(t)?;
            for (what, got, want) in [("state", state, want.state), ("frame", frame, want.frame)] {
                if let Some(d) = difference(&got, &want) {
                    out.push(format!("round {round} start {what}{d}"));
                }
            }
        }
        if round == 3 && t >= at {
            let (state, frame) = observe(&mut p, t);
            let sound = demo.listen(&mut p, t)?;
            let got = Measured {
                state,
                frame,
                sound,
            };
            for d in compare(&got, &demo.reference(t)?) {
                out.push(format!("round 3 at {:.3} s {d}", t.0));
            }
            return Ok(out);
        }
    }
    Err("the clip never reached the moment in its third round".into())
}

/// Reach `at` every way and say which differ. True when none does
fn check(demo: &Demo, at: SourceTime) -> Result<bool, String> {
    let begun = Instant::now();
    let (store, end) = demo.index()?;
    let indexed = begun.elapsed().as_secs_f64();
    if at.0 > end {
        return Err(format!("the moment is past the demo's end at {end:.3} s"));
    }

    let want = demo.reference(at)?;
    let mut same = true;
    same &= report("played in frames", &compare(&played(demo, at)?, &want));
    same &= report("seek", &compare(&sought(demo, &store, None, at)?, &want));
    let later = SourceTime((at.0 + 30.).min(end));
    let back = sought(demo, &store, Some(later), at)?;
    same &= report("seek back", &compare(&back, &want));
    let (second, waiting) = restore_points(demo, at)?;
    let kept = from_checkpoint(demo, second, at)?;
    same &= report("seek from a second before", &compare(&kept, &want));
    match waiting {
        Some(group) => {
            let kept = from_checkpoint(demo, group, at)?;
            same &= report("seek from an event waiting", &compare(&kept, &want));
        }
        None => println!("    seek from an event waiting: none waits in the two seconds before"),
    }
    same &= report("drags", &dragged(demo, at, end)?);
    same &= report("clip loop", &looped(demo, &store, at)?);
    println!(
        "    ({end:.0} s indexed in {indexed:.1} s, all ways in {:.1} s)",
        begun.elapsed().as_secs_f64()
    );
    Ok(same)
}

/// When the session `at` falls in began, its first group's time
fn session_start(demo: &Demo, at: SourceTime) -> Result<f64, String> {
    let mut p = demo.sequential()?;
    let mut start = 0.;
    let mut segment = None;
    let mut g = 0;
    loop {
        p.seek_group(g + 1).map_err(|m| m.text)?;
        let Some(position) = p.position().filter(|q| q.group == g + 1 && q.time <= at) else {
            return Ok(start);
        };
        g = position.group;
        if segment != Some(position.segment) {
            segment = Some(position.segment);
            start = position.time.0;
        }
    }
}

/// Export a slice round `at` into `folder`, to 4 seconds after it and
/// from 8 seconds before it, or later in its session when the export
/// refuses that start, and where `at` falls in it, or why every start was
/// refused
fn slice(
    demo: &Demo,
    at: f64,
    folder: &Path,
    name: &str,
) -> Result<Result<(Demo, f64), String>, String> {
    let micros = |t: f64| DemoTime::from_micros((t * 1e6).round() as i64);
    let session = session_start(demo, SourceTime(at))?;
    let mut refused = Vec::new();
    for (i, before) in [8., 6., 4., 2.].into_iter().enumerate() {
        let start = (at - before).max(session + 0.1);
        let output = folder.join(format!("{name}-{i}.dem"));
        match run_restart(
            &demo.path,
            &output,
            micros(start),
            Some(micros(at + 4.)),
            Limits::default(),
        ) {
            Ok(report) => {
                // The slice's times start at its first group, give or take
                // a frame
                let first: DemoTime = report
                    .effective_start
                    .parse()
                    .map_err(|_| format!("effective start {}", report.effective_start))?;
                return Ok(Ok((demo.other(&output), at - first.secs_f64())));
            }
            Err(e) => refused.push(format!("from {start:.1} s, {e}")),
        }
    }
    Ok(Err(refused.join("; ")))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, manifest, cases @ ..] = args.as_slice() else {
        eprintln!("usage: fixture_equivalence INSTALL MANIFEST [CASE...]");
        std::process::exit(2);
    };
    let manifest = load_manifest(Path::new(manifest)).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    // A case no fixture has, misspelt perhaps, would check nothing and pass
    let known: Vec<&String> = manifest
        .fixtures
        .iter()
        .flat_map(|f| f.moments.iter().map(|m| &m.case))
        .collect();
    let unknown: Vec<&String> = cases.iter().filter(|c| !known.contains(c)).collect();
    if !unknown.is_empty() {
        eprintln!("no fixture has the cases {unknown:?}");
        std::process::exit(2);
    }
    if known.is_empty() {
        eprintln!("no fixture has moments to check");
        std::process::exit(2);
    }

    let folder =
        std::env::temp_dir().join(format!("lambda-slicer-equivalence-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("scratch folder");

    let mut all = true;
    for fixture in &manifest.fixtures {
        let moments: Vec<_> = fixture
            .moments
            .iter()
            .filter(|m| cases.is_empty() || cases.contains(&m.case))
            .collect();
        if moments.is_empty() {
            continue;
        }
        let demo = match Demo::open(Path::new(install), &fixture.path) {
            Ok(demo) => demo,
            Err(e) => {
                eprintln!("{}: {e}", fixture.path.display());
                all = false;
                continue;
            }
        };
        let name = fixture
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        for (i, moment) in moments.iter().enumerate() {
            println!("{} at {} s in {name}", moment.case, moment.at);
            let outcome = check(&demo, SourceTime(moment.at)).and_then(|same| {
                let slug = format!("{}-{i}", name.trim_end_matches(".dem"));
                match slice(&demo, moment.at, &folder, &slug)? {
                    Ok((slice, at)) => {
                        println!(
                            "  the slice from {:.1} s, at {at:.3} s in it",
                            moment.at - at
                        );
                        Ok(same & check(&slice, SourceTime(at))?)
                    }
                    // An export's refusal is its own rule, not a difference
                    Err(refused) => {
                        println!("  no slice: {refused}");
                        Ok(same)
                    }
                }
            });
            match outcome {
                Ok(same) => all &= same,
                Err(e) => {
                    println!("    failed: {e}");
                    all = false;
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&folder);
    std::process::exit(if all { 0 } else { 1 });
}
