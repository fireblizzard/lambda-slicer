//! Bounded little-endian bit reader matching the engine's `MSG_ReadBits`
//! family. Byte-level reads require byte alignment; bit sections end with
//! [`BitReader::align`], mirroring `MSG_EndBitReading`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BitError {
    #[error("read of {wanted} bits at bit {at} past end ({len} bits)")]
    Eof {
        at: usize,
        wanted: usize,
        len: usize,
    },
    #[error("byte read at unaligned bit {0}")]
    Unaligned(usize),
    #[error("string at bit {at} exceeds {limit} bytes")]
    StringTooLong { at: usize, limit: usize },
}

pub struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
}

/// Longest string the engine's static buffers accept.
pub const MAX_STRING: usize = 8192;

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn bit_pos(&self) -> usize {
        self.pos
    }

    pub fn byte_pos(&self) -> usize {
        self.pos / 8
    }

    pub fn bits_left(&self) -> usize {
        self.data.len() * 8 - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.bits_left() == 0
    }

    fn ensure(&self, n: usize) -> Result<(), BitError> {
        if n > self.bits_left() {
            return Err(BitError::Eof {
                at: self.pos,
                wanted: n,
                len: self.data.len() * 8,
            });
        }
        Ok(())
    }

    /// Read `n` (0..=32) bits, least significant first.
    pub fn bits(&mut self, n: u32) -> Result<u32, BitError> {
        debug_assert!(n <= 32);
        let n = n as usize;
        self.ensure(n)?;
        let mut value: u64 = 0;
        let mut got = 0;
        while got < n {
            let byte = self.data[self.pos / 8];
            let shift = self.pos % 8;
            let take = (8 - shift).min(n - got);
            let chunk = (u64::from(byte) >> shift) & ((1u64 << take) - 1);
            value |= chunk << got;
            got += take;
            self.pos += take;
        }
        Ok(value as u32)
    }

    pub fn bit(&mut self) -> Result<bool, BitError> {
        Ok(self.bits(1)? != 0)
    }

    pub fn peek_bits(&self, n: u32) -> Result<u32, BitError> {
        let mut copy = BitReader {
            data: self.data,
            pos: self.pos,
        };
        copy.bits(n)
    }

    /// Engine `ReadSBits`: one sign bit, then `n - 1` magnitude bits.
    pub fn sbits(&mut self, n: u32) -> Result<SignMagnitude, BitError> {
        let negative = self.bit()?;
        let magnitude = self.bits(n.saturating_sub(1))?;
        Ok(SignMagnitude {
            negative,
            magnitude,
        })
    }

    /// Bytes up to and excluding a NUL terminator, read bit-wise.
    pub fn bit_string(&mut self) -> Result<Vec<u8>, BitError> {
        let at = self.pos;
        let mut out = Vec::new();
        loop {
            let c = self.bits(8)? as u8;
            if c == 0 {
                return Ok(out);
            }
            if out.len() >= MAX_STRING {
                return Err(BitError::StringTooLong {
                    at,
                    limit: MAX_STRING,
                });
            }
            out.push(c);
        }
    }

    /// Advance to the next byte boundary.
    pub fn align(&mut self) {
        self.pos = self.pos.div_ceil(8) * 8;
    }

    fn aligned(&self) -> Result<(), BitError> {
        if !self.pos.is_multiple_of(8) {
            return Err(BitError::Unaligned(self.pos));
        }
        Ok(())
    }

    pub fn u8(&mut self) -> Result<u8, BitError> {
        self.aligned()?;
        Ok(self.bits(8)? as u8)
    }

    pub fn u16(&mut self) -> Result<u16, BitError> {
        self.aligned()?;
        Ok(self.bits(16)? as u16)
    }

    pub fn u32(&mut self) -> Result<u32, BitError> {
        self.aligned()?;
        self.bits(32)
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8], BitError> {
        self.aligned()?;
        self.ensure(n * 8)?;
        let start = self.pos / 8;
        self.pos += n * 8;
        Ok(&self.data[start..start + n])
    }

    /// NUL-terminated byte string (terminator consumed, not returned).
    pub fn string(&mut self) -> Result<Vec<u8>, BitError> {
        self.aligned()?;
        self.bit_string()
    }

    /// The bytes between two byte offsets, for raw-span preservation.
    pub fn slice(&self, start: usize, end: usize) -> &'a [u8] {
        &self.data[start..end]
    }
}

/// A sign-magnitude value as sent on the wire. `-0` is representable and
/// preserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignMagnitude {
    pub negative: bool,
    pub magnitude: u32,
}

impl SignMagnitude {
    pub fn value(self) -> i64 {
        let m = i64::from(self.magnitude);
        if self.negative { -m } else { m }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_lsb_first_across_bytes() {
        let data = [0b1010_1101, 0b0000_0011];
        let mut r = BitReader::new(&data);
        assert_eq!(r.bits(3).unwrap(), 0b101);
        assert_eq!(r.bits(7).unwrap(), 0b11_10101);
        assert_eq!(r.bits(6).unwrap(), 0);
        assert!(r.bits(1).is_err());
    }

    #[test]
    fn reads_32_bits_unaligned() {
        let data = [0xFF, 0x12, 0x34, 0x56, 0x78];
        let mut r = BitReader::new(&data);
        r.bits(4).unwrap();
        assert_eq!(r.bits(32).unwrap(), 0x8563_412F);
    }

    #[test]
    fn sign_magnitude_and_alignment() {
        let data = [0b0000_1011, 0x41, 0x42, 0x00];
        let mut r = BitReader::new(&data);
        let v = r.sbits(4).unwrap();
        assert_eq!(v.value(), -5);
        assert!(r.u8().is_err());
        r.align();
        assert_eq!(r.string().unwrap(), b"AB");
        assert!(r.is_empty());
    }
}
