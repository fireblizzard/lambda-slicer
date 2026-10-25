//! Cross-check decoded server state against what the recording client
//! itself wrote, meaning the demo info block of every network record and the
//! standalone `ClientData` records
//!
//! These values come from the client, independently of our decoder, so
//! agreement is evidence that decoded clientdata and entity state are what
//! the game computed. The client may record a frame's values before or after
//! applying that frame's packet, so every check counts both alignments

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::container::{ClientDataRecord, DemoReader, FrameBody, Limits, NetworkFrame};
use crate::protocol::profile;
use crate::state::{Decoder, FrameLocation, Snapshot};

pub const CROSSCHECK_SCHEMA: u32 = 1;
const MAX_EXAMPLES: usize = 5;
/// `FL_ONGROUND`
const FL_ONGROUND: i64 = 1 << 9;

/// What a check can prove
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    /// Values both sides must agree on exactly in every frame
    Identity,
    /// The client's stored copy of server clientdata. Agreement tests the
    /// decoded clientdata itself, and rare misses are timing at record edges
    ServerCopy,
    /// Client-side predicted or HUD values. Agreement depends on prediction
    /// settings, so this is informational, not a decoder test
    ClientView,
}

/// Overall verdict of a cross-check. Client-view checks never affect it
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Decoding completed without inferred delta bases, and every identity
    /// and server-copy sample agrees
    Verified,
    /// Nothing contradicts the decode, but it cannot be called verified
    /// because some delta bases were inferred, or because there were no
    /// server-copy samples to compare (e.g. spectator or HLTV recordings)
    Inconclusive,
    /// Decoding stopped, or an identity or server-copy sample disagrees
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct Example {
    pub entry: u32,
    pub frame_number: i32,
    pub time: f32,
    pub recorded: String,
    pub before: String,
    pub after: String,
}

/// An exact comparison. `before`/`after` refer to the decoded state before
/// and after the network packet of the same recorded frame
#[derive(Clone, Debug, Serialize)]
pub struct ExactCheck {
    pub name: &'static str,
    pub kind: CheckKind,
    pub compared: u64,
    pub match_before: u64,
    pub match_after: u64,
    pub match_either: u64,
    /// Samples matching only the state one or two packets earlier (the
    /// client applies the change late). Counted for lag-tolerant checks
    pub match_lagged: u64,
    /// Samples that agree in no alignment while the decoded state compared
    /// was inferred. They do not prove a decoder fault
    pub misses_on_inferred: u64,
    pub examples: Vec<Example>,
}

