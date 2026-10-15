//! Development probe that counts `svc_sound` starts and stops per sample, to
//! find sounds that keep playing until stopped
//!
//! Usage is `sound_probe <demos...>`

use std::collections::BTreeMap;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

/// `SND_STOP`, `SND_CHANGE_VOL`, `SND_CHANGE_PITCH` and `SND_SPAWNING`
const STOP: u16 = 1 << 5;
const CHANGE: u16 = (1 << 6) | (1 << 7);
const SPAWNING: u16 = 1 << 8;

#[derive(Default)]
struct Counts {
    starts: u64,
    stops: u64,
    changes: u64,
    spawning: u64,
    channels: BTreeMap<u8, u64>,
}

fn main() {
    let mut by_sample: BTreeMap<String, Counts> = BTreeMap::new();

    for path in std::env::args().skip(1) {
        let (_, _) = decode_demo(path.as_ref(), &DecodeOptions::default(), |_, d, m| {
            let Message::Sound(s) = &m.message else {
                return;
            };

            let name = d
                .resources()
                .iter()
                .find(|r| r.kind == 0 && s.sound_index == r.index)
                .map_or_else(
                    || format!("#{}", s.sound_index),
                    |r| String::from_utf8_lossy(&r.name).into_owned(),
                );
            let c = by_sample.entry(name).or_default();
            if s.flags & STOP != 0 {
                c.stops += 1;
            } else if s.flags & CHANGE != 0 {
                c.changes += 1;
            } else {
                c.starts += 1;
            }
            if s.flags & SPAWNING != 0 {
                c.spawning += 1;
            }
            *c.channels.entry(s.channel).or_default() += 1;
        });
    }

    for (name, c) in by_sample
        .iter()
        .filter(|(_, c)| c.stops + c.changes + c.spawning > 0)
    {
        println!(
            "{name:40} starts {:6} stops {:6} changes {:6} spawning {:4} channels {:?}",
            c.starts, c.stops, c.changes, c.spawning, c.channels
        );
    }
}
