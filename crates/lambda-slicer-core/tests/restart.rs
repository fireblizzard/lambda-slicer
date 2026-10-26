//! Restart exports of synthetic demos whose packets decode

mod common;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use common::stream::{
    CLIENT, ENTITY, EVENT_RESOURCE, SOUND_RESOURCE, delta, full_packet, setup_on, setup_with, time,
};
use common::{build, frame};
use lambda_slicer_core::container::{
    DemoReader, DirectoryEntry, EventRecord, FrameBody, Limits, NetworkFrame, tag,
};
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::delta::Value;
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_core::slice::ExportError;
use lambda_slicer_core::slice::restart::{nearest_start, run_restart, run_restart_until};
use lambda_slicer_core::state::{DecodeOptions, DecodeStatus, decode_demo};
use lambda_slicer_core::timeline::time::DemoTime;

/// Resources of the synthetic session, as (type, name, resource index)
const RESOURCES: &[(u32, &str, u32)] = &[
    (EVENT_RESOURCE, "events/gauss.sc", 3),
    (EVENT_RESOURCE, "events/gaussspin.sc", 4),
    (EVENT_RESOURCE, "events/egon_fire.sc", 5),
    (EVENT_RESOURCE, "events/egon_stop.sc", 6),
    (EVENT_RESOURCE, "events/mystery.sc", 7),
    (SOUND_RESOURCE, "doors/doormove1.wav", 20),
    (SOUND_RESOURCE, "common/null.wav", 21),
];

/// Seconds between playback groups
const STEP: f32 = 0.1;

/// The first sequence of playback, carried by the full update
const FIRST: i32 = 10;

fn entry(entry_type: i32, desc: &str) -> DirectoryEntry {
    let mut description_raw = [0u8; 64];
    description_raw[..desc.len()].copy_from_slice(desc.as_bytes());

    DirectoryEntry {
        entry_type,
        description_raw,
        flags: 0,
        cd_track: -1,
        track_time_bits: 0,
        frame_count: 0,
        offset: 0,
        length: 0,
    }
}

/// A network record whose demo info matches the synthetic server info
fn network(
    tag: u8,
    time: f32,
    frame_number: i32,
    sequence: i32,
    payload: Vec<u8>,
) -> lambda_slicer_core::container::Frame {
    let mut demo_info = Box::new([0u8; 436]);
    demo_info[..4].copy_from_slice(&time.to_le_bytes());
    demo_info[176..180].copy_from_slice(&2i32.to_le_bytes());

    frame(
        tag,
        time,
        frame_number,
        FrameBody::Network(Box::new(NetworkFrame {
            demo_info,
            sequence: [sequence, sequence - 1, 0, 0, sequence, 0, 0],
            message: payload,
        })),
    )
}

/// Clientdata health and entity 1's model, both as deltas from `from`
fn step(seconds: f32, from: u8, value: i64, extra: &[u8], animtime: Option<i64>) -> Vec<u8> {
    let mut w = BitWriter::new();
    time(&mut w, 10. + seconds);
    w.bytes(extra);

    w.u8(svc::CLIENTDATA);
    w.bit(true);
    w.bits(u32::from(from), 8);
    delta(&mut w, CLIENT, &[(0, value % 100)]);
    w.bit(false);
    w.align();

    w.u8(svc::DELTAPACKETENTITIES);
    w.u16(3);
    w.u8(from);
    w.bit(false);
    w.bit(true);
    w.bits(1, 11);
    w.bit(false);
    match animtime {
        Some(offset) => delta(&mut w, ENTITY, &[(1, value), (2, offset)]),
        None => delta(&mut w, ENTITY, &[(1, value)]),
    }
    w.bits(0, 16);

    w.finish()
}

/// Playback groups, each with the delta reference and extra messages of
/// its packet. Group 0 is the full update
struct Recording {
    groups: Vec<(u8, Vec<u8>)>,
    /// Weapon bits in the client's ClientData records
    recorded_weapons: u32,
    /// Demo records written before the network record of a group, by group
    records: Vec<(i32, FrameBody)>,
    /// Groups that also set entity 1's animtime, as (group, offset back
    /// from the packet's server time in hundredths)
    animtimes: Vec<(i32, i64)>,
    /// A group where a new session sets itself up, as after a level change,
    /// with its first full update in the next group
    session_at: Option<i32>,
    /// The new session numbers its packets from [`FIRST`] again, as after a
    /// reconnect, reusing sequences the previous session sent
    sequence_reset: bool,
    /// The new session's clock starts over, so its groups have the times of
    /// the groups [`CLOCK_BACK`] before them
    clock_reset: bool,
}

