//! Development probe that lists each `HudText` message of the given demos,
//! with its time in the playback and the name it sends, and each
//! `GameTitle`, which shows the game's logo
//!
//! `hudtext_probe DEMO...`

use std::path::Path;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    for path in std::env::args().skip(1) {
        println!("{path}");
        let options = DecodeOptions::default();
        decode_demo(Path::new(&path), &options, |at, d, m| {
            let Message::User { id, body } = &m.message else {
                return;
            };
            let named = d.user_message(*id).map(|u| u.name_lossy());
            if named.as_deref() == Some("GameTitle") {
                println!(
                    "  {:>9.3} s  part {}  GameTitle {body:?}",
                    at.time(),
                    at.entry
                );
                return;
            }
            if named.as_deref() != Some("HudText") {
                return;
            }

            let end = body.iter().position(|&b| b == 0).unwrap_or(body.len());
            let name = String::from_utf8_lossy(&body[..end]);
            println!("  {:>9.3} s  part {}  {name}", at.time(), at.entry);
        });
    }
}
