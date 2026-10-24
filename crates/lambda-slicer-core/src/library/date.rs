//! When a demo was recorded, as far as anything says
//!
//! The header has no date. Adrenaline Gamer and OpenAG name their automatic
//! recordings `YYYYMMDD_HHMMSS_map.dem` in the recording machine's local
//! time, and other clients such as BugfixedHL `map-YYYYMMDD-HHMMSS.dem`,
//! which is the best source. Otherwise the file's modification time,
//! usually when recording ended, is used in this machine's local time.
//! Copying usually keeps it, but downloads and archive tools often reset
//! it, so the source is kept alongside the date

use std::path::Path;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    /// A `YYYYMMDD_HHMMSS` or `YYYYMMDD-HHMMSS` timestamp in the file name
    FileName,
    /// The file's modification time
    Modified,
}

impl DateSource {
    pub fn name(self) -> &'static str {
        match self {
            DateSource::FileName => "file_name",
            DateSource::Modified => "modified",
        }
    }
}

/// A `YYYYMMDD_HHMMSS` or `YYYYMMDD-HHMMSS` timestamp anywhere in `name`,
/// with a plausible date
pub fn from_name(name: &str) -> Option<NaiveDateTime> {
    let bytes = name.as_bytes();
    (0..bytes.len().saturating_sub(14)).find_map(|i| {
        let window = &bytes[i..i + 15];
        let digits = |r: std::ops::Range<usize>| {
            window[r.clone()]
                .iter()
                .all(u8::is_ascii_digit)
                .then(|| std::str::from_utf8(&window[r]).ok()?.parse::<u32>().ok())
                .flatten()
        };
        // Not part of a longer run of digits
        let clean_start = i == 0 || !bytes[i - 1].is_ascii_digit();
        let clean_end = bytes.get(i + 15).is_none_or(|b| !b.is_ascii_digit());
        let separated = matches!(window[8], b'_' | b'-');
        if !(clean_start && clean_end && separated) {
            return None;
        }
        let (y, m, d) = (digits(0..4)?, digits(4..6)?, digits(6..8)?);
        let (hh, mm, ss) = (digits(9..11)?, digits(11..13)?, digits(13..15)?);
        if !(1998..=2100).contains(&y) {
            return None;
        }
        let date = NaiveDate::from_ymd_opt(y as i32, m, d)?;
        let time = NaiveTime::from_hms_opt(hh, mm, ss)?;
        Some(NaiveDateTime::new(date, time))
    })
}

/// When `path` was recorded, as `YYYY-MM-DD HH:MM:SS` local time, and where
/// that came from
pub fn recorded(path: &Path, modified_ns: Option<i64>) -> Option<(String, DateSource)> {
    let format = |t: NaiveDateTime| t.format("%Y-%m-%d %H:%M:%S").to_string();
    if let Some(t) = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(from_name)
    {
        return Some((format(t), DateSource::FileName));
    }
    Some((local(modified_ns?)?, DateSource::Modified))
}

/// A file time in nanoseconds since 1970 as `YYYY-MM-DD HH:MM:SS` in this
/// machine's local time
pub fn local(ns: i64) -> Option<String> {
    let utc = DateTime::from_timestamp(ns.div_euclid(1_000_000_000), 0)?;
    let local = utc.with_timezone(&Local).naive_local();
    Some(local.format("%Y-%m-%d %H:%M:%S").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_recording_names_carry_their_date() {
        let t = from_name("20190815_210848_scary_1_[Ehll]_.dem").unwrap();
        assert_eq!(t.to_string(), "2019-08-15 21:08:48");
        assert!(from_name("cross_naz_420.dem").is_none());
        assert!(from_name("20191340_210848_x.dem").is_none());
        assert!(from_name("120190815_210848.dem").is_none());
        assert_eq!(
            from_name("kz_20230909_224143.dem").unwrap().to_string(),
            "2023-09-09 22:41:43"
        );
    }

    #[test]
    fn names_with_a_dash_between_date_and_time_carry_their_date() {
        let t = from_name("dm_dust2-20261007-212429.dem").unwrap();
        assert_eq!(t.to_string(), "2026-10-07 21:24:29");
        assert!(from_name("dm_dust2-20261307-212429.dem").is_none());
        assert!(from_name("map-20261007-2124290.dem").is_none());
    }

    #[test]
    fn names_win_over_modification_times() {
        let named = recorded(Path::new("x/20190815_210848_a.dem"), Some(0)).unwrap();
        assert_eq!(named, ("2019-08-15 21:08:48".into(), DateSource::FileName));
        let (_, source) =
            recorded(Path::new("x/plain.dem"), Some(1_600_000_000_000_000_000)).unwrap();
        assert_eq!(source, DateSource::Modified);
        assert!(recorded(Path::new("x/plain.dem"), None).is_none());
    }
}
