//! Development probe that prints the model each player holds, by its
//! `weaponmodel`, around the given times, and every kill with the weapon
//! its killer held then
//!
//! `held_probe DEMO`

use std::collections::BTreeMap;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut held: BTreeMap<u16, String> = BTreeMap::new();
    let options = DecodeOptions::default();
    decode_demo(path.as_ref(), &options, |at, d, m| {
        let model_name = |index: i64| {
            d.resources()
                .iter()
                .find(|r| r.kind == 2 && i64::from(r.index) == index)
                .map(|r| String::from_utf8_lossy(&r.name).into_owned())
        };
        if let (Some(snap), Some(schema)) = (d.latest_snapshot(), d.schema("entity_state_player_t"))
            && let Some(entities) = &snap.entities
        {
            for e in entities.iter() {
                if e.number < 1 || e.number > u16::from(d.max_clients) {
                    continue;
                }
                let model = e
                    .fields
                    .value(&schema, "weaponmodel")
                    .and_then(|v| v.as_i64())
                    .and_then(model_name)
                    .unwrap_or_default();
                held.insert(e.number, model);
            }
        }

        let Message::User { id, body } = &m.message else {
            return;
        };
        let name = d
            .user_message(*id)
            .map(|u| u.name_lossy())
            .unwrap_or_default();
        if name == "DeathMsg" {
            let killer = u16::from(body[0]);
            let rest = &body[2..];
            let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
            let weapon = String::from_utf8_lossy(&rest[..end]);
            println!(
                "{:>9.3} s  killer {killer}  {weapon}  holding {:?}  recorder {}",
                at.playback_time(),
                held.get(&killer),
                u16::from(d.player_num) + 1,
            );
        }
        if name == "CurWeapon" && body.first() == Some(&1) {
            println!(
                "{:>9.3} s  recorder's weapon id {}",
                at.playback_time(),
                body[1]
            );
        }
    });
}
