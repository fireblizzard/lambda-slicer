//! Synthetic message streams through the decoder. Encodings follow the
//! engine writers documented in docs/protocol-profile-5-48.md

mod common;

use std::sync::Arc;

use common::stream::*;
use lambda_slicer_core::container::NetworkFrame;
use lambda_slicer_core::protocol::bitwrite::BitWriter;
use lambda_slicer_core::protocol::delta::{StructState, Value};
use lambda_slicer_core::protocol::encode::{
    EncodeStats, FullEntity, differing_fields, write_full_clientdata, write_full_entities,
    write_time,
};
use lambda_slicer_core::protocol::messages::svc;
use lambda_slicer_core::state::{
    ActorResolution, DecodeErrorKind, Decoder, EntityState, EventSource, FrameLocation,
    NameChangeKind, Snapshot,
};

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
    assert_eq!(d.stats.bases_count_search, 1);
    assert_eq!(d.stats.inferred_snapshots, 1);

    let snap = d.snapshot_for_byte(100).unwrap();
    assert_eq!(snap.sequence, 11);
    assert!(snap.inferred);
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

fn model_of(d: &Decoder, seq: u32, entity: u16) -> Option<Value> {
    let snap = d.latest_snapshot().filter(|s| s.sequence == seq)?;
    let e = snap
        .entities
        .as_ref()?
        .iter()
        .find(|e| e.number == entity)?;
    d.entity_value(e, "modelindex")
}

#[test]
fn exact_reference_picks_the_named_frame_among_equal_count_candidates() {
    // Frames 10 and 11 hold the same three entities with different models
    // for entity 1. A delta naming 10 must build on 10, not on the newer 11
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(11, model_update(10, 1, 3)), |_, _| {})
        .unwrap();
    assert_eq!(model_of(&d, 11, 1), Some(Value::Int(3)));

    d.network_frame(loc(3), &frame(12, model_update(10, 5, 8)), |_, _| {})
        .unwrap();

    assert_eq!(model_of(&d, 12, 1), Some(Value::Int(2)));
    assert_eq!(model_of(&d, 12, 5), Some(Value::Int(8)));
    assert!(!d.latest_snapshot().unwrap().inferred);
    assert_eq!(d.stats.bases_strict, 2);
    assert_eq!(d.stats.inferred_snapshots, 0);
}

#[test]
fn equal_count_base_without_an_exact_reference_is_inferred() {
    // No frame carries byte 200, and frames 10 and 11 both reproduce the
    // declared count. The search takes one of them, but the result must not
    // be reported as proven
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(11, model_update(10, 1, 3)), |_, _| {})
        .unwrap();
    d.network_frame(loc(3), &frame(12, model_update(200, 5, 8)), |_, _| {})
        .unwrap();

    let snap = d.latest_snapshot().unwrap();
    assert_eq!(snap.sequence, 12);
    assert!(snap.inferred);
    assert_eq!(d.stats.bases_count_search, 1);
    assert_eq!(d.stats.inferred_snapshots, 1);
    assert_eq!(d.stats.first_inferred, Some(loc(3)));

    // Inference carries into deltas built on the inferred frame, even when
    // the reference to it is exact
    d.network_frame(loc(4), &frame(13, model_update(12, 5, 9)), |_, _| {})
        .unwrap();
    assert!(d.latest_snapshot().unwrap().inferred);
    assert_eq!(d.stats.inferred_snapshots, 2);

    // A full update rebuilds from baselines alone and clears it
    d.network_frame(loc(5), &frame(14, full_packet(14)), |_, _| {})
        .unwrap();
    assert!(!d.latest_snapshot().unwrap().inferred);
}

