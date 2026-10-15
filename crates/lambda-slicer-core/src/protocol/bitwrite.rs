//! LSB-first bit writer, the inverse of [`super::bit::BitReader`].

#[derive(Default)]
pub struct BitWriter {
    out: Vec<u8>,
    bit: usize,
}

impl BitWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bit_len(&self) -> usize {
        self.bit
    }

    /// Write the low `n` (0..=32) bits of `v`.
    pub fn bits(&mut self, v: u32, n: u32) {
        for i in 0..n {
            if self.bit.is_multiple_of(8) {
                self.out.push(0);
            }
            if (v >> i) & 1 != 0 {
                *self.out.last_mut().unwrap() |= 1 << (self.bit % 8);
            }
            self.bit += 1;
        }
    }

    pub fn bit(&mut self, b: bool) {
        self.bits(u32::from(b), 1);
    }

    /// Sign bit then `n - 1` magnitude bits.
    pub fn sbits(&mut self, v: i64, n: u32) {
        self.bit(v < 0);
        self.bits(v.unsigned_abs() as u32, n - 1);
    }

    /// Bytes followed by a NUL, bit-wise.
    pub fn string(&mut self, s: &[u8]) {
        for &c in s {
            self.bits(c.into(), 8);
        }
        self.bits(0, 8);
    }

    /// Pad to the next byte boundary with zero bits.
    pub fn align(&mut self) {
        self.bit = self.bit.div_ceil(8) * 8;
    }

    pub fn u8(&mut self, v: u8) {
        self.align();
        self.bits(v.into(), 8);
    }

    pub fn u16(&mut self, v: u16) {
        self.align();
        self.bits(v.into(), 16);
    }

    pub fn u32(&mut self, v: u32) {
        self.align();
        self.bits(v, 32);
    }

    pub fn bytes(&mut self, b: &[u8]) {
        self.align();
        for &c in b {
            self.bits(c.into(), 8);
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.align();
        self.out
    }
}
