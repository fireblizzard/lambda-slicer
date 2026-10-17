//! Development survey of the death animation a killed player plays
//!
//! Usage is `death_probe <demos...>`. For each kill, reads the victim
//! entity's `sequence` in the snapshots right after the death notice. The
//! server picks the death animation from the last body part hit, so a head
//! hit plays the headshot death (`ACT_DIE_HEADSHOT`)

use std::collections::BTreeMap;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    for path in std::env::args().skip(1) {
        let options = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };
        // Victims waiting for a sequence, as entity, weapon and time
        let mut waiting: Vec<(u16, String, f32)> = Vec::new();
        // (weapon, sequence) -> count
        let mut seen: BTreeMap<(String, i64), u32> = BTreeMap::new();
        let mut missed = 0;
        decode_demo(path.as_ref(), &options, |loc, d, m| {
            if let Message::User { id, body } = &m.message
                && d.user_message(*id)
                    .is_some_and(|u| u.name_lossy() == "DeathMsg")
                && body.len() >= 2
            {
                let rest = &body[2..];
                let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
                let weapon = String::from_utf8_lossy(&rest[..end]).into_owned();
                waiting.push((u16::from(body[1]), weapon, loc.time()));
            }
            let Some(snap) = d.latest_snapshot() else {
                return;
            };
            let (Some(entities), Some(schema)) =
                (&snap.entities, d.schema("entity_state_player_t"))
            else {
                return;
            };
            waiting.retain(|(entity, weapon, time)| {
                if loc.time() - time > 1. {
                    missed += 1;
                    return false;
                }
                let Some(e) = entities.iter().find(|e| e.number == *entity) else {
                    return true;
                };
                let seq = match e.fields.value(&schema, "sequence") {
                    Some(v) => v.as_i64().unwrap_or(-1),
                    None => return true,
                };
                // Wait for the packet after the death notice
                if loc.time() <= *time {
                    return true;
                }
                *seen.entry((weapon.clone(), seq)).or_default() += 1;
                false
            });
        });

        println!("{path}  (no sequence seen within 1 s: {missed})");
        let mut by_seq: BTreeMap<i64, u32> = BTreeMap::new();
        for ((_, s), n) in &seen {
            *by_seq.entry(*s).or_default() += n;
        }
        println!("  sequences overall {by_seq:?}");
        let mut weapons: BTreeMap<&str, Vec<(i64, u32)>> = BTreeMap::new();
        for ((w, s), n) in &seen {
            weapons.entry(w).or_default().push((*s, *n));
        }
        for (w, v) in weapons {
            println!("  {w:12} {v:?}");
        }
    }
}