/// Distance between a recorded vector and the decoded one, binned by the
/// better of the two alignments
#[derive(Clone, Debug, Serialize)]
pub struct DistanceCheck {
    pub name: &'static str,
    pub kind: CheckKind,
    pub compared: u64,
    /// Upper bounds of the bins in world units. The last bin is unbounded
    pub bins: Vec<f64>,
    pub counts: Vec<u64>,
    pub max: f64,
    /// Inexact samples whose decoded state was inferred
    pub inexact_on_inferred: u64,
    /// First samples farther than 1/32 unit, as (entry, frame number, time, distance)
    pub outliers: Vec<(u32, i32, f32, f64)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CrossCheckReport {
    pub schema: u32,
    pub path: String,
    /// Overall verdict, see [`Outcome`]
    pub outcome: Outcome,
    /// Why the outcome is not `verified`
    pub reasons: Vec<String>,
    pub decode_error: Option<String>,
    /// Set when the recording's protocol is not supported, so nothing was
    /// decoded or compared
    pub unsupported: Option<String>,
    pub inferred_delta_bases: bool,
    /// Frames skipped because the client was spectating or in
    /// intermission, where recorded view values follow another entity
    pub skipped_spectator_frames: u64,
    /// Playback records before the first playback network packet, when only
    /// loading-segment state exists to compare against
    pub skipped_before_first_packet: u64,
    /// Frames of a level change inside the recording, from a new
    /// `svc_serverinfo` to the new session's first full entity update. The
    /// client keeps recording old-session values while it signs on
    pub skipped_transition_frames: u64,
    /// ClientData records whose origin was not compared because the view
    /// was set to another entity (camera), so they hold its position
    pub skipped_camera_view_records: u64,
    pub exact: Vec<ExactCheck>,
    pub distance: Vec<DistanceCheck>,
}

impl CrossCheckReport {
    /// Compute `outcome` and `reasons` from the collected checks
    fn judge(&mut self) {
        let mut failed = Vec::new();
        let mut inconclusive = Vec::new();

        if let Some(e) = &self.decode_error {
            failed.push(format!("decode stopped: {e}"));
        }

        for c in &self.exact {
            if c.kind == CheckKind::ClientView {
                continue;
            }

            let disagree = c.compared - (c.match_either + c.match_lagged);
            let proven = disagree - c.misses_on_inferred;
            if proven > 0 {
                failed.push(format!("{}: {proven} of {} disagree", c.name, c.compared));
            }
            if c.misses_on_inferred > 0 {
                inconclusive.push(format!(
                    "{}: {} of {} disagree on inferred state",
                    c.name, c.misses_on_inferred, c.compared
                ));
            }
        }

        for c in &self.distance {
            if c.kind != CheckKind::ServerCopy {
                continue;
            }

            let inexact = c.compared - c.counts[0];
            let proven = inexact - c.inexact_on_inferred;
            if proven > 0 {
                failed.push(format!(
                    "{}: {proven} of {} not exact (max {:.4})",
                    c.name, c.compared, c.max
                ));
            }
            if c.inexact_on_inferred > 0 {
                inconclusive.push(format!(
                    "{}: {} of {} not exact on inferred state",
                    c.name, c.inexact_on_inferred, c.compared
                ));
            }
        }

        if let Some(u) = &self.unsupported {
            inconclusive.push(u.clone());
        }
        if self.inferred_delta_bases {
            inconclusive.push("some delta bases were inferred, not proven".to_owned());
        }
        if self.server_copy_agreement().1 == 0 && self.unsupported.is_none() {
            inconclusive.push("no server-copy samples to compare".to_owned());
        }

        self.outcome = if !failed.is_empty() {
            Outcome::Failed
        } else if !inconclusive.is_empty() {
            Outcome::Inconclusive
        } else {
            Outcome::Verified
        };
        failed.extend(inconclusive);
        self.reasons = failed;
    }

    /// Identity checks with any sample that matched in neither alignment
    pub fn identity_failures(&self) -> Vec<&ExactCheck> {
        self.exact
            .iter()
            .filter(|c| {
                c.kind == CheckKind::Identity && c.match_either + c.match_lagged < c.compared
            })
            .collect()
    }

    /// Server-copy samples that agreed / were compared, over all such checks
    pub fn server_copy_agreement(&self) -> (u64, u64) {
        let exact = self
            .exact
            .iter()
            .filter(|c| c.kind == CheckKind::ServerCopy)
            .map(|c| (c.match_either + c.match_lagged, c.compared));
        let dist = self
            .distance
            .iter()
            .filter(|c| c.kind == CheckKind::ServerCopy)
            .map(|c| (c.counts[0], c.compared));

        exact
            .chain(dist)
            .fold((0, 0), |(a, b), (x, y)| (a + x, b + y))
    }
}

const BINS: [f64; 6] = [0., 1. / 128., 1. / 32., 0.5, 4., 64.];

impl DistanceCheck {
    fn new(name: &'static str, kind: CheckKind) -> Self {
        Self {
            name,
            kind,
            compared: 0,
            bins: BINS.to_vec(),
            counts: vec![0; BINS.len() + 1],
            max: 0.,
            inexact_on_inferred: 0,
            outliers: Vec::new(),
        }
    }

