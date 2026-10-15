//! Development check of the full-update encoders on real demos
//!
//! Usage is `encode_check <every> <demos...>`
//!
//! Every `every`th proven snapshot of the playback entry is encoded as
//! `svc_time`, full clientdata and full packet entities, decoded again by
//! the same decoder under the same sequence, and compared field by field.
//! The re-decoded snapshot then serves as later packets' base, so an
//! encoding error also shows up as decode failures or cross-check misses

use std::sync::Arc;

use lambda_slicer_core::container::{DemoReader, FrameBody, Limits, NetworkFrame};
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::delta::Kind;
use lambda_slicer_core::protocol::encode::{
    EncodeStats, FullEntity, differing_fields, write_full_clientdata, write_full_entities,
    write_time,
};
use lambda_slicer_core::state::{Decoder, FrameLocation, Snapshot};

#[derive(Default)]
struct Totals {
    checked: u64,
    entities: u64,
    mismatched_snapshots: u64,
    /// Differences in fields other than time windows, which must not occur
    other_differences: u64,
    max_payload: usize,
    stats: EncodeStats,
}

fn encode(d: &Decoder, snap: &Snapshot, stats: &mut EncodeStats) -> Option<Vec<u8>> {
    let mut w = BitWriter::new();
    write_time(&mut w, snap.server_time_bits);

    let client_schema = d.schema("clientdata_t")?;
    let weapon_schema = d.schema("weapon_data_t");
    let client = snap.clientdata.as_ref()?;
    let weapons: Vec<_> = snap
        .weapons
        .as_ref()?
        .iter()
        .map(|s| s.as_deref())
        .collect();
    write_full_clientdata(
        &mut w,
        &client_schema,
        client,
        weapon_schema.as_deref(),
        &weapons,
        snap.server_time_bits,
        stats,
    );

    let entities = snap.entities.as_ref()?;
    let empty = Default::default();
    let schemas: Vec<_> = entities
        .iter()
        .map(|e| d.entity_schema(e.number, e.custom))
        .collect::<Option<_>>()?;
    let full: Vec<_> = entities
        .iter()
        .zip(&schemas)
        .map(|(e, schema)| FullEntity {
            number: e.number,
            custom: e.custom,
            schema,
            baseline: d.baseline(e.number).map_or(&empty, |b| &b.fields),
            state: &e.fields,
        })
        .collect();
    write_full_entities(
        &mut w,
        &full,
        d.instanced_baselines() > 0,
        snap.server_time_bits,
        stats,
    );

    Some(w.finish())
}

fn differences(d: &Decoder, a: &Snapshot, b: &Snapshot) -> Vec<String> {
    let mut out = Vec::new();
    let (Some(ea), Some(eb)) = (&a.entities, &b.entities) else {
        return vec!["entities missing".into()];
    };

    let na: Vec<_> = ea.iter().map(|e| e.number).collect();
    let nb: Vec<_> = eb.iter().map(|e| e.number).collect();
    if na != nb {
        out.push(format!("entity numbers {na:?} vs {nb:?}"));
        return out;
    }

    for (x, y) in ea.iter().zip(eb.iter()) {
        let schema = d.entity_schema(x.number, x.custom).unwrap();
        for i in differing_fields(&schema, &x.fields, &y.fields) {
            let window = matches!(
                schema.fields[i].kind(),
                Kind::TimeWindow8 | Kind::TimeWindowBig
            );
            let tag = if window { "" } else { "OTHER " };
            out.push(format!(
                "{tag}entity {} {}",
                x.number,
                schema.fields[i].name_lossy()
            ));
        }
    }

    let cschema = d.schema("clientdata_t").unwrap();
    if let (Some(x), Some(y)) = (&a.clientdata, &b.clientdata) {
        for i in differing_fields(&cschema, x, y) {
            out.push(format!(
                "OTHER clientdata {}",
                cschema.fields[i].name_lossy()
            ));
        }
    }

    if let (Some(wschema), Some(x), Some(y)) = (d.schema("weapon_data_t"), &a.weapons, &b.weapons) {
        let empty = Default::default();
        for slot in 0..64 {
            let left = x[slot].as_deref().unwrap_or(&empty);
            let right = y[slot].as_deref().unwrap_or(&empty);
            for i in differing_fields(&wschema, left, right) {
                out.push(format!(
                    "OTHER weapon {slot} {}",
                    wschema.fields[i].name_lossy()
                ));
            }
        }
    }

    out
}

fn check(path: &str, every: u64, totals: &mut Totals) {
    let f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    let mut d = Decoder::new();
    let mut seen = 0u64;

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
            if let Err(e) = d.network_frame(loc, n, |_, _| {}) {
                println!("{path}: decode stopped: {e}");
                return;
            }

            let Some(snap) = d.latest_snapshot().cloned() else {
                continue;
            };
            if entry == 0 || snap.inferred || snap.sequence != n.sequence[0] as u32 {
                continue;
            }

            seen += 1;
            if !seen.is_multiple_of(every) {
                continue;
            }

            let Some(payload) = encode(&d, &snap, &mut totals.stats) else {
                continue;
            };
            totals.max_payload = totals.max_payload.max(payload.len());

            let frame = NetworkFrame {
                demo_info: n.demo_info.clone(),
                sequence: n.sequence,
                message: payload,
            };
            if let Err(e) = d.network_frame(loc, &frame, |_, _| {}) {
                println!(
                    "{path}: encoded packet failed to decode at {}: {e}",
                    lf.frame.time()
                );
                return;
            }

            let again: Arc<Snapshot> = d.latest_snapshot().cloned().unwrap();
            totals.checked += 1;
            totals.entities += snap.entities.as_ref().map_or(0, |e| e.len() as u64);

            let diff = differences(&d, &snap, &again);
            let other: Vec<_> = diff.iter().filter(|x| x.starts_with("OTHER")).collect();
            totals.other_differences += other.len() as u64;
            if !other.is_empty() && totals.other_differences <= 20 {
                println!("{path} t {}: {other:?}", lf.frame.time());
            }
            if !diff.is_empty() {
                totals.mismatched_snapshots += 1;
                if totals.mismatched_snapshots <= 3 {
                    println!(
                        "{path} t {}: {} differences, first {:?}",
                        lf.frame.time(),
                        diff.len(),
                        &diff[..diff.len().min(5)]
                    );
                }
            }
        }
    }
}

fn main() {
    let mut a = std::env::args().skip(1);
    let every: u64 = a.next().unwrap().parse().unwrap();
    let mut totals = Totals::default();

    for path in a {
        check(&path, every, &mut totals);
    }

    println!(
        "{} snapshots checked ({} entities), {} with differences, {} outside time windows, {} inexact time windows ({} clamped), largest payload {} bytes",
        totals.checked,
        totals.entities,
        totals.mismatched_snapshots,
        totals.other_differences,
        totals.stats.inexact_time_windows,
        totals.stats.clamped_time_windows,
        totals.max_payload
    );
}
