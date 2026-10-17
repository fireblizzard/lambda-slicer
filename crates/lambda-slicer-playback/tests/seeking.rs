//! Seeking through checkpoints reaches exactly the state sequential
//! playback reaches (implementation plan §13.9, the equivalence test)

#[path = "../../lambda-slicer-core/tests/common/mod.rs"]
mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use common::stream::{self, SOUND_RESOURCE};
use common::{build, frame, network};
use lambda_slicer_core::container::{DirectoryEntry, FrameBody, Limits, SoundRecord, tag};
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_playback::audio::flags;
use lambda_slicer_playback::checkpoint::Checkpoints;
use lambda_slicer_playback::controller::{CameraMode, Command, Controller, Reply, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;
use lambda_slicer_playback::worker::Worker;

const MODEL_RESOURCE: u32 = 2;
const GAUSS: u32 = 1;
const HUM: u32 = 2;

/// Playback groups, a quarter of a second apart
const GROUPS: i32 = 48;
/// The group where a second session begins
const SECOND_SESSION: i32 = 25;
/// The group whose time is a little earlier than the one before
const STEP_BACK: i32 = 14;

fn sound(flags: u16, channel: u32, entity: u32, index: u32, pitch: u8) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(svc::SOUND);
    w.bits(u32::from(flags | flags::PITCH), 9);
    w.bits(channel, 3);
    w.bits(entity, 11);
    w.bits(index, 8);
    w.bits(0, 3);
    w.bits(u32::from(pitch), 8);
    w.align();
    w.finish()
}

fn stop_sound(entity: u16, channel: u16) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(svc::STOPSOUND);
    w.u16(entity << 3 | channel);
    w.finish()
}

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

fn group_time(k: i32) -> f32 {
    let time = k as f32 * 0.25;
    if k == STEP_BACK { time - 0.375 } else { time }
}

