//! Restart export, a slice `[start, end)` that plays on its own
//!
//! The output keeps the LOADING entry of the session the start is in, then
//! opens the playback
//! entry with one bootstrap group. That group holds a full update for every
//! frame before the start that the kept packets still reference as a delta
//! base, sent under the frame's own sequence numbers. The kept frame groups
//! follow with their bytes unchanged and only their outer times and frame
//! numbers rebased. The client therefore rebuilds every later state itself,
//! and the bootstrap updates are the only reconstructed state written.
//!
//! Only proven state is used. The source must not fail its cross-check,
//! and neither the bootstrap frames nor the kept packets may rest on
//! inferred bases or fall in a run the client dropped as too old.
//! Persistent state that changed after LOADING (user info, light styles,
//! HUD messages) is replayed at the start of the bootstrap group, see
//! [`super::persistent`]

use std::cell::RefCell;
use std::collections::{BTreeSet, VecDeque};
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use super::ongoing::{Ongoing, PendingSounds};
use super::output::{Counting, Published, io_err, publish_new, to_i32};
use super::persistent::Persistent;
use super::{ExportError, check_stop};
use crate::container::{
    DemoReader, Frame, FrameBody, HEADER_DIR_OFFSET_FIELD, Limits, NetworkFrame, encode_directory,
    encode_frame, encode_header, tag,
};
use crate::protocol::bitwrite::BitWriter;
use crate::protocol::delta::{DeltaSchema, Kind, StructState, Value};
use crate::protocol::encode::{
    Approximation, EncodeStats, Fit, FullEntity, Place, differing_fields, write_full_clientdata,
    write_full_entities, write_time,
};
use crate::protocol::messages::Message;
use crate::protocol::profile;
use crate::state::{Decoder, FrameLocation, Snapshot};
use crate::timeline::time::DemoTime;
use crate::validate::crosscheck::{Outcome, crosscheck_demo_until};
use crate::validate::structural::{InspectStatus, inspect};

/// Pre-start frames kept as bootstrap candidates. The client accepts bases
/// up to 62 packets back
const BOOTSTRAP_WINDOW: u32 = 62;

/// How far from a refused start other starts are tried, in seconds, the
/// nearest first. A later start keeps at least half the clip
const SEARCH_STEPS: [f64; 7] = [0.1, 0.25, 0.5, 1., 2., 3., 5.];

/// How many starts are checked at once, as each reads the demo through
const CHECKS_AT_ONCE: usize = 4;

/// How many times the gap between the refused start and the nearest that
/// works is narrowed, each a quarter of the last
const NARROWINGS: usize = 2;

