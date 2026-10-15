//! Synthetic message streams through the decoder. Encodings follow the
//! engine writers documented in docs/protocol-profile-5-48.md

use lambda_slicer_core::container::NetworkFrame;
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::delta::{Value, dt};
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_core::state::{
    ActorResolution, DecodeErrorKind, Decoder, EventSource, FrameLocation,
};

type Field = (&'static str, u32, u32, u32, u32); // name, flags, offset, bits, premultiply

const ENTITY: &[Field] = &[
    ("origin[0]", dt::FLOAT | dt::SIGNED, 0, 16, 8),
    ("modelindex", dt::INTEGER, 4, 10, 1),
    ("animtime", dt::TIMEWINDOW_8, 8, 8, 1),
];
const CLIENT: &[Field] = &[
    ("health", dt::FLOAT | dt::SIGNED, 0, 10, 1),
    ("weapons", dt::INTEGER, 4, 32, 1),
];
const WEAPON: &[Field] = &[
    ("m_iId", dt::INTEGER, 0, 5, 1),
    ("m_iClip", dt::INTEGER | dt::SIGNED, 4, 10, 1),
];
const EVENT: &[Field] = &[
    ("entindex", dt::INTEGER, 0, 11, 1),
    ("bparam1", dt::INTEGER, 4, 1, 1),
    ("bparam2", dt::INTEGER, 8, 1, 1),
];

fn description(w: &mut BitWriter, name: &str, fields: &[Field]) {
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
fn delta(w: &mut BitWriter, fields: &[Field], sent: &[(usize, i64)]) {
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

fn setup() -> Vec<u8> {
    let mut w = BitWriter::new();

    w.u8(svc::SERVERINFO);
    w.u32(48);
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

    // Resource list with one event script at index 3
    w.u8(svc::RESOURCELIST);
    w.bits(1, 12);
    w.bits(5, 4);
    w.string(b"events/gauss.sc");
    w.bits(3, 12);
    w.bits(0, 24);
    w.bits(0, 3);
    w.bit(false);
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

fn frame(seq: i32, payload: Vec<u8>) -> NetworkFrame {
    NetworkFrame {
        demo_info: Box::new([0; 436]),
        sequence: [seq, 0, 0, 0, 0, 0, 0],
        message: payload,
    }
}

fn loc(n: u64) -> FrameLocation {
    FrameLocation {
        entry: 1,
        ordinal: n,
        offset: 0,
        time_bits: 0,
        frame_number: n as i32,
    }
}

fn time(w: &mut BitWriter, t: f32) {
    w.u8(svc::TIME);
    w.u32(t.to_bits());
}

fn full_packet(seq: i32) -> Vec<u8> {
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

fn delta_packet(from: u8) -> Vec<u8> {
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

#[test]
fn reconstructs_full_and_delta_packets() {
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    assert_eq!(d.max_clients, 2);
    assert_eq!(d.event_name(3), Some(&b"events/gauss.sc"[..]));

    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    let snap = d.snapshot_for_byte(10).unwrap().clone();
    let ents = snap.entities.as_ref().unwrap();
    assert_eq!(ents.iter().map(|e| e.number).collect::<Vec<_>>(), [0, 1, 5]);
    assert_eq!(d.entity_value(&ents[1], "modelindex"), Some(Value::Int(2)));
    assert_eq!(
        d.entity_value(&ents[1], "origin[0]"),
        Some(Value::Float(-10.))
    );
    assert_eq!(d.entity_value(&ents[2], "modelindex"), Some(Value::Int(7)));
    assert_eq!(
        d.entity_value(&ents[2], "animtime"),
        Some(Value::Float(9.95))
    );

    d.network_frame(loc(2), &frame(11, delta_packet(10)), |_, _| {})
        .unwrap();
    let snap = d.snapshot_for_byte(11).unwrap().clone();
    let ents = snap.entities.as_ref().unwrap();
    assert_eq!(ents.iter().map(|e| e.number).collect::<Vec<_>>(), [0, 1, 7]);

    // Entity 1 keeps its origin from the delta base and takes a new model
    assert_eq!(
        d.entity_value(&ents[1], "origin[0]"),
        Some(Value::Float(-10.))
    );
    assert_eq!(d.entity_value(&ents[1], "modelindex"), Some(Value::Int(3)));
    assert_eq!(d.entity_value(&ents[2], "modelindex"), Some(Value::Int(9)));

    // Entity 0 is carried over unchanged (same shared state)
    assert!(std::sync::Arc::ptr_eq(
        &ents[0],
        &d.snapshot_for_byte(10).unwrap().entities.as_ref().unwrap()[0]
    ));

    let client = d.schema("clientdata_t").unwrap();
    let cd = snap.clientdata.as_ref().unwrap();
    assert_eq!(cd.value(&client, "health"), Some(Value::Float(-3.)));
    assert_eq!(cd.value(&client, "weapons"), Some(Value::Int(1 << 9)));

    let weapon = d.schema("weapon_data_t").unwrap();
    let w9 = snap.weapons.as_ref().unwrap()[9].as_ref().unwrap();
    assert_eq!(w9.value(&weapon, "m_iId"), Some(Value::Int(9)));
    assert_eq!(w9.value(&weapon, "m_iClip"), Some(Value::Int(4)));

    let ev: Vec<_> = d
        .events
        .iter()
        .filter(|e| e.source != EventSource::Demo)
        .collect();
    assert_eq!(ev.len(), 2);
    assert_eq!(
        ev[0].actor,
        ActorResolution::PacketEntity {
            packet_index: 1,
            entity: 1
        }
    );
    assert_eq!(ev[0].resource.as_deref(), Some(&b"events/gauss.sc"[..]));

    let event_t = d.schema("event_t").unwrap();
    assert_eq!(
        ev[0].args.as_ref().unwrap().value(&event_t, "bparam2"),
        Some(Value::Int(1))
    );

    assert_eq!(ev[1].source, EventSource::SvcReliable);
    assert_eq!(ev[1].actor, ActorResolution::Explicit { entity: 2 });
    assert_eq!(ev[1].delay_raw, Some(50));

    assert_eq!(d.stats.user_messages.get("CurWeapon"), Some(&1));
    assert_eq!(d.stats.entities_without_baseline, 1);
}

#[test]
fn delta_without_any_reproducing_base_is_an_error() {
    // No frame received yet, so the empty base cannot give the declared count
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();

    let err = d
        .network_frame(loc(1), &frame(11, delta_packet(10)), |_, _| {})
        .unwrap_err();

    assert!(
        matches!(
            err.kind,
            DecodeErrorKind::CountMismatch {
                declared: 3,
                actual: 2
            }
        ),
        "{err}"
    );
    assert_eq!(d.stats.empty_delta_bases, 1);
}

#[test]
fn shifted_sequence_numbering_is_recovered_by_entity_count() {
    // The recording numbers the full packet 10 but the server calls it 99
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(11, delta_packet(99)), |_, _| {})
        .unwrap();

    assert_eq!(d.stats.sequence_offset_changes, 1);
    assert_eq!(d.stats.sequence_offset, 10u8.wrapping_sub(99));

    let snap = d.snapshot_for_byte(100).unwrap();
    assert_eq!(snap.sequence, 11);
    let ents = snap.entities.as_ref().unwrap();
    assert_eq!(ents.iter().map(|e| e.number).collect::<Vec<_>>(), [0, 1, 7]);

    // Clientdata used the same base, so the health delta from 100 is applied
    // to it
    let client = d.schema("clientdata_t").unwrap();
    let weapon = d.schema("weapon_data_t").unwrap();
    let w9 = snap.weapons.as_ref().unwrap()[9].as_ref().unwrap();
    assert_eq!(w9.value(&weapon, "m_iId"), Some(Value::Int(9)));
    assert_eq!(
        snap.clientdata.as_ref().unwrap().value(&client, "weapons"),
        Some(Value::Int(1 << 9))
    );
}

#[test]
fn delta_base_is_found_by_ring_slot() {
    // With the multiplayer ring of 64, reference byte 74 resolves to
    // sequence 10
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(80, delta_packet(74)), |_, _| {})
        .unwrap();

    assert_eq!(d.backup(), 64);
    assert_eq!(d.stats.delta_base_byte_mismatches, 2);
    assert_eq!(d.stats.empty_delta_bases, 0);
}

#[test]
fn declared_count_mismatch_is_an_error() {
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();

    let mut p = full_packet(10);
    // Declared count lives right after the svc_packetentities opcode
    let at = p.iter().position(|&b| b == svc::PACKETENTITIES).unwrap();
    p[at + 1] = 4;

    let err = d
        .network_frame(loc(1), &frame(10, p), |_, _| {})
        .unwrap_err();

    assert!(
        matches!(
            err.kind,
            DecodeErrorKind::CountMismatch {
                declared: 4,
                actual: 3
            }
        ),
        "{err}"
    );
}

#[test]
fn unknown_opcode_and_unregistered_user_message_stop_decoding() {
    for op in [svc::BAD, svc::DAMAGE, 60, 200] {
        let mut d = Decoder::new();
        let err = d
            .network_frame(loc(0), &frame(1, vec![svc::NOP, op, 1, 2, 3]), |_, _| {})
            .unwrap_err();

        assert_eq!(err.message_ordinal, 1);
        assert_eq!(err.payload_offset, 1);
        assert_eq!(err.opcode, op);
    }
}

#[test]
fn truncated_messages_never_panic() {
    let mut payload = setup();
    payload.extend(full_packet(10));

    for cut in 0..payload.len() {
        let mut d = Decoder::new();
        let _ = d.network_frame(loc(0), &frame(1, payload[..cut].to_vec()), |_, _| {});
    }
}