/// How many groups back a clock that starts over takes its times from
const CLOCK_BACK: i32 = 20;

impl Recording {
    /// `count` groups, each referencing the packet `lag` sequences back
    fn steady(count: i32, lag: i32) -> Self {
        let groups = (1..count)
            .map(|k| (((FIRST + k - lag).max(FIRST)) as u8, Vec::new()))
            .collect();

        Self {
            groups,
            recorded_weapons: 1 << 9,
            records: Vec::new(),
            animtimes: Vec::new(),
            session_at: None,
            sequence_reset: false,
            clock_reset: false,
        }
    }

    fn bytes(&self) -> Vec<u8> {
        let loading = vec![
            network(tag::NETWORK_START, 3., 100, 5, setup_with(48, RESOURCES)),
            frame(tag::NEXT_SECTION, 3.1, 101, FrameBody::NextSection),
        ];

        // The client's own copy of the weapon bits the full update sets
        let mut client = [0u8; 32];
        client[24..28].copy_from_slice(&self.recorded_weapons.to_le_bytes());

        let mut playback = vec![frame(tag::DEMO_START, 0., 0, FrameBody::DemoStart)];
        let records = &self.records;
        let session = self.session_at.filter(|_| self.sequence_reset);
        let sequence = |n: i32| match session {
            Some(s) if n >= s => FIRST + n - s,
            _ => FIRST + n,
        };
        let clock = self.session_at.filter(|_| self.clock_reset);
        let time_of = |n: i32| match clock {
            Some(s) if n >= s => (n - CLOCK_BACK) as f32 * STEP,
            _ => n as f32 * STEP,
        };
        let mut group = |t: f32, n: i32, payload: Vec<u8>| {
            playback.push(frame(
                tag::DEMO_BUFFER,
                t,
                n,
                FrameBody::DemoBuffer(vec![n as u8; 8]),
            ));
            playback.push(frame(tag::CLIENT_DATA, t, n, FrameBody::ClientData(client)));
            for (_, body) in records.iter().filter(|(g, _)| *g == n) {
                let tag = match body {
                    FrameBody::WeaponAnim { .. } => tag::WEAPON_ANIM,
                    _ => tag::EVENT,
                };
                playback.push(frame(tag, t, n, body.clone()));
            }
            playback.push(network(tag::NETWORK, t, n, sequence(n), payload));
        };

        group(0., 0, full_packet(FIRST));
        for (k, (from, extra)) in self.groups.iter().enumerate() {
            let n = k as i32 + 1;
            let t = time_of(n);
            let animtime = self
                .animtimes
                .iter()
                .find(|(g, _)| *g == n)
                .map(|(_, o)| *o);
            let payload = match self.session_at {
                // The setup ends with the sign-on, as a real one does
                Some(s) if s == n => [
                    setup_on(48, RESOURCES, b"maps/second.bsp"),
                    vec![svc::SIGNONNUM, 1],
                ]
                .concat(),
                Some(s) if s + 1 == n => full_packet(sequence(n)),
                Some(s) if self.sequence_reset && n > s + 1 => {
                    let from = (sequence(n) - 1) as u8;
                    step(t, from, i64::from(n), extra, animtime)
                }
                _ => step(t, *from, i64::from(n), extra, animtime),
            };
            group(t, n, payload);
        }

        let end = time_of(self.groups.len() as i32 + 1);
        playback.push(frame(
            tag::NEXT_SECTION,
            end,
            self.groups.len() as i32 + 1,
            FrameBody::NextSection,
        ));

        build(&[
            (entry(0, "LOADING"), loading),
            (entry(1, "Playback"), playback),
        ])
    }
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, bytes).unwrap();
    p
}

fn at(group: i32) -> DemoTime {
    DemoTime::from_f32(group as f32 * STEP).unwrap()
}

fn slice(
    rec: &Recording,
    start: i32,
    end: Option<i32>,
) -> Result<
    (
        PathBuf,
        tempfile::TempDir,
        lambda_slicer_core::slice::restart::RestartReport,
    ),
    ExportError,
> {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let out = dir.path().join("out.dem");

    let report = run_restart(&src, &out, at(start), end.map(at), Limits::default())?;
    Ok((out, dir, report))
}

fn last_model(path: &Path) -> Option<Value> {
    let (report, d) = decode_demo(path, &DecodeOptions::default(), |_, _, _| {});
    assert_eq!(report.status, DecodeStatus::Complete);

    let snap = d.latest_snapshot()?;
    let schema = d.entity_schema(1, false)?;
    let e = snap.entities.as_ref()?.iter().find(|e| e.number == 1)?;
    e.fields.value(&schema, "modelindex")
}