#[derive(Clone, Debug, Serialize)]
pub struct RestartValidation {
    /// Kept packets whose decoded output state was compared with the source
    pub compared_snapshots: u64,
    /// Time-window fields that differ by exactly a bootstrap approximation,
    /// inherited unchanged since the bootstrap
    pub approximated_differences: u64,
    /// Any other differing field, which fails the export
    pub other_differences: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RestartReport {
    pub source: String,
    pub published: Published,
    pub requested_start: String,
    /// Source time of the first kept frame group
    pub effective_start: String,
    pub requested_end: Option<String>,
    /// Source time of the first excluded group, or of the source terminal
    pub effective_end: String,
    /// Sequence of the first kept network record
    pub start_sequence: u32,
    /// Pre-start frames written as full updates
    pub bootstrap_sequences: Vec<u32>,
    /// Persistent messages replayed before them (user info, light styles,
    /// HUD state), and their size
    pub persistent_messages: usize,
    /// Timed HUD notices before the start that were no longer showing and
    /// so were not replayed
    pub expired_notices: usize,
    /// Effects under way at the start that were carried over it
    pub carried: Vec<String>,
    /// Effects under way at the start that the clip leaves out, gibs and
    /// thrown models still in the air
    pub left_out: Vec<String>,
    pub persistent_bytes: usize,
    pub bootstrap_bytes: usize,
    pub kept_records: u64,
    pub kept_network_records: u64,
    pub inexact_time_windows: u32,
    pub clamped_time_windows: u32,
    /// Every accepted approximation, once per place and field with its
    /// largest error
    pub approximations: Vec<String>,
    pub validation: RestartValidation,
}

/// Everything learnt from the planning pass
struct Plan {
    /// Ordinal of the first kept record in the playback entry
    first_ordinal: u64,
    /// Ordinal of the first excluded record, or of the source terminal
    end_ordinal: u64,
    first_time: f32,
    first_frame_number: i32,
    /// Spacing kept between the bootstrap group and the first kept group
    lead: f64,
    /// Time of the first excluded group or of the source terminal
    end_time: f32,
    start_sequence: u32,
    /// DemoBuffer and ClientData records of the last group before the start
    prelude: Vec<Frame>,
    /// Demo records of effects under way at the start, written after the
    /// bootstrap updates
    carried: Vec<Frame>,
    /// Network messages of effects under way at the start (sound loops,
    /// beams, HUD text), written in the record after the bootstrap group
    late: Vec<u8>,
    /// One line per carried effect
    notes: Vec<String>,
    /// One line per effect under way at the start that is left out
    left_out: Vec<String>,
    /// (snapshot, the source record that carried it) per bootstrap frame
    bootstrap: Vec<(Arc<Snapshot>, NetworkFrame)>,
    /// The network records that set up the start's session, when it is a
    /// later session than the source's LOADING entry describes. They become
    /// the output's LOADING entry
    setup: Option<Vec<Frame>>,
    /// That session's map, from its server info, for the output's header
    setup_map: Option<Vec<u8>>,
    /// Persistent messages replayed before the bootstrap updates, and how
    /// many there are
    persistent: Vec<u8>,
    persistent_count: usize,
    /// Persistent user messages (HUD state), written in their own group
    /// after the bootstrap group
    hud: Vec<u8>,
    hud_count: usize,
    /// Timed HUD notices no longer showing at the start
    expired_notices: usize,
    /// The record whose sequence fields the persistent messages travel with
    persistent_record: Option<NetworkFrame>,
    kept_records: u64,
    kept_network_records: u64,
}

fn decode_err(e: impl std::fmt::Display) -> ExportError {
    ExportError::Decode(e.to_string())
}

fn location(entry: u32, ordinal: u64, offset: u64, frame: &Frame) -> FrameLocation {
    FrameLocation {
        entry,
        ordinal,
        offset,
        time_bits: frame.time_bits,
        frame_number: frame.frame_number,
    }
}

fn open(path: &Path, limits: Limits) -> Result<DemoReader<BufReader<File>>, ExportError> {
    let file = File::open(path).map_err(io_err(path))?;
    let reader = DemoReader::new(BufReader::new(file), limits)?;
    profile::check(&reader.header)?;

    let types: Vec<i32> = reader.directory.iter().map(|d| d.entry_type).collect();
    if types != [0, 1] {
        return Err(ExportError::Unsupported(format!(
            "a restart needs exactly LOADING(0) and Playback(1) entries, found types {types:?}"
        )));
    }

    Ok(reader)
}

/// Decode the LOADING entry
fn decode_loading(
    reader: &mut DemoReader<BufReader<File>>,
    d: &mut Decoder,
) -> Result<(), ExportError> {
    for item in reader.frames(0)? {
        let lf = item?;
        if let FrameBody::Network(n) = &lf.frame.body {
            let loc = location(0, lf.ordinal, lf.offset, &lf.frame);
            d.network_frame(loc, n, |_, _| {}).map_err(decode_err)?;
        }
    }

    Ok(())
}

fn at(frame: &Frame) -> String {
    DemoTime::from_f32(frame.time()).map_or_else(|| frame.time().to_string(), |t| t.to_string())
}

/// The sequence a delta reference names, counted back from `sequence` the
/// way the client does
fn named(sequence: u32, from: u8) -> u32 {
    sequence.wrapping_sub(u32::from((sequence as u8).wrapping_sub(from)))
}

fn plan(
    path: &Path,
    start: DemoTime,
    end: Option<DemoTime>,
    limits: Limits,
    stop: &AtomicBool,
) -> Result<Plan, ExportError> {
    let mut reader = open(path, limits)?;
    let mut d = Decoder::new();
    decode_loading(&mut reader, &mut d)?;

    let mut recent: VecDeque<NetworkFrame> = VecDeque::new();
    let mut prelude: Vec<Frame> = Vec::new();
    let mut previous: Option<(i32, f32)> = None;
    let mut window: Vec<Arc<Snapshot>> = Vec::new();
    let mut referenced: BTreeSet<u32> = BTreeSet::new();
    let mut kept: Option<Plan> = None;
    let mut session = 0;
    let mut dropped = 0;
    let mut persistent = Persistent::default();
    let mut ongoing = Ongoing::default();
    let mut pending: Option<PendingSounds> = None;
    let loading_session = d.segment;

    // A later session sets itself up inside the playback entry, from its
    // server info to `svc_signonnum`. Those records are kept for a LOADING
    // entry of its own. A start is possible once the setup is complete and a
    // full update arrived, so there is a state to start from
    let mut setup: Option<Vec<Frame>> = None;
    let mut session_setup: Option<Vec<Frame>> = None;
    let mut session_map: Option<Vec<u8>> = None;
    let (mut signed_on, mut full_update) = (true, true);

    // Past the end, packets are still decoded until every sound that may
    // be playing at the start is confirmed or ruled out
    let mut past_end = false;

    for item in reader.frames(1)? {
        check_stop(stop)?;
        let lf = item?;
        let f = &lf.frame;
        let t = DemoTime::from_f32(f.time()).unwrap_or(DemoTime::ZERO);
        let new_group = previous.map(|(n, _)| n) != Some(f.frame_number);

        if past_end && !pending.as_ref().is_some_and(PendingSounds::unresolved) {
            break;
        }

        if matches!(f.body, FrameBody::NextSection) {
            if past_end {
                break;
            }
            let Some(p) = kept.as_mut() else {
                return Err(ExportError::RangeBeyondEnd {
                    end: start,
                    segment_end: t,
                });
            };
            if let Some(e) = end
                && e > t
            {
                return Err(ExportError::RangeBeyondEnd {
                    end: e,
                    segment_end: t,
                });
            }

            p.end_ordinal = lf.ordinal;
            p.end_time = f.time();
            break;
        }

        if new_group && kept.is_none() && t >= start && !matches!(f.body, FrameBody::DemoStart) {
            if !(signed_on && full_update) {
                return Err(ExportError::StartDuringLoading);
            }
            let start_setup = match d.segment == loading_session {
                true => None,
                false => Some(
                    session_setup
                        .clone()
                        .ok_or(ExportError::StartDuringLoading)?,
                ),
            };
            if let Some(b) = persistent.blocker.take() {
                return Err(ExportError::UnknownPersistentMessage(b));
            }
            let persistent_kept = persistent.split(f.time());
            let Some((_, previous_time)) = previous else {
                return Err(ExportError::StartAtFirstGroup);
            };

            // Bootstrap candidates are the stored frames just before the start
            let latest = d.latest_snapshot().cloned();
            if let Some(latest) = &latest {
                let newest = latest.sequence;
                window = (newest.saturating_sub(BOOTSTRAP_WINDOW)..=newest)
                    .filter_map(|s| d.snapshot(s).cloned())
                    .collect();
            }

            let present = |e: u16| {
                latest
                    .as_ref()
                    .and_then(|s| s.entities.as_ref())
                    .is_some_and(|list| list.iter().any(|x| x.number == e))
            };
            let (carried, sounds) = ongoing
                .at_start(f.time(), present)
                .map_err(ExportError::ActiveEffect)?;
            pending = Some(sounds);

            session = d.segment;
            dropped = d.stats.discarded_packets;
            kept = Some(Plan {
                first_ordinal: lf.ordinal,
                end_ordinal: 0,
                first_time: f.time(),
                first_frame_number: f.frame_number,
                lead: (f64::from(f.time()) - f64::from(previous_time)).max(0.),
                end_time: f.time(),
                start_sequence: 0,
                prelude: std::mem::take(&mut prelude),
                carried: carried.records,
                late: carried.messages,
                notes: carried.notes,
                left_out: carried.left_out,
                bootstrap: Vec::new(),
                setup_map: start_setup.as_ref().and(session_map.clone()),
                setup: start_setup,
                persistent: persistent_kept.engine,
                persistent_count: persistent_kept.engine_count,
                hud: persistent_kept.hud,
                hud_count: persistent_kept.hud_count,
                expired_notices: persistent_kept.expired,
                persistent_record: recent.back().cloned(),
                kept_records: 0,
                kept_network_records: 0,
            });
        }

        if new_group
            && !past_end
            && let (Some(p), Some(e)) = (kept.as_mut(), end)
            && t >= e
        {
            p.end_ordinal = lf.ordinal;
            p.end_time = f.time();
            past_end = true;
        }

        if new_group && kept.is_none() {
            prelude.clear();
        }
        previous = Some((f.frame_number, f.time()));

        let FrameBody::Network(n) = &f.body else {
            match kept.as_mut() {
                Some(p) if !past_end => p.kept_records += 1,
                Some(_) => {}
                None => {
                    ongoing.observe_record(&d, f);

                    // Only the client's state records, never its events,
                    // sounds or commands, which already happened
                    if matches!(f.body, FrameBody::DemoBuffer(_) | FrameBody::ClientData(_)) {
                        prelude.push(f.clone());
                    }
                }
            }
            continue;
        };

        let sequence = n.sequence[0] as u32;
        let loc = location(1, lf.ordinal, lf.offset, f);
        let mut froms: Vec<u8> = Vec::new();
        let before = kept.is_none();
        let time = f.time();
        let segment = d.segment;
        let (mut signon_here, mut full_here) = (false, false);
        d.network_frame(loc, n, |dec, m| {
            if before {
                persistent.observe(dec, m, &n.message, time);
                ongoing.observe_message(dec, m, &n.message, time);
            } else if let Some(s) = pending.as_mut() {
                s.observe(m);
            }
            match &m.message {
                Message::DeltaPacketEntities(p) => froms.extend(p.delta_from),
                Message::ClientData(c) => froms.extend(c.delta_from),
                Message::PacketEntities(_) => full_here = true,
                Message::SignOnNum(_) => signon_here = true,
                Message::ServerInfo(info) if before => session_map = Some(info.map_name.clone()),
                _ => {}
            }
        })
        .map_err(decode_err)?;

        if before {
            if d.segment != segment {
                // A new session may number its packets again, so a bootstrap
                // must never take a previous session's record for its own
                recent.clear();
                setup = Some(Vec::new());
                session_setup = None;
                (signed_on, full_update) = (false, false);
            }
            if let Some(records) = setup.as_mut() {
                records.push(f.clone());
                if signon_here {
                    session_setup = setup.take();
                    // The previous map's state is gone, and the setup's own
                    // messages arrive with the LOADING entry
                    persistent = Persistent::default();
                    ongoing = Ongoing::default();
                }
            }
            signed_on |= signon_here;
            full_update |= full_here;
        }

        let Some(p) = kept.as_mut() else {
            recent.push_back((**n).clone());
            if recent.len() > 2 * BOOTSTRAP_WINDOW as usize {
                recent.pop_front();
            }
            continue;
        };
        if past_end {
            continue;
        }

        if p.kept_network_records == 0 {
            p.start_sequence = sequence;
        }
        p.kept_records += 1;
        p.kept_network_records += 1;

        if d.segment != session {
            return Err(ExportError::CrossSegmentRange(at(f)));
        }
        if d.stats.discarded_packets != dropped {
            return Err(ExportError::ClientDroppedPackets(at(f)));
        }
        if let Some(s) = d.latest_snapshot()
            && s.sequence == sequence
            && s.inferred
        {
            return Err(ExportError::UnprovenState(at(f)));
        }

        for from in froms {
            let base = named(sequence, from);
            if base < p.start_sequence {
                referenced.insert(base);
            }
        }
    }

    let mut p = kept.ok_or(ExportError::RangeBeyondEnd {
        end: start,
        segment_end: DemoTime::ZERO,
    })?;
    if p.kept_network_records == 0 {
        return Err(ExportError::EmptyRange);
    }
    if p.end_ordinal == 0 {
        return Err(ExportError::Unsupported(
            "the playback entry has no terminal record".into(),
        ));
    }

    // Sound loops still playing at the start go with the other effects
    if let Some(s) = &pending {
        let (bytes, notes) = s.confirmed(&d);
        p.late.extend_from_slice(&bytes);
        p.notes.extend(notes);
    }

    // The newest frame before the start is the state the client shows until
    // the first kept packet, so it is written even when nothing references it
    if let Some(newest) = window.last() {
        referenced.insert(newest.sequence);
    }

    for sequence in referenced {
        let snap = window.iter().find(|s| s.sequence == sequence);
        let record = recent.iter().find(|n| n.sequence[0] as u32 == sequence);
        let (Some(snap), Some(record)) = (snap, record) else {
            return Err(ExportError::MissingDeltaBase(sequence));
        };
        if snap.inferred || snap.entities.is_none() || snap.clientdata.is_none() {
            return Err(ExportError::UnprovenState(format!(
                "bootstrap frame {sequence}"
            )));
        }

        p.bootstrap.push((snap.clone(), record.clone()));
    }
    if let Some((_, first)) = p.bootstrap.first() {
        p.persistent_record = Some(first.clone());
    }

    Ok(p)
}

/// `svc_time`, full clientdata and full packet entities for one frame
fn bootstrap_payload(
    d: &Decoder,
    snap: &Snapshot,
    stats: &mut EncodeStats,
) -> Result<Vec<u8>, ExportError> {
    let missing = |what: &str| ExportError::Unsupported(format!("bootstrap needs {what}"));
    let time = snap.server_time_bits;
    let mut w = BitWriter::new();
    write_time(&mut w, time);

    let client_schema = d
        .schema("clientdata_t")
        .ok_or_else(|| missing("clientdata_t"))?;
    let weapon_schema = d.schema("weapon_data_t");
    let client = snap
        .clientdata
        .as_ref()
        .ok_or_else(|| missing("clientdata"))?;
    let weapons: Vec<Option<&StructState>> = match &snap.weapons {
        Some(w) => w.iter().map(|s| s.as_deref()).collect(),
        None => Vec::new(),
    };
    write_full_clientdata(
        &mut w,
        &client_schema,
        client,
        weapon_schema.as_deref(),
        &weapons,
        time,
        stats,
    );

    let entities = snap.entities.as_ref().ok_or_else(|| missing("entities"))?;
    let schemas = entities
        .iter()
        .map(|e| d.entity_schema(e.number, e.custom))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| missing("entity schemas"))?;
    let empty = StructState::default();
    let full: Vec<FullEntity<'_>> = entities
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
    write_full_entities(&mut w, &full, d.instanced_baselines() > 0, time, stats);