fn demo(dir: &Path) -> PathBuf {
    let resources = [
        (SOUND_RESOURCE, "weapons/gauss2.wav", GAUSS),
        (SOUND_RESOURCE, "ambience/hum.wav", HUM),
        (MODEL_RESOURCE, "models/player.mdl", 2),
        (MODEL_RESOURCE, "models/w_gauss.mdl", 7),
        (MODEL_RESOURCE, "models/rpgrocket.mdl", 9),
    ];
    let loading = vec![
        network(
            tag::NETWORK_START,
            3.,
            99,
            &stream::setup_with(48, &resources),
        ),
        network(tag::NETWORK_START, 3., 100, &stream::full_packet(100)),
        frame(tag::NEXT_SECTION, 3.1, 101, FrameBody::NextSection),
    ];

    let mut playback = vec![frame(tag::DEMO_START, 0., 0, FrameBody::DemoStart)];
    let mut base = 100u8;
    for k in 1..=GROUPS {
        let n = 100 + 2 * k;
        let time = group_time(k);

        // A new session, set up and given a full update in two groups
        if k == SECOND_SESSION {
            let setup = stream::setup_on(48, &resources, b"maps/second.bsp");
            playback.push(network(tag::NETWORK, time, n - 1, &setup));
            playback.push(network(tag::NETWORK, time, n, &stream::full_packet(n)));
            base = n as u8;
            continue;
        }

        let model = if k % 4 < 2 { 9 } else { 7 };
        let payload: Vec<u8> = match k % 6 {
            1 => sound(0, 1, 1, GAUSS, 100),
            2 => [
                stream::model_update(base, 5, model),
                sound(0, 2, 5, HUM, 100),
            ]
            .concat(),
            3 => sound(flags::CHANGE_PITCH, 2, 5, HUM, 150),
            4 => sound(0, 1, 1, HUM, 100),
            5 => stop_sound(5, 2),
            _ => stream::model_update(base, 1, 2),
        };
        playback.push(network(tag::NETWORK, time, n, &payload));
        if k % 6 == 0 {
            playback.push(frame(
                tag::SOUND,
                time,
                n,
                FrameBody::Sound(SoundRecord {
                    channel: 3,
                    sample: b"player/pl_step1.wav".to_vec(),
                    attenuation_bits: 0.8f32.to_bits(),
                    volume_bits: 1f32.to_bits(),
                    flags: 0,
                    pitch: 100,
                }),
            ));
            playback.push(frame(
                tag::WEAPON_ANIM,
                time,
                n,
                FrameBody::WeaponAnim {
                    sequence: k,
                    body: 0,
                },
            ));
        }
    }
    let end = group_time(GROUPS) + 0.25;
    playback.push(frame(
        tag::NEXT_SECTION,
        end,
        100 + 2 * GROUPS + 1,
        FrameBody::NextSection,
    ));

    let bytes = build(&[
        (entry(0, "LOADING"), loading),
        (entry(1, "Playback"), playback),
    ]);
    let path = dir.join("seeking.dem");
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Playback that never starts from a checkpoint, the reference
fn sequential(path: &Path) -> Playback {
    Playback::open_with(
        path,
        Limits::default(),
        Checkpoints::shared(f64::INFINITY, u64::MAX),
    )
    .unwrap()
}

/// Everything that must agree however a moment was reached, the decoder's
/// own counters and the change revisions included
fn digest(p: &mut Playback, at: SourceTime) -> String {
    let d = p.decoder();
    let decoder = format!(
        "{:?}",
        (
            d.time_bits,
            d.segment,
            d.stats.network_frames,
            d.stats.messages,
            d.latest_snapshot().map(|s| s.sequence),
            &d.players,
            d.resources().len(),
        )
    );
    let f = p.present(Stamp::default(), at, CameraMode::Recorder, false);
    format!(
        "{decoder} {:?}",
        (
            f.position,
            f.at,
            &f.map,
            f.scene_revision,
            &f.entities,
            f.recorder,
            f.viewmodel,
            &f.light_styles,
            &f.sounds,
            f.audio_revision,
            f.end,
            &f.stopped,
        )
    )
}

/// The reference state at each time, worked out once
struct Reference {
    path: PathBuf,
    known: HashMap<u64, String>,
}

impl Reference {
    fn at(&mut self, time: SourceTime) -> String {
        let path = self.path.clone();
        self.known
            .entry(time.0.to_bits())
            .or_insert_with(|| {
                let mut p = sequential(&path);
                p.advance_to(time).unwrap();
                digest(&mut p, time)
            })
            .clone()
    }
}

fn times() -> Vec<SourceTime> {
    let mut times: Vec<SourceTime> = (0..=52).map(|i| SourceTime(f64::from(i) * 0.25)).collect();
    // Between groups, and on both sides of the step back
    times.extend([0.1, 3.0625, 3.1, 3.2, 3.4, 6.2, 9.9, 100.].map(SourceTime));
    times
}

#[test]
fn the_synthetic_demo_has_what_the_tests_rely_on() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = sequential(&demo(dir.path()));
    p.advance_to(SourceTime(100.)).unwrap();
    assert!(p.position().unwrap().segment == 2);
    assert_eq!(p.scene().map.as_deref(), Some("maps/second.bsp"));
    assert!(p.at_end());

    // The step back puts group 14 (3.125) before group 13 (3.25) in time,
    // but it still waits for group 13
    let mut p = sequential(&demo(dir.path()));
    p.advance_to(SourceTime(3.2)).unwrap();
    assert_eq!(p.position().unwrap().time, SourceTime(3.));
    p.advance_to(SourceTime(3.4)).unwrap();
    assert_eq!(p.position().unwrap().time, SourceTime(3.125));
}

#[test]
fn seeking_from_checkpoints_matches_sequential_playback() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut reference = Reference {
        path: path.clone(),
        known: HashMap::new(),
    };

    let store = Checkpoints::shared(0.5, u64::MAX);
    let mut indexer = Playback::open_with(&path, Limits::default(), store.clone()).unwrap();
    while !indexer.index(7) {}
    assert!(store.lock().unwrap().len() > 15);
    assert!(store.lock().unwrap().indexed().1);

    for at in times() {
        let expected = reference.at(at);

        let mut forward = Playback::open_with(&path, Limits::default(), store.clone()).unwrap();
        forward.advance_to(at).unwrap();
        assert_eq!(digest(&mut forward, at), expected, "forward to {at:?}");

        let mut backward = Playback::open_with(&path, Limits::default(), store.clone()).unwrap();
        backward.advance_to(SourceTime(100.)).unwrap();
        backward.advance_to(at).unwrap();
        assert_eq!(digest(&mut backward, at), expected, "back to {at:?}");

        // Going on from a restored checkpoint stays equal
        let later = SourceTime(at.0 + 1.3);
        backward.advance_to(later).unwrap();
        assert_eq!(
            digest(&mut backward, later),
            reference.at(later),
            "on to {later:?}"
        );
    }
}