#[test]
fn client_drops_deltas_once_their_base_is_63_packets_back() {
    // A lagging client keeps acknowledging frame 10 while the server streams
    // on. Up to 62 packets back the client applies the delta, from 63 it
    // drops the packet as too old until a full update arrives
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    for seq in 11..=72 {
        d.network_frame(loc(2), &frame(seq, model_update(10, 5, 8)), |_, _| {})
            .unwrap();
    }
    assert_eq!(model_of(&d, 72, 5), Some(Value::Int(8)));

    for seq in 73..=80 {
        d.network_frame(loc(3), &frame(seq, model_update(10, 1, 3)), |_, _| {})
            .unwrap();
    }

    // The dropped packets left no state and are not used as bases
    let latest = d.latest_snapshot().unwrap();
    assert_eq!(latest.sequence, 72);
    assert!(!latest.inferred);
    assert_eq!(d.stats.discarded_packets, 8);
    assert_eq!(d.stats.bases_strict, 62);

    // A full update ends the run, and deltas from it apply again
    d.network_frame(loc(4), &frame(81, full_packet(81)), |_, _| {})
        .unwrap();
    d.network_frame(loc(5), &frame(82, model_update(81, 1, 3)), |_, _| {})
        .unwrap();
    assert_eq!(model_of(&d, 82, 1), Some(Value::Int(3)));
    assert_eq!(d.stats.discarded_packets, 8);
}

#[test]
fn a_dropped_run_ends_at_a_reference_to_a_kept_frame() {
    // The client's next acknowledgement names frame 72, which it kept, so
    // the packet after the dropped one applies again
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    for seq in 11..=73 {
        d.network_frame(loc(2), &frame(seq, model_update(10, 5, 8)), |_, _| {})
            .unwrap();
    }
    assert_eq!(d.stats.discarded_packets, 1);

    d.network_frame(loc(3), &frame(74, model_update(72, 1, 3)), |_, _| {})
        .unwrap();

    assert_eq!(model_of(&d, 74, 1), Some(Value::Int(3)));
    assert_eq!(model_of(&d, 74, 5), Some(Value::Int(8)));
    assert!(!d.latest_snapshot().unwrap().inferred);
    assert_eq!(d.stats.discarded_packets, 1);
}

#[test]
fn a_dropped_run_past_255_packets_stays_dropped_when_the_byte_wraps() {
    // After 256 packets the reference byte comes back into range, but it
    // names a frame the client dropped, so the run goes on
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    for seq in 11..=270 {
        d.network_frame(loc(2), &frame(seq, model_update(10, 5, 8)), |_, _| {})
            .unwrap();
    }

    assert_eq!(d.latest_snapshot().unwrap().sequence, 72);
    assert_eq!(d.stats.discarded_packets, 270 - 72);
}

#[test]
fn a_dropped_run_goes_on_across_a_gap_in_the_recording() {
    // A lag burst leaves sequences 74 to 139 unrecorded. That starts a new
    // numbering run, but the client still drops packets naming frame 10
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    for seq in 11..=73 {
        d.network_frame(loc(2), &frame(seq, model_update(10, 5, 8)), |_, _| {})
            .unwrap();
    }
    d.network_frame(loc(3), &frame(140, model_update(10, 1, 3)), |_, _| {})
        .unwrap();

    assert_eq!(d.latest_snapshot().unwrap().sequence, 72);
    assert_eq!(d.stats.discarded_packets, 2);
    assert_eq!(d.stats.bases_count_search, 0);
    assert_eq!(d.stats.inferred_snapshots, 0);
}

#[test]
fn a_sudden_far_reference_is_inferred_not_dropped() {
    // Byte 10 is 64 packets back, but no run led there, so this is not the
    // client dropping a lagging packet. It may be shifted numbering, and the
    // ring occupant is not trusted either
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(74, model_update(10, 5, 8)), |_, _| {})
        .unwrap();

    let latest = d.latest_snapshot().unwrap();
    assert_eq!(latest.sequence, 74);
    assert!(latest.inferred);
    assert_eq!(d.stats.discarded_packets, 0);
    assert_eq!(d.stats.bases_strict, 0);
    assert_eq!(d.stats.bases_count_search, 1);
}

