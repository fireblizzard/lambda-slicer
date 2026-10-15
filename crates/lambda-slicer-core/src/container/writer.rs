//! Byte-exact encoders for the container structures read by `reader`

use super::frame::{Frame, FrameBody};
use super::{DIR_ENTRY_LEN, DirectoryEntry, HEADER_LEN, Header, MAGIC};

pub fn encode_header(h: &Header) -> [u8; HEADER_LEN as usize] {
    let mut b = [0u8; HEADER_LEN as usize];
    b[0..8].copy_from_slice(MAGIC);
    b[8..12].copy_from_slice(&h.demo_protocol.to_le_bytes());
    b[12..16].copy_from_slice(&h.network_protocol.to_le_bytes());
    b[16..276].copy_from_slice(&h.map_name_raw[..]);
    b[276..536].copy_from_slice(&h.game_dir_raw[..]);
    b[536..540].copy_from_slice(&h.map_checksum.to_le_bytes());
    b[540..544].copy_from_slice(&h.directory_offset.to_le_bytes());

    b
}

/// Entry count followed by 92-byte records
pub fn encode_directory(entries: &[DirectoryEntry]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + entries.len() * DIR_ENTRY_LEN as usize);
    let count = i32::try_from(entries.len()).expect("directory entry count fits in i32");
    out.extend_from_slice(&count.to_le_bytes());

    for e in entries {
        out.extend_from_slice(&e.entry_type.to_le_bytes());
        out.extend_from_slice(&e.description_raw);
        out.extend_from_slice(&e.flags.to_le_bytes());
        out.extend_from_slice(&e.cd_track.to_le_bytes());
        out.extend_from_slice(&e.track_time_bits.to_le_bytes());
        out.extend_from_slice(&e.frame_count.to_le_bytes());
        out.extend_from_slice(&e.offset.to_le_bytes());
        out.extend_from_slice(&e.length.to_le_bytes());
    }

    out
}

fn sized(out: &mut Vec<u8>, bytes: &[u8]) {
    let len = i32::try_from(bytes.len()).expect("payload length fits in i32");
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
}

/// Append the encoding of `f` to `out`
pub fn encode_frame(f: &Frame, out: &mut Vec<u8>) {
    out.push(f.tag);
    out.extend_from_slice(&f.time_bits.to_le_bytes());
    out.extend_from_slice(&f.frame_number.to_le_bytes());

    match &f.body {
        FrameBody::Network(n) => {
            out.extend_from_slice(&n.demo_info[..]);
            for s in n.sequence {
                out.extend_from_slice(&s.to_le_bytes());
            }
            sized(out, &n.message);
        }
        FrameBody::DemoStart | FrameBody::NextSection => {}
        FrameBody::ConsoleCommand(c) => out.extend_from_slice(&c[..]),
        FrameBody::ClientData(c) => out.extend_from_slice(c),
        FrameBody::Event(e) => {
            out.extend_from_slice(&e.flags.to_le_bytes());
            out.extend_from_slice(&e.index.to_le_bytes());
            out.extend_from_slice(&e.delay_bits.to_le_bytes());
            out.extend_from_slice(&e.args);
        }
        FrameBody::WeaponAnim { sequence, body } => {
            out.extend_from_slice(&sequence.to_le_bytes());
            out.extend_from_slice(&body.to_le_bytes());
        }
        FrameBody::Sound(s) => {
            out.extend_from_slice(&s.channel.to_le_bytes());
            sized(out, &s.sample);
            out.extend_from_slice(&s.attenuation_bits.to_le_bytes());
            out.extend_from_slice(&s.volume_bits.to_le_bytes());
            out.extend_from_slice(&s.flags.to_le_bytes());
            out.extend_from_slice(&s.pitch.to_le_bytes());
        }
        FrameBody::DemoBuffer(b) => sized(out, b),
    }
}