#[test]
fn scrubbing_stepping_and_looping_end_where_sequential_playback_does() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut reference = Reference {
        path: path.clone(),
        known: HashMap::new(),
    };
    let times = times();

    // Scrubbing at random while checkpoints are still being added
    let store = Checkpoints::shared(0.5, u64::MAX);
    let mut p = Playback::open_with(&path, Limits::default(), store.clone()).unwrap();
    let mut indexer = Playback::open_with(&path, Limits::default(), store).unwrap();
    let mut seed = 0x2545_f491_u32;
    for i in 0..300 {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let at = times[seed as usize % times.len()];
        p.advance_to(at).unwrap();
        assert_eq!(digest(&mut p, at), reference.at(at), "scrub {i} to {at:?}");
        if i % 10 == 0 {
            indexer.index(3);
        }
    }

    // Stepping group by group, forward and back
    let mut stepping = sequential(&path);
    let mut by_group = Playback::open(&path, Limits::default()).unwrap();
    for group in [3, 4, 5, 40, 39, 12, 13, 2, 60] {
        by_group.seek_group(group).unwrap();
        let at = by_group.position().unwrap().time;
        let mut fresh = sequential(&path);
        fresh.seek_group(group).unwrap();
        assert_eq!(
            digest(&mut by_group, at),
            digest(&mut fresh, at),
            "group {group}"
        );
    }
    stepping.seek_group(20).unwrap();

    // A clip looped three times, played in display-sized steps
    let (start, end) = (SourceTime(5.1), SourceTime(7.6));
    let mut looping =
        Playback::open_with(&path, Limits::default(), Checkpoints::shared(0.5, u64::MAX)).unwrap();
    for round in 0..3 {
        looping.advance_to(start).unwrap();
        assert_eq!(
            digest(&mut looping, start),
            reference.at(start),
            "round {round} start"
        );
        let mut t = start.0;
        while t < end.0 {
            looping.advance_to(SourceTime(t)).unwrap();
            t += 1. / 60.;
        }
        looping.advance_to(end).unwrap();
        assert_eq!(
            digest(&mut looping, end),
            reference.at(end),
            "round {round} end"
        );
    }
}

#[test]
fn the_worker_indexes_while_idle_and_seeks_answer_the_same() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut reference = Reference {
        path: path.clone(),
        known: HashMap::new(),
    };
    let worker = Worker::spawn(Limits::default(), || {});
    let mut controller = Controller::default();

    let load = controller.command(Command::Load(path), 0.).unwrap();
    worker.send(load);
    worker.wait().unwrap();

    // Asking again until the index covers the demo
    let mut complete = false;
    for _ in 0..500 {
        let seek = controller
            .command(Command::SeekTo(SourceTime(1.)), 0.)
            .unwrap();
        worker.send(seek);
        let Some(Reply::Frame(frame)) = worker.wait() else {
            panic!("no frame");
        };
        if frame.indexed.1 {
            complete = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(complete, "the background index never finished");

    for at in [9.5, 0.25, 6.2, 3.1, 12.] {
        let at = SourceTime(at);
        let seek = controller.command(Command::SeekTo(at), 0.).unwrap();
        worker.send(seek.clone());
        let reply = worker.wait().unwrap();
        assert!(controller.accept(&reply, 0.));
        let Reply::Frame(frame) = reply else {
            panic!("no frame");
        };
        let mut fresh = sequential(&frame.demo);
        fresh.advance_to(at).unwrap();
        let mut expected = fresh.present(seek.stamp, at, CameraMode::Recorder, false);
        expected.indexed = frame.indexed;
        assert_eq!(format!("{frame:?}"), format!("{expected:?}"), "{at:?}");
        let _ = reference.at(at);
    }
}