#[test]
fn a_base_from_before_the_recording_is_inferred_without_suspecting_the_numbering() {
    // Playback numbering starts at 1000, unrelated to the loading segment,
    // and the first delta names 996, which the client received before
    // recording began. The guess is inferred, but it is no offset
    let before = 996u32 as u8;
    let first = 1000u32 as u8;

    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(1000, full_packet(1000)), |_, _| {})
        .unwrap();
    d.network_frame(loc(3), &frame(1001, full_packet(1001)), |_, _| {})
        .unwrap();
    d.network_frame(loc(4), &frame(1002, model_update(before, 5, 8)), |_, _| {})
        .unwrap();

    let latest = d.latest_snapshot().unwrap();
    assert_eq!(latest.sequence, 1002);
    assert!(latest.inferred);
    assert_eq!(d.stats.unrecorded_bases, 1);
    assert_eq!(d.stats.bases_count_search, 1);
    assert_eq!(d.stats.sequence_offset_changes, 0);
    assert_eq!(d.stats.first_inferred, Some(loc(4)));

    // The next reference names a recorded frame and stays strict
    d.network_frame(loc(5), &frame(1003, model_update(first, 1, 3)), |_, _| {})
        .unwrap();
    let latest = d.latest_snapshot().unwrap();
    assert!(!latest.inferred);
    assert_eq!(model_of(&d, 1003, 1), Some(Value::Int(3)));
    assert_eq!(model_of(&d, 1003, 5), Some(Value::Int(7)));
    assert_eq!(d.stats.bases_strict, 1);
    assert_eq!(d.stats.inferred_snapshots, 1);
    assert_eq!(d.snapshot_for_byte(first).unwrap().sequence, 1000);
}

#[test]
fn stale_frame_outside_the_server_window_is_not_trusted() {
    // Frame 10 has the reference's low byte and the right entity count, but
    // it is 66 packets old, so the server's slot no longer held it
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(75, full_packet(75)), |_, _| {})
        .unwrap();
    d.network_frame(loc(3), &frame(76, model_update(10, 5, 8)), |_, _| {})
        .unwrap();

    assert_eq!(d.backup(), 64);
    assert!(d.latest_snapshot().unwrap().inferred);
    assert_eq!(d.stats.bases_strict, 0);
    assert_eq!(d.stats.bases_count_search, 1);
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

fn userinfo(w: &mut BitWriter, slot: u8, user_id: u32, info: &str) {
    w.u8(svc::UPDATEUSERINFO);
    w.u8(slot);
    w.u32(user_id);
    w.bytes(info.as_bytes());
    w.bytes(&[0]);
    w.bytes(&[0; 16]);
}

fn reliable_event(w: &mut BitWriter, entity: i64) {
    w.u8(svc::EVENT_RELIABLE);
    w.bits(3, 10);
    delta(w, EVENT, &[(0, entity)]);
    w.bit(false);
    w.align();
}

#[test]
fn events_keep_the_player_identity_of_their_time() {
    // Alice fires from slot 1 (entity 2), renames, leaves, and Bob takes
    // the slot. Each event must keep the name it had when it happened
    let mut w = BitWriter::new();
    userinfo(&mut w, 1, 5, r"\name\Alice\model\gordon");
    reliable_event(&mut w, 2);
    userinfo(&mut w, 1, 5, r"\name\Ally\model\gordon");
    reliable_event(&mut w, 2);
    userinfo(&mut w, 1, 5, "");
    reliable_event(&mut w, 2);
    userinfo(&mut w, 1, 6, r"\name\Bob");
    reliable_event(&mut w, 2);
    // Resending identical userinfo is not a change
    userinfo(&mut w, 1, 6, r"\name\Bob");

    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(2, w.finish()), |_, _| {})
        .unwrap();

    let who: Vec<_> = d
        .events
        .iter()
        .map(|e| {
            e.player
                .as_ref()
                .map(|p| (p.name.as_str(), p.incarnation, p.user_id))
        })
        .collect();
    assert_eq!(
        who,
        [
            Some(("Alice", 0, 5)),
            Some(("Ally", 0, 5)),
            None,
            Some(("Bob", 1, 6)),
        ]
    );

    let history: Vec<_> = d
        .name_history
        .iter()
        .map(|c| (c.kind, c.player.name.as_str(), c.player.incarnation))
        .collect();
    assert_eq!(
        history,
        [
            (NameChangeKind::Joined, "Alice", 0),
            (NameChangeKind::Renamed, "Ally", 0),
            (NameChangeKind::Left, "Ally", 0),
            (NameChangeKind::Joined, "Bob", 1),
        ]
    );

    // Only the current occupant remains in the player table
    assert_eq!(d.players[&1].name, "Bob");
}

