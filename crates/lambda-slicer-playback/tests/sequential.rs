//! Sequential playback, the controller and the worker on a synthetic demo

#[path = "../../lambda-slicer-core/tests/common/mod.rs"]
mod common;

use std::path::Path;

use common::stream::{self, SOUND_RESOURCE};
use common::{build, frame, network};
use lambda_slicer_core::container::{DirectoryEntry, FrameBody, Limits, SoundRecord, tag};
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_playback::audio::{SoundName, flags};
use lambda_slicer_playback::controller::{CameraMode, Command, Controller, Reply, Stamp, Target};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;
use lambda_slicer_playback::worker::Worker;

const MODEL_RESOURCE: u32 = 2;
const GAUSS: u32 = 1;
const HUM: u32 = 2;

/// Playback groups, 0.5 s apart from 0.5
const GROUPS: i32 = 8;

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

/// A demo whose playback changes entity 5's model at 1.0 and plays sounds
/// on entities 1 and 5. The full update is sequence 100 and every delta
/// comes from it
fn demo(dir: &Path) -> std::path::PathBuf {
    let setup = stream::setup_with(
        48,
        &[
            (SOUND_RESOURCE, "weapons/gauss2.wav", GAUSS),
            (SOUND_RESOURCE, "ambience/hum.wav", HUM),
            (MODEL_RESOURCE, "models/player.mdl", 2),
            (MODEL_RESOURCE, "models/w_gauss.mdl", 7),
            (MODEL_RESOURCE, "models/rpgrocket.mdl", 9),
        ],
    );
    let loading = vec![
        network(tag::NETWORK_START, 3., 99, &setup),
        network(tag::NETWORK_START, 3., 100, &stream::full_packet(100)),
        frame(tag::NEXT_SECTION, 3.1, 101, FrameBody::NextSection),
    ];

    let mut playback = vec![frame(tag::DEMO_START, 0., 0, FrameBody::DemoStart)];
    for k in 1..=GROUPS {
        let n = 100 + k;
        let time = k as f32 * 0.5;
        let payload: Vec<u8> = match k {
            1 => sound(0, 1, 1, GAUSS, 100),
            2 => [stream::model_update(100, 5, 9), sound(0, 2, 5, HUM, 100)].concat(),
            3 => sound(flags::CHANGE_PITCH, 2, 5, HUM, 150),
            4 => sound(0, 1, 1, HUM, 100),
            5 => stop_sound(5, 2),
            _ => stream::model_update(100, 1, 2),
        };
        playback.push(network(tag::NETWORK, time, n, &payload));
        if k == 6 {
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
        }
        if k == 7 {
            playback.push(frame(
                tag::WEAPON_ANIM,
                time,
                n,
                FrameBody::WeaponAnim {
                    sequence: 4,
                    body: 0,
                },
            ));
        }
    }
    let end = (GROUPS + 1) as f32 * 0.5;
    playback.push(frame(
        tag::NEXT_SECTION,
        end,
        100 + GROUPS + 1,
        FrameBody::NextSection,
    ));

    let bytes = build(&[
        (entry(0, "LOADING"), loading),
        (entry(1, "Playback"), playback),
    ]);
    let path = dir.join("synthetic.dem");
    std::fs::write(&path, bytes).unwrap();
    path
}

fn open(path: &Path) -> Playback {
    Playback::open(path, Limits::default()).unwrap()
}

/// Everything that must agree however a moment was reached
fn digest(p: &mut Playback, at: SourceTime) -> String {
    let f = p.present(Stamp::default(), at, CameraMode::Recorder, false);
    format!(
        "{:?}",
        (
            f.position,
            f.at,
            &f.entities,
            &f.sounds,
            f.recorder,
            f.viewmodel,
            f.end,
            &f.stopped
        )
    )
}

fn sounds(p: &mut Playback, at: f64) -> Vec<(u16, u8, String, u8)> {
    p.advance_to(SourceTime(at)).unwrap();
    let f = p.present(Stamp::default(), SourceTime(at), CameraMode::Recorder, true);
    f.sounds
        .iter()
        .map(|c| {
            let name = match &c.sound {
                SoundName::File(n) => n.to_string(),
                other => format!("{other:?}"),
            };
            (c.entity, c.channel, name, c.pitch_at(SourceTime(at)))
        })
        .collect()
}

