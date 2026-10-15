//! Whole-file sequential decode, feeding container frames in physical order
//! through one [`Decoder`]

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Serialize;

use super::world::{DecodeStats, Decoder, FrameLocation, PlayerInfo};
use crate::container::{DemoReader, FrameBody, Limits};
use crate::protocol::messages::ParsedMessage;

pub const DECODE_REPORT_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DecodeStatus {
    /// Every network message in every entry decoded and applied
    Complete,
    /// Decoding stopped at `error`, and state before it is valid
    Partial,
    /// Header or directory unusable
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct SchemaSummary {
    pub name: String,
    pub fields: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct DecodeReport {
    pub schema: u32,
    pub path: String,
    pub status: DecodeStatus,
    pub error: Option<String>,
    pub max_clients: u8,
    pub player_num: u8,
    pub hltv: bool,
    /// Some delta bases were not found under the recorded sequence numbers
    /// and were selected by reproducing the server's declared entity count.
    /// The state is then inferred, not proven
    pub inferred_delta_bases: bool,
    pub stats: DecodeStats,
    pub delta_schemas: Vec<SchemaSummary>,
    pub players: Vec<PlayerInfo>,
}

pub struct DecodeOptions {
    pub limits: Limits,
    pub collect_events: bool,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            collect_events: true,
        }
    }
}

/// Decode `path`, calling `on_message` for every applied network message.
/// Returns the report and the final decoder (with collected events)
pub fn decode_demo(
    path: &Path,
    options: &DecodeOptions,
    mut on_message: impl FnMut(&FrameLocation, &Decoder, &ParsedMessage),
) -> (DecodeReport, Decoder) {
    let mut decoder = Decoder::new();
    decoder.collect_events = options.collect_events;

    let mut report = DecodeReport {
        schema: DECODE_REPORT_SCHEMA,
        path: path.display().to_string(),
        status: DecodeStatus::Failed,
        error: None,
        max_clients: 0,
        player_num: 0,
        hltv: false,
        inferred_delta_bases: false,
        stats: DecodeStats::default(),
        delta_schemas: Vec::new(),
        players: Vec::new(),
    };

    let reader = File::open(path).map_err(|e| e.to_string()).and_then(|f| {
        DemoReader::new(BufReader::new(f), options.limits).map_err(|e| e.to_string())
    });
    let mut reader = match reader {
        Ok(r) => r,
        Err(e) => {
            report.error = Some(e);
            return (report, decoder);
        }
    };

    report.status = DecodeStatus::Complete;
    'entries: for entry in 0..reader.directory.len() as u32 {
        let frames = match reader.frames(entry) {
            Ok(f) => f,
            Err(e) => {
                report.error = Some(e.to_string());
                report.status = DecodeStatus::Partial;
                break;
            }
        };

        for item in frames {
            let lf = match item {
                Ok(lf) => lf,
                Err(e) => {
                    report.error = Some(e.to_string());
                    report.status = DecodeStatus::Partial;
                    break 'entries;
                }
            };

            let location = FrameLocation {
                entry,
                ordinal: lf.ordinal,
                offset: lf.offset,
                time_bits: lf.frame.time_bits,
                frame_number: lf.frame.frame_number,
            };
            match &lf.frame.body {
                FrameBody::Network(n) => {
                    if let Err(e) =
                        decoder.network_frame(location, n, |d, m| on_message(&location, d, m))
                    {
                        report.error = Some(e.to_string());
                        report.status = DecodeStatus::Partial;
                        break 'entries;
                    }
                }
                FrameBody::Event(ev) => decoder.demo_event(location, ev),
                _ => {}
            }
        }
    }

    report.max_clients = decoder.max_clients;
    report.player_num = decoder.player_num;
    report.hltv = decoder.hltv;
    report.inferred_delta_bases =
        decoder.stats.sequence_offset_changes > 0 || decoder.stats.empty_delta_bases > 0;
    report.stats = decoder.stats.clone();

    let mut schemas: Vec<SchemaSummary> = [
        "entity_state_t",
        "entity_state_player_t",
        "custom_entity_state_t",
        "clientdata_t",
        "weapon_data_t",
        "event_t",
        "usercmd_t",
    ]
    .iter()
    .filter_map(|n| {
        decoder.schema(n).map(|s| SchemaSummary {
            name: s.name_lossy(),
            fields: s.fields.len(),
        })
    })
    .collect();
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    report.delta_schemas = schemas;
    report.players = decoder.players.values().cloned().collect();

    (report, decoder)
}
