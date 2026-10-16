//! Whole-file sequential decode, feeding container frames in physical order
//! through one [`Decoder`]

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use super::world::{DecodeStats, Decoder, FrameLocation, NameChange, PlayerInfo};
use crate::container::{DemoReader, FrameBody, Limits};
use crate::message::{Coded, Message, io_message};
use crate::protocol::messages::ParsedMessage;
use crate::protocol::profile;

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
    /// The recording uses a protocol this decoder does not support, so it
    /// was not decoded. Inspection still reads its metadata
    Unsupported,
    /// Stopped on request. State before the stop is valid but the result
    /// is incomplete and must not stand in for a full decode
    Cancelled,
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
    pub error: Option<Message>,
    pub max_clients: u8,
    pub player_num: u8,
    pub hltv: bool,
    /// Some delta bases were not found under the recorded sequence numbers
    /// and were selected by reproducing the server's declared entity count.
    /// The state is then inferred, not proven
    pub inferred_delta_bases: bool,
    pub stats: DecodeStats,
    pub delta_schemas: Vec<SchemaSummary>,
    /// Occupants at the end of the recording
    pub players: Vec<PlayerInfo>,
    /// Joins, renames and departures, so earlier events keep their names
    pub name_history: Vec<NameChange>,
    /// Playback time covered by network records, and the part of it the
    /// recording client spent as a spectator, either a spectator client
    /// (demo info `spectator`) or a player in observer mode (client data
    /// `iuser1`). Each record's state lasts until the next record
    pub playback_seconds: f64,
    pub spectator_seconds: f64,
}

/// Whether the recording player is in observer mode, which the game marks
/// with a non-zero `iuser1` (`OBS_*` in `common/const.h`)
fn observing(d: &Decoder) -> bool {
    let (Some(snap), Some(schema)) = (d.latest_snapshot(), d.schema("clientdata_t")) else {
        return false;
    };
    let Some(client) = &snap.clientdata else {
        return false;
    };

    client
        .value(&schema, "iuser1")
        .and_then(|v| v.as_i64())
        .is_some_and(|m| m != 0)
}

pub struct DecodeOptions {
    pub limits: Limits,
    pub collect_events: bool,
    /// Checked before every frame. Setting it stops decoding with
    /// [`DecodeStatus::Cancelled`]
    pub cancel: Option<Arc<AtomicBool>>,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            collect_events: true,
            cancel: None,
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
        name_history: Vec::new(),
        playback_seconds: 0.,
        spectator_seconds: 0.,
    };
    let mut previous: Option<(f64, bool)> = None;

    let reader = File::open(path)
        .map_err(|e| io_message(&e))
        .and_then(|f| DemoReader::new(BufReader::new(f), options.limits).map_err(|e| e.message()));
    let mut reader = match reader {
        Ok(r) => r,
        Err(e) => {
            report.error = Some(e);
            return (report, decoder);
        }
    };

    if let Err(e) = profile::check(&reader.header) {
        report.status = DecodeStatus::Unsupported;
        report.error = Some(e.message());
        return (report, decoder);
    }

    report.status = DecodeStatus::Complete;
    'entries: for entry in 0..reader.directory.len() as u32 {
        let frames = match reader.frames(entry) {
            Ok(f) => f,
            Err(e) => {
                report.error = Some(e.message());
                report.status = DecodeStatus::Partial;
                break;
            }
        };

        for item in frames {
            if options
                .cancel
                .as_ref()
                .is_some_and(|c| c.load(Ordering::Relaxed))
            {
                report.status = DecodeStatus::Cancelled;
                break 'entries;
            }
            let lf = match item {
                Ok(lf) => lf,
                Err(e) => {
                    report.error = Some(e.message());
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
                        report.error = Some(e.message());
                        report.status = DecodeStatus::Partial;
                        break 'entries;
                    }

                    if entry > 0 {
                        let now = f64::from(lf.frame.time());
                        if let Some((then, spectating)) = previous {
                            let span = (now - then).max(0.);
                            report.playback_seconds += span;
                            if spectating {
                                report.spectator_seconds += span;
                            }
                        }
                        let spectating = n.info_spectator() != 0 || observing(&decoder);
                        previous = Some((now, spectating));
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
    report.inferred_delta_bases = decoder.stats.inferred_snapshots > 0;
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
    report.name_history = decoder.name_history.clone();

    (report, decoder)
}
