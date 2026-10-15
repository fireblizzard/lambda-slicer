//! Synthetic network message streams built the way the engine writes
//! them, see docs/protocol-profile-5-48.md

use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::delta::dt;
use lambda_slicer_core::protocol::messages::svc;

pub type Field = (&'static str, u32, u32, u32, u32); // name, flags, offset, bits, premultiply

pub const ENTITY: &[Field] = &[
    ("origin[0]", dt::FLOAT | dt::SIGNED, 0, 16, 8),
    ("modelindex", dt::INTEGER, 4, 10, 1),
    ("animtime", dt::TIMEWINDOW_8, 8, 8, 1),
];
pub const CLIENT: &[Field] = &[
    ("health", dt::FLOAT | dt::SIGNED, 0, 10, 1),
    ("weapons", dt::INTEGER, 4, 32, 1),
];
pub const WEAPON: &[Field] = &[
    ("m_iId", dt::INTEGER, 0, 5, 1),
    ("m_iClip", dt::INTEGER | dt::SIGNED, 4, 10, 1),
];
pub const EVENT: &[Field] = &[
    ("entindex", dt::INTEGER, 0, 11, 1),
    ("bparam1", dt::INTEGER, 4, 1, 1),
    ("bparam2", dt::INTEGER, 8, 1, 1),
];

pub fn description(w: &mut BitWriter, name: &str, fields: &[Field]) {
    w.u8(svc::DELTADESCRIPTION);
    w.bytes(name.as_bytes());
    w.bytes(&[0]);
    w.bits(fields.len() as u32, 16);

    for &(fname, flags, offset, bits, pre) in fields {
        w.bits(1, 3);
        w.bits(0x7F, 8);
        w.bits(flags, 32);
        w.string(fname.as_bytes());
        w.bits(offset, 16);
        w.bits(4, 8);
        w.bits(bits, 8);
        w.bits(pre * 4000, 32);
        w.bits(4000, 32);
    }
}

/// Delta with the given (field index, raw value) pairs. Signed fields take
/// a signed raw value
pub fn delta(w: &mut BitWriter, fields: &[Field], sent: &[(usize, i64)]) {
    let mut mask = 0u64;
    for (i, _) in sent {
        mask |= 1 << i;
    }

    let bytes = if mask == 0 {
        0
    } else {
        (64 - mask.leading_zeros()).div_ceil(8)
    };
    w.bits(bytes, 3);
    for b in 0..bytes {
        w.bits(((mask >> (8 * b)) & 0xFF) as u32, 8);
    }

    let mut sorted = sent.to_vec();
    sorted.sort();
    for (i, v) in sorted {
        let (_, flags, _, bits, _) = fields[i];
        if flags & dt::TIMEWINDOW_8 != 0 {
            w.sbits(v, 8);
        } else if flags & dt::SIGNED != 0 {
            w.sbits(v, bits);
        } else {
            w.bits(v as u32, bits);
        }
    }
}

pub fn setup() -> Vec<u8> {
    setup_with_protocol(48)
}

pub fn setup_with_protocol(protocol: u32) -> Vec<u8> {
    setup_with(protocol, &[("events/gauss.sc", 3)])
}

/// Session setup announcing `protocol`, with these event scripts as
/// (name, resource index)
pub fn setup_with(protocol: u32, events: &[(&str, u32)]) -> Vec<u8> {
    let mut w = BitWriter::new();

    w.u8(svc::SERVERINFO);
    w.u32(protocol);
    w.u32(1);
    w.u32(0);
    w.bytes(&[0; 16]);
    w.u8(2); // max clients
    w.u8(0);
    w.u8(1);
    for s in [&b"ag"[..], b"host", b"maps/test.bsp", b""] {
        w.bytes(s);
        w.bytes(&[0]);
    }
    w.u8(0);

    for name in [
        "entity_state_t",
        "entity_state_player_t",
        "custom_entity_state_t",
    ] {
        description(&mut w, name, ENTITY);
    }
    description(&mut w, "clientdata_t", CLIENT);
    description(&mut w, "weapon_data_t", WEAPON);
    description(&mut w, "event_t", EVENT);
    w.align();

    // Resource list of event scripts, each with no extra data, and no
    // consistency list
    w.u8(svc::RESOURCELIST);
    w.bits(events.len() as u32, 12);
    for &(name, index) in events {
        w.bits(5, 4);
        w.string(name.as_bytes());
        w.bits(index, 12);
        w.bits(0, 24);
        w.bits(0, 3);
        w.bit(false);
    }
    w.bit(false);
    w.align();

    // User message 64, fixed size 2
    w.u8(svc::NEWUSERMSG);
    w.u8(64);
    w.u8(2);
    let mut name = [0u8; 16];
    name[..9].copy_from_slice(b"CurWeapon");
    w.bytes(&name);

    // Baselines for 0, 1 (player), 5
    w.u8(svc::SPAWNBASELINE);
    for (num, model) in [(0u32, 1i64), (1, 2), (5, 7)] {
        w.bits(num, 11);
        w.bits(1, 2);
        delta(&mut w, ENTITY, &[(1, model)]);
    }
    w.bits(0xFFFF, 16);
    w.bits(0, 6);
    w.align();

    w.finish()
}

pub fn time(w: &mut BitWriter, t: f32) {
    w.u8(svc::TIME);
    w.u32(t.to_bits());
}

pub fn full_packet(seq: i32) -> Vec<u8> {
    let mut w = BitWriter::new();
    time(&mut w, 10.);
    w.u8(svc::CLIENTDATA);
    w.bit(false);
    delta(&mut w, CLIENT, &[(0, 100), (1, 1 << 9)]);
    w.bit(true);
    w.bits(9, 6);
    delta(&mut w, WEAPON, &[(0, 9), (1, 0)]);
    w.bit(false);
    w.align();

    w.u8(svc::PACKETENTITIES);
    w.u16(3);
    // 0 from baseline (absolute header), then 1 and 5
    w.bit(false);
    w.bit(true);
    w.bits(0, 11);
    w.bit(false); // custom
    w.bit(false); // no offset
    delta(&mut w, ENTITY, &[]);
    w.bit(true); // numbase + 1
    w.bit(false);
    w.bit(false);
    delta(&mut w, ENTITY, &[(0, -80)]);
    w.bit(false);
    w.bit(false);
    w.bits(4, 6);
    w.bit(false);
    w.bit(false);
    delta(&mut w, ENTITY, &[(2, 5)]);
    w.bits(0, 16);
    w.align();

    let _ = seq;
    w.finish()
}

pub fn delta_packet(from: u8) -> Vec<u8> {
    let mut w = BitWriter::new();
    time(&mut w, 10.5);
    w.u8(svc::CLIENTDATA);
    w.bit(true);
    w.bits(u32::from(from), 8);
    delta(&mut w, CLIENT, &[(0, -3)]);
    w.bit(true);
    w.bits(9, 6);
    delta(&mut w, WEAPON, &[(1, 4)]);
    w.bit(false);
    w.align();

    w.u8(svc::DELTAPACKETENTITIES);
    w.u16(3);
    w.u8(from);
    // Update 1, remove 5, add 7 from a zero baseline
    w.bit(false);
    w.bit(true);
    w.bits(1, 11);
    w.bit(false);
    delta(&mut w, ENTITY, &[(1, 3)]);
    w.bit(true);
    w.bit(false);
    w.bits(4, 6);
    w.bit(false);
    w.bit(false);
    w.bits(2, 6);
    w.bit(false);
    delta(&mut w, ENTITY, &[(1, 9)]);
    w.bits(0, 16);
    w.align();

    // Event for packet index 1 (entity 1) with the stop flag set, then a
    // reliable event for 2
    w.u8(svc::EVENT);
    w.bits(1, 5);
    w.bits(3, 10);
    w.bit(true);
    w.bits(1, 11);
    w.bit(true);
    delta(&mut w, EVENT, &[(2, 1)]);
    w.bit(false);
    w.align();

    w.u8(svc::EVENT_RELIABLE);
    w.bits(3, 10);
    delta(&mut w, EVENT, &[(0, 2), (1, 1)]);
    w.bit(true);
    w.bits(50, 16);
    w.align();

    w.u8(64);
    w.bytes(&[1, 9]);

    w.finish()
}

/// Delta entity packet from `from` that only sets `entity`'s model, so
/// every candidate base keeps the same entity count
pub fn model_update(from: u8, entity: u32, model: i64) -> Vec<u8> {
    let mut w = BitWriter::new();
    time(&mut w, 11.);

    w.u8(svc::DELTAPACKETENTITIES);
    w.u16(3);
    w.u8(from);
    w.bit(false);
    w.bit(true);
    w.bits(entity, 11);
    w.bit(false);
    delta(&mut w, ENTITY, &[(1, model)]);
    w.bits(0, 16);

    w.finish()
}