#[test]
fn entities_keep_their_identity_until_their_model_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = open(&demo(dir.path()));

    p.advance_to(SourceTime::ZERO).unwrap();
    let models = |p: &Playback| -> Vec<(u16, u32, Option<String>)> {
        p.scene()
            .entities
            .iter()
            .map(|e| {
                (
                    e.id.number,
                    e.id.life,
                    e.model.as_deref().map(str::to_owned),
                )
            })
            .collect()
    };
    assert_eq!(p.scene().map.as_deref(), Some("maps/test.bsp"));
    assert_eq!(
        models(&p),
        [
            (1, 0, Some("models/player.mdl".into())),
            (5, 0, Some("models/w_gauss.mdl".into())),
        ]
    );
    // Sent as -80 under the schema's multiplier of 8
    assert_eq!(p.scene().entities[0].origin[0], -10.);

    p.advance_to(SourceTime(1.)).unwrap();
    assert_eq!(
        models(&p),
        [
            (1, 0, Some("models/player.mdl".into())),
            (5, 1, Some("models/rpgrocket.mdl".into())),
        ]
    );
    let position = p.position().unwrap();
    assert_eq!((position.time, position.segment), (SourceTime(1.), 1));
}

#[test]
fn sounds_start_change_replace_and_stop() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = open(&demo(dir.path()));

    assert_eq!(sounds(&mut p, 0.), []);
    assert_eq!(
        sounds(&mut p, 0.5),
        [(1, 1, "weapons/gauss2.wav".into(), 100)]
    );
    assert_eq!(
        sounds(&mut p, 1.5),
        [
            (1, 1, "weapons/gauss2.wav".into(), 100),
            (5, 2, "ambience/hum.wav".into(), 150),
        ]
    );
    assert_eq!(
        sounds(&mut p, 2.),
        [
            (5, 2, "ambience/hum.wav".into(), 150),
            (1, 1, "ambience/hum.wav".into(), 100),
        ]
    );
    assert_eq!(
        sounds(&mut p, 2.5),
        [(1, 1, "ambience/hum.wav".into(), 100)]
    );

    // A sound record plays on the entity the recorder views from
    assert_eq!(
        sounds(&mut p, 3.),
        [
            (1, 1, "ambience/hum.wav".into(), 100),
            (1, 3, "player/pl_step1.wav".into(), 100),
        ]
    );

    // Unknown lengths count as finished after ten seconds
    let late = sounds(&mut p, 12.);
    assert_eq!(late, [(1, 3, "player/pl_step1.wav".into(), 100)]);
    assert_eq!(p.scene().viewmodel.map(|v| v.sequence), Some(4));
}

#[test]
fn every_way_of_reaching_a_moment_gives_the_same_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());

    for at in [0., 0.75, 1.5, 2.75, 3.5, 12.] {
        let at = SourceTime(at);

        let mut direct = open(&path);
        direct.advance_to(at).unwrap();
        let expected = digest(&mut direct, at);

        let mut stepping = open(&path);
        let mut t = 0.;
        while t < at.0 {
            stepping.advance_to(SourceTime(t)).unwrap();
            t += 0.05;
        }
        stepping.advance_to(at).unwrap();
        assert_eq!(digest(&mut stepping, at), expected, "stepping to {at:?}");

        let mut back = open(&path);
        back.advance_to(SourceTime(100.)).unwrap();
        back.advance_to(at).unwrap();
        assert_eq!(
            digest(&mut back, at),
            expected,
            "back from the end to {at:?}"
        );

        let mut by_group = open(&path);
        by_group
            .seek_group(direct.position().unwrap().group)
            .unwrap();
        by_group.advance_to(at).unwrap();
        assert_eq!(digest(&mut by_group, at), expected, "by group to {at:?}");
    }
}

/// Replies until one answers `stamp`, judged by the controller as they come
fn settle(worker: &Worker, controller: &mut Controller, stamp: Stamp) -> Vec<(Stamp, bool, Reply)> {
    let mut seen = Vec::new();
    loop {
        let reply = worker.wait().expect("worker replies");
        let accepted = controller.accept(&reply, 0.);
        let done = reply.stamp() == stamp;
        seen.push((reply.stamp(), accepted, reply));
        if done {
            return seen;
        }
    }
}

