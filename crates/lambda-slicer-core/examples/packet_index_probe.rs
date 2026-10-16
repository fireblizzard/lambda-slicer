//! Development probe of unreliable event packet indices against the
//! packet's entity list
//!
//! Usage is `packet_index_probe <demo>`. For every event resolved through
//! the packet, prints how far its index is from the end of the list and
//! whether the entities from there on are custom (beam) entities

use std::collections::BTreeMap;

use lambda_slicer_core::container::{DemoReader, FrameBody, Limits};
use lambda_slicer_core::state::{ActorResolution, Decoder, FrameLocation};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let f = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    let mut d = Decoder::new();
    let mut from_end: BTreeMap<(usize, bool, bool), u64> = BTreeMap::new();
    let mut seen = 0;
    let mut previous_len = 0;
    let mut shown = 0;

    for entry in 0..r.directory.len() as u32 {
        for lf in r.frames(entry).unwrap() {
            let lf = lf.unwrap();
            let FrameBody::Network(n) = &lf.frame.body else {
                continue;
            };
            let loc = FrameLocation {
                entry,
                ordinal: lf.ordinal,
                offset: lf.offset,
                time_bits: lf.frame.time_bits,
                frame_number: lf.frame.frame_number,
            };
            if d.network_frame(loc, n, |_, _| {}).is_err() {
                return;
            }

            let Some(entities) = d.latest_snapshot().and_then(|s| s.entities.clone()) else {
                seen = d.events.len();
                continue;
            };
            for e in &d.events[seen..] {
                let ActorResolution::PacketEntity { packet_index, .. } = e.actor else {
                    continue;
                };
                let pi = usize::from(packet_index);
                if shown < 12 {
                    let normal = entities.iter().filter(|x| !x.custom).count();
                    println!(
                        "t {:.3} index {pi} len {} previous len {previous_len} not custom {normal} at index {}",
                        e.location.time(),
                        entities.len(),
                        entities[pi].number
                    );
                    shown += 1;
                }
                let below = |limit: u16| entities.iter().filter(|x| x.number < limit).count();
                for limit in [256u16, 512] {
                    *from_end
                        .entry((limit as usize, pi == below(limit), false))
                        .or_default() += 1;
                }
                let tail_custom = entities[pi..].iter().all(|x| x.custom);
                let player =
                    entities[pi].number >= 1 && entities[pi].number <= u16::from(d.max_clients);
                *from_end
                    .entry((entities.len() - pi, tail_custom, player))
                    .or_default() += 1;
            }
            seen = d.events.len();
            previous_len = entities.len();
        }
    }

    if let Some(schema) = d.schema("event_t") {
        let names: Vec<_> = schema.fields.iter().map(|f| f.name_lossy()).collect();
        println!("event_t fields: {names:?}");
    }

    println!(
        "(entities from the index to the end, all of them custom, index names a player): events"
    );
    for (k, v) in &from_end {
        println!("  {k:?}: {v}");
    }
}