    /// Record one distance. `inferred` marks a sample compared against
    /// inferred decoded state
    fn add(&mut self, at: &FrameLocation, d: f64, inferred: bool) {
        if d > 0. && inferred {
            self.inexact_on_inferred += 1;
        }

        if d > 1. / 32. && self.outliers.len() < MAX_EXAMPLES {
            self.outliers
                .push((at.entry, at.frame_number, at.time(), d));
        }

        self.compared += 1;
        self.max = self.max.max(d);

        let i = BINS.iter().position(|&b| d <= b).unwrap_or(BINS.len());
        self.counts[i] += 1;
    }
}

struct Exact {
    check: ExactCheck,
}

impl Exact {
    fn new(name: &'static str, kind: CheckKind) -> Self {
        Self {
            check: ExactCheck {
                name,
                kind,
                compared: 0,
                match_before: 0,
                match_after: 0,
                match_either: 0,
                match_lagged: 0,
                misses_on_inferred: 0,
                examples: Vec::new(),
            },
        }
    }

    fn add<T: PartialEq + std::fmt::Debug>(
        &mut self,
        at: &FrameLocation,
        recorded: T,
        before: Option<T>,
        after: Option<T>,
    ) {
        self.add_lagged(at, recorded, before, after, [None, None], false);
    }

    /// Like `add`, also accepting states up to two packets before `before`.
    /// `inferred` marks a sample compared against inferred decoded state
    fn add_lagged<T: PartialEq + std::fmt::Debug>(
        &mut self,
        at: &FrameLocation,
        recorded: T,
        before: Option<T>,
        after: Option<T>,
        older: [Option<T>; 2],
        inferred: bool,
    ) {
        if before.is_none() && after.is_none() {
            return;
        }

        let c = &mut self.check;
        c.compared += 1;

        let b = before.as_ref() == Some(&recorded);
        let a = after.as_ref() == Some(&recorded);
        c.match_before += u64::from(b);
        c.match_after += u64::from(a);
        c.match_either += u64::from(a || b);

        let lagged = !(a || b) && older.iter().any(|o| o.as_ref() == Some(&recorded));
        c.match_lagged += u64::from(lagged);
        c.misses_on_inferred += u64::from(!(a || b || lagged) && inferred);

        if !(a || b || lagged) && c.examples.len() < MAX_EXAMPLES {
            c.examples.push(Example {
                entry: at.entry,
                frame_number: at.frame_number,
                time: at.time(),
                recorded: format!("{recorded:?}"),
                before: format!("{before:?}"),
                after: format!("{after:?}"),
            });
        }
    }
}

fn dist(a: [f32; 3], b: [f64; 3]) -> f64 {
    let d: f64 = (0..3).map(|i| (f64::from(a[i]) - b[i]).powi(2)).sum();
    d.sqrt()
}

/// Converted clientdata field of a snapshot
fn cd(d: &Decoder, s: Option<&Arc<Snapshot>>, name: &str) -> Option<f64> {
    let schema = d.schema("clientdata_t")?;
    s?.clientdata.as_ref()?.value(&schema, name)?.as_f64()
}

fn cd_vec(d: &Decoder, s: Option<&Arc<Snapshot>>, base: &str) -> Option<[f64; 3]> {
    Some([
        cd(d, s, &format!("{base}[0]"))?,
        cd(d, s, &format!("{base}[1]"))?,
        cd(d, s, &format!("{base}[2]"))?,
    ])
}

/// Physics part of `movevars_t` from demo info compared field by field with
/// the `svc_newmovevars` body (16 floats, footsteps, rollangle, rollspeed,
/// 6 sky floats, sky name)
fn movevars_equal(info: &[u8], msg: &[u8]) -> bool {
    if msg.len() < 97 {
        return false;
    }

    let footsteps_info = i32::from_le_bytes(info[64..68].try_into().unwrap());
    info[0..64] == msg[0..64]
        && (footsteps_info != 0) == (msg[64] != 0)
        && info[100..132] == msg[65..97]
}

/// Sky names of demo info `movevars_t` and `svc_newmovevars`
fn sky_names(info: &[u8], msg: &[u8]) -> (String, String) {
    (
        crate::container::c_string_lossy(&info[68..100]),
        crate::container::c_string_lossy(msg.get(97..).unwrap_or_default()),
    )
}

/// Decoder state around one network packet
struct PacketContext<'a> {
    before: Option<&'a Arc<Snapshot>>,
    after: Option<&'a Arc<Snapshot>>,
    view_before: Option<u16>,
    movevars_before: Option<&'a [u8]>,
}

