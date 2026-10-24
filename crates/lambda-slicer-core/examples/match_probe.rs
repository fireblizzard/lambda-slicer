//! Development probe that lists what starts and ends the matches of the
//! given demos, each `Countdown`, `Settings`, intermission and new session,
//! with its session and time in the playback
//!
//! `match_probe DEMO...`

use std::path::Path;

use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    for path in std::env::args().skip(1) {
        println!("{path}");
        let options = DecodeOptions::default();
        let mut kills = 0;
        decode_demo(Path::new(&path), &options, |at, d, m| {
            let time = at.playback_time();
            let what = match &m.message {
                Message::ServerInfo(info) => {
                    format!("new session {}", String::from_utf8_lossy(&info.map_name))
                }
                Message::Intermission => format!("intermission after {kills} kills"),
                Message::User { id, body } => {
                    let named = d.user_message(*id).map(|u| u.name_lossy());
                    match named.as_deref() {
                        Some("Countdown") => {
                            format!("Countdown {:?}", body.first().map(|&b| b as i8))
                        }
                        Some("Settings") => format!("Settings match {:?}", body.first()),
                        Some("DeathMsg") => {
                            kills += 1;
                            return;
                        }
                        _ => return,
                    }
                }
                _ => return,
            };
            println!(
                "  part {}  {time:>9.3} s  {what}  ({kills} kills)",
                d.segment
            );
        });
    }
}
