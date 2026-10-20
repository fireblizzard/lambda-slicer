//! AG's countdown as playback shows it and speaks it, on a synthetic demo

#[path = "../../lambda-slicer-core/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};

use common::stream;
use common::{build, frame, network};
use lambda_slicer_core::container::{DirectoryEntry, FrameBody, Limits, tag};
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_playback::audio::SoundName;
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::hud::WAITING;
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

/// The id the setup gives `Countdown`
const COUNTDOWN: u8 = 65;

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

/// `Countdown` registered with a body of any length, as AG's server does
fn register() -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(svc::NEWUSERMSG);
    w.u8(COUNTDOWN);
    w.u8(255);
    let mut name = [0u8; 16];
    name[..9].copy_from_slice(b"Countdown");
    w.bytes(&name);
    w.finish()
}

/// A `Countdown` message, `seconds` left, spoken or not, between two names
fn countdown(seconds: i8, spoken: bool, names: [&str; 2]) -> Vec<u8> {
    let mut body = vec![seconds as u8, u8::from(spoken)];
    for name in names {
        body.extend_from_slice(name.as_bytes());
        body.push(0);
    }
    let mut w = BitWriter::new();
    w.u8(COUNTDOWN);
    w.u8(body.len() as u8);
    w.bytes(&body);
    w.finish()
}

/// A demo whose groups come a second apart from 1 s to 10 s, three spoken
/// at 1 s, hidden at 2 s, two spoken at 3 s, and the match waiting at 9 s
/// with the last round's winner
fn demo(dir: &Path) -> PathBuf {
    let setup = [stream::setup_with(48, &[]), register()].concat();
    let loading = vec![
        network(tag::NETWORK_START, 3., 99, &setup),
        network(tag::NETWORK_START, 3., 100, &stream::full_packet(100)),
        frame(tag::NEXT_SECTION, 3.1, 101, FrameBody::NextSection),
    ];

    let mut playback = vec![frame(tag::DEMO_START, 0., 0, FrameBody::DemoStart)];
    for k in 1..=10 {
        let payload = match k {
            1 => countdown(3, true, ["", ""]),
            2 => countdown(-1, false, ["", ""]),
            3 => countdown(2, true, ["", ""]),
            9 => countdown(WAITING, false, ["red", ""]),
            _ => stream::model_update(100, 1, 2),
        };
        playback.push(network(tag::NETWORK, k as f32, 100 + k, &payload));
    }
    playback.push(frame(tag::NEXT_SECTION, 10.5, 111, FrameBody::NextSection));

    let bytes = build(&[
        (entry(0, "LOADING"), loading),
        (entry(1, "Playback"), playback),
    ]);
    let path = dir.join("countdown.dem");
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn the_countdown_shows_for_five_seconds_from_each_spoken_number() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = Playback::open(&demo(dir.path()), Limits::default()).unwrap();
    let mut shown = |at: f64| {
        let at = SourceTime(at);
        p.advance_to(at).unwrap();
        let frame = p.present(Stamp::default(), at, CameraMode::Recorder, false);
        frame.countdown.map(|c| (c.seconds, c.names))
    };
    let none = || [String::new(), String::new()];

    assert_eq!(shown(0.5), None);
    assert_eq!(shown(1.5), Some((3, none())));
    // Hidden by a negative count
    assert_eq!(shown(2.5), None);
    assert_eq!(shown(3.5), Some((2, none())));
    assert_eq!(shown(7.9), Some((2, none())));
    // Five seconds after the last spoken number
    assert_eq!(shown(8.1), None);
    // Waiting for players shows until the next message, whatever the time
    let waiting = Some((WAITING, ["red".to_owned(), String::new()]));
    assert_eq!(shown(9.5), waiting);
    assert_eq!(shown(10.4), waiting);
}

#[test]
fn each_spoken_number_plays_its_sound_where_the_listener_is() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = Playback::open(&demo(dir.path()), Limits::default()).unwrap();
    let playing = |p: &Playback, sample: &str| {
        p.audio()
            .channels()
            .iter()
            .find(|c| c.sound == SoundName::File(sample.into()))
            .cloned()
    };

    p.advance_to(SourceTime(1.5)).unwrap();
    let three = playing(&p, "fvox/three.wav").expect("three spoken");
    assert_eq!(three.attenuation, 0.);

    p.advance_to(SourceTime(3.5)).unwrap();
    assert!(playing(&p, "fvox/two.wav").is_some());
    // The waiting message is not spoken
    p.advance_to(SourceTime(10.4)).unwrap();
    assert!(p.audio().channels().iter().all(|c| c.started.0 < 9.));
}