#[test]
fn player_identities_do_not_cross_a_session_boundary() {
    // Alice holds slot 1 in the first session. In the second, an event from
    // entity 2 arrives before any userinfo and must not be credited to her
    let mut first = BitWriter::new();
    userinfo(&mut first, 1, 5, r"\name\Alice");
    reliable_event(&mut first, 2);

    let mut early = BitWriter::new();
    reliable_event(&mut early, 2);

    let mut late = BitWriter::new();
    userinfo(&mut late, 1, 9, r"\name\Carol");
    reliable_event(&mut late, 2);

    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(2, first.finish()), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(3, setup()), |_, _| {})
        .unwrap();
    assert!(d.players.is_empty());

    d.network_frame(loc(3), &frame(4, early.finish()), |_, _| {})
        .unwrap();
    d.network_frame(loc(4), &frame(5, late.finish()), |_, _| {})
        .unwrap();

    let who: Vec<_> = d
        .events
        .iter()
        .map(|e| {
            e.player
                .as_ref()
                .map(|p| (p.segment, p.name.as_str(), p.incarnation))
        })
        .collect();
    assert_eq!(who, [Some((1, "Alice", 0)), None, Some((2, "Carol", 0))]);

    // The session change itself is not a departure
    let history: Vec<_> = d
        .name_history
        .iter()
        .map(|c| (c.kind, c.player.segment, c.player.name.as_str()))
        .collect();
    assert_eq!(
        history,
        [
            (NameChangeKind::Joined, 1, "Alice"),
            (NameChangeKind::Joined, 2, "Carol"),
        ]
    );
}

#[test]
fn non_player_entities_have_no_player_identity() {
    let mut w = BitWriter::new();
    userinfo(&mut w, 1, 5, r"\name\Alice");
    reliable_event(&mut w, 40);

    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(2, w.finish()), |_, _| {})
        .unwrap();

    assert_eq!(d.events[0].actor, ActorResolution::Explicit { entity: 40 });
    assert_eq!(d.events[0].player, None);
}

#[test]
fn serverinfo_with_another_protocol_stops_decoding() {
    let mut d = Decoder::new();
    let err = d
        .network_frame(loc(0), &frame(1, setup_with_protocol(47)), |_, _| {})
        .unwrap_err();

    assert_eq!(err.kind, DecodeErrorKind::UnsupportedProtocol(47));
    assert_eq!(err.opcode, svc::SERVERINFO);
}

/// Encode the latest snapshot as a full update, the way a restart export
/// bootstraps a slice
fn encode_latest(d: &Decoder, stats: &mut EncodeStats) -> Vec<u8> {
    let snap = d.latest_snapshot().unwrap();
    let mut w = BitWriter::new();
    write_time(&mut w, snap.server_time_bits);

    let weapons: Vec<_> = snap
        .weapons
        .as_ref()
        .unwrap()
        .iter()
        .map(|s| s.as_deref())
        .collect();
    write_full_clientdata(
        &mut w,
        &d.schema("clientdata_t").unwrap(),
        snap.clientdata.as_ref().unwrap(),
        d.schema("weapon_data_t").as_deref(),
        &weapons,
        snap.server_time_bits,
        stats,
    );

    let empty = StructState::default();
    let entities = snap.entities.as_ref().unwrap();
    let schemas: Vec<_> = entities
        .iter()
        .map(|e| d.entity_schema(e.number, e.custom).unwrap())
        .collect();
    let full: Vec<_> = entities
        .iter()
        .zip(&schemas)
        .map(|(e, schema)| FullEntity {
            number: e.number,
            custom: e.custom,
            schema,
            baseline: d.baseline(e.number).map_or(&empty, |b| &b.fields),
            state: &e.fields,
        })
        .collect();
    write_full_entities(
        &mut w,
        &full,
        d.instanced_baselines() > 0,
        snap.server_time_bits,
        stats,
    );

    w.finish()
}

fn assert_same_state(d: &Decoder, a: &Snapshot, b: &Snapshot) {
    let (ea, eb) = (a.entities.as_ref().unwrap(), b.entities.as_ref().unwrap());
    let numbers = |e: &[Arc<EntityState>]| e.iter().map(|x| x.number).collect::<Vec<_>>();
    assert_eq!(numbers(ea), numbers(eb));
    for (x, y) in ea.iter().zip(eb.iter()) {
        let schema = d.entity_schema(x.number, x.custom).unwrap();
        assert_eq!(
            differing_fields(&schema, &x.fields, &y.fields),
            Vec::<usize>::new()
        );
    }

    let client = d.schema("clientdata_t").unwrap();
    let (ca, cb) = (
        a.clientdata.as_ref().unwrap(),
        b.clientdata.as_ref().unwrap(),
    );
    assert_eq!(differing_fields(&client, ca, cb), Vec::<usize>::new());

    let weapon = d.schema("weapon_data_t").unwrap();
    let empty = StructState::default();
    let (wa, wb) = (a.weapons.as_ref().unwrap(), b.weapons.as_ref().unwrap());
    for slot in 0..64 {
        let left = wa[slot].as_deref().unwrap_or(&empty);
        let right = wb[slot].as_deref().unwrap_or(&empty);
        assert_eq!(differing_fields(&weapon, left, right), Vec::<usize>::new());
    }
}