    Ok(w.finish())
}

/// The header's form of a server info map, `maps/c3a2e.bsp` becoming
/// `c3a2e`, zero padded
fn header_map_name(map: &[u8]) -> [u8; 260] {
    let name = map
        .rsplit(|b| *b == b'/' || *b == b'\\')
        .next()
        .unwrap_or(map);
    let name = match name.len().checked_sub(4) {
        Some(n) if name[n..].eq_ignore_ascii_case(b".bsp") => &name[..n],
        _ => name,
    };
    let mut raw = [0u8; 260];
    let n = name.len().min(259);
    raw[..n].copy_from_slice(&name[..n]);
    raw
}

/// Outer time of a kept record, measured from the bootstrap group at zero
fn rebased(p: &Plan, time: f32) -> u32 {
    let t = f64::from(time) - f64::from(p.first_time) + p.lead;

    (t.max(0.) as f32).to_bits()
}

struct Written {
    bootstrap_bytes: usize,
    stats: EncodeStats,
}

/// Write the clip `p` plans of the demo at `path`, marked as a clip of the
/// demo named `origin`
fn write(
    path: &Path,
    out: &mut File,
    p: &Plan,
    origin: &str,
    limits: Limits,
    stop: &AtomicBool,
) -> Result<Written, ExportError> {
    let mut reader = open(path, limits)?;
    let mut d = Decoder::new();
    match &p.setup {
        // The bootstrap is encoded with the schemas and baselines of the
        // session the client loads
        Some(records) => {
            for f in records {
                if let FrameBody::Network(n) = &f.body {
                    let loc = location(0, 0, 0, f);
                    d.network_frame(loc, n, |_, _| {}).map_err(decode_err)?;
                }
            }
        }
        None => decode_loading(&mut reader, &mut d)?,
    }

    let dest = Path::new("<output>");
    let mut header = reader.header.clone();
    let mut directory = reader.directory.clone();
    let mut w = Counting {
        inner: BufWriter::new(&mut *out),
        pos: 0,
    };
    header.directory_offset = 0;
    // A later session's own map names the output. The header's checksum
    // is the client's own sum of the map file, which the recording does not
    // hold for that map, so it is 0, as some recordings already have it
    if let Some(map) = &p.setup_map {
        *header.map_name_raw = header_map_name(map);
        header.map_checksum = 0;
    }
    w.put(&encode_header(&header)).map_err(io_err(dest))?;

    let mut buf = Vec::with_capacity(64 * 1024);
    let mut put = |w: &mut Counting<_>, f: &Frame| -> Result<(), ExportError> {
        buf.clear();
        encode_frame(f, &mut buf);
        w.put(&buf).map_err(io_err(dest))
    };

    // LOADING, unchanged, or the later session's setup records and a
    // terminal record of its own
    let start = w.pos;
    match &p.setup {
        Some(records) => {
            // LOADING holds startup records, which the game reads at once.
            // As ordinary network records they would wait for the playback
            // clock to reach their time in the source, as the game showed
            for f in records {
                let startup = Frame {
                    tag: tag::NETWORK_START,
                    ..f.clone()
                };
                put(&mut w, &startup)?;
            }
            let last = records.last().ok_or_else(|| {
                ExportError::Unsupported("the session has no setup records".into())
            })?;
            let terminal = Frame {
                tag: tag::NEXT_SECTION,
                time_bits: last.time_bits,
                frame_number: last.frame_number + 1,
                body: FrameBody::NextSection,
            };
            put(&mut w, &terminal)?;
            if directory[0].frame_count != 0 {
                directory[0].frame_count = to_i32(records.len() as u64, "LOADING record count")?;
            }
        }
        None => {
            for item in reader.frames(0)? {
                put(&mut w, &item?.frame)?;
            }
        }
    }
    directory[0].offset = to_i32(start, "entry offset")?;
    directory[0].length = to_i32(w.pos - start, "entry length")?;

    // The playback entry opens with DemoStart and the bootstrap group, and
    // the kept groups follow
    let start = w.pos;
    let zero = |body: FrameBody, tag: u8| Frame {
        tag,
        time_bits: 0,
        frame_number: 0,
        body,
    };
    put(&mut w, &zero(FrameBody::DemoStart, tag::DEMO_START))?;
    for f in &p.prelude {
        put(&mut w, &zero(f.body.clone(), f.tag))?;
    }

    if !p.persistent.is_empty() {
        let Some(record) = &p.persistent_record else {
            return Err(ExportError::Unsupported(
                "no record before the start to carry persistent state".into(),
            ));
        };
        let network = NetworkFrame {
            demo_info: record.demo_info.clone(),
            sequence: record.sequence,
            message: p.persistent.clone(),
        };
        put(
            &mut w,
            &zero(FrameBody::Network(Box::new(network)), tag::NETWORK),
        )?;
    }

    let mut stats = EncodeStats::default();
    let mut bootstrap_bytes = 0;
    for (snap, record) in &p.bootstrap {
        let message = bootstrap_payload(&d, snap, &mut stats)?;
        bootstrap_bytes += message.len();
        let network = NetworkFrame {
            demo_info: record.demo_info.clone(),
            sequence: record.sequence,
            message,
        };
        put(
            &mut w,
            &zero(FrameBody::Network(Box::new(network)), tag::NETWORK),
        )?;
    }

    for f in &p.carried {
        put(&mut w, &zero(f.body.clone(), f.tag))?;
    }

    // HUD elements time their display from the HUD clock, which only moves
    // when a frame is drawn. Read in the bootstrap group, a message would be
    // timed from the clock before the restart and expire at once, so HUD
    // messages arrive in a group of their own once a frame was drawn
    let hud_group = i32::from(!p.hud.is_empty() || !p.late.is_empty());
    if hud_group > 0 {
        // The newest bootstrap frame's numbers, so the sequence never steps back
        let newest = p.bootstrap.last().map(|(_, r)| r);
        let Some(record) = newest.or(p.persistent_record.as_ref()) else {
            return Err(ExportError::Unsupported(
                "no record before the start to carry HUD state".into(),
            ));
        };
        let network = NetworkFrame {
            demo_info: record.demo_info.clone(),
            sequence: record.sequence,
            message: [p.late.as_slice(), p.hud.as_slice()].concat(),
        };
        let lead = f32::from_bits(rebased(p, p.first_time));
        put(
            &mut w,
            &Frame {
                tag: tag::NETWORK,
                time_bits: (lead / 2.).to_bits(),
                frame_number: 1,
                body: FrameBody::Network(Box::new(network)),
            },
        )?;
    }

    let persistent_records = u64::from(!p.persistent.is_empty()) + hud_group as u64;
    let mut network_records = p.bootstrap.len() as u64 + persistent_records;
    for item in reader.frames(1)? {
        check_stop(stop)?;
        let lf = item?;
        if lf.ordinal < p.first_ordinal {
            continue;
        }
        if lf.ordinal >= p.end_ordinal {
            break;
        }

        let mut f = lf.frame;
        if matches!(f.body, FrameBody::Network(_)) {
            network_records += 1;
        }
        f.time_bits = rebased(p, f.time());
        f.frame_number = f.frame_number - p.first_frame_number + 1 + hud_group;
        put(&mut w, &f)?;
    }

    let terminal = Frame {
        tag: tag::NEXT_SECTION,
        time_bits: rebased(p, p.end_time),
        frame_number: to_i32(p.end_ordinal - p.first_ordinal, "terminal frame number")?,
        body: FrameBody::NextSection,
    };
    put(&mut w, &terminal)?;

    let d1 = &mut directory[1];
    d1.mark_clip(origin);
    d1.offset = to_i32(start, "entry offset")?;
    d1.length = to_i32(w.pos - start, "entry length")?;
    d1.frame_count = to_i32(network_records, "network record count")?;
    d1.track_time_bits = terminal.time_bits;

    let dir_offset = to_i32(w.pos, "directory offset")?;
    w.put(&encode_directory(&directory)).map_err(io_err(dest))?;
    w.inner.flush().map_err(io_err(dest))?;
    drop(w);

    out.seek(SeekFrom::Start(HEADER_DIR_OFFSET_FIELD as u64))
        .map_err(io_err(dest))?;
    out.write_all(&dir_offset.to_le_bytes())
        .map_err(io_err(dest))?;
    out.flush().map_err(io_err(dest))?;

    accept(&stats.approximations)?;

    Ok(Written {
        bootstrap_bytes,
        stats,
    })
}

