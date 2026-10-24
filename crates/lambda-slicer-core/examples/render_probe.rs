//! Development probe that prints how entities of a model are drawn, their
//! render mode, amount, colour and effect, each time those change
//!
//! `render_probe DEMO MODEL_TEXT`

use std::collections::BTreeMap;

use lambda_slicer_core::state::{DecodeOptions, decode_demo};

const FIELDS: &[&str] = &[
    "rendermode",
    "renderamt",
    "rendercolor.r",
    "rendercolor.g",
    "rendercolor.b",
    "renderfx",
    "effects",
];

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let wanted = a.next().unwrap();
    let mut last: BTreeMap<u16, String> = BTreeMap::new();

    let options = DecodeOptions {
        collect_events: false,
        ..Default::default()
    };
    decode_demo(path.as_ref(), &options, |loc, d, _| {
        if loc.entry == 0 {
            return;
        }
        let Some(entities) = d.latest_snapshot().and_then(|s| s.entities.as_ref()) else {
            return;
        };

        let model_name = |index: i64| {
            d.resources()
                .iter()
                .find(|r| r.kind == 2 && i64::from(r.index) == index)
                .map(|r| String::from_utf8_lossy(&r.name).into_owned())
        };
        for e in entities.iter() {
            let Some(schema) = d.entity_schema(e.number, e.custom) else {
                continue;
            };
            let model = e
                .fields
                .value(&schema, "modelindex")
                .and_then(|v| v.as_i64())
                .and_then(model_name)
                .unwrap_or_default();
            if !model.contains(&wanted) {
                continue;
            }

            let values: Vec<String> = FIELDS
                .iter()
                .map(|f| match e.fields.value(&schema, f) {
                    Some(v) => format!("{f}={v:?}"),
                    None => format!("{f}=-"),
                })
                .collect();
            let line = format!("{model} {}", values.join(" "));
            if last.get(&e.number) != Some(&line) {
                println!(
                    "{:>9.3} s  part {}  entity {}  {line}",
                    loc.playback_time(),
                    d.segment,
                    e.number
                );
                last.insert(e.number, line);
            }
        }
    });
}