#[test]
fn the_controller_shows_only_answers_to_its_newest_seek() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let worker = Worker::spawn(Limits::default(), || {});
    let mut controller = Controller::default();

    let load = controller.command(Command::Load(path.clone()), 0.).unwrap();
    let stamp = load.stamp;
    worker.send(load);
    let seen = settle(&worker, &mut controller, stamp);
    assert!(seen.iter().all(|(_, accepted, _)| *accepted));

    // Two seeks in a row. Whatever answers the first is refused
    let first = controller
        .command(Command::SeekTo(SourceTime(3.)), 0.)
        .unwrap();
    worker.send(first.clone());
    let second = controller
        .command(Command::SeekTo(SourceTime(1.)), 0.)
        .unwrap();
    worker.send(second.clone());
    for (stamp, accepted, _) in settle(&worker, &mut controller, second.stamp) {
        assert_eq!(accepted, stamp == second.stamp, "{stamp:?}");
    }
    assert_eq!(controller.time(0.), SourceTime(1.));

    // A step goes on from the group shown and moves the playhead to it
    let step = controller.command(Command::StepGroup(1), 0.).unwrap();
    assert!(matches!(step.target, Target::Group(_)));
    worker.send(step.clone());
    let seen = settle(&worker, &mut controller, step.stamp);
    let Some((_, true, Reply::Frame(frame))) = seen.last() else {
        panic!("the step was not shown");
    };
    assert_eq!(frame.at, SourceTime(1.5));
    assert_eq!(controller.time(5.), SourceTime(1.5));

    // Loading again refuses everything from the earlier load
    let old = controller
        .command(Command::SeekTo(SourceTime(2.)), 0.)
        .unwrap();
    let reload = controller.command(Command::Load(path), 0.).unwrap();
    worker.send(old.clone());
    worker.send(reload.clone());
    for (stamp, accepted, _) in settle(&worker, &mut controller, reload.stamp) {
        assert_eq!(accepted, stamp == reload.stamp, "{stamp:?}");
    }
}

#[test]
fn playing_follows_the_clock_and_loops() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut controller = Controller::default();
    controller.command(Command::Load(path), 0.).unwrap();

    let play = controller.command(Command::Play, 10.).unwrap();
    assert_eq!(play.target, Target::Time(SourceTime::ZERO));
    assert!(play.audible);
    let tick = controller.tick(11.5).unwrap();
    assert_eq!(tick.target, Target::Time(SourceTime(1.5)));
    assert_eq!(tick.stamp, play.stamp);

    controller.command(
        Command::SetLoop(Some(lambda_slicer_playback::controller::LoopRange {
            start: SourceTime(0.5),
            end: SourceTime(2.),
        })),
        11.5,
    );
    // Sound is mixed no further than the loop's end
    assert_eq!(controller.until(), Some(SourceTime(2.)));
    let wrapped = controller.tick(12.).unwrap();
    assert_eq!(wrapped.target, Target::Time(SourceTime(0.5)));
    assert!(wrapped.stamp.revision > tick.stamp.revision);

    controller.command(Command::Pause, 12.25);
    assert_eq!(controller.tick(20.), None);
    assert_eq!(controller.time(20.), SourceTime(0.75));
}

#[test]
fn playing_a_range_pauses_at_its_end_unless_the_playhead_moves() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut controller = Controller::default();
    controller.command(Command::Load(path), 0.).unwrap();

    let range = Command::PlayRange {
        from: SourceTime(0.5),
        to: SourceTime(1.),
    };
    let start = controller.command(range.clone(), 0.).unwrap();
    assert_eq!(start.target, Target::Time(SourceTime(0.5)));
    assert!(start.audible);
    let during = controller.tick(0.3).unwrap();
    assert_eq!(during.target, Target::Time(SourceTime(0.8)));
    assert_eq!(controller.until(), Some(SourceTime(1.)));

    // Past the end it stops there, paused, with one last frame asked for
    let end = controller.tick(0.6).unwrap();
    assert_eq!(end.target, Target::Time(SourceTime(1.)));
    assert!(!end.audible && !controller.playing());
    assert_eq!(controller.tick(1.), None);
    assert_eq!(controller.time(5.), SourceTime(1.));

    // A seek during the range forgets its end
    controller.command(range, 10.).unwrap();
    controller
        .command(Command::SeekTo(SourceTime(2.)), 10.5)
        .unwrap();
    let on = controller.tick(11.).unwrap();
    assert_eq!(on.target, Target::Time(SourceTime(2.5)));
    assert!(controller.playing());
}