/// Field differences found so far. A time-window difference counts as
/// approximated only when it is exactly a bootstrap approximation of the
/// same place and field, the source still holding the value meant and the
/// output the value sent. Everything else is unexplained
#[derive(Default)]
struct Tally {
    windows: u64,
    other: u64,
    /// The first unexplained fields, for the error
    first: Vec<String>,
}

impl Tally {
    fn add(
        &mut self,
        place: Place,
        schema: &DeltaSchema,
        x: &StructState,
        y: &StructState,
        approximations: &[Approximation],
    ) {
        for i in differing_fields(schema, x, y) {
            let field = &schema.fields[i];
            let name = field.name_lossy();
            let bits = |s: &StructState| match s.value(schema, &name) {
                Some(Value::Float(f)) => Some(f.to_bits()),
                _ => None,
            };

            let window = matches!(field.kind(), Kind::TimeWindow8 | Kind::TimeWindowBig);
            let explained = window
                && approximations.iter().any(|a| {
                    a.place == place
                        && a.field == name
                        && Some(a.original_bits) == bits(x)
                        && Some(a.written_bits) == bits(y)
                });
            if explained {
                self.windows += 1;
                continue;
            }

            self.other += 1;
            if self.first.len() < 3 {
                self.first.push(format!("{} {name}", place_name(place)));
            }
        }
    }
}

