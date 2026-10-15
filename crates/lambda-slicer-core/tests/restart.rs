//! S1 restart exports of synthetic demos whose packets decode

mod common;

use std::path::{Path, PathBuf};

use common::stream::{CLIENT, ENTITY, delta, full_packet, setup_with, time};
use common::{build, frame};
use lambda_slicer_core::container::{
    DemoReader, DirectoryEntry, EventRecord, FrameBody, Limits, NetworkFrame, tag,
};
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::delta::Value;
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_core::slice::ExportError;
use lambda_slicer_core::slice::restart::run_restart;
use lambda_slicer_core::state::{DecodeOptions, DecodeStatus, decode_demo};
use lambda_slicer_core::timeline::time::DemoTime;

/// Event scripts of the synthetic session, as (name, resource index)
const EVENTS: &[(&str, u32)] = &[("events/gauss.sc", 3), ("events/gaussspin.sc", 4)];

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
fn step(seconds: f32, from: u8, value: i64, extra: &[u8]) -> Vec<u8> {
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
    delta(&mut w, ENTITY, &[(1, value)]);
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
}

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
        }
    }

    fn bytes(&self) -> Vec<u8> {
        let loading = vec![
            network(tag::NETWORK_START, 3., 100, 5, setup_with(48, EVENTS)),
            frame(tag::NEXT_SECTION, 3.1, 101, FrameBody::NextSection),
        ];

        // The client's own copy of the weapon bits the full update sets
        let mut client = [0u8; 32];
        client[24..28].copy_from_slice(&self.recorded_weapons.to_le_bytes());

        let mut playback = vec![frame(tag::DEMO_START, 0., 0, FrameBody::DemoStart)];
        let records = &self.records;
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
            playback.push(network(tag::NETWORK, t, n, FIRST + n, payload));
        };

        group(0., 0, full_packet(FIRST));
        for (k, (from, extra)) in self.groups.iter().enumerate() {
            let n = k as i32 + 1;
            let t = n as f32 * STEP;
            group(t, n, step(t, *from, i64::from(n), extra));
        }

        let end = (self.groups.len() + 1) as f32 * STEP;
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

    let err = slice(&rec, 30, None).unwrap_err();
    assert!(
        matches!(&err, ExportError::UnknownPersistentMessage(m) if m.contains("Mystery")),
        "{err}"
    );
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
    assert_eq!(r.carried_records, 2);

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
    assert_eq!(r.carried_records, 0);
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
