//! Compact per-entry frame index. Holds locations and scheduling fields only,
//! never payloads, so large recordings stay cheap to index.

use std::io::{Read, Seek};

use serde::Serialize;

use super::ContainerError;
use super::frame::{FrameBody, tag};
use super::reader::DemoReader;

#[derive(Clone, Debug, Serialize)]
pub struct FrameSpan {
    pub ordinal: u64,
    pub offset: u64,
    pub len: u32,
    pub tag: u8,
    pub time_bits: u32,
    pub frame_number: i32,
    /// Network records only: the seven recorded sequence integers.
    pub sequence: Option<[i32; 7]>,
}

impl FrameSpan {
    pub fn time(&self) -> f32 {
        f32::from_bits(self.time_bits)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TagCounts {
    pub network_start: u64,
    pub network: u64,
    pub demo_start: u64,
    pub console_command: u64,
    pub client_data: u64,
    pub next_section: u64,
    pub event: u64,
    pub weapon_anim: u64,
    pub sound: u64,
    pub demo_buffer: u64,
}

impl TagCounts {
    fn add(&mut self, t: u8) {
        let slot = match t {
            tag::NETWORK_START => &mut self.network_start,
            tag::NETWORK => &mut self.network,
            tag::DEMO_START => &mut self.demo_start,
            tag::CONSOLE_COMMAND => &mut self.console_command,
            tag::CLIENT_DATA => &mut self.client_data,
            tag::NEXT_SECTION => &mut self.next_section,
            tag::EVENT => &mut self.event,
            tag::WEAPON_ANIM => &mut self.weapon_anim,
            tag::SOUND => &mut self.sound,
            tag::DEMO_BUFFER => &mut self.demo_buffer,
            _ => return,
        };
        *slot += 1;
    }

    pub fn network_records(&self) -> u64 {
        self.network_start + self.network
    }

    pub fn total(&self) -> u64 {
        self.network_start
            + self.network
            + self.demo_start
            + self.console_command
            + self.client_data
            + self.next_section
            + self.event
            + self.weapon_anim
            + self.sound
            + self.demo_buffer
    }
}

#[derive(Debug)]
pub struct EntryIndex {
    pub entry: u32,
    /// Empty unless spans were requested.
    pub spans: Vec<FrameSpan>,
    pub counts: TagCounts,
    pub first_time: Option<f32>,
    pub last_time: Option<f32>,
    pub min_frame_number: Option<i32>,
    pub max_frame_number: Option<i32>,
    /// Non-terminal records whose time is lower than the preceding
    /// non-terminal record's. Repeated NextSection records carry unrelated
    /// times and are excluded.
    pub time_decreases: u64,
    pub max_network_message: u64,
    /// Time of the first NextSection record.
    pub first_terminal_time: Option<f32>,
    /// NextSection records; real recordings may contain more than one.
    pub terminal_records: u64,
    /// Non-terminal records after the first NextSection (structural warning).
    pub records_after_terminal: u64,
    /// First structural error; frames before it are indexed.
    pub error: Option<ContainerError>,
}

impl EntryIndex {
    pub fn is_complete(&self) -> bool {
        self.terminal_records > 0 && self.error.is_none()
    }
}

/// Index one directory entry. Errors in frame data are recorded in the
/// result; only an unusable directory entry is returned as `Err`.
pub fn index_entry<R: Read + Seek>(
    reader: &mut DemoReader<R>,
    entry: u32,
    keep_spans: bool,
) -> Result<EntryIndex, ContainerError> {
    let mut idx = EntryIndex {
        entry,
        spans: Vec::new(),
        counts: TagCounts::default(),
        first_time: None,
        last_time: None,
        min_frame_number: None,
        max_frame_number: None,
        time_decreases: 0,
        max_network_message: 0,
        first_terminal_time: None,
        terminal_records: 0,
        records_after_terminal: 0,
        error: None,
    };
    let mut prev_record_time: Option<f32> = None;
    let mut iter = reader.frames(entry)?;
    for item in iter.by_ref() {
        let lf = match item {
            Ok(lf) => lf,
            Err(e) => {
                idx.error = Some(e);
                break;
            }
        };
        let f = &lf.frame;
        let t = f.time();
        idx.counts.add(f.tag);
        if !matches!(f.body, FrameBody::NextSection) {
            if prev_record_time.is_some_and(|last| t < last) {
                idx.time_decreases += 1;
            }
            prev_record_time = Some(t);
        }
        idx.first_time.get_or_insert(t);
        idx.last_time = Some(t);
        idx.min_frame_number = Some(
            idx.min_frame_number
                .map_or(f.frame_number, |m| m.min(f.frame_number)),
        );
        idx.max_frame_number = Some(
            idx.max_frame_number
                .map_or(f.frame_number, |m| m.max(f.frame_number)),
        );
        let sequence = match &f.body {
            FrameBody::Network(n) => {
                idx.max_network_message = idx.max_network_message.max(n.message.len() as u64);
                Some(n.sequence)
            }
            FrameBody::NextSection => {
                idx.first_terminal_time.get_or_insert(t);
                None
            }
            _ => None,
        };
        if keep_spans {
            idx.spans.push(FrameSpan {
                ordinal: lf.ordinal,
                offset: lf.offset,
                // A frame is bounded by its entry, whose length is an i32.
                len: u32::try_from(lf.len).unwrap_or(u32::MAX),
                tag: f.tag,
                time_bits: f.time_bits,
                frame_number: f.frame_number,
                sequence,
            });
        }
    }
    idx.terminal_records = iter.terminal_records();
    idx.records_after_terminal = iter.records_after_terminal();
    Ok(idx)
}
