//! Synthetic demo builder for tests. Produces structurally valid containers
//! with opaque network payloads, but it says nothing about game playability

#![allow(dead_code)]

use lambda_slicer_core::container::{
    DirectoryEntry, EventRecord, Frame, FrameBody, Header, NetworkFrame, SoundRecord,
    encode_directory, encode_frame, encode_header, tag,
};

pub fn header() -> Header {
    let mut map = Box::new([0u8; 260]);
    map[..9].copy_from_slice(b"crossfire");

    let mut game = Box::new([0u8; 260]);
    game[..2].copy_from_slice(b"ag");

    Header {
        demo_protocol: 5,
        network_protocol: 48,
        map_name_raw: map,
        game_dir_raw: game,
        map_checksum: 0x1234_5678,
        directory_offset: 0,
    }
}

pub fn frame(tag: u8, time: f32, frame_number: i32, body: FrameBody) -> Frame {
    Frame {
        tag,
        time_bits: time.to_bits(),
        frame_number,
        body,
    }
}

pub fn network(tag: u8, time: f32, n: i32, payload: &[u8]) -> Frame {
    let mut demo_info = Box::new([0u8; 436]);
    demo_info[..4].copy_from_slice(&time.to_le_bytes());

    frame(
        tag,
        time,
        n,
        FrameBody::Network(Box::new(NetworkFrame {
            demo_info,
            sequence: [n, n - 1, n, n, n + 1, n, n],
            message: payload.to_vec(),
        })),
    )
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

/// One playback frame group at `time` with every record type
pub fn group(time: f32, n: i32) -> Vec<Frame> {
    let mut cmd = Box::new([0u8; 64]);
    cmd[..7].copy_from_slice(b"+attack");

    vec![
        frame(
            tag::CONSOLE_COMMAND,
            time,
            n,
            FrameBody::ConsoleCommand(cmd),
        ),
        frame(
            tag::DEMO_BUFFER,
            time,
            n,
            FrameBody::DemoBuffer(vec![1, 2, 3]),
        ),
        frame(tag::CLIENT_DATA, time, n, FrameBody::ClientData([7; 32])),
        frame(
            tag::EVENT,
            time,
            n,
            FrameBody::Event(EventRecord {
                flags: 0,
                index: 9,
                delay_bits: 0f32.to_bits(),
                args: [3; 72],
            }),
        ),
        frame(
            tag::SOUND,
            time,
            n,
            FrameBody::Sound(SoundRecord {
                channel: 1,
                sample: b"weapons/gauss2.wav".to_vec(),
                attenuation_bits: 0.8f32.to_bits(),
                volume_bits: 1.0f32.to_bits(),
                flags: 0,
                pitch: 100,
            }),
        ),
        frame(
            tag::WEAPON_ANIM,
            time,
            n,
            FrameBody::WeaponAnim {
                sequence: 2,
                body: 0,
            },
        ),
        network(tag::NETWORK, time, n, &[0xAB; 40]),
    ]
}

/// A two-entry demo with `groups` playback frame groups 0.5 s apart
pub fn demo(groups: i32) -> Vec<u8> {
    let loading = vec![
        network(tag::NETWORK_START, 3., 100, &[0xCD; 200]),
        frame(tag::NEXT_SECTION, 3.1, 101, FrameBody::NextSection),
    ];

    let mut playback = vec![frame(tag::DEMO_START, 0., 0, FrameBody::DemoStart)];
    for n in 0..groups {
        playback.extend(group(n as f32 * 0.5, n));
    }

    let end = groups as f32 * 0.5;
    playback.push(frame(
        tag::NEXT_SECTION,
        end,
        groups,
        FrameBody::NextSection,
    ));

    build(&[
        (entry(0, "LOADING"), loading),
        (entry(1, "Playback"), playback),
    ])
}

pub fn build(entries: &[(DirectoryEntry, Vec<Frame>)]) -> Vec<u8> {
    let mut out = encode_header(&header()).to_vec();
    let mut dir = Vec::new();

    for (e, frames) in entries {
        let start = out.len();
        for f in frames {
            encode_frame(f, &mut out);
        }

        let mut e = e.clone();
        e.offset = start as i32;
        e.length = (out.len() - start) as i32;
        if e.entry_type != 0 {
            e.frame_count = frames
                .iter()
                .filter(|f| matches!(f.body, FrameBody::Network(_)))
                .count() as i32;
            e.track_time_bits = frames.last().map_or(0, |f| f.time_bits);
        }

        dir.push(e);
    }

    let dir_offset = out.len() as i32;
    out.extend(encode_directory(&dir));
    out[540..544].copy_from_slice(&dir_offset.to_le_bytes());

    out
}
