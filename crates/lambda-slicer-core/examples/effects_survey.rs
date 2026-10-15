//! Development survey of temp entity kinds and stopped sounds across demos
//!
//! Usage is `effects_survey --list FILE`

use std::collections::BTreeMap;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

const STOP: u16 = 1 << 5;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [flag, list] = args.as_slice() else {
        eprintln!("usage: effects_survey --list FILE");
        std::process::exit(2);
    };
    assert_eq!(flag, "--list");

    let mut kinds: BTreeMap<u8, (u64, u64, usize)> = BTreeMap::new();
    let mut stopped: BTreeMap<String, (u64, u64)> = BTreeMap::new();

    for path in std::fs::read_to_string(list)
        .unwrap()
        .lines()
        .map(str::trim)
    {
        if path.is_empty() {
            continue;
        }

        let mut seen_kinds = std::collections::BTreeSet::new();
        let mut seen_stops = std::collections::BTreeSet::new();
        decode_demo(
            path.as_ref(),
            &DecodeOptions::default(),
            |_, d, m| match &m.message {
                Message::TempEntity { kind, body } => {
                    let e = kinds.entry(*kind).or_default();
                    e.0 += 1;
                    e.2 = e.2.max(body.len());
                    if seen_kinds.insert(*kind) {
                        e.1 += 1;
                    }
                }
                Message::Sound(s) if s.flags & STOP != 0 => {
                    let name = d
                        .resources()
                        .iter()
                        .find(|r| r.kind == 0 && s.sound_index == r.index)
                        .map_or_else(
                            || format!("#{}", s.sound_index),
                            |r| String::from_utf8_lossy(&r.name).into_owned(),
                        );
                    let e = stopped.entry(name.clone()).or_default();
                    e.0 += 1;
                    if seen_stops.insert(name) {
                        e.1 += 1;
                    }
                }
                _ => {}
            },
        );
    }

    println!("temp entities (kind: messages, files, largest body)");
    for (k, (n, f, len)) in &kinds {
        println!("  {k:4}  {n:9} {f:5} {len:4}");
    }
    println!("stopped sounds (stops, files)");
    for (name, (n, f)) in &stopped {
        println!("  {name:40} {n:8} {f:5}");
    }
}
