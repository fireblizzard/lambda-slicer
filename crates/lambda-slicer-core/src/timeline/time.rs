//! Demo time as checked integer microseconds

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DemoTime(pub i64);

impl DemoTime {
    pub const ZERO: DemoTime = DemoTime(0);

    pub fn from_micros(us: i64) -> Self {
        DemoTime(us)
    }

    pub fn micros(self) -> i64 {
        self.0
    }

    /// Exact conversion of a recorded f32 time, rounded to the nearest
    /// microsecond. `None` for non-finite or out-of-range values
    pub fn from_f32(t: f32) -> Option<Self> {
        let us = (f64::from(t) * 1e6).round();
        (us.is_finite() && us.abs() < i64::MAX as f64).then_some(DemoTime(us as i64))
    }

    pub fn secs_f64(self) -> f64 {
        self.0 as f64 / 1e6
    }
}

impl fmt::Display for DemoTime {
    /// Formats as `HH:MM:SS.mmm`, truncated to milliseconds
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let ms = self.0.unsigned_abs() / 1000;
        let (h, m, s, ms) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000);
        write!(f, "{sign}{h:02}:{m:02}:{s:02}.{ms:03}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid time {0:?}: expected seconds, M:SS, or HH:MM:SS[.fff]")]
pub struct ParseTimeError(String);

impl FromStr for DemoTime {
    type Err = ParseTimeError;

    /// Accepts `96`, `96.5`, `1:36`, and `00:01:36.250`. Minutes and seconds
    /// after the first component must be below 60
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad = || ParseTimeError(s.to_owned());
        let parts: Vec<&str> = s.trim().split(':').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(bad());
        }

        let (last, whole) = parts.split_last().ok_or_else(bad)?;
        let (sec, frac) = match last.split_once('.') {
            Some((a, b)) => (a, b),
            None => (*last, ""),
        };
        if frac.len() > 6 || !frac.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad());
        }

        let int = |p: &str| -> Result<i64, ParseTimeError> {
            if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                return Err(bad());
            }

            p.parse::<i64>().map_err(|_| bad())
        };

        let sec = int(sec)?;
        let frac_us = if frac.is_empty() {
            0
        } else {
            int(frac)? * 10i64.pow(6 - frac.len() as u32)
        };

        let mut total: i64 = 0;
        for (i, p) in whole.iter().enumerate() {
            let v = int(p)?;
            if i > 0 && v >= 60 {
                return Err(bad());
            }

            total = total
                .checked_mul(60)
                .and_then(|t| t.checked_add(v))
                .ok_or_else(bad)?;
        }

        if !whole.is_empty() && sec >= 60 {
            return Err(bad());
        }

        total
            .checked_mul(60)
            .and_then(|t| {
                if whole.is_empty() {
                    Some(sec)
                } else {
                    t.checked_add(sec)
                }
            })
            .and_then(|t| t.checked_mul(1_000_000))
            .and_then(|t| t.checked_add(frac_us))
            .map(DemoTime)
            .ok_or_else(bad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_documented_forms() {
        assert_eq!("96".parse(), Ok(DemoTime(96_000_000)));
        assert_eq!("1:36".parse(), Ok(DemoTime(96_000_000)));
        assert_eq!("4:48".parse(), Ok(DemoTime(288_000_000)));
        assert_eq!("00:01:36.250".parse(), Ok(DemoTime(96_250_000)));
        assert_eq!("0.5".parse(), Ok(DemoTime(500_000)));
    }

    #[test]
    fn rejects_malformed() {
        for s in [
            "",
            ":",
            "1:60",
            "1:2:3:4",
            "-1",
            "1.2.3",
            "a",
            "1:",
            "0.1234567",
        ] {
            assert!(s.parse::<DemoTime>().is_err(), "{s}");
        }
    }

    #[test]
    fn displays_hms() {
        assert_eq!(DemoTime(96_000_000).to_string(), "00:01:36.000");
        assert_eq!(DemoTime(192_000_000).to_string(), "00:03:12.000");
        assert_eq!(DemoTime(3_723_456_789).to_string(), "01:02:03.456");
    }
}