fn place_name(place: Place) -> String {
    match place {
        Place::Entity(n) => format!("entity {n}"),
        Place::Client => "clientdata".into(),
        Place::Weapon(slot) => format!("weapon {slot}"),
    }
}

/// Field differences between two snapshots
fn compare(d: &Decoder, a: &Snapshot, b: &Snapshot, approximations: &[Approximation]) -> Tally {
    let mut tally = Tally::default();
    match (&a.entities, &b.entities) {
        (Some(x), Some(y)) if x.iter().map(|e| e.number).eq(y.iter().map(|e| e.number)) => {
            for (e, f) in x.iter().zip(y.iter()) {
                let place = Place::Entity(e.number);
                match d.entity_schema(e.number, e.custom) {
                    Some(s) => tally.add(place, &s, &e.fields, &f.fields, approximations),
                    None => tally.other += 1,
                }
            }
        }
        (None, None) => {}
        _ => tally.other += 1,
    }

    let empty = StructState::default();
    if let Some(s) = d.schema("clientdata_t") {
        let x = a.clientdata.as_deref().unwrap_or(&empty);
        let y = b.clientdata.as_deref().unwrap_or(&empty);
        tally.add(Place::Client, &s, x, y, approximations);
    }
    if let (Some(s), Some(x), Some(y)) = (d.schema("weapon_data_t"), &a.weapons, &b.weapons) {
        for slot in 0..64 {
            let left = x[slot].as_deref().unwrap_or(&empty);
            let right = y[slot].as_deref().unwrap_or(&empty);
            tally.add(Place::Weapon(slot as u8), &s, left, right, approximations);
        }
    }

    tally
}

