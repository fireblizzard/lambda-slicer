//! Development probe showing which `movevars_t` floats differ between demo
//! info and the last decoded `svc_newmovevars`, by field index

use std::collections::BTreeMap;

use lambda_slicer_core::container::{DemoReader, FrameBody, Limits};
use lambda_slicer_core::state::{Decoder, FrameLocation};

fn f(b: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

fn main() {
    let p = std::env::args().nth(1).unwrap();
    let file = std::io::BufReader::new(std::fs::File::open(&p).unwrap());
    let mut r = DemoReader::new(file, Limits::default()).unwrap();

    let mut d = Decoder::new();
    d.collect_events = false;
    let mut diffs = BTreeMap::<usize, (u64, f32, f32)>::new();

    for entry in 0..r.directory.len() as u32 {
        for lf in r.frames(entry).unwrap() {
            let lf = lf.unwrap();
            let FrameBody::Network(n) = &lf.frame.body else {
                continue;
            };

            let at = FrameLocation {
                entry,
                ordinal: lf.ordinal,
                offset: lf.offset,
                time_bits: lf.frame.time_bits,
                frame_number: lf.frame.frame_number,
            };
            d.network_frame(at, n, |_, _| {}).unwrap();
            let (Some(m), true) = (&d.movevars, entry > 0) else {
                continue;
            };

            let info = n.info_movevars();
            // 16 leading floats, then 8 trailing floats
            let pairs = (0..16)
                .map(|i| (i, i * 4, i * 4))
                .chain((0..8).map(|i| (16 + i, 100 + i * 4, 65 + i * 4)));
            for (field, a, b) in pairs {
                let (x, y) = (f(info, a), f(m, b));
                if x.to_bits() != y.to_bits() {
                    diffs.entry(field).or_insert((0, x, y)).0 += 1;
                }
            }
        }
    }

    for (field, (n, x, y)) in diffs {
        println!("field {field}: {n} frames, e.g. demo info {x} vs svc {y}");
    }
}