#[test]
fn a_restart_bootstraps_the_referenced_frame_and_keeps_the_rest() {
    let rec = Recording::steady(60, 1);
    let (out, _dir, r) = slice(&rec, 30, Some(50)).unwrap();

    // Group 30 carries sequence 40, which deltas from 39
    assert_eq!(r.start_sequence, 40);
    assert_eq!(r.bootstrap_sequences, [39]);
    assert_eq!(r.kept_network_records, 20);
    assert_eq!(r.validation.compared_snapshots, 20);
    assert_eq!(r.validation.other_differences, 0);

    // The output ends on group 49's state
    assert_eq!(last_model(&out), Some(Value::Int(49)));
}

#[test]
fn a_clip_is_marked_with_the_demo_it_came_from() {
    let rec = Recording::steady(60, 1);
    let (out, _dir, _) = slice(&rec, 30, Some(50)).unwrap();

    // The game reads the playback entry's description alone, and the mark
    // after it names the source without its extension
    let reader = DemoReader::new(std::fs::File::open(&out).unwrap(), Limits::default()).unwrap();
    let playback = &reader.directory[1];
    assert_eq!(playback.description(), "Playback");
    assert_eq!(playback.clip_of().as_deref(), Some("src"));
    assert_eq!(reader.directory[0].clip_of(), None);
}

#[test]
fn a_stopped_export_leaves_nothing_behind() {
    let rec = Recording::steady(60, 1);
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let out = dir.path().join("out.dem");

    let stop = AtomicBool::new(true);
    let limits = Limits::default();
    let stopped = run_restart_until(&src, &out, at(30), Some(at(50)), None, limits, &stop);
    assert!(matches!(stopped, Err(ExportError::Stopped)));

    // Neither the clip nor the copies it was made from are left
    let left: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(left.len(), 1);
}

#[test]
fn lagging_references_bootstrap_every_frame_still_used() {
    let rec = Recording::steady(60, 3);
    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();

    assert_eq!(r.bootstrap_sequences, [37, 38, 39]);
    assert_eq!(r.validation.other_differences, 0);
}

#[test]
fn a_start_after_inferred_state_is_refused() {
    // Group 20 names byte 200, which no frame carries, so its base is a
    // guess and every later delta inherits it
    let mut rec = Recording::steady(60, 1);
    rec.groups[19].0 = 200;

    let err = slice(&rec, 30, None).unwrap_err();
    assert!(matches!(err, ExportError::UnprovenState(_)), "{err}");
}

#[test]
fn a_range_with_packets_the_client_drops_is_refused() {
    // The acknowledgement stalls at sequence 10, so from sequence 73 the
    // base is 63 or more packets back and the client drops the packets
    let mut rec = Recording::steady(90, 1);
    for g in rec.groups.iter_mut().take(70) {
        g.0 = FIRST as u8;
    }

    let err = slice(&rec, 60, None).unwrap_err();
    assert!(matches!(err, ExportError::ClientDroppedPackets(_)), "{err}");
}

/// `CurWeapon` (registered as message 64 with 2 bytes) with this body
fn cur_weapon(body: [u8; 2]) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(64);
    w.bytes(&body);
    w.finish()
}

#[test]
fn persistent_messages_before_the_start_are_replayed_once_per_key() {
    // Weapon 2 is updated twice and weapon 3 once, so two messages remain
    let mut rec = Recording::steady(60, 1);
    rec.groups[4].1 = cur_weapon([1, 2]);
    rec.groups[9].1 = cur_weapon([1, 3]);
    rec.groups[14].1 = cur_weapon([1, 2]);

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.persistent_messages, 2);
    assert_eq!(r.persistent_bytes, 6);
}

#[test]
fn an_unclassified_persistent_message_is_refused() {
    // A message registered during playback whose effect is not known
    let mut w = BitWriter::new();
    w.u8(svc::NEWUSERMSG);
    w.u8(70);
    w.u8(1);
    let mut name = [0u8; 16];
    name[..7].copy_from_slice(b"Mystery");
    w.bytes(&name);
    w.u8(70);
    w.u8(9);

    let mut rec = Recording::steady(60, 1);
    rec.groups[9].1 = w.finish();

    // Only the name, so the translated sentence around it has no English
    let err = slice(&rec, 30, None).unwrap_err();
    assert!(
        matches!(&err, ExportError::UnknownPersistentMessage(m) if m == "Mystery"),
        "{err}"
    );
}