/// Fields whose clamped value only sets an animation's phase. A clamped
/// `animtime` restarts the animation at a different frame, while clamping
/// any other time field would move an event in time
const CLAMP_ALLOWED: &[&str] = &["animtime"];

/// Refuse approximations beyond what the field's resolution explains, and
/// clamping of any field but an entity's animation time
fn accept(approximations: &[Approximation]) -> Result<(), ExportError> {
    for a in approximations {
        let where_ = format!("{} {}", place_name(a.place), a.field);
        match a.fit {
            Fit::Exact => {}
            Fit::Rounded if a.error() <= a.resolution => {}
            Fit::Rounded => {
                return Err(ExportError::UnrepresentableTimeField(format!(
                    "{where_} rounds by {:.4} s, more than its {:.4} s resolution",
                    a.error(),
                    a.resolution
                )));
            }
            Fit::Clamped
                if matches!(a.place, Place::Entity(_))
                    && CLAMP_ALLOWED.contains(&a.field.as_str()) => {}
            Fit::Clamped => {
                return Err(ExportError::UnrepresentableTimeField(format!(
                    "{where_} lies {:.2} s outside its time window",
                    a.error()
                )));
            }
        }
    }

    Ok(())
}

/// One line per place and field, with the largest error
fn summarize(approximations: &[Approximation]) -> Vec<String> {
    let mut worst: Vec<(Place, &str, Fit, f32)> = Vec::new();
    for a in approximations {
        match worst
            .iter_mut()
            .find(|(p, f, fit, _)| *p == a.place && *f == a.field && *fit == a.fit)
        {
            Some(w) => w.3 = w.3.max(a.error()),
            None => worst.push((a.place, &a.field, a.fit, a.error())),
        }
    }

    worst
        .into_iter()
        .map(|(place, field, fit, error)| {
            let how = if fit == Fit::Clamped {
                "clamped"
            } else {
                "rounded"
            };
            format!("{} {field} {how} by up to {error:.3} s", place_name(place))
        })
        .collect()
}

