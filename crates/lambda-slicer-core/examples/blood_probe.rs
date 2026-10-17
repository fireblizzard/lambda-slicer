//! Development survey of blood sprites, the server's trace of a hit
//!
//! Usage is `blood_probe <demos...>`. Counts `TE_BLOODSPRITE` effects and
//! their sizes, and for each one finds the nearest visible player to report
//! the height of the hit above that player's origin
//!
//! A hit runs `TraceAttack`, which spawns blood at the impact point with a
//! size of `clamp(damage * 2 / 10, 3, 16)` in multiplayer, damage already
//! multiplied for the body part (`sk_player_head` is 3)

use std::collections::BTreeMap;

use lambda_slicer_core::analysis::motion::Motion;
use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

const TE_BLOODSPRITE: u8 = 115;

fn coord(b: &[u8]) -> f32 {
    f32::from(i16::from_le_bytes([b[0], b[1]])) / 8.
}

fn main() {
    for path in std::env::args().skip(1) {
        let options = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };
        let mut motion = Motion::default();
        let mut sizes: BTreeMap<u8, u32> = BTreeMap::new();
        // Height above the nearest player's origin, in 4 unit bins, by size
        let mut heights: BTreeMap<(i32, bool), u32> = BTreeMap::new();
        let mut near = 0;
        let mut far = 0;
        let mut total = 0;
        decode_demo(path.as_ref(), &options, |_, d, m| {
            motion.observe(d);
            let Message::TempEntity { kind, body } = &m.message else {
                return;
            };
            if *kind != TE_BLOODSPRITE || body.len() < 12 {
                return;
            }
            total += 1;
            let p = [coord(&body[0..2]), coord(&body[2..4]), coord(&body[4..6])];
            let size = body[11];
            *sizes.entry(size).or_default() += 1;

            // Nearest player by horizontal distance at their latest sample
            let mut best: Option<(f32, f32)> = None;
            for entity in 1..=u16::from(d.max_clients) {
                let Some(s) = motion.samples(entity).last().copied() else {
                    continue;
                };
                let dx = s.origin[0] - p[0];
                let dy = s.origin[1] - p[1];
                let h = (dx * dx + dy * dy).sqrt();
                if best.is_none_or(|b| h < b.0) {
                    best = Some((h, p[2] - s.origin[2]));
                }
            }
            match best {
                Some((h, dz)) if h < 24. => {
                    near += 1;
                    *heights
                        .entry(((dz / 4.).floor() as i32 * 4, size >= 7))
                        .or_default() += 1;
                }
                _ => far += 1,
            }
        });

        println!("{path}");
        println!("  blood sprites {total}, near a visible player {near}, not {far}");
        println!("  sizes {sizes:?}");
        println!("  height above origin (4 unit bins), small sizes | size 7 and up");
        let bins: std::collections::BTreeSet<i32> = heights.keys().map(|k| k.0).collect();
        for b in bins {
            let small = heights.get(&(b, false)).copied().unwrap_or(0);
            let big = heights.get(&(b, true)).copied().unwrap_or(0);
            println!("    {b:4} {small:6} | {big:6}");
        }
    }
}