/// `StatusIcon` with this status and icon name, registered as user message
/// 71 with a body of any length, and a colour when the icon shows
fn status_icon(status: u8, icon: &[u8]) -> Vec<u8> {
    let mut body = vec![status];
    body.extend_from_slice(icon);
    body.push(0);
    if status != 0 {
        body.extend_from_slice(&[255, 160, 0]);
    }

    let mut w = BitWriter::new();
    w.u8(svc::NEWUSERMSG);
    w.u8(71);
    w.u8(255);
    let mut name = [0u8; 16];
    name[..10].copy_from_slice(b"StatusIcon");
    w.bytes(&name);
    w.u8(71);
    w.u8(body.len() as u8);
    w.bytes(&body);
    w.finish()
}

#[test]
fn status_icons_before_the_start_are_replayed_once_per_icon() {
    // The first icon is shown and later hidden, and a second one shown, so
    // one message per icon remains besides the registration
    let mut rec = Recording::steady(60, 1);
    rec.groups[4].1 = status_icon(1, b"dmg_rad");
    rec.groups[9].1 = status_icon(1, b"dmg_poison");
    rec.groups[14].1 = status_icon(0, b"dmg_rad");

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.persistent_messages, 3);
}

#[test]
fn a_start_at_or_past_the_end_is_refused() {
    let rec = Recording::steady(20, 1);

    let err = slice(&rec, 25, None).unwrap_err();
    assert!(matches!(err, ExportError::RangeBeyondEnd { .. }), "{err}");
}

#[test]
fn a_source_that_fails_its_cross_check_is_refused() {
    // The client's records disagree with the weapons the packets give
    let mut rec = Recording::steady(60, 1);
    rec.recorded_weapons = 1 << 3;

    let err = slice(&rec, 30, None).unwrap_err();
    assert!(matches!(err, ExportError::SourceFailedCrossCheck), "{err}");
}

/// A demo event record of the recorder (entity 1)
fn event(index: i32, bparam1: i32, pitch: i32) -> FrameBody {
    let mut args = [0u8; 72];
    args[4..8].copy_from_slice(&1i32.to_le_bytes());
    args[56..60].copy_from_slice(&pitch.to_le_bytes());
    args[64..68].copy_from_slice(&bparam1.to_le_bytes());

    FrameBody::Event(EventRecord {
        flags: 0,
        index,
        delay_bits: 0,
        args,
    })
}

/// A charge from group 25 with spin-up and spin start, pitch updates in
/// every group, and the shot in group 36
fn charging() -> Recording {
    let mut rec = Recording::steady(60, 1);
    rec.records.push((
        25,
        FrameBody::WeaponAnim {
            sequence: 3,
            body: 0,
        },
    ));
    rec.records.push((25, event(4, 0, 110)));
    for g in 26..36 {
        rec.records.push((g, event(4, 1, 110 + g)));
    }
    rec.records.push((36, event(3, 0, 0)));

    rec
}

/// Records of the output's bootstrap group (frame 0 after DemoStart)
fn bootstrap_records(path: &Path) -> Vec<FrameBody> {
    let f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    r.frames(1)
        .unwrap()
        .map(|lf| lf.unwrap().frame)
        .take_while(|f| f.frame_number == 0)
        .map(|f| f.body)
        .filter(|b| !matches!(b, FrameBody::DemoStart | FrameBody::Network(_)))
        .collect()
}

#[test]
fn a_gauss_charge_under_way_at_the_start_is_carried_over() {
    let (out, _dir, r) = slice(&charging(), 30, None).unwrap();
    assert_eq!(r.carried, ["gauss charge (spin-up and spin sound)"]);

    // The spin-up animation, then the latest spin turned into a sound start
    // at its pitch, after the client's state records
    let records = bootstrap_records(&out);
    assert!(matches!(records[0], FrameBody::DemoBuffer(_)));
    assert!(matches!(records[1], FrameBody::ClientData(_)));
    assert_eq!(
        records[2],
        FrameBody::WeaponAnim {
            sequence: 3,
            body: 0
        }
    );
    let FrameBody::Event(spin) = &records[3] else {
        panic!("{:?}", records[3]);
    };
    assert_eq!((spin.index, spin.bparam1(), spin.iparam1()), (4, 0, 139));
    assert_eq!(records.len(), 4);
}

#[test]
fn a_finished_charge_is_not_carried_over() {
    let (out, _dir, r) = slice(&charging(), 40, None).unwrap();
    assert!(r.carried.is_empty(), "{:?}", r.carried);
    assert_eq!(bootstrap_records(&out).len(), 2);
}

