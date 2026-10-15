//! Delta and message encoding, the inverse of the decoders. The layouts
//! follow the engine writers (ReHLDS `DELTA_WriteDelta`,
//! `SV_WriteDeltaHeader`, `SV_CreatePacketEntities` for full updates and
//! `SV_WriteClientdataToMessage`)
//!
//! Values are written in their wire form, so a decoded state re-encodes
//! exactly. Time-window fields are the exception, because they are sent
//! relative to the server time of the packet carrying them. They are
//! re-encoded against the new packet's time with an offset that reproduces
//! the decoded value bit for bit where one exists, and counted otherwise

use super::bit::SignMagnitude;
use super::bitwrite::BitWriter;
use super::delta::{DeltaField, DeltaSchema, Kind, StructState, Value, Wire, convert};
use super::messages::svc;

/// How far from the engine's own offset a time-window search goes, which
/// covers the whole range of 8-bit windows
const SEARCH_SPAN: i64 = 1024;

/// Fields an encoder could not reproduce exactly
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EncodeStats {
    /// Time-window fields with no offset in range that decodes to the same
    /// float bits under the new server time
    pub inexact_time_windows: u32,
    /// Of those, the fields whose value lies outside the window altogether,
    /// so the offset is clamped and the value moves by more than rounding
    pub clamped_time_windows: u32,
}

/// A value compared the way the engine compares struct contents, with
/// floats by their bits
#[derive(PartialEq)]
enum Compared {
    Int(i64),
    Float(u32),
    Str(Vec<u8>),
}

fn compared(field: &DeltaField, wire: Option<&Wire>) -> Compared {
    let value = match wire {
        Some(w) => convert(field, w),
        None => zero_value(field.kind()),
    };

    match value {
        Value::Int(i) => Compared::Int(i),
        Value::Float(f) => Compared::Float(f.to_bits()),
        Value::Str(s) => Compared::Str(s),
    }
}

fn zero_value(kind: Kind) -> Value {
    match kind {
        Kind::String => Value::Str(Vec::new()),
        Kind::Float | Kind::Angle | Kind::TimeWindow8 | Kind::TimeWindowBig => Value::Float(0.),
        _ => Value::Int(0),
    }
}

fn time_window_value(field: &DeltaField, offset: i64, time_bits: u32) -> u32 {
    let wire = Wire::TimeWindow {
        offset: SignMagnitude {
            negative: offset < 0,
            magnitude: offset.unsigned_abs() as u32,
        },
        base_time_bits: time_bits,
    };

    match convert(field, &wire) {
        Value::Float(f) => f.to_bits(),
        _ => 0,
    }
}

/// How a time-window value was encoded
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fit {
    Exact,
    Rounded,
    Clamped,
}

/// The offset to send so the client decodes `target` under `time_bits`
fn time_window_offset(field: &DeltaField, target: u32, time_bits: u32) -> (i64, Fit) {
    let width = match field.kind() {
        Kind::TimeWindow8 => 8,
        _ => field.bits,
    };
    let limit = (1i64 << (width - 1)) - 1;

    // The engine's own choice first, then its neighbours, then the range
    let scale = match field.kind() {
        Kind::TimeWindow8 => 100.,
        _ => field.premultiply(),
    };
    let time = f64::from(f32::from_bits(time_bits));
    let value = f64::from(f32::from_bits(target));
    let unclamped = (time * scale) as i64 - (value * scale) as i64;
    let engine = unclamped.clamp(-limit, limit);

    // Wide fields are searched only around the engine's choice
    let span = limit.min(SEARCH_SPAN);
    let near = (-3..=3).map(|d| engine + d);
    let found = near
        .chain(engine - span..=engine + span)
        .filter(|o| (-limit..=limit).contains(o))
        .find(|&o| time_window_value(field, o, time_bits) == target);

    match found {
        Some(o) => (o, Fit::Exact),
        None if unclamped != engine => (engine, Fit::Clamped),
        None => (engine, Fit::Rounded),
    }
}

fn write_value(
    w: &mut BitWriter,
    field: &DeltaField,
    wire: Option<&Wire>,
    time_bits: u32,
    stats: &mut EncodeStats,
) {
    match field.kind() {
        Kind::String => match wire {
            Some(Wire::Str(s)) => w.string(s),
            _ => w.string(b""),
        },
        Kind::TimeWindow8 | Kind::TimeWindowBig => {
            let width = match field.kind() {
                Kind::TimeWindow8 => 8,
                _ => field.bits,
            };
            let target = match compared(field, wire) {
                Compared::Float(bits) => bits,
                _ => 0,
            };

            let (offset, fit) = time_window_offset(field, target, time_bits);
            if fit != Fit::Exact {
                stats.inexact_time_windows += 1;
            }
            if fit == Fit::Clamped {
                stats.clamped_time_windows += 1;
            }
            w.sbits(offset, width);
        }
        _ => match wire {
            Some(Wire::Unsigned(u)) => w.bits(*u, field.bits),
            Some(Wire::Signed(s)) => {
                w.bit(s.negative);
                w.bits(s.magnitude, field.bits - 1);
            }
            _ if field.signed() && field.kind() != Kind::Angle => w.sbits(0, field.bits),
            _ => w.bits(0, field.bits),
        },
    }
}

