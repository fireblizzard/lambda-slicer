//! Counts the sounds playback starts over a stretch of a demo, by where
//! they come from and their sample
//!
//! `sound_starts DEMO FROM TO`

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use lambda_slicer_core::container::Limits;
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [demo, from, to, ..] = args.as_slice() else {
        eprintln!("usage: sound_starts DEMO FROM TO");
        std::process::exit(2);
    };
    let (from, to): (f64, f64) = (from.parse().unwrap(), to.parse().unwrap());
    let mut playback = Playback::open(Path::new(demo), Limits::default()).expect("demo");
    playback.advance_to(SourceTime(from)).expect("playback");
    let mut seen: HashSet<u64> = playback.audio().channels().iter().map(|c| c.id).collect();
    let mut starts: BTreeMap<String, u32> = BTreeMap::new();
    let mut at = from;
    while at < to {
        at += 0.01;
        playback.advance_to(SourceTime(at)).expect("playback");
        for c in playback.audio().channels() {
            if seen.insert(c.id) {
                *starts
                    .entry(format!("{:?} {:?}", c.source, c.sound))
                    .or_insert(0) += 1;
            }
        }
    }
    for (sound, count) in starts {
        println!("{count:5} {sound}");
    }
}
