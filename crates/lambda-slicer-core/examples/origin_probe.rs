//! Development probe that lists ClientData records whose origin disagrees
//! with decoded clientdata, with the sequence and delta references of the
//! packets around each one
//!
//! Usage is `origin_probe <demo> [threshold] [context]`

use std::collections::VecDeque;
use std::sync::Arc;

use lambda_slicer_core::container::{ClientDataRecord, DemoReader, FrameBody, Limits};
use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{Decoder, FrameLocation, Snapshot};

struct PacketInfo {
    time: f32,
    frame_number: i32,
    sequence: u32,
    entity_from: Option<u8>,
    clientdata_from: Option<Option<u8>>,
    declared: Option<u16>,
    inferred: bool,
    origin: Option<[f64; 3]>,
}

fn origin(d: &Decoder, s: Option<&Arc<Snapshot>>) -> Option<[f64; 3]> {
    let schema = d.schema("clientdata_t")?;
    let c = s?.clientdata.as_ref()?;
    let v = |i: usize| c.value(&schema, &format!("origin[{i}]"))?.as_f64();

    Some([v(0)?, v(1)?, v(2)?])
}

fn dist(a: [f32; 3], b: [f64; 3]) -> f64 {
    (0..3)
        .map(|i| (f64::from(a[i]) - b[i]).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let threshold: f64 = a.next().map_or(1.0, |s| s.parse().unwrap());
    let context: usize = a.next().map_or(3, |s| s.parse().unwrap());

    let f = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    let mut d = Decoder::new();
    let mut recent: VecDeque<PacketInfo> = VecDeque::new();
    let mut pending: Vec<([f32; 3], f32, i32)> = Vec::new();
    let mut mismatches = 0u64;

    for entry in 0..r.directory.len() as u32 {
        for lf in r.frames(entry).unwrap() {
            let lf = lf.unwrap();
            match &lf.frame.body {
                FrameBody::ClientData(c) if entry > 0 => {
                    let rec = ClientDataRecord(c);
                    pending.push((rec.origin(), lf.frame.time(), lf.frame.frame_number));
                }
                FrameBody::Network(n) => {
                    let loc = FrameLocation {
                        entry,
                        ordinal: lf.ordinal,
                        offset: lf.offset,
                        time_bits: lf.frame.time_bits,
                        frame_number: lf.frame.frame_number,
                    };

                    let before = d.latest_snapshot().cloned();
                    let mut info = PacketInfo {
                        time: lf.frame.time(),
                        frame_number: lf.frame.frame_number,
                        sequence: n.sequence[0] as u32,
                        entity_from: None,
                        clientdata_from: None,
                        declared: None,
                        inferred: false,
                        origin: None,
                    };
                    let res = d.network_frame(loc, n, |_, m| match &m.message {
                        Message::DeltaPacketEntities(p) => {
                            info.entity_from = p.delta_from;
                            info.declared = Some(p.declared_count);
                        }
                        Message::PacketEntities(p) => info.declared = Some(p.declared_count),
                        Message::ClientData(c) => info.clientdata_from = Some(c.delta_from),
                        _ => {}
                    });
                    if let Err(e) = res {
                        println!("decode error: {e}");
                        return;
                    }

                    let after = d.latest_snapshot().cloned();
                    info.inferred = after.as_ref().is_some_and(|s| s.inferred);
                    info.origin = origin(&d, after.as_ref());

                    let candidates = [origin(&d, before.as_ref()), origin(&d, after.as_ref())];
                    for (rec, t, fnum) in pending.drain(..) {
                        let best = candidates
                            .iter()
                            .flatten()
                            .map(|o| dist(rec, *o))
                            .min_by(f64::total_cmp);
                        let Some(best) = best else { continue };
                        if best <= threshold {
                            continue;
                        }

                        mismatches += 1;
                        if mismatches > 0 {
                            println!("M t {t} f {fnum} d {best:.2} rec {rec:?}");
                            continue;
                        }

                        println!("MISMATCH t {t} frame {fnum} distance {best:.3} recorded {rec:?}");
                        let start = recent.len().saturating_sub(context);
                        for p in recent.iter().skip(start).chain(std::iter::once(&info)) {
                            println!(
                                "    t {:<10} f {:<7} seq {:<7} (low {:>3}) ent_from {:>4} cd_from {:>9} count {:>4} inf {} origin {:?}",
                                p.time,
                                p.frame_number,
                                p.sequence,
                                p.sequence & 0xff,
                                p.entity_from.map_or("-".into(), |x| x.to_string()),
                                p.clientdata_from.map_or("-".into(), |x| x
                                    .map_or("full".into(), |v| v.to_string())),
                                p.declared.map_or("-".into(), |x| x.to_string()),
                                p.inferred,
                                p.origin.map(|o| [o[0] as f32, o[1] as f32, o[2] as f32]),
                            );
                        }
                    }

                    if let Ok(w) = std::env::var("WINDOW") {
                        let (from, to) = w.split_once(',').unwrap();
                        let (from, to): (f32, f32) = (from.parse().unwrap(), to.parse().unwrap());
                        if info.time >= from && info.time < to {
                            let back = info
                                .entity_from
                                .map(|f| (info.sequence as u8).wrapping_sub(f));
                            println!(
                                "  PKT t {:<10} f {:<7} seq {:<7} (low {:>3}) ent_from {:?} back {:?} cd_from {:?} count {:?} origin {:?}",
                                info.time,
                                info.frame_number,
                                info.sequence,
                                info.sequence & 0xff,
                                info.entity_from,
                                back,
                                info.clientdata_from,
                                info.declared,
                                info.origin.map(|o| [o[0] as f32, o[1] as f32, o[2] as f32]),
                            );
                        }
                    }

                    recent.push_back(info);
                    if recent.len() > 300 {
                        recent.pop_front();
                    }
                }
                _ => {}
            }
        }
    }

    println!(
        "{mismatches} mismatches over {threshold}; stats {:?}",
        (
            d.stats.bases_strict,
            d.stats.bases_offset,
            d.stats.bases_count_search,
            d.stats.bases_empty,
            d.stats.max_delta_distance,
            d.stats.duplicate_sequences
        )
    );
}
