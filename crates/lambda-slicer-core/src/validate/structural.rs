//! Container inspection and structural validation.
//!
//! This level checks the container only: header, directory, region layout,
//! frame grammar, and terminal records. Network payloads are not decoded.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Serialize;

use crate::container::{
    ContainerError, DemoReader, HEADER_LEN, Limits, TagCounts, c_string_lossy, index_entry,
};

pub const REPORT_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub code: &'static str,
    pub entry: Option<u32>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeaderReport {
    pub demo_protocol: i32,
    pub network_protocol: i32,
    pub map_name: String,
    pub game_dir: String,
    pub map_checksum: u32,
    pub directory_offset: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct EntryReport {
    pub index: u32,
    pub entry_type: i32,
    pub description: String,
    pub flags: i32,
    pub cd_track: i32,
    pub track_time: f32,
    pub frame_count: i32,
    pub offset: i32,
    pub length: i32,
    pub records: u64,
    pub network_records: u64,
    pub counts: TagCounts,
    pub first_time: Option<f32>,
    pub last_time: Option<f32>,
    pub first_terminal_time: Option<f32>,
    pub min_frame_number: Option<i32>,
    pub max_frame_number: Option<i32>,
    pub time_decreases: u64,
    pub max_network_message: u64,
    pub terminal_records: u64,
    pub records_after_terminal: u64,
    pub complete: bool,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InspectStatus {
    /// Container fully read with no structural errors.
    Complete,
    /// Some entries could not be fully read.
    Partial,
    /// Header or directory unusable.
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct InspectReport {
    pub schema: u32,
    pub path: String,
    pub size: u64,
    pub status: InspectStatus,
    pub header: Option<HeaderReport>,
    pub entries: Vec<EntryReport>,
    pub findings: Vec<Finding>,
}

impl InspectReport {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }
}

/// Supported demo/network protocol profile for the initial implementation.
pub const SUPPORTED_PROFILE: (i32, i32) = (5, 48);

pub fn inspect(path: &Path, limits: Limits) -> InspectReport {
    let mut report = InspectReport {
        schema: REPORT_SCHEMA,
        path: path.display().to_string(),
        size: 0,
        status: InspectStatus::Failed,
        header: None,
        entries: Vec::new(),
        findings: Vec::new(),
    };
    let mut findings = Vec::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            add(&mut findings, Severity::Error, "open", None, e.to_string());
            report.findings = findings;
            return report;
        }
    };
    let mut reader = match DemoReader::new(BufReader::new(file), limits) {
        Ok(r) => r,
        Err(e) => {
            let code = match e {
                ContainerError::BadMagic => "bad_magic",
                ContainerError::NoDirectory => "no_directory",
                ContainerError::TooShort { .. } => "too_short",
                _ => "header",
            };
            add(&mut findings, Severity::Error, code, None, e.to_string());
            report.findings = findings;
            return report;
        }
    };
    let h = &reader.header;
    report.size = reader.file_len;
    report.header = Some(HeaderReport {
        demo_protocol: h.demo_protocol,
        network_protocol: h.network_protocol,
        map_name: h.map_name(),
        game_dir: h.game_dir(),
        map_checksum: h.map_checksum,
        directory_offset: h.directory_offset,
    });
    if (h.demo_protocol, h.network_protocol) != SUPPORTED_PROFILE {
        add(
            &mut findings,
            Severity::Warning,
            "unsupported_profile",
            None,
            format!(
                "protocol {}/{} is outside the supported {}/{} profile",
                h.demo_protocol, h.network_protocol, SUPPORTED_PROFILE.0, SUPPORTED_PROFILE.1
            ),
        );
    }
    let dir_offset = h.directory_offset as u64;
    let directory = reader.directory.clone();
    let mut regions = Vec::new();

    for (i, d) in directory.iter().enumerate() {
        let i = i as u32;
        match d.region(reader.file_len) {
            Ok(r) => regions.push((r, i)),
            Err(e) => {
                add(
                    &mut findings,
                    Severity::Error,
                    "entry_region",
                    Some(i),
                    e.to_string(),
                );
                continue;
            }
        }
        let idx = match index_entry(&mut reader, i, false) {
            Ok(idx) => idx,
            Err(e) => {
                add(
                    &mut findings,
                    Severity::Error,
                    "entry_region",
                    Some(i),
                    e.to_string(),
                );
                continue;
            }
        };
        if let Some(e) = &idx.error {
            add(
                &mut findings,
                Severity::Error,
                "frame",
                Some(i),
                e.to_string(),
            );
        }
        if idx.records_after_terminal > 0 {
            add(
                &mut findings,
                Severity::Warning,
                "records_after_terminal",
                Some(i),
                format!(
                    "{} records follow the first NextSection",
                    idx.records_after_terminal
                ),
            );
        }
        if idx.terminal_records > 1 {
            add(
                &mut findings,
                Severity::Info,
                "repeated_terminal",
                Some(i),
                format!("{} NextSection records", idx.terminal_records),
            );
        }
        if idx.time_decreases > 0 {
            add(
                &mut findings,
                Severity::Warning,
                "time_decreases",
                Some(i),
                format!("record time decreases {} times", idx.time_decreases),
            );
        }
        let net = idx.counts.network_records();
        if idx.error.is_none() && i64::from(d.frame_count) != net as i64 && d.entry_type != 0 {
            add(
                &mut findings,
                Severity::Warning,
                "frame_count_mismatch",
                Some(i),
                format!(
                    "directory frame count {} but {} network records",
                    d.frame_count, net
                ),
            );
        }
        if d.entry_type != 0
            && idx.error.is_none()
            && idx.first_terminal_time.map(f32::to_bits) != Some(d.track_time_bits)
        {
            add(
                &mut findings,
                Severity::Warning,
                "track_time_mismatch",
                Some(i),
                format!(
                    "directory track time {} but first NextSection at {:?}",
                    d.track_time(),
                    idx.first_terminal_time
                ),
            );
        }
        report.entries.push(EntryReport {
            index: i,
            entry_type: d.entry_type,
            description: c_string_lossy(&d.description_raw),
            flags: d.flags,
            cd_track: d.cd_track,
            track_time: d.track_time(),
            frame_count: d.frame_count,
            offset: d.offset,
            length: d.length,
            records: idx.counts.total(),
            network_records: net,
            first_time: idx.first_time,
            last_time: idx.last_time,
            first_terminal_time: idx.first_terminal_time,
            min_frame_number: idx.min_frame_number,
            max_frame_number: idx.max_frame_number,
            time_decreases: idx.time_decreases,
            max_network_message: idx.max_network_message,
            terminal_records: idx.terminal_records,
            records_after_terminal: idx.records_after_terminal,
            complete: idx.is_complete(),
            error: idx.error.as_ref().map(ToString::to_string),
            counts: idx.counts,
        });
    }

    // Region layout: entries must not overlap each other or the directory;
    // uncovered bytes are reported.
    regions.sort();
    let mut cursor = HEADER_LEN;
    for ((start, end), i) in &regions {
        if *start < cursor {
            add(
                &mut findings,
                Severity::Error,
                "region_overlap",
                Some(*i),
                format!("entry starts at {start}, before previous region end {cursor}"),
            );
        } else if *start > cursor {
            add(
                &mut findings,
                Severity::Warning,
                "region_gap",
                Some(*i),
                format!("{} unreferenced bytes before entry", start - cursor),
            );
        }
        cursor = cursor.max(*end);
    }
    if dir_offset < cursor {
        add(
            &mut findings,
            Severity::Error,
            "region_overlap",
            None,
            "directory overlaps an entry".into(),
        );
    } else if dir_offset > cursor {
        add(
            &mut findings,
            Severity::Warning,
            "region_gap",
            None,
            format!(
                "{} unreferenced bytes before directory",
                dir_offset - cursor
            ),
        );
    }
    let dir_end = dir_offset + 4 + directory.len() as u64 * crate::container::DIR_ENTRY_LEN;
    if dir_end < reader.file_len {
        add(
            &mut findings,
            Severity::Warning,
            "trailing_file_bytes",
            None,
            format!("{} bytes after the directory", reader.file_len - dir_end),
        );
    }
    if directory.is_empty() {
        add(
            &mut findings,
            Severity::Error,
            "empty_directory",
            None,
            "directory has no entries".into(),
        );
    }

    let failed_entries = report.entries.iter().filter(|e| !e.complete).count()
        + (directory.len() - report.entries.len());
    report.findings = findings;
    report.status = if failed_entries == 0 {
        InspectStatus::Complete
    } else {
        InspectStatus::Partial
    };
    report
}

fn add(
    findings: &mut Vec<Finding>,
    severity: Severity,
    code: &'static str,
    entry: Option<u32>,
    message: String,
) {
    findings.push(Finding {
        severity,
        code,
        entry,
        message,
    });
}
