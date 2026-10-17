//! Development survey of each session's server info, beside the header's
//! map name and checksum

use std::path::PathBuf;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    for arg in std::env::args().skip(1) {
        let path = PathBuf::from(arg);
        let options = DecodeOptions {
            collect_events: false,
            ..DecodeOptions::default()
        };
        println!("{}", path.display());
        let _ = decode_demo(&path, &options, |loc, _, m| {
            if let Message::ServerInfo(i) = &m.message {
                println!(
                    "  entry {} map {} crc {:#010x}",
                    loc.entry,
                    String::from_utf8_lossy(&i.map_name),
                    i.map_crc as u32
                );
            }
        });
    }
}