/// Write `to` as a delta from `base`, sending every field whose value
/// differs. With `force` the mask is written even when nothing differs,
/// otherwise nothing is written and `false` returned
pub fn write_delta(
    w: &mut BitWriter,
    schema: &DeltaSchema,
    base: &StructState,
    to: &StructState,
    force: bool,
    time_bits: u32,
    stats: &mut EncodeStats,
) -> bool {
    let sent: Vec<usize> = schema
        .fields
        .iter()
        .enumerate()
        .take(64)
        .filter(|(_, f)| {
            let before = base.get_wire(f.offset).map(|(_, w)| w);
            let after = to.get_wire(f.offset).map(|(_, w)| w);

            compared(f, before) != compared(f, after)
        })
        .map(|(i, _)| i)
        .collect();
    if sent.is_empty() && !force {
        return false;
    }

    let mut mask = [0u8; 8];
    for &i in &sent {
        mask[i / 8] |= 1 << (i % 8);
    }
    let mask_len = sent.last().map_or(0, |&i| i / 8 + 1);

    w.bits(mask_len as u32, 3);
    for &b in &mask[..mask_len] {
        w.bits(b.into(), 8);
    }

    for i in sent {
        let field = &schema.fields[i];
        let wire = to.get_wire(field.offset).map(|(_, w)| w);
        write_value(w, field, wire, time_bits, stats);
    }

    true
}

/// Indices of the fields of `schema` whose values differ between `a` and
/// `b`, compared as the client stores them
pub fn differing_fields(schema: &DeltaSchema, a: &StructState, b: &StructState) -> Vec<usize> {
    schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            let left = a.get_wire(f.offset).map(|(_, w)| w);
            let right = b.get_wire(f.offset).map(|(_, w)| w);

            compared(f, left) != compared(f, right)
        })
        .map(|(i, _)| i)
        .collect()
}

/// Whether `state` differs from an all-zero struct of `schema`
pub fn differs_from_zero(schema: &DeltaSchema, state: &StructState) -> bool {
    schema.fields.iter().any(|f| {
        let wire = state.get_wire(f.offset).map(|(_, w)| w);

        compared(f, wire) != compared(f, None)
    })
}

pub fn write_time(w: &mut BitWriter, time_bits: u32) {
    w.u8(svc::TIME);
    w.u32(time_bits);
}

/// One entity of a full update, with the schema it is sent with and the
/// spawn baseline it deltas from
pub struct FullEntity<'a> {
    pub number: u16,
    pub custom: bool,
    pub schema: &'a DeltaSchema,
    pub baseline: &'a StructState,
    pub state: &'a StructState,
}

/// `svc_packetentities` with every entity sent from its spawn baseline,
/// never from an instanced baseline or an earlier entity
pub fn write_full_entities(
    w: &mut BitWriter,
    entities: &[FullEntity<'_>],
    instanced_baselines: bool,
    time_bits: u32,
    stats: &mut EncodeStats,
) {
    w.u8(svc::PACKETENTITIES);
    w.u16(entities.len() as u16);

    let mut numbase = 0u16;
    for e in entities {
        let step = i32::from(e.number) - i32::from(numbase);
        if step == 1 {
            w.bit(true);
        } else {
            w.bit(false);
            if step <= 0 || step > 63 {
                w.bit(true);
                w.bits(e.number.into(), 11);
            } else {
                w.bit(false);
                w.bits(step as u32, 6);
            }
        }
        numbase = e.number;

        w.bit(e.custom);
        if instanced_baselines {
            w.bit(false);
        }
        w.bit(false);

        write_delta(w, e.schema, e.baseline, e.state, true, time_bits, stats);
    }

    w.bits(0, 16);
    w.align();
}

/// `svc_clientdata` without a delta base, with every weapon slot that is
/// not all zero
pub fn write_full_clientdata(
    w: &mut BitWriter,
    client_schema: &DeltaSchema,
    client: &StructState,
    weapon_schema: Option<&DeltaSchema>,
    weapons: &[Option<&StructState>],
    time_bits: u32,
    stats: &mut EncodeStats,
) {
    w.u8(svc::CLIENTDATA);
    w.bit(false);

    let zero = StructState::default();
    write_delta(w, client_schema, &zero, client, true, time_bits, stats);

    if let Some(schema) = weapon_schema {
        for (slot, state) in weapons.iter().enumerate().take(64) {
            let Some(state) = state else { continue };
            if !differs_from_zero(schema, state) {
                continue;
            }

            w.bit(true);
            w.bits(slot as u32, 6);
            write_delta(w, schema, &zero, state, true, time_bits, stats);
        }
    }

    w.bit(false);
    w.align();
}
