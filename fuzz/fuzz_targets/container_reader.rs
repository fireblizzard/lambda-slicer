#![no_main]

use std::io::Cursor;

use lambda_slicer_core::container::{DemoReader, Limits, index_entry};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = Limits {
        max_network_message: 1 << 16,
        max_demo_buffer: 1 << 16,
        ..Limits::default()
    };
    if let Ok(mut r) = DemoReader::new(Cursor::new(data), limits) {
        for e in 0..r.directory.len() as u32 {
            let _ = index_entry(&mut r, e, true);
        }
    }
});