struct Pending {
    location: FrameLocation,
    record: [u8; 32],
}

struct Checks {
    /// Client fov recorded while clientdata fov is 0 ("use default")
    default_fov: Option<u32>,
    /// ClientData records taken while viewing through another entity
    camera_view_records: u64,
    maxclients: Exact,
    playernum: Exact,
    viewentity: Exact,
    movevars: Exact,
    sky_name: Exact,
    health: Exact,
    waterlevel: Exact,
    onground: Exact,
    viewheight: Exact,
    weapon_bits: Exact,
    fov: Exact,
    simorg: DistanceCheck,
    simvel: DistanceCheck,
    record_origin: DistanceCheck,
}

impl Checks {
    fn new() -> Self {
        Self {
            default_fov: None,
            camera_view_records: 0,
            maxclients: Exact::new(
                "demo info max_clients = svc_serverinfo max_clients",
                CheckKind::Identity,
            ),
            playernum: Exact::new(
                "demo info playernum = svc_serverinfo player_num",
                CheckKind::Identity,
            ),
            viewentity: Exact::new(
                "demo info viewentity = svc_setview entity",
                CheckKind::Identity,
            ),
            movevars: Exact::new(
                "demo info movevars physics = svc_newmovevars",
                CheckKind::Identity,
            ),
            sky_name: Exact::new(
                "demo info sky name = svc_newmovevars sky name",
                CheckKind::ClientView,
            ),
            health: Exact::new(
                "demo info health = clientdata health",
                CheckKind::ClientView,
            ),
            waterlevel: Exact::new(
                "demo info waterlevel = clientdata waterlevel",
                CheckKind::ClientView,
            ),
            onground: Exact::new(
                "demo info onground = clientdata flags & FL_ONGROUND",
                CheckKind::ClientView,
            ),
            viewheight: Exact::new(
                "demo info viewheight[2] = clientdata view_ofs[2]",
                CheckKind::ClientView,
            ),
            weapon_bits: Exact::new(
                "ClientData weapon bits = clientdata weapons",
                CheckKind::ServerCopy,
            ),
            fov: Exact::new(
                "ClientData fov = clientdata fov (0 = client default; up to two packets of lag allowed)",
                CheckKind::ClientView,
            ),
            simorg: DistanceCheck::new(
                "demo info simorg ~ clientdata origin",
                CheckKind::ClientView,
            ),
            simvel: DistanceCheck::new(
                "demo info simvel ~ clientdata velocity",
                CheckKind::ClientView,
            ),
            record_origin: DistanceCheck::new(
                "ClientData origin ~ clientdata origin",
                CheckKind::ServerCopy,
            ),
        }
    }

