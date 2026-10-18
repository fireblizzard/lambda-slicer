//! Counts a demo's sound records by sample, the sounds the recording client
//! started itself and wrote into the demo
//!
//! `record_sounds DEMO [FROM TO]` with times in seconds

use std::collections::BTreeMap;

use lambda_slicer_core::container::{DemoReader, FrameBody, Limits};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("demo");
    let from: f32 = args.next().map_or(0., |s| s.parse().unwrap());
    let to: f32 = args.next().map_or(f32::INFINITY, |s| s.parse().unwrap());

    let file = std::io::BufReader::new(std::fs::File::open(&path).expect("demo"));
    let mut reader = DemoReader::new(file, Limits::default()).expect("demo");
    let mut counts: BTreeMap<(String, i32), u32> = BTreeMap::new();
    for entry in 0..2 {
        let Ok(frames) = reader.frames(entry) else {
            continue;
        };
        for located in frames {
            let Ok(located) = located else {
                break;
            };
            let time = located.frame.time();
            let FrameBody::Sound(sound) = &located.frame.body else {
                continue;
            };
            if time < from || time >= to {
                continue;
            }
            let name = String::from_utf8_lossy(&sound.sample).into_owned();
            *counts.entry((name, sound.channel)).or_insert(0) += 1;
        }
    }
    for ((name, channel), count) in counts {
        println!("{count:6} {name} channel {channel}");
    }
}
