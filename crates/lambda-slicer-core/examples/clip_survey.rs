//! Development survey that tries clips across many demos and counts why
//! clips are refused, so the common reasons can be fixed first
//!
//! `clip_survey --list FILE SCRATCH [--workers N]`
//!
//! Each demo in the list, one path per line, is cross-checked once, then a
//! 10 second clip is tried at a fifth, half and four fifths of its playback.
//! The demos are only read. Each clip is written to a file in `SCRATCH`
//! that is deleted at once, and the source is neither copied nor validated
//! against the clip, so a clip that passes here can still fail an export

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use lambda_slicer_core::container::{DemoReader, Limits};
use lambda_slicer_core::message::Coded;
use lambda_slicer_core::slice::ExportError;
use lambda_slicer_core::slice::restart::check_restart;
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_core::validate::crosscheck::{Outcome, crosscheck_demo};

/// Where in the playback each clip starts, as shares of its length
const STARTS: [f32; 3] = [0.2, 0.5, 0.8];

/// How long each clip is, in seconds
const CLIP_SECONDS: f32 = 10.;

/// Examples kept for each reason
const EXAMPLES: usize = 3;

/// Characters of a reason's detail kept, so long diagnostics group together
const DETAIL_CHARS: usize = 80;

/// What became of one demo
enum Survey {
    /// Not a clip candidate, with the reason
    Skipped(String),
    /// Each clip tried, as its start and its refusal if any
    Clips(Vec<(DemoTime, Option<String>)>),
}

/// The reason a clip was refused, as its message code and, where the code
/// alone says too little, its detail
fn reason(e: &ExportError) -> String {
    let detail = match e {
        ExportError::UnknownPersistentMessage(d)
        | ExportError::ActiveEffect(d)
        | ExportError::Unsupported(d)
        | ExportError::Decode(d)
        | ExportError::Validation(d) => d.clone(),
        _ => return e.code().to_string(),
    };

    let detail: String = detail.chars().take(DETAIL_CHARS).collect();
    format!("{} | {detail}", e.code())
}

/// The playback entry's length in seconds, none when the demo has no
/// playback entry or cannot be read
fn playback_length(path: &Path) -> Option<f32> {
    let file = File::open(path).ok()?;
    let reader = DemoReader::new(BufReader::new(file), Limits::default()).ok()?;
    reader
        .directory
        .iter()
        .find(|d| d.entry_type == 1)
        .map(|d| d.track_time())
}

fn survey(path: &Path, scratch: &Path) -> Survey {
    let Some(length) = playback_length(path) else {
        return Survey::Skipped("unreadable or no playback".into());
    };
    if length < CLIP_SECONDS * 2. {
        return Survey::Skipped("shorter than 20 s".into());
    }

    // The same check an export makes before anything else, once per demo
    let check = crosscheck_demo(path, Limits::default());
    if check.outcome == Outcome::Failed {
        return Survey::Skipped("fails its cross-check".into());
    }

    let mut clips = Vec::new();
    for share in STARTS {
        let start = DemoTime::from_f32(length * share).unwrap_or(DemoTime::ZERO);
        // Short demos would otherwise ask for a clip past their end
        let end = DemoTime::from_f32((length * share + CLIP_SECONDS).min(length));
        let refused = check_restart(path, scratch, start, end, Limits::default())
            .err()
            .map(|e| reason(&e));
        clips.push((start, refused));
    }

    Survey::Clips(clips)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (list, scratch) = match args.as_slice() {
        [flag, list, scratch, ..] if flag == "--list" => (list, PathBuf::from(scratch)),
        _ => panic!("usage: clip_survey --list FILE SCRATCH [--workers N]"),
    };
    let workers = match args.as_slice() {
        [.., flag, n] if flag == "--workers" => n.parse().expect("workers"),
        _ => 4,
    };

    let text = std::fs::read_to_string(list).expect("read list");
    let files: Vec<PathBuf> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .collect();
    std::fs::create_dir_all(&scratch).expect("create scratch folder");

    // Workers take the next demo in turn until none are left
    let started = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(PathBuf, Survey)>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(path) = files.get(i) else {
                        break;
                    };

                    let result = survey(path, &scratch);
                    let mut results = results.lock().unwrap();
                    results.push((path.clone(), result));
                    if results.len().is_multiple_of(100) {
                        eprintln!("{} of {} demos", results.len(), files.len());
                    }
                }
            });
        }
    });

    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.0.cmp(&b.0));

    let mut skipped = BTreeMap::<String, u64>::new();
    let mut refused = BTreeMap::<String, Vec<String>>::new();
    let (mut clips, mut passed, mut demos_refused) = (0u64, 0u64, 0u64);
    for (path, result) in &results {
        match result {
            Survey::Skipped(why) => *skipped.entry(why.clone()).or_default() += 1,
            Survey::Clips(list) => {
                let mut any = false;
                for (start, refusal) in list {
                    clips += 1;
                    match refusal {
                        None => passed += 1,
                        Some(why) => {
                            any = true;
                            let example = format!("{} at {start}", path.display());
                            refused.entry(why.clone()).or_default().push(example);
                        }
                    }
                }
                demos_refused += u64::from(any);
            }
        }
    }

    println!(
        "{} demos in {:.0} s, {clips} clips tried, {passed} passed, {} demos with a refused clip",
        results.len(),
        started.elapsed().as_secs_f64(),
        demos_refused
    );

    println!("\nSkipped demos:");
    for (why, n) in &skipped {
        println!("  {n:>5}  {why}");
    }

    // Most common reasons first
    let mut reasons: Vec<(&String, &Vec<String>)> = refused.iter().collect();
    reasons.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    println!("\nRefused clips by reason:");
    for (why, examples) in reasons {
        println!("  {:>5}  {why}", examples.len());
        for example in examples.iter().take(EXAMPLES) {
            println!("         {example}");
        }
    }
}