#[test]
fn playing_backward_is_silent_wraps_loops_and_stops_at_the_start() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut controller = Controller::default();
    controller.command(Command::Load(path), 0.).unwrap();
    controller
        .command(Command::SeekTo(SourceTime(2.)), 0.)
        .unwrap();

    let back = controller.command(Command::PlayBackward, 1.).unwrap();
    assert_eq!(back.target, Target::Time(SourceTime(2.)));
    assert!(!back.audible);
    let tick = controller.tick(1.5).unwrap();
    assert_eq!(tick.target, Target::Time(SourceTime(1.5)));

    // Past the start of a loop it goes on from the loop's end
    controller.command(
        Command::SetLoop(Some(lambda_slicer_playback::controller::LoopRange {
            start: SourceTime(1.),
            end: SourceTime(1.75),
        })),
        1.5,
    );
    let wrapped = controller.tick(2.).unwrap();
    assert_eq!(wrapped.target, Target::Time(SourceTime(1.75)));
    assert!(wrapped.stamp.revision > tick.stamp.revision);
    controller.command(Command::SetLoop(None), 2.);

    // At the start it pauses there
    let start = controller.tick(10.).unwrap();
    assert_eq!(start.target, Target::Time(SourceTime::ZERO));
    assert!(!controller.playing());

    // Play goes forward again, with sound
    let forward = controller.command(Command::Play, 11.).unwrap();
    assert!(forward.audible);
    assert_eq!(controller.time(12.), SourceTime(1.));
}

#[test]
fn scrubbing_holds_playback_silent_and_resumes_or_goes_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut controller = Controller::default();
    controller.command(Command::Load(path), 0.).unwrap();
    controller.command(Command::Play, 0.).unwrap();

    // Playing at 1 s, a drag holds the playhead where it is dragged
    let first = controller
        .command(Command::Scrub(SourceTime(3.)), 1.)
        .unwrap();
    assert!(!first.audible && controller.scrubbing() && !controller.playing());
    let second = controller
        .command(Command::Scrub(SourceTime(2.5)), 1.5)
        .unwrap();
    assert!(second.stamp.revision > first.stamp.revision);
    assert_eq!(controller.time(5.), SourceTime(2.5));

    // Released, it plays on from there with sound
    let resumed = controller
        .command(Command::EndScrub { cancel: false }, 2.)
        .unwrap();
    assert_eq!(resumed.target, Target::Time(SourceTime(2.5)));
    assert!(resumed.audible && controller.playing() && !controller.scrubbing());

    // Cancelled, it goes back to where the drag began, here paused
    assert_eq!(controller.command(Command::Pause, 2.5), None);
    controller.command(Command::Scrub(SourceTime(0.5)), 3.);
    let back = controller
        .command(Command::EndScrub { cancel: true }, 3.5)
        .unwrap();
    assert_eq!(back.target, Target::Time(SourceTime(3.)));
    assert!(!controller.playing());
    assert_eq!(
        controller.command(Command::EndScrub { cancel: false }, 4.),
        None
    );
}

#[test]
fn playing_after_a_step_runs_on_from_the_group_reached() {
    let dir = tempfile::tempdir().unwrap();
    let path = demo(dir.path());
    let mut controller = Controller::default();
    controller.command(Command::Load(path.clone()), 0.).unwrap();
    let mut p = open(&path);
    let answer = |controller: &mut Controller, p: &mut Playback, stamp, at, wall| {
        let frame = p.present(stamp, at, CameraMode::Recorder, false);
        assert!(controller.accept(&Reply::Frame(Box::new(frame)), wall));
    };

    let seek = controller
        .command(Command::SeekTo(SourceTime(1.)), 0.)
        .unwrap();
    p.advance_to(SourceTime(1.)).unwrap();
    answer(&mut controller, &mut p, seek.stamp, SourceTime(1.), 0.);

    // A step pauses at the next group, its answer moving the playhead there
    let step = controller.command(Command::StepGroup(1), 1.).unwrap();
    let Target::Group(group) = step.target else {
        panic!("a step asks for a group");
    };
    p.seek_group(group).unwrap();
    let reached = p.position().unwrap().time;
    answer(&mut controller, &mut p, step.stamp, reached, 1.);
    assert_eq!(controller.time(2.), reached);

    // Played, the answers to its requests leave the clock running
    let play = controller.command(Command::Play, 2.).unwrap();
    answer(&mut controller, &mut p, play.stamp, reached, 2.5);
    assert_eq!(controller.time(3.), SourceTime(reached.0 + 1.));
    assert!(controller.playing());
}