    fn client_record(
        &mut self,
        d: &Decoder,
        p: &Pending,
        before: Option<&Arc<Snapshot>>,
        after: Option<&Arc<Snapshot>>,
        older: [Option<&Arc<Snapshot>>; 2],
    ) {
        let r = ClientDataRecord(&p.record);
        let at = &p.location;
        let inferred = [before, after].iter().flatten().any(|s| s.inferred);

        self.weapon_bits.add_lagged(
            at,
            i64::from(r.weapon_bits()),
            cd(d, before, "weapons").map(|v| v as i64 as i32 as i64),
            cd(d, after, "weapons").map(|v| v as i64 as i32 as i64),
            [None, None],
            inferred,
        );

        let recorded_fov = r.fov().to_bits();
        if cd(d, before, "fov") == Some(0.) && self.default_fov.is_none() {
            self.default_fov = Some(recorded_fov);
        }

        let default = self.default_fov;
        let fov = |s| {
            cd(d, s, "fov").and_then(|v| {
                if v == 0. {
                    default
                } else {
                    Some((v as f32).to_bits())
                }
            })
        };
        self.fov.add_lagged(
            at,
            recorded_fov,
            fov(before),
            fov(after),
            [fov(older[0]), fov(older[1])],
            false,
        );

        // While the view is set to another entity (a camera), the record
        // holds that entity's position, not the player's
        let own_view = d
            .view_entity
            .is_none_or(|v| v == u16::from(d.player_num) + 1);
        if !own_view {
            self.camera_view_records += 1;
            return;
        }

        let best = [cd_vec(d, before, "origin"), cd_vec(d, after, "origin")]
            .into_iter()
            .flatten()
            .map(|o| dist(r.origin(), o))
            .min_by(f64::total_cmp);
        if let Some(b) = best {
            self.record_origin.add(at, b, inferred);
        }
    }

    fn network(
        &mut self,
        d: &Decoder,
        at: &FrameLocation,
        n: &NetworkFrame,
        ctx: PacketContext<'_>,
    ) {
        let PacketContext {
            before,
            after,
            view_before,
            movevars_before,
        } = ctx;

        // Identities that do not depend on timing
        if d.max_clients > 0 {
            let mc = Some(i64::from(d.max_clients));
            self.maxclients
                .add(at, i64::from(n.info_maxclients()), mc, mc);

            let pn = Some(i64::from(d.player_num));
            self.playernum
                .add(at, i64::from(n.info_playernum()), pn, pn);
        }

        if d.view_entity.is_some() {
            let ve = |v: Option<u16>| v.map(i64::from);
            self.viewentity.add(
                at,
                i64::from(n.info_viewentity()),
                ve(view_before.or(d.view_entity)),
                ve(d.view_entity),
            );
        }

        if let Some(m) = &d.movevars {
            let info = n.info_movevars();
            let prior = movevars_before.unwrap_or(m);
            let eq = |mv: &[u8]| Some(movevars_equal(info, mv));
            self.movevars.add(at, true, eq(prior), eq(m));

            let (rec, sent) = sky_names(info, m);
            let (_, sent_before) = sky_names(info, prior);
            self.sky_name.add(at, rec, Some(sent_before), Some(sent));
        }

        let int = |s, name| cd(d, s, name).map(|v| v as i64);
        self.health.add(
            at,
            i64::from(n.info_health()),
            int(before, "health"),
            int(after, "health"),
        );
        self.waterlevel.add(
            at,
            i64::from(n.info_waterlevel()),
            int(before, "waterlevel"),
            int(after, "waterlevel"),
        );

        let on = |s| int(s, "flags").map(|f| i64::from(f & FL_ONGROUND != 0));
        self.onground
            .add(at, i64::from(n.info_onground() != 0), on(before), on(after));

        let vh = |s| cd(d, s, "view_ofs[2]").map(|v| (v as f32).to_bits());
        self.viewheight
            .add(at, n.info_viewheight()[2].to_bits(), vh(before), vh(after));

        for (check, rec, field) in [
            (&mut self.simorg, n.info_simorg(), "origin"),
            (&mut self.simvel, n.info_simvel(), "velocity"),
        ] {
            let best = [cd_vec(d, before, field), cd_vec(d, after, field)]
                .into_iter()
                .flatten()
                .map(|o| dist(rec, o))
                .min_by(f64::total_cmp);
            if let Some(b) = best {
                check.add(at, b, false);
            }
        }
    }
}

/// Decode `path` and run all cross-checks
pub fn crosscheck_demo(path: &Path, limits: Limits) -> CrossCheckReport {
    crosscheck_demo_until(path, limits, &AtomicBool::new(false))
}

