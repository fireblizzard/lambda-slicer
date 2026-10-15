//! Development probe that prints sequence numbers and opcodes per network frame

use lambda_slicer_core::container::{DemoReader, FrameBody, Limits};
use lambda_slicer_core::protocol::messages::{Message, svc};
use lambda_slicer_core::state::{Decoder, FrameLocation};

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let limit: u64 = a.next().map_or(40, |s| s.parse().unwrap());

    let f = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    let mut d = Decoder::new();
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

            let mut ops = Vec::new();
            let res = d.network_frame(loc, n, |_, m| {
                ops.push(match &m.message {
                    Message::PacketEntities(p) => format!("FULL({})", p.declared_count),
                    Message::DeltaPacketEntities(p) => format!(
                        "DELTA({} from {} upd {})",
                        p.declared_count,
                        p.delta_from.unwrap(),
                        p.updates.len()
                    ),
                    Message::ClientData(c) => format!("CD(from {:?})", c.delta_from),
                    _ => svc::name(m.opcode).trim_start_matches("svc_").to_string(),
                })
            });

            println!(
                "e{entry} #{:<5} t {:<10} seq {:?} off {} hist {:?} | {}{}",
                lf.ordinal,
                lf.frame.time(),
                &n.sequence[..],
                d.stats.sequence_offset,
                d.snapshot_for_byte(0).map(|s| s.sequence),
                ops.join(" "),
                res.err()
                    .map(|e| format!("  !! {}", e.kind))
                    .unwrap_or_default()
            );

            shown += 1;
            if shown >= limit {
                return;
            }
        }
    }
}
