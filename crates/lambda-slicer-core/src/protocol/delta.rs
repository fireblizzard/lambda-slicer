//! Dynamic delta schemas and delta decoding
//!
//! Schemas arrive in `svc_deltadescription`, themselves delta-encoded with a
//! fixed meta-schema. Decoded values are kept in wire form so they can be
//! re-encoded exactly. Converted values are computed on demand with the
//! engine's formulas (ReHLDS `DELTA_ParseDelta`, client side)

use std::sync::Arc;

use super::bit::{BitError, BitReader, SignMagnitude};

/// `fieldType` flags
pub mod dt {
    pub const BYTE: u32 = 1 << 0;
    pub const SHORT: u32 = 1 << 1;
    pub const FLOAT: u32 = 1 << 2;
    pub const INTEGER: u32 = 1 << 3;
    pub const ANGLE: u32 = 1 << 4;
    pub const TIMEWINDOW_8: u32 = 1 << 5;
    pub const TIMEWINDOW_BIG: u32 = 1 << 6;
    pub const STRING: u32 = 1 << 7;
    pub const SIGNED: u32 = 1 << 31;
}

/// The send mask is at most 7 bytes (3-bit count), so 56 fields
pub const MAX_SENDABLE_FIELDS: usize = 56;
/// Hard cap on schema size accepted from a file
pub const MAX_SCHEMA_FIELDS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Byte,
    Short,
    Float,
    Integer,
    Angle,
    TimeWindow8,
    TimeWindowBig,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeltaError {
    #[error(transparent)]
    Bits(#[from] BitError),
    #[error("field {field:?}: unsupported field type {flags:#x}")]
    BadType { field: String, flags: u32 },
    #[error("field {field:?}: {bits} significant bits is out of range")]
    BadBits { field: String, bits: u32 },
    #[error("schema {name:?} has {count} fields (limit {MAX_SCHEMA_FIELDS})")]
    TooManyFields { name: String, count: usize },
    #[error("empty delta description name")]
    EmptyName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeltaField {
    pub name: Vec<u8>,
    /// Raw `fieldType` flags
    pub flags: u32,
    pub offset: u32,
    pub size: u32,
    pub bits: u32,
    /// Wire value, and the multiplier is `raw / 4000`
    pub premultiply_raw: u32,
    pub postmultiply_raw: u32,
    kind: Kind,
}

fn multiplier(raw: u32) -> f64 {
    // Meta-schema float with premultiply 4000, stored by the engine as f32
    f64::from((f64::from(raw) / 4000.) as f32)
}

fn not_one(m: f64) -> bool {
    m <= 0.9999 || m >= 1.0001
}

impl DeltaField {
    pub fn new(
        name: &[u8],
        flags: u32,
        offset: u32,
        size: u32,
        bits: u32,
        premultiply_raw: u32,
        postmultiply_raw: u32,
    ) -> Result<Self, DeltaError> {
        let display = String::from_utf8_lossy(name).into_owned();
        let kind = match flags & !dt::SIGNED {
            dt::BYTE => Kind::Byte,
            dt::SHORT => Kind::Short,
            dt::FLOAT => Kind::Float,
            dt::INTEGER => Kind::Integer,
            dt::ANGLE => Kind::Angle,
            dt::TIMEWINDOW_8 => Kind::TimeWindow8,
            dt::TIMEWINDOW_BIG => Kind::TimeWindowBig,
            dt::STRING => Kind::String,
            _ => {
                return Err(DeltaError::BadType {
                    field: display,
                    flags,
                });
            }
        };

        let needs_bits = !matches!(kind, Kind::String | Kind::TimeWindow8);
        if needs_bits && !(1..=32).contains(&bits) {
            return Err(DeltaError::BadBits {
                field: display,
                bits,
            });
        }

        Ok(Self {
            name: name.to_vec(),
            flags,
            offset,
            size,
            bits,
            premultiply_raw,
            postmultiply_raw,
            kind,
        })
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn signed(&self) -> bool {
        self.flags & dt::SIGNED != 0
    }

    pub fn premultiply(&self) -> f64 {
        multiplier(self.premultiply_raw)
    }

    pub fn postmultiply(&self) -> f64 {
        multiplier(self.postmultiply_raw)
    }

    pub fn name_lossy(&self) -> String {
        String::from_utf8_lossy(&self.name).into_owned()
    }

    fn read(&self, r: &mut BitReader<'_>, time_base_bits: u32) -> Result<Wire, BitError> {
        Ok(match self.kind {
            Kind::String => Wire::Str(r.bit_string()?.into()),
            Kind::TimeWindow8 => Wire::TimeWindow {
                offset: r.sbits(8)?,
                base_time_bits: time_base_bits,
            },
            Kind::TimeWindowBig => Wire::TimeWindow {
                offset: r.sbits(self.bits)?,
                base_time_bits: time_base_bits,
            },
            Kind::Angle => Wire::Unsigned(r.bits(self.bits)?),
            _ if self.signed() => Wire::Signed(r.sbits(self.bits)?),
            _ => Wire::Unsigned(r.bits(self.bits)?),
        })
    }
}

/// A field value exactly as transmitted
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Wire {
    Unsigned(u32),
    Signed(SignMagnitude),
    Str(Box<[u8]>),
    /// Time-window offset relative to the server time of the packet that
    /// carried it (`svc_time`, kept as f32 bits)
    TimeWindow {
        offset: SignMagnitude,
        base_time_bits: u32,
    },
}

/// Converted value as the engine would store it
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f32),
    Str(Vec<u8>),
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(f64::from(*f)),
            Value::Str(_) => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::Str(_) => None,
        }
    }
}