/// Decode source and output side by side over the kept packets and compare
/// every snapshot both store
fn validate(
    source: &Path,
    output: &Path,
    p: &Plan,
    approximations: &[Approximation],
    limits: Limits,
    stop: &AtomicBool,
) -> Result<(RestartValidation, Option<String>), ExportError> {
    let mut first_unexplained: Option<String> = None;
    let mut src = open(source, limits)?;
    let mut src_d = Decoder::new();
    decode_loading(&mut src, &mut src_d)?;

    let mut out = open(output, limits)?;
    let mut out_d = Decoder::new();
    decode_loading(&mut out, &mut out_d)?;

    // Output network records in order, decoding the bootstrap group as it
    // passes. The kept records pair up with the source's one for one
    let mut out_frames = out.frames(1)?;
    let late = !p.hud.is_empty() || !p.late.is_empty();
    let mut bootstrap_left =
        p.bootstrap.len() + usize::from(!p.persistent.is_empty()) + usize::from(late);
    let mut next_output = |out_d: &mut Decoder| -> Result<Option<NetworkFrame>, ExportError> {
        for item in out_frames.by_ref() {
            let lf = item?;
            let loc = location(1, lf.ordinal, lf.offset, &lf.frame);
            let FrameBody::Network(n) = lf.frame.body else {
                continue;
            };

            out_d
                .network_frame(loc, &n, |_, _| {})
                .map_err(|e| ExportError::Validation(format!("output does not decode: {e}")))?;
            if bootstrap_left > 0 {
                bootstrap_left -= 1;
                continue;
            }

            return Ok(Some(*n));
        }

        Ok(None)
    };

    let mut v = RestartValidation {
        compared_snapshots: 0,
        approximated_differences: 0,
        other_differences: 0,
    };
    for item in src.frames(1)? {
        check_stop(stop)?;
        let lf = item?;
        if lf.ordinal >= p.end_ordinal {
            break;
        }
        let FrameBody::Network(n) = &lf.frame.body else {
            continue;
        };

        let loc = location(1, lf.ordinal, lf.offset, &lf.frame);
        src_d.network_frame(loc, n, |_, _| {}).map_err(decode_err)?;
        if lf.ordinal < p.first_ordinal {
            continue;
        }

        let Some(o) = next_output(&mut out_d)? else {
            return Err(ExportError::Validation(
                "output ends before the source range".into(),
            ));
        };
        if o.sequence != n.sequence || o.message != n.message {
            return Err(ExportError::Validation(format!(
                "kept record at {} differs from the source",
                at(&lf.frame)
            )));
        }

        let sequence = n.sequence[0] as u32;
        let (Some(a), Some(b)) = (src_d.latest_snapshot(), out_d.latest_snapshot()) else {
            continue;
        };
        if a.sequence != sequence || b.sequence != sequence {
            continue;
        }

        let tally = compare(&src_d, a, b, approximations);
        v.compared_snapshots += 1;
        v.approximated_differences += tally.windows;
        v.other_differences += tally.other;
        if tally.other > 0 && first_unexplained.is_none() {
            first_unexplained = Some(format!("{} at {}", tally.first.join(", "), at(&lf.frame)));
        }
    }

    if next_output(&mut out_d)?.is_some() {
        return Err(ExportError::Validation(
            "output has records past the source range".into(),
        ));
    }

    Ok((v, first_unexplained))
}

/// Whether a restart of `[start, end)` of `source` would be refused, found
/// by reading the source where it is. The plan and write passes run as an
/// export's do, writing to a file in `scratch` that is deleted at once.
/// The source is not copied and the cross-check and final validation are
/// left out, so the caller checks the source once for all its clips, and an
/// export that passes this can still fail its validation
pub fn check_restart(
    source: &Path,
    scratch: &Path,
    start: DemoTime,
    end: Option<DemoTime>,
    limits: Limits,
) -> Result<(), ExportError> {
    let never = AtomicBool::new(false);
    check(source, scratch, start, end, limits, &never)?;
    Ok(())
}

/// [`check_restart`] that gives up once `stop` is set, returning the time
/// the clip would start at, the first frame group's at or after `start`
fn check(
    source: &Path,
    scratch: &Path,
    start: DemoTime,
    end: Option<DemoTime>,
    limits: Limits,
    stop: &AtomicBool,
) -> Result<f32, ExportError> {
    let p = plan(source, start, end, limits, stop)?;
    let mut out = tempfile::tempfile_in(scratch).map_err(io_err(scratch))?;
    write(source, &mut out, &p, "", limits, stop)?;
    Ok(p.first_time)
}

/// Each of `starts` checked at once as [`check`] does, with the time it
/// would start at, none for one that is refused
fn check_all(
    source: &Path,
    scratch: &Path,
    starts: &[DemoTime],
    end: DemoTime,
    limits: Limits,
    stop: &AtomicBool,
) -> Result<Vec<Option<f32>>, ExportError> {
    let results: Vec<Result<f32, ExportError>> = std::thread::scope(|s| {
        let mut checks = Vec::new();
        for &start in starts {
            checks.push(s.spawn(move || check(source, scratch, start, Some(end), limits, stop)));
        }
        let mut results = Vec::new();
        for c in checks {
            let failed = Err(ExportError::Unsupported(
                "the check stopped unexpectedly".into(),
            ));
            results.push(c.join().unwrap_or(failed));
        }
        results
    });

    let mut out = Vec::new();
    for result in results {
        match result {
            Ok(first) => out.push(Some(first)),
            Err(ExportError::Stopped) => return Err(ExportError::Stopped),
            Err(_) => out.push(None),
        }
    }
    Ok(out)
}

/// The start nearest `start` that a clip of `source` ending at `end` can
/// have, for a start refused for where it is
/// ([`ExportError::depends_on_start`]). Starts a little earlier and later
/// are tried, nearest first and earlier before later, then the gap between
/// `start` and the nearest that works is narrowed. Returns the time that
/// start begins at, none when no start within 5 seconds works. Gives up
/// once `stop` is set. A start found passes the same check as
/// [`check_restart`], so its export can still fail its validation
pub fn nearest_start(
    source: &Path,
    scratch: &Path,
    start: DemoTime,
    end: DemoTime,
    limits: Limits,
    stop: &AtomicBool,
) -> Result<Option<DemoTime>, ExportError> {
    let at = |offset: f64| DemoTime::from_micros(start.micros() + (offset * 1e6).round() as i64);
    let half = (end.secs_f64() - start.secs_f64()) / 2.;

    let mut offsets = Vec::new();
    for step in SEARCH_STEPS {
        if start.secs_f64() - step >= 0. {
            offsets.push(-step);
        }
        if step <= half {
            offsets.push(step);
        }
    }

    // A batch is nearer than the next, and in order within itself, so the
    // first start that works is the nearest
    let mut found = None;
    for batch in offsets.chunks(CHECKS_AT_ONCE) {
        let starts: Vec<DemoTime> = batch.iter().map(|&o| at(o)).collect();
        let checked = check_all(source, scratch, &starts, end, limits, stop)?;
        if let Some(i) = checked.iter().position(Option::is_some) {
            found = Some((batch[i], checked[i]));
            break;
        }
    }
    let Some((mut works, mut first)) = found else {
        return Ok(None);
    };

    // The three quarter points between the refused start and the one that
    // works, from the refused end, the first that works the new nearest
    let mut refused = 0.;
    for _ in 0..NARROWINGS {
        let points: Vec<f64> = (1..4)
            .map(|k| refused + (works - refused) * f64::from(k) / 4.)
            .collect();
        let starts: Vec<DemoTime> = points.iter().map(|&o| at(o)).collect();
        let checked = check_all(source, scratch, &starts, end, limits, stop)?;
        match checked.iter().position(Option::is_some) {
            Some(i) => {
                if i > 0 {
                    refused = points[i - 1];
                }
                works = points[i];
                first = checked[i];
            }
            None => refused = points[2],
        }
    }

    Ok(first.and_then(DemoTime::from_f32))
}

