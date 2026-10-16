//! Development probe that prints chosen fields of entities at a time
//!
//! Usage is `entity_probe <demo> <time> <entity...>`

use lambda_slicer_core::state::{DecodeOptions, decode_demo};

const FIELDS: &[&str] = &[
    "modelindex",
    "movetype",
    "aiment",
    "owner",
    "effects",
    "rendermode",
    "solid",
    "origin[0]",
    "origin[1]",
    "origin[2]",
];

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let time: f32 = a.next().unwrap().parse().unwrap();
    let wanted: Vec<u16> = a.map(|s| s.parse().unwrap()).collect();
    let mut done = false;

    let options = DecodeOptions {
        collect_events: false,
        ..Default::default()
    };
    decode_demo(path.as_ref(), &options, |loc, d, _| {
        if done || loc.entry == 0 || loc.time() < time {
            return;
        }
        let Some(snap) = d.latest_snapshot() else {
            return;
        };
        let Some(entities) = &snap.entities else {
            return;
        };
        done = true;
        let numbers: Vec<u16> = entities.iter().map(|e| e.number).collect();
        println!(
            "snapshot {} has {} entities: {numbers:?}",
            snap.sequence,
            numbers.len()
        );

        let model_name = |index: i64| {
            d.resources()
                .iter()
                .find(|r| r.kind == 2 && i64::from(r.index) == index)
                .map(|r| String::from_utf8_lossy(&r.name).into_owned())
        };
        for e in entities.iter().filter(|e| wanted.contains(&e.number)) {
            let Some(schema) = d.entity_schema(e.number, e.custom) else {
                continue;
            };
            let values: Vec<String> = FIELDS
                .iter()
                .filter_map(|f| e.fields.value(&schema, f).map(|v| format!("{f}={v:?}")))
                .collect();
            let model = e
                .fields
                .value(&schema, "modelindex")
                .and_then(|v| v.as_i64())
                .and_then(model_name);
            println!("entity {} model {model:?}: {}", e.number, values.join(" "));
        }
    });
}