#[test]
fn a_full_update_reencodes_the_decoded_state() {
    // Entities 0, 1 and 7 after a delta that removed 5, with clientdata and
    // weapon slot 9 built over two packets
    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(11, delta_packet(10)), |_, _| {})
        .unwrap();
    let before = d.latest_snapshot().unwrap().clone();

    let mut stats = EncodeStats::default();
    let payload = encode_latest(&d, &mut stats);
    d.network_frame(loc(3), &frame(12, payload), |_, _| {})
        .unwrap();

    let after = d.latest_snapshot().unwrap().clone();
    assert_eq!(after.sequence, 12);
    assert!(!after.inferred);
    assert_same_state(&d, &before, &after);
    assert!(
        stats.approximations.is_empty(),
        "{:?}",
        stats.approximations
    );
}

#[test]
fn time_windows_are_reencoded_against_the_new_server_time() {
    // animtime 10.45 arrives at server time 10.5 (offset 5). Encoded at
    // 11.0 it needs offset 55 to decode to the same float
    let mut w = BitWriter::new();
    time(&mut w, 10.5);
    w.u8(svc::DELTAPACKETENTITIES);
    w.u16(3);
    w.u8(10);
    w.bit(false);
    w.bit(true);
    w.bits(5, 11);
    w.bit(false);
    delta(&mut w, ENTITY, &[(2, 5)]);
    w.bits(0, 16);

    let mut d = Decoder::new();
    d.network_frame(loc(0), &frame(1, setup()), |_, _| {})
        .unwrap();
    d.network_frame(loc(1), &frame(10, full_packet(10)), |_, _| {})
        .unwrap();
    d.network_frame(loc(2), &frame(11, w.finish()), |_, _| {})
        .unwrap();
    let before = d.latest_snapshot().unwrap().clone();

    let mut stats = EncodeStats::default();
    let mut moved = (*before).clone();
    moved.server_time_bits = 11f32.to_bits();
    let mut w = BitWriter::new();
    write_time(&mut w, moved.server_time_bits);
    let entities = moved.entities.as_ref().unwrap();
    let schemas: Vec<_> = entities
        .iter()
        .map(|e| d.entity_schema(e.number, e.custom).unwrap())
        .collect();
    let baselines: Vec<_> = entities
        .iter()
        .map(|e| {
            d.baseline(e.number)
                .map_or_else(StructState::default, |b| b.fields.clone())
        })
        .collect();
    let full: Vec<_> = entities
        .iter()
        .zip(schemas.iter().zip(&baselines))
        .map(|(e, (schema, baseline))| FullEntity {
            number: e.number,
            custom: e.custom,
            schema,
            baseline,
            state: &e.fields,
        })
        .collect();
    write_full_entities(&mut w, &full, false, moved.server_time_bits, &mut stats);
    d.network_frame(loc(3), &frame(12, w.finish()), |_, _| {})
        .unwrap();

    let schema = d.entity_schema(5, false).unwrap();
    let animtime = |s: &Snapshot| {
        let e = s
            .entities
            .as_ref()
            .unwrap()
            .iter()
            .find(|e| e.number == 5)
            .unwrap();
        e.fields.value(&schema, "animtime")
    };
    assert_eq!(animtime(&before), Some(Value::Float(10.45)));
    assert_eq!(
        animtime(d.latest_snapshot().unwrap()),
        Some(Value::Float(10.45))
    );
    assert_eq!(stats.inexact_time_windows, 0);

    // 2 s back is outside the 8-bit window, so the offset is clamped
    let mut stats = EncodeStats::default();
    let mut w = BitWriter::new();
    write_full_entities(&mut w, &full, false, 12.5f32.to_bits(), &mut stats);
    assert_eq!(stats.clamped_time_windows, 1);
}