/// [`crosscheck_demo`] that stops between records once `stop` is set. The
/// report it then gives fails, with a decode error that says so
pub fn crosscheck_demo_until(path: &Path, limits: Limits, stop: &AtomicBool) -> CrossCheckReport {
    let mut report = CrossCheckReport {
        schema: CROSSCHECK_SCHEMA,
        path: path.display().to_string(),
        outcome: Outcome::Failed,
        reasons: Vec::new(),
        decode_error: None,
        unsupported: None,
        inferred_delta_bases: false,
        skipped_spectator_frames: 0,
        skipped_before_first_packet: 0,
        skipped_transition_frames: 0,
        skipped_camera_view_records: 0,
        exact: Vec::new(),
        distance: Vec::new(),
    };

    let mut checks = Checks::new();
    let mut d = Decoder::new();
    d.collect_events = false;

    let reader = File::open(path)
        .map_err(|e| e.to_string())
        .and_then(|f| DemoReader::new(BufReader::new(f), limits).map_err(|e| e.to_string()));
    let mut reader = match reader {
        Ok(r) => r,
        Err(e) => {
            report.decode_error = Some(e);
            report.judge();
            return report;
        }
    };

    if let Err(e) = profile::check(&reader.header) {
        report.unsupported = Some(e.to_string());
        report.judge();
        return report;
    }

    let mut pending: Option<Pending> = None;
    let mut playback_started = false;
    let mut in_transition = false;

    // Packetless ClientData records awaiting the next network packet, stored
    // as (record, latest snapshot, two older snapshots)
    type Deferred = (Pending, Option<Arc<Snapshot>>, [Option<Arc<Snapshot>>; 2]);
    let mut deferred: Vec<Deferred> = Vec::new();

    // Snapshot that was latest before the previous network packet
    let mut older: [Option<Arc<Snapshot>>; 2] = [None, None];

    'entries: for entry in 0..reader.directory.len() as u32 {
        let frames = match reader.frames(entry) {
            Ok(f) => f,
            Err(e) => {
                report.decode_error = Some(e.to_string());
                break;
            }
        };

        for item in frames {
            if stop.load(Ordering::Relaxed) {
                report.decode_error = Some("stopped".into());
                break 'entries;
            }
            let lf = match item {
                Ok(lf) => lf,
                Err(e) => {
                    report.decode_error = Some(e.to_string());
                    break 'entries;
                }
            };

            let at = FrameLocation {
                entry,
                ordinal: lf.ordinal,
                offset: lf.offset,
                time_bits: lf.frame.time_bits,
                frame_number: lf.frame.frame_number,
            };

            // A ClientData record waits for a network packet of its frame,
            // and any other frame number resolves it against the latest state
            if let Some(p) = &pending
                && p.location.frame_number != at.frame_number
            {
                if in_transition {
                    report.skipped_transition_frames += 1;
                } else if playback_started {
                    deferred.push((
                        Pending {
                            location: p.location,
                            record: p.record,
                        },
                        d.latest_snapshot().cloned(),
                        older.clone(),
                    ));
                } else {
                    report.skipped_before_first_packet += 1;
                }

                pending = None;
            }

            match &lf.frame.body {
                FrameBody::ClientData(rec) if entry > 0 => {
                    pending = Some(Pending {
                        location: at,
                        record: *rec,
                    });
                }
                FrameBody::Network(n) => {
                    let before = d.latest_snapshot().cloned();
                    let older_now = older.clone();
                    older = [before.clone(), older[0].clone()];
                    let view_before = d.view_entity;
                    let movevars_before = d.movevars.clone();
                    let (segment, full, snaps) =
                        (d.segment, d.stats.full_packets, d.stats.snapshots);

                    if let Err(e) = d.network_frame(at, n, |_, _| {}) {
                        report.decode_error = Some(e.to_string());
                        break 'entries;
                    }
                    let after = d.latest_snapshot().cloned();

                    if entry == 0 {
                        continue;
                    }

                    let was_in_transition = in_transition;
                    let new_session = playback_started && d.segment != segment;
                    if new_session {
                        in_transition = true;
                    }

                    // The new session's first full update can arrive in the
                    // same packet as its serverinfo
                    if in_transition && d.stats.full_packets > full {
                        in_transition = false;
                    }

                    if was_in_transition || in_transition || new_session {
                        // Packetless records just before a level change were
                        // already written with the client's reset state
                        report.skipped_transition_frames += 1 + deferred.len() as u64;
                        deferred.clear();
                        pending = None;
                        continue;
                    }

                    // Empty packets (common while a level loads) keep
                    // records deferred until a packet carries state
                    let updated = d.stats.snapshots > snaps;
                    if updated {
                        for (p, latest, old) in deferred.drain(..) {
                            checks.client_record(
                                &d,
                                &p,
                                latest.as_ref(),
                                latest.as_ref(),
                                [old[0].as_ref(), old[1].as_ref()],
                            );
                        }
                    }

                    let first = !playback_started;
                    playback_started = true;
                    if first {
                        // The record pending now saw only loading state
                        if pending.take().is_some() {
                            report.skipped_before_first_packet += 1;
                        }
                    }

                    if n.info_spectator() != 0 || n.info_intermission() != 0 || d.hltv {
                        report.skipped_spectator_frames += 1;
                        pending = None;
                        continue;
                    }

                    if let Some(p) = pending.take() {
                        if updated {
                            checks.client_record(
                                &d,
                                &p,
                                before.as_ref(),
                                after.as_ref(),
                                [older_now[0].as_ref(), older_now[1].as_ref()],
                            );
                        } else {
                            deferred.push((p, before.clone(), older_now.clone()));
                        }
                    }

                    checks.network(
                        &d,
                        &at,
                        n,
                        PacketContext {
                            before: before.as_ref(),
                            after: after.as_ref(),
                            view_before,
                            movevars_before: movevars_before.as_deref(),
                        },
                    );
                }
                _ => {}
            }
        }
    }

    for (p, latest, old) in deferred.drain(..) {
        checks.client_record(
            &d,
            &p,
            latest.as_ref(),
            latest.as_ref(),
            [old[0].as_ref(), old[1].as_ref()],
        );
    }

    report.inferred_delta_bases = d.stats.inferred_snapshots > 0;
    report.skipped_camera_view_records = checks.camera_view_records;

    let c = checks;
    report.exact = [
        c.maxclients,
        c.playernum,
        c.viewentity,
        c.movevars,
        c.sky_name,
        c.health,
        c.waterlevel,
        c.onground,
        c.viewheight,
        c.weapon_bits,
        c.fov,
    ]
    .into_iter()
    .map(|e| e.check)
    .collect();
    report.distance = vec![c.simorg, c.simvel, c.record_origin];

    report.judge();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> FrameLocation {
        FrameLocation {
            entry: 1,
            ordinal: 0,
            offset: 0,
            time_bits: 0,
            frame_number: 0,
        }
    }

    #[test]
    fn movevars_compare_field_by_field() {
        // demo info layout is 16 floats, int footsteps, 32-byte sky name, 8 floats
        let mut info = vec![0u8; 132];
        let mut msg = vec![0u8; 97];
        for i in 0..64 {
            info[i] = i as u8;
            msg[i] = i as u8;
        }

        info[64] = 1;
        msg[64] = 1;

        info[68..74].copy_from_slice(b"desert");
        msg.extend_from_slice(b"desert\0");

        for i in 0..32 {
            info[100 + i] = 200 - i as u8;
            msg[65 + i] = 200 - i as u8;
        }

        assert!(movevars_equal(&info, &msg));

        let mut other = msg.clone();
        other[3] ^= 1;
        assert!(!movevars_equal(&info, &other));

        let mut sky = msg.clone();
        sky[97] = b'D';
        assert!(movevars_equal(&info, &sky));
        assert_eq!(sky_names(&info, &sky), ("desert".into(), "Desert".into()));
    }

    #[test]
    fn exact_checks_count_both_alignments() {
        let mut e = Exact::new("x", CheckKind::Identity);
        e.add_lagged(&loc(), 5, Some(1), Some(2), [None, Some(5)], false);
        e.add_lagged(&loc(), 6, Some(1), Some(2), [None, None], true);
        e.add(&loc(), 1, Some(1), Some(2));
        e.add(&loc(), 2, Some(1), Some(2));
        e.add(&loc(), 3, Some(1), Some(2));
        e.add(&loc(), 4, None, None);

        let c = e.check;
        assert_eq!(
            (c.compared, c.match_before, c.match_after, c.match_either),
            (5, 1, 1, 2)
        );
        assert_eq!(c.match_lagged, 1);
        assert_eq!(c.misses_on_inferred, 1);
        assert_eq!(c.examples.len(), 2);
    }

    fn report(exact: Vec<ExactCheck>, inferred: bool) -> CrossCheckReport {
        CrossCheckReport {
            schema: CROSSCHECK_SCHEMA,
            path: String::new(),
            outcome: Outcome::Failed,
            reasons: Vec::new(),
            decode_error: None,
            unsupported: None,
            inferred_delta_bases: inferred,
            skipped_spectator_frames: 0,
            skipped_before_first_packet: 0,
            skipped_transition_frames: 0,
            skipped_camera_view_records: 0,
            exact,
            distance: Vec::new(),
        }
    }

    fn check(kind: CheckKind, agree: u64, compared: u64) -> ExactCheck {
        let mut e = Exact::new("x", kind).check;
        e.compared = compared;
        e.match_either = agree;
        e
    }

    #[test]
    fn outcome_rules() {
        let mut r = report(vec![check(CheckKind::ServerCopy, 10, 10)], false);
        r.judge();
        assert_eq!(r.outcome, Outcome::Verified);

        // Client-view disagreement never matters
        let mut r = report(
            vec![
                check(CheckKind::ServerCopy, 10, 10),
                check(CheckKind::ClientView, 1, 10),
            ],
            false,
        );
        r.judge();
        assert_eq!(r.outcome, Outcome::Verified);

        // Inferred bases with full agreement are inconclusive
        let mut r = report(vec![check(CheckKind::ServerCopy, 10, 10)], true);
        r.judge();
        assert_eq!(r.outcome, Outcome::Inconclusive);

        // Nothing compared is inconclusive
        let mut r = report(vec![check(CheckKind::ServerCopy, 0, 0)], false);
        r.judge();
        assert_eq!(r.outcome, Outcome::Inconclusive);

        // Disagreement only on inferred state is inconclusive, not failed
        let mut miss = check(CheckKind::ServerCopy, 8, 10);
        miss.misses_on_inferred = 2;
        let mut r = report(vec![miss], true);
        r.judge();
        assert_eq!(r.outcome, Outcome::Inconclusive);
        assert_eq!(r.reasons.len(), 2);

        // Any identity or proven server-copy disagreement fails, even when
        // other state was inferred
        for kind in [CheckKind::Identity, CheckKind::ServerCopy] {
            let mut r = report(
                vec![check(kind, 9, 10), check(CheckKind::ServerCopy, 5, 5)],
                true,
            );
            r.judge();
            assert_eq!(r.outcome, Outcome::Failed);
            assert_eq!(r.reasons.len(), 2);
        }

        let mut r = report(vec![check(CheckKind::ServerCopy, 10, 10)], false);
        r.decode_error = Some("stopped".into());
        r.judge();
        assert_eq!(r.outcome, Outcome::Failed);
    }

    #[test]
    fn distance_bins() {
        let mut d = DistanceCheck::new("x", CheckKind::ServerCopy);
        for v in [0., 0.005, 0.02, 0.3, 2., 50., 100.] {
            d.add(&loc(), v, false);
        }

        assert_eq!(d.counts, vec![1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(d.outliers.len(), 4);
        assert_eq!(d.max, 100.);
    }
}