#[test]
fn events_just_before_the_start_are_not_replayed() {
    // A shot in the group right before the start already happened
    let mut rec = Recording::steady(60, 1);
    rec.records.push((29, event(3, 0, 0)));

    let (out, _dir, _) = slice(&rec, 30, None).unwrap();
    let records = bootstrap_records(&out);
    assert!(
        records.iter().all(|b| !matches!(b, FrameBody::Event(_))),
        "{records:?}"
    );
}

#[test]
fn the_newest_frame_before_the_start_is_written_even_if_unreferenced() {
    // Kept packets reference 37 and 38, then sequence 42 jumps to the kept
    // frame 40, so nothing names 39. The client still showed 39 last
    let mut rec = Recording::steady(60, 3);
    rec.groups[31].0 = (FIRST + 30) as u8;

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.bootstrap_sequences, [37, 38, 39]);
}

#[test]
fn replayed_hud_messages_arrive_after_the_bootstrap_group() {
    // HUD elements time their display from the HUD clock, which moves only
    // when a frame is drawn, so the HUD record is a later group of its own
    let mut rec = Recording::steady(60, 1);
    rec.groups[4].1 = cur_weapon([1, 2]);

    let (out, _dir, _) = slice(&rec, 30, None).unwrap();
    let f = std::io::BufReader::new(std::fs::File::open(&out).unwrap());
    let mut r = DemoReader::new(f, Limits::default()).unwrap();
    let frames: Vec<_> = r.frames(1).unwrap().map(|lf| lf.unwrap().frame).collect();

    let hud: Vec<_> = frames.iter().filter(|f| f.frame_number == 1).collect();
    assert_eq!(hud.len(), 1);
    let FrameBody::Network(n) = &hud[0].body else {
        panic!("{:?}", hud[0].body);
    };
    assert_eq!(n.message, cur_weapon([1, 2]));

    let first_kept = frames.iter().find(|f| f.frame_number == 2).unwrap();
    assert!(hud[0].time() > 0. && hud[0].time() < first_kept.time());
}

#[test]
fn egon_fire_under_way_is_carried_as_a_startup_event() {
    // The local beam exists only when the startup event runs
    let mut rec = Recording::steady(60, 1);
    for g in 20..40 {
        rec.records.push((g, event(5, i32::from(g == 20), 0)));
    }

    let (out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.carried, ["egon fire (startup event)"]);
    let records = bootstrap_records(&out);
    let FrameBody::Event(e) = records.last().unwrap() else {
        panic!("{records:?}");
    };
    assert_eq!((e.index, e.bparam1()), (5, 1));

    // Once stopped, nothing is carried
    rec.records.push((25, event(6, 0, 0)));
    rec.records
        .retain(|(g, b)| !(*g > 25 && matches!(b, FrameBody::Event(_))));
    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert!(r.carried.is_empty(), "{:?}", r.carried);
}

/// `svc_sound` of `sample` on entity 5's static channel, started or stopped
fn sound(sample: u32, stop: bool) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(svc::SOUND);
    w.bits(if stop { 1 << 5 } else { 0 }, 9);
    w.bits(6, 3);
    w.bits(5, 11);
    w.bits(sample, 8);
    w.bits(0, 3);
    w.finish()
}

#[test]
fn a_sound_loop_stopped_after_the_start_is_started_again() {
    let mut rec = Recording::steady(60, 1);
    rec.groups[19].1 = sound(20, false);
    rec.groups[44].1 = sound(20, true);

    let (_out, _dir, r) = slice(&rec, 30, Some(40)).unwrap();
    assert_eq!(r.carried, ["sound doors/doormove1.wav on entity 5"]);
}

#[test]
fn a_sound_stopped_before_the_start_or_never_is_not_restarted() {
    let mut rec = Recording::steady(60, 1);
    rec.groups[9].1 = sound(20, false);
    rec.groups[14].1 = sound(20, true);
    // A one-shot nothing ever stops
    rec.groups[19].1 = sound(21, false);

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert!(r.carried.is_empty(), "{:?}", r.carried);
}

/// `svc_temp_entity` of `kind` with this body
fn temp(kind: u8, body: &[u8]) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(svc::TEMPENTITY);
    w.u8(kind);
    w.bytes(body);
    w.finish()
}

#[test]
fn a_beam_still_alive_is_sent_again_with_its_remaining_life() {
    // TE_BEAMPOINTS with a 2 s life at group 29, 0.1 s before the start
    let mut body = [0u8; 24];
    body[16] = 20;
    let mut rec = Recording::steady(60, 1);
    rec.groups[28].1 = temp(0, &body);

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.carried, ["temp entity 0 (1.9 s left)"]);

    // Started 2.5 s before the start, it has ended
    rec.groups[28].1 = Vec::new();
    rec.groups[4].1 = temp(0, &body);
    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert!(r.carried.is_empty(), "{:?}", r.carried);
}