/// Engine conversion for a field value decoded with `field`
pub fn convert(field: &DeltaField, wire: &Wire) -> Value {
    let scaled = |d: f64| {
        let mut d = d;
        let pre = field.premultiply();
        if not_one(pre) {
            d /= pre;
        }

        let post = field.postmultiply();
        if not_one(post) {
            d *= post;
        }

        d
    };

    let raw = match wire {
        Wire::Str(s) => return Value::Str(s.to_vec()),
        Wire::TimeWindow {
            offset,
            base_time_bits,
        } => {
            let time = f64::from(f32::from_bits(*base_time_bits));
            let addt = offset.value() as f64;
            let t = match field.kind {
                Kind::TimeWindow8 => (time * 100. - addt) / 100.,
                _ => {
                    let pre = field.premultiply();
                    if not_one(pre) {
                        (time * pre - addt) / pre
                    } else {
                        time - addt
                    }
                }
            };

            return Value::Float(t as f32);
        }
        Wire::Unsigned(u) => f64::from(*u),
        Wire::Signed(s) => s.value() as f64,
    };

    let signed = field.signed();
    match field.kind {
        Kind::Angle => Value::Float((raw * (360. / (1u64 << field.bits) as f64)) as f32),
        Kind::Float => Value::Float(scaled(raw) as f32),
        Kind::Byte if signed => Value::Int(i64::from(scaled(raw) as i8)),
        Kind::Byte => Value::Int(i64::from(scaled(raw) as u8)),
        Kind::Short if signed => Value::Int(i64::from(scaled(raw) as i16)),
        Kind::Short => Value::Int(i64::from(scaled(raw) as u16)),
        Kind::Integer if signed => Value::Int(i64::from(scaled(raw) as i32)),
        _ => Value::Int(i64::from(scaled(raw) as u32)),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeltaSchema {
    pub name: Vec<u8>,
    pub fields: Vec<Arc<DeltaField>>,
}

impl DeltaSchema {
    pub fn name_lossy(&self) -> String {
        String::from_utf8_lossy(&self.name).into_owned()
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name.as_bytes())
    }
}

/// The fixed schema used to transmit field descriptions
pub fn meta_schema() -> DeltaSchema {
    let f = |name: &str, flags, bits, pre| {
        Arc::new(DeltaField::new(name.as_bytes(), flags, 0, 0, bits, pre, 4000).unwrap())
    };

    DeltaSchema {
        name: b"delta_description_t".to_vec(),
        fields: vec![
            f("fieldType", dt::INTEGER, 32, 4000),
            f("fieldName", dt::STRING, 1, 4000),
            f("fieldOffset", dt::INTEGER, 16, 4000),
            f("fieldSize", dt::INTEGER, 8, 4000),
            f("significant_bits", dt::INTEGER, 8, 4000),
            f("premultiply", dt::FLOAT, 32, 4000 * 4000),
            f("postmultiply", dt::FLOAT, 32, 4000 * 4000),
        ],
    }
}

/// One decoded delta, holding the transmitted mask and the sent field values
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeltaUpdate {
    pub mask_len: u8,
    pub mask: [u8; 8],
    /// (field index, value) in field order
    pub values: Vec<(u16, Wire)>,
}

impl DeltaUpdate {
    pub fn get(&self, index: usize) -> Option<&Wire> {
        self.values
            .iter()
            .find(|(i, _)| usize::from(*i) == index)
            .map(|(_, w)| w)
    }
}

/// Decode one delta against `schema`. `time_base_bits` is the current
/// `svc_time` as f32 bits, attached to time-window fields
pub fn parse_delta(
    r: &mut BitReader<'_>,
    schema: &DeltaSchema,
    time_base_bits: u32,
) -> Result<DeltaUpdate, DeltaError> {
    let mask_len = r.bits(3)? as u8;
    let mut mask = [0u8; 8];
    for b in mask.iter_mut().take(mask_len.into()) {
        *b = r.bits(8)? as u8;
    }

    let mut values = Vec::new();
    for (i, field) in schema.fields.iter().enumerate().take(64) {
        if mask[i / 8] & (1 << (i % 8)) != 0 {
            values.push((i as u16, field.read(r, time_base_bits)?));
        }
    }

    Ok(DeltaUpdate {
        mask_len,
        mask,
        values,
    })
}

/// Decode the body of `svc_deltadescription` after its name string
pub fn parse_description(
    r: &mut BitReader<'_>,
    name: Vec<u8>,
    meta: &DeltaSchema,
) -> Result<DeltaSchema, DeltaError> {
    if name.is_empty() {
        return Err(DeltaError::EmptyName);
    }

    let count = r.bits(16)? as usize;
    if count > MAX_SCHEMA_FIELDS {
        return Err(DeltaError::TooManyFields {
            name: String::from_utf8_lossy(&name).into_owned(),
            count,
        });
    }

    let mut fields = Vec::with_capacity(count);
    for _ in 0..count {
        let u = parse_delta(r, meta, 0)?;
        let uint = |i: usize| match u.get(i) {
            Some(Wire::Unsigned(v)) => *v,
            _ => 0,
        };
        let fname = match u.get(1) {
            Some(Wire::Str(s)) => s.to_vec(),
            _ => Vec::new(),
        };

        fields.push(Arc::new(DeltaField::new(
            &fname,
            uint(0),
            uint(2),
            uint(3),
            uint(4),
            uint(5),
            uint(6),
        )?));
    }

    Ok(DeltaSchema { name, fields })
}

/// Field values of one struct instance (entity_state_t, clientdata_t, ...),
/// keyed by struct offset like the engine's byte copies. Absent offsets are
/// zero
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StructState {
    entries: Vec<(u32, Arc<DeltaField>, Wire)>,
}

impl StructState {
    /// Engine `ParseDelta(from, to)` into a zeroed `to`. Sent fields take the
    /// new value, other fields of `schema` copy `base` at the same offset,
    /// and offsets outside `schema` become zero
    pub fn apply(base: &StructState, schema: &DeltaSchema, update: &DeltaUpdate) -> StructState {
        let mut sent = update.values.iter().peekable();
        let mut entries = Vec::with_capacity(schema.fields.len());
        for (i, field) in schema.fields.iter().enumerate() {
            if let Some((_, w)) = sent.next_if(|(j, _)| usize::from(*j) == i) {
                entries.push((field.offset, field.clone(), w.clone()));
            } else if let Some(e) = base.entry(field.offset) {
                entries.push(e.clone());
            }
        }

        entries.sort_by_key(|e| e.0);
        entries.dedup_by_key(|e| e.0);

        StructState { entries }
    }

    fn entry(&self, offset: u32) -> Option<&(u32, Arc<DeltaField>, Wire)> {
        self.entries
            .binary_search_by_key(&offset, |e| e.0)
            .ok()
            .map(|i| &self.entries[i])
    }

    pub fn get_wire(&self, offset: u32) -> Option<(&DeltaField, &Wire)> {
        self.entry(offset).map(|(_, f, w)| (f.as_ref(), w))
    }

    /// Converted value of the named field of `schema`, zero when absent
    pub fn value(&self, schema: &DeltaSchema, name: &str) -> Option<Value> {
        self.value_at(schema, schema.field_index(name)?)
    }

    /// [`StructState::value`] by the field's position in `schema`, for
    /// readers that look the position up once
    pub fn value_at(&self, schema: &DeltaSchema, index: usize) -> Option<Value> {
        let field = schema.fields.get(index)?;
        Some(match self.entry(field.offset) {
            Some((_, f, w)) => convert(f, w),
            None if field.kind() == Kind::String => Value::Str(Vec::new()),
            None if matches!(
                field.kind(),
                Kind::Float | Kind::Angle | Kind::TimeWindow8 | Kind::TimeWindowBig
            ) =>
            {
                Value::Float(0.)
            }
            None => Value::Int(0),
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeltaField, &Wire)> {
        self.entries.iter().map(|(_, f, w)| (f.as_ref(), w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::protocol::bitwrite::BitWriter;

    fn write_field(w: &mut BitWriter, name: &[u8], flags: u32, offset: u32, bits: u32, pre: u32) {
        // Send all 7 meta fields, so the mask byte is 0b0111_1111
        w.bits(1, 3);
        w.bits(0x7F, 8);

        w.bits(flags, 32);
        w.string(name);
        w.bits(offset, 16);
        w.bits(4, 8);
        w.bits(bits, 8);
        w.bits(pre, 32);
        w.bits(4000, 32);
    }

    #[test]
    fn description_and_delta_roundtrip() {
        let mut w = BitWriter::new();
        w.bits(3, 16);
        write_field(
            &mut w,
            b"origin[0]",
            dt::FLOAT | dt::SIGNED,
            8,
            18,
            8 * 4000,
        );
        write_field(&mut w, b"animtime", dt::TIMEWINDOW_8, 12, 8, 4000);
        write_field(&mut w, b"model", dt::STRING, 16, 1, 4000);

        let meta = meta_schema();
        let bytes = w.finish();
        let mut r = BitReader::new(&bytes);
        let schema = parse_description(&mut r, b"test_t".to_vec(), &meta).unwrap();

        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].premultiply(), 8.);
        assert_eq!(schema.fields[1].kind(), Kind::TimeWindow8);

        // Delta with fields 0 and 1 sent, where origin = -100.5 -> -804 at 1/8
        let mut w = BitWriter::new();
        w.bits(1, 3);
        w.bits(0b011, 8);
        w.bits(1, 1);
        w.bits(804, 17);
        w.bits(0, 1);
        w.bits(5, 7);

        let bytes = w.finish();
        let mut r = BitReader::new(&bytes);
        let time = 12.5f32.to_bits();
        let u = parse_delta(&mut r, &schema, time).unwrap();
        let s = StructState::apply(&StructState::default(), &schema, &u);

        assert_eq!(s.value(&schema, "origin[0]"), Some(Value::Float(-100.5)));
        assert_eq!(s.value(&schema, "animtime"), Some(Value::Float(12.45)));
        assert_eq!(s.value(&schema, "model"), Some(Value::Str(Vec::new())));

        // Second delta sends only the string, so others inherit by offset
        let mut w = BitWriter::new();
        w.bits(1, 3);
        w.bits(0b100, 8);
        w.string(b"x");

        let bytes = w.finish();
        let mut r = BitReader::new(&bytes);
        let u2 = parse_delta(&mut r, &schema, 99f32.to_bits()).unwrap();
        let s2 = StructState::apply(&s, &schema, &u2);

        assert_eq!(s2.value(&schema, "origin[0]"), Some(Value::Float(-100.5)));
        assert_eq!(s2.value(&schema, "animtime"), Some(Value::Float(12.45)));
        assert_eq!(s2.value(&schema, "model"), Some(Value::Str(b"x".to_vec())));
    }

    #[test]
    fn rejects_bad_field_types() {
        assert!(DeltaField::new(b"x", dt::BYTE | dt::SHORT, 0, 1, 8, 4000, 4000).is_err());
        assert!(DeltaField::new(b"x", dt::INTEGER, 0, 4, 33, 4000, 4000).is_err());
        assert!(DeltaField::new(b"x", dt::INTEGER, 0, 4, 0, 4000, 4000).is_err());
    }
}