pub fn run_restart(
    source: &Path,
    dest: &Path,
    start: DemoTime,
    end: Option<DemoTime>,
    limits: Limits,
) -> Result<RestartReport, ExportError> {
    run_restart_until(source, dest, start, end, limits, &AtomicBool::new(false))
}

/// [`run_restart`] that gives up with [`ExportError::Stopped`] within a
/// moment of `stop` being set, as every pass checks it between records.
/// Nothing is left behind, as with any other failure
pub fn run_restart_until(
    source: &Path,
    dest: &Path,
    start: DemoTime,
    end: Option<DemoTime>,
    limits: Limits,
    stop: &AtomicBool,
) -> Result<RestartReport, ExportError> {
    // Every pass reads the hashed snapshot, never the source path again, so
    // the name the clip is marked with is taken now
    let planned: RefCell<Option<(PathBuf, Plan, Written)>> = RefCell::new(None);
    let origin = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut validation = None;

    let published = publish_new(
        source,
        dest,
        |_, snapshot, out| {
            check_stop(stop)?;
            let check = crosscheck_demo_until(snapshot, limits, stop);
            check_stop(stop)?;
            if check.outcome == Outcome::Failed {
                return Err(ExportError::SourceFailedCrossCheck);
            }

            let p = plan(snapshot, start, end, limits, stop)?;
            let written = write(snapshot, out, &p, &origin, limits, stop)?;
            *planned.borrow_mut() = Some((snapshot.to_path_buf(), p, written));
            Ok(())
        },
        |tmp| {
            let report = inspect(tmp, limits);
            if report.status != InspectStatus::Complete || report.has_errors() {
                let msgs: Vec<String> = report
                    .findings
                    .iter()
                    .map(|f| format!("{}: {}", f.code, f.message))
                    .collect();
                return Err(ExportError::Validation(msgs.join("; ")));
            }

            let planned = planned.borrow();
            let Some((snapshot, p, written)) = planned.as_ref() else {
                return Err(ExportError::Validation("no plan".into()));
            };
            let approximations = &written.stats.approximations;
            let (v, first) = validate(snapshot, tmp, p, approximations, limits, stop)?;
            if v.other_differences > 0 {
                return Err(ExportError::Validation(format!(
                    "{} decoded fields differ from the source, first {}",
                    v.other_differences,
                    first.unwrap_or_default()
                )));
            }

            validation = Some(v);
            Ok(())
        },
    )?;

    let (_, p, written) = planned.into_inner().expect("planned before publishing");
    let validation = validation.expect("validated before publishing");
    let time = |f: f32| DemoTime::from_f32(f).map_or_else(|| f.to_string(), |t| t.to_string());

    Ok(RestartReport {
        source: source.display().to_string(),
        published,
        requested_start: start.to_string(),
        effective_start: time(p.first_time),
        requested_end: end.map(|e| e.to_string()),
        effective_end: time(p.end_time),
        start_sequence: p.start_sequence,
        bootstrap_sequences: p.bootstrap.iter().map(|(s, _)| s.sequence).collect(),
        persistent_messages: p.persistent_count + p.hud_count,
        expired_notices: p.expired_notices,
        carried: p.notes.clone(),
        left_out: p.left_out.clone(),
        persistent_bytes: p.persistent.len() + p.hud.len(),
        bootstrap_bytes: written.bootstrap_bytes,
        kept_records: p.kept_records,
        kept_network_records: p.kept_network_records,
        inexact_time_windows: written.stats.inexact_time_windows,
        clamped_time_windows: written.stats.clamped_time_windows,
        approximations: summarize(&written.stats.approximations),
        validation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approximation(place: Place, field: &str, fit: Fit, error: f32) -> Approximation {
        Approximation {
            place,
            field: field.into(),
            fit,
            original_bits: 10f32.to_bits(),
            written_bits: (10. + error).to_bits(),
            resolution: 0.01,
        }
    }

    #[test]
    fn rounding_is_accepted_within_the_field_resolution() {
        let within = approximation(Place::Entity(3), "starttime", Fit::Rounded, 0.004);
        assert!(accept(&[within]).is_ok());

        let beyond = approximation(Place::Entity(3), "starttime", Fit::Rounded, 0.03);
        assert!(matches!(
            accept(&[beyond]),
            Err(ExportError::UnrepresentableTimeField(_))
        ));
    }

    #[test]
    fn only_an_entity_animtime_may_be_clamped() {
        let animtime = approximation(Place::Entity(3), "animtime", Fit::Clamped, 2.);
        assert!(accept(&[animtime]).is_ok());

        for refused in [
            approximation(Place::Entity(3), "starttime", Fit::Clamped, 2.),
            approximation(Place::Entity(3), "impacttime", Fit::Clamped, 2.),
            approximation(Place::Client, "animtime", Fit::Clamped, 2.),
            approximation(Place::Weapon(4), "fuser1", Fit::Clamped, 2.),
        ] {
            assert!(matches!(
                accept(&[refused]),
                Err(ExportError::UnrepresentableTimeField(_))
            ));
        }
    }
}