#[test]
fn gibs_still_flying_at_the_start_are_left_out_and_reported() {
    // TE_BREAKMODEL with a 2 s life, which moves under physics, at group
    // 29, 0.1 s before the start
    let mut body = [0u8; 24];
    body[22] = 20;
    let mut rec = Recording::steady(60, 1);
    rec.groups[28].1 = temp(108, &body);

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.left_out, ["temp entity 108 (1.9 s left)"]);
    assert!(r.carried.is_empty(), "{:?}", r.carried);

    // Landed 2.5 s before the start, nothing is left out
    rec.groups[28].1 = Vec::new();
    rec.groups[4].1 = temp(108, &body);
    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert!(r.left_out.is_empty(), "{:?}", r.left_out);
}

#[test]
fn an_unknown_event_script_before_the_start_is_refused() {
    let mut rec = Recording::steady(60, 1);
    rec.records.push((10, event(7, 0, 0)));

    let err = slice(&rec, 30, None).unwrap_err();
    assert!(
        matches!(&err, ExportError::ActiveEffect(m) if m.contains("mystery")),
        "{err}"
    );
}

#[test]
fn a_start_refused_for_an_effect_just_before_it_finds_the_nearest_that_works() {
    // An unknown event at group 28 refuses every start after it
    let mut rec = Recording::steady(60, 1);
    rec.records.push((28, event(7, 0, 0)));
    let err = slice(&rec, 30, Some(40)).unwrap_err();
    assert!(err.depends_on_start(), "{err}");

    // Starting at the event's own group keeps it in the clip
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let never = AtomicBool::new(false);
    let limits = Limits::default();
    let nearest = nearest_start(&src, dir.path(), at(30), at(40), None, limits, &never);
    assert_eq!(nearest.unwrap(), Some(at(28)));
    slice(&rec, 28, Some(40)).unwrap();
}

/// Registers `Vote` as user message 70 with a 4-byte body, then sends one
/// with this status
fn vote(status: u8) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(svc::NEWUSERMSG);
    w.u8(70);
    w.u8(4);
    let mut name = [0u8; 16];
    name[..4].copy_from_slice(b"Vote");
    w.bytes(&name);
    w.u8(70);
    w.bytes(&[status, 1, 0, 3]);
    w.finish()
}

#[test]
fn a_vote_still_showing_at_the_start_is_replayed() {
    // OpenAG shows a vote for 4 s after each message. At group 10 it has
    // 2 s left at the start in group 30
    let mut rec = Recording::steady(60, 1);
    rec.groups[9].1 = vote(1);

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.expired_notices, 0);
    // The registration and the vote
    assert_eq!(r.persistent_messages, 2);
}

#[test]
fn an_expired_or_hidden_vote_is_not_replayed() {
    // 4.5 s before the start, the vote has gone
    let mut rec = Recording::steady(60, 1);
    rec.groups[4].1 = vote(1);
    let (_out, _dir, r) = slice(&rec, 50, None).unwrap();
    assert_eq!(r.expired_notices, 1);
    assert_eq!(r.persistent_messages, 1);

    // Status 0 hides it, so nothing shows however recent
    rec.groups[4].1 = Vec::new();
    rec.groups[28].1 = vote(0);
    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.expired_notices, 1);
}

#[test]
fn an_old_animtime_is_clamped_reported_and_explained() {
    // Entity 1's animation started at group 5 and entity 5's with the full
    // update, both too long before the start for the 8-bit window. Later
    // deltas leave them alone, so the output keeps the clamped values
    let mut rec = Recording::steady(60, 1);
    rec.animtimes.push((5, 3));

    let (_out, _dir, r) = slice(&rec, 30, None).unwrap();
    assert_eq!(r.clamped_time_windows, 2);
    let fields: Vec<_> = r
        .approximations
        .iter()
        .map(|a| a.split(" clamped").next().unwrap())
        .collect();
    assert_eq!(fields, ["entity 1 animtime", "entity 5 animtime"]);
    assert_eq!(r.validation.approximated_differences, 2 * 30);
    assert_eq!(r.validation.other_differences, 0);
}

#[test]
fn the_group_index_snaps_ranges_as_the_restart_does() {
    let rec = Recording::steady(60, 1);
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let (report, _) = decode_demo(&src, &DecodeOptions::default(), |_, _, _| {});

    // Between groups, on a group, and to the end of the recording
    let half = |g: f32| DemoTime::from_f32(g * STEP).unwrap();
    let ranges = [
        (half(30.5), Some(half(49.5))),
        (at(30), Some(at(50))),
        (half(12.2), None),
        // The DemoStart group is never a start, so this starts at group 1
        (at(0), Some(at(5))),
    ];
    for (k, (start, end)) in ranges.into_iter().enumerate() {
        let snapped = report.groups.snap(start, end, None).unwrap();
        let out = dir.path().join(format!("out{k}.dem"));
        let r = run_restart(&src, &out, start, end, Limits::default()).unwrap();
        assert_eq!(snapped.start.to_string(), r.effective_start, "{start}");
        assert_eq!(snapped.end.to_string(), r.effective_end, "{start}");
    }

    // The same refusals
    let err = report.groups.snap(at(25), Some(at(80)), None).unwrap_err();
    assert!(matches!(err, ExportError::RangeBeyondEnd { .. }), "{err}");
}

#[test]
fn a_restart_starts_in_a_later_session_with_its_own_setup() {
    // A new session sets up in group 30 and has its full update in group 31
    let mut rec = Recording::steady(60, 1);
    rec.session_at = Some(30);
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let (report, _) = decode_demo(&src, &DecodeOptions::default(), |_, _, _| {});
    assert_eq!(report.groups.ready.get(&2), Some(&32));

    // The full update's group is still loading
    let path = dir.path().join("loading.dem");
    let restart = run_restart(&src, &path, at(31), None, Limits::default()).unwrap_err();
    assert!(
        matches!(restart, ExportError::StartDuringLoading),
        "{restart}"
    );
    let snap = report.groups.snap(at(31), None, None).unwrap_err();
    assert!(matches!(snap, ExportError::StartDuringLoading), "{snap}");

    let out = dir.path().join("later.dem");
    let r = run_restart(&src, &out, at(40), Some(at(50)), Limits::default()).unwrap();
    assert_eq!(r.validation.other_differences, 0);
    assert_eq!(r.kept_network_records, 10);
    let snapped = report.groups.snap(at(40), Some(at(50)), None).unwrap();
    assert_eq!(snapped.start.to_string(), r.effective_start);
    assert_eq!(snapped.end.to_string(), r.effective_end);

    // The output's LOADING entry is the new session's setup, as startup
    // records like a real one, so it decodes as one session ending on group
    // 49's state
    let mut reader =
        DemoReader::new(std::fs::File::open(&out).unwrap(), Limits::default()).unwrap();
    let tags: Vec<u8> = reader
        .frames(0)
        .unwrap()
        .map(|f| f.unwrap().frame.tag)
        .collect();
    assert_eq!(tags.last(), Some(&tag::NEXT_SECTION));
    assert!(
        tags[..tags.len() - 1]
            .iter()
            .all(|t| *t == tag::NETWORK_START),
        "{tags:?}"
    );
    // and its header names the session's own map
    assert_eq!(reader.header.map_name(), "second");
    assert_eq!(reader.header.map_checksum, 0);
    let (out_report, _) = decode_demo(&out, &DecodeOptions::default(), |_, _, _| {});
    assert_eq!(out_report.status, DecodeStatus::Complete);
    assert_eq!(out_report.stats.segments, 1);
    assert_eq!(last_model(&out), Some(Value::Int(49)));

    // Group 32 deltas from the full update, which is not in the range, so it
    // is written as the bootstrap
    let r = run_restart(
        &src,
        &dir.path().join("ready.dem"),
        at(32),
        Some(at(35)),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(r.bootstrap_sequences, [41]);
    assert_eq!(r.validation.other_differences, 0);
}

#[test]
fn a_later_session_that_reuses_sequences_bootstraps_from_its_own_records() {
    // The new session from group 30 numbers its packets from 10 again, so
    // sequence 14 is both group 4's and group 34's
    let mut rec = Recording::steady(60, 1);
    rec.session_at = Some(30);
    rec.sequence_reset = true;
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let out = dir.path().join("out.dem");
    let r = run_restart(&src, &out, at(35), Some(at(40)), Limits::default()).unwrap();
    assert_eq!(r.bootstrap_sequences, [14]);
    assert_eq!(r.validation.other_differences, 0);

    // The bootstrap record carries group 34's demo info, not group 4's
    let f = std::io::BufReader::new(std::fs::File::open(&out).unwrap());
    let mut reader = DemoReader::new(f, Limits::default()).unwrap();
    let bootstrap = reader
        .frames(1)
        .unwrap()
        .map(|lf| lf.unwrap().frame)
        .take_while(|f| f.frame_number == 0)
        .find_map(|f| match f.body {
            FrameBody::Network(n) if n.sequence[0] == 14 => Some(n),
            _ => None,
        })
        .unwrap();
    let time = f32::from_le_bytes(bootstrap.demo_info[..4].try_into().unwrap());
    assert_eq!(DemoTime::from_f32(time), Some(at(34)));
}

#[test]
fn a_restart_naming_a_later_session_starts_there_when_the_clock_started_over() {
    // The new session from group 30 takes its times from 20 groups back, so
    // group 40 has group 20's time
    let mut rec = Recording::steady(60, 1);
    rec.session_at = Some(30);
    rec.clock_reset = true;
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let (report, _) = decode_demo(&src, &DecodeOptions::default(), |_, _, _| {});

    // Without a session the clip is the earliest with those times
    let first = dir.path().join("first.dem");
    run_restart(&src, &first, at(20), Some(at(25)), Limits::default()).unwrap();
    assert_eq!(last_model(&first), Some(Value::Int(24)));

    let later = dir.path().join("later.dem");
    let (limits, never) = (Limits::default(), AtomicBool::new(false));
    let r = run_restart_until(&src, &later, at(20), Some(at(25)), Some(2), limits, &never).unwrap();
    assert_eq!(r.validation.other_differences, 0);
    assert_eq!(last_model(&later), Some(Value::Int(44)));
    let snapped = report.groups.snap(at(20), Some(at(25)), Some(2)).unwrap();
    assert_eq!(snapped.start.to_string(), r.effective_start);
    assert_eq!(snapped.end.to_string(), r.effective_end);
}

/// A slice of `rec` from group `start` to `end`, checked against the
/// group index, decoding as a whole
fn crossing(rec: &Recording, start: i32, end: i32) -> (PathBuf, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let (report, _) = decode_demo(&src, &DecodeOptions::default(), |_, _, _| {});
    let out = dir.path().join("out.dem");
    let r = run_restart(&src, &out, at(start), Some(at(end)), Limits::default()).unwrap();
    assert_eq!(r.validation.other_differences, 0);
    let snapped = report.groups.snap(at(start), Some(at(end)), None).unwrap();
    assert_eq!(snapped.start.to_string(), r.effective_start);
    assert_eq!(snapped.end.to_string(), r.effective_end);

    let (out_report, _) = decode_demo(&out, &DecodeOptions::default(), |_, _, _| {});
    assert_eq!(out_report.status, DecodeStatus::Complete);
    assert_eq!(out_report.stats.segments, 2);
    (out, dir)
}

#[test]
fn a_restart_goes_on_across_a_map_change() {
    // The new session sets up in group 30, so a clip from 25 to 40 loads
    // the next map partway and ends on group 39's state
    let mut rec = Recording::steady(60, 1);
    rec.session_at = Some(30);
    let (out, _dir) = crossing(&rec, 25, 40);
    assert_eq!(last_model(&out), Some(Value::Int(39)));

    // The setup's records are kept where they were, in the playback entry
    let f = std::io::BufReader::new(std::fs::File::open(&out).unwrap());
    let mut reader = DemoReader::new(f, Limits::default()).unwrap();
    let numbers: Vec<i32> = reader
        .frames(1)
        .unwrap()
        .map(|lf| lf.unwrap().frame.frame_number)
        .collect();
    assert!(numbers.windows(2).all(|w| w[0] <= w[1]), "{numbers:?}");
}

#[test]
fn a_restart_goes_on_across_a_reconnect_that_numbers_packets_again() {
    // From group 30 packets are numbered from 10 again, so only the first
    // session's packets need a bootstrap
    let mut rec = Recording::steady(60, 1);
    rec.session_at = Some(30);
    rec.sequence_reset = true;
    let (out, _dir) = crossing(&rec, 25, 40);
    assert_eq!(last_model(&out), Some(Value::Int(39)));
}

#[test]
fn a_restart_stops_where_the_clock_starts_over() {
    // From group 30 the clock goes back 20 groups
    let mut rec = Recording::steady(60, 1);
    rec.session_at = Some(30);
    rec.clock_reset = true;
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &rec.bytes());
    let (report, _) = decode_demo(&src, &DecodeOptions::default(), |_, _, _| {});

    let out = dir.path().join("out.dem");
    let err = run_restart(&src, &out, at(25), None, Limits::default()).unwrap_err();
    assert!(matches!(err, ExportError::ClockStartsOver(_)), "{err}");
    let err = report.groups.snap(at(25), None, None).unwrap_err();
    assert!(matches!(err, ExportError::ClockStartsOver(_)), "{err}");

    // Ending before it is fine
    run_restart(&src, &out, at(25), Some(at(29)), Limits::default()).unwrap();
}
