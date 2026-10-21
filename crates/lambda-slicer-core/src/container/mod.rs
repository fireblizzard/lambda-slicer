//! GoldSrc demo container with the header, directory, and outer frame records
//!
//! Everything here is byte-exact. Raw string buffers and float bits are
//! preserved so a frame can be written back unchanged. Network message
//! payloads are opaque at this layer

mod frame;
mod index;
mod reader;
mod writer;

pub use frame::{ClientDataRecord, EventRecord, Frame, FrameBody, NetworkFrame, SoundRecord, tag};
pub use index::{EntryIndex, FrameSpan, TagCounts, index_entry};
pub use reader::{
    DemoReader, FrameIter, FramePosition, Limits, LocatedFrame, read_directory, read_header,
};
pub use writer::{encode_directory, encode_frame, encode_header};

use std::fmt;

pub const MAGIC: &[u8; 8] = b"HLDEMO\0\0";
pub const HEADER_LEN: u64 = 544;
pub const DIR_ENTRY_LEN: u64 = 92;
/// Offset of the directory-offset field inside the header
pub const HEADER_DIR_OFFSET_FIELD: usize = 540;

/// Fixed 544-byte demo header. String fields are kept as raw buffers
#[derive(Clone, PartialEq, Eq)]
pub struct Header {
    pub demo_protocol: i32,
    pub network_protocol: i32,
    pub map_name_raw: Box<[u8; 260]>,
    pub game_dir_raw: Box<[u8; 260]>,
    pub map_checksum: u32,
    pub directory_offset: i32,
}

impl Header {
    pub fn map_name(&self) -> String {
        c_string_lossy(&self.map_name_raw[..])
    }

    pub fn game_dir(&self) -> String {
        c_string_lossy(&self.game_dir_raw[..])
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("demo_protocol", &self.demo_protocol)
            .field("network_protocol", &self.network_protocol)
            .field("map_name", &self.map_name())
            .field("game_dir", &self.game_dir())
            .field("map_checksum", &self.map_checksum)
            .field("directory_offset", &self.directory_offset)
            .finish()
    }
}

/// One 92-byte directory record. Values are raw and may be invalid, so use
/// [`DirectoryEntry::region`] to obtain a checked byte range
#[derive(Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub entry_type: i32,
    pub description_raw: [u8; 64],
    pub flags: i32,
    pub cd_track: i32,
    pub track_time_bits: u32,
    pub frame_count: i32,
    pub offset: i32,
    pub length: i32,
}

/// What starts the mark of a clip Lambda Slicer exported, which follows the
/// end of its playback entry's description and names the demo it came from
pub const CLIP_MARK: &[u8] = b"LS#";

/// Ends a source name cut short to fit the mark
const CUT: &[u8] = b"..";

impl DirectoryEntry {
    pub fn description(&self) -> String {
        c_string_lossy(&self.description_raw)
    }

    /// The name of the demo this entry's file is a clip of, from the mark
    /// after the end of its description, none for a demo with no mark
    pub fn clip_of(&self) -> Option<String> {
        let raw = &self.description_raw;
        let end = raw.iter().position(|&b| b == 0)?;
        let after = raw.get(end + 1..)?.strip_prefix(CLIP_MARK)?;
        let name = c_string_lossy(after);
        (!name.is_empty()).then_some(name)
    }

    /// Mark this entry's file as a clip of the demo named `source`, after
    /// the end of its description, which the game reads alone. A name too
    /// long for the room left is cut between whole letters and ends in `..`,
    /// and the last byte stays empty to end the mark
    pub fn mark_clip(&mut self, source: &str) {
        let raw = &mut self.description_raw;
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len() - 1);
        let room = raw.len().saturating_sub(end + 2 + CLIP_MARK.len());
        if room <= CUT.len() {
            return;
        }

        let mut name = source.as_bytes().to_vec();
        if name.len() > room {
            let mut keep = room - CUT.len();
            while !source.is_char_boundary(keep) {
                keep -= 1;
            }
            name.truncate(keep);
            name.extend_from_slice(CUT);
        }

        let mark = [CLIP_MARK, &name].concat();
        raw[end + 1..].fill(0);
        raw[end + 1..end + 1 + mark.len()].copy_from_slice(&mark);
    }

    pub fn track_time(&self) -> f32 {
        f32::from_bits(self.track_time_bits)
    }

    /// Checked `[start, end)` byte range of this entry within a file of
    /// `file_len` bytes
    pub fn region(&self, file_len: u64) -> Result<(u64, u64), ContainerError> {
        let start = u64::try_from(self.offset).map_err(|_| ContainerError::Range {
            what: "directory entry offset",
            detail: format!("negative offset {}", self.offset),
        })?;
        let len = u64::try_from(self.length).map_err(|_| ContainerError::Range {
            what: "directory entry length",
            detail: format!("negative length {}", self.length),
        })?;
        let end = start.checked_add(len).ok_or(ContainerError::Range {
            what: "directory entry",
            detail: "offset + length overflows".into(),
        })?;
        if start < HEADER_LEN || end > file_len {
            return Err(ContainerError::Range {
                what: "directory entry",
                detail: format!("region {start}..{end} is outside {HEADER_LEN}..{file_len}"),
            });
        }

        Ok((start, end))
    }
}

impl fmt::Debug for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DirectoryEntry")
            .field("entry_type", &self.entry_type)
            .field("description", &self.description())
            .field("flags", &self.flags)
            .field("cd_track", &self.cd_track)
            .field("track_time", &self.track_time())
            .field("frame_count", &self.frame_count)
            .field("offset", &self.offset)
            .field("length", &self.length)
            .finish()
    }
}

/// Why a frame could not be read
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FrameCause {
    #[error("truncated {field}: needs {needed} bytes, {available} remain in the entry")]
    Truncated {
        field: &'static str,
        needed: u64,
        available: u64,
    },
    #[error("unknown frame tag {0}")]
    UnknownTag(u8),
    #[error("non-finite frame time (bits {0:#010x})")]
    NonFiniteTime(u32),
    #[error("negative {field} length {len}")]
    NegativeLength { field: &'static str, len: i64 },
    #[error("{field} length {len} exceeds limit {limit}")]
    OverLimit {
        field: &'static str,
        len: u64,
        limit: u64,
    },
    #[error("entry ended without a NextSection record")]
    MissingTerminal,
}

#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("I/O error at byte {offset}: {source}")]
    Io {
        offset: u64,
        #[source]
        source: std::io::Error,
    },
    #[error("file is {len} bytes, shorter than the 544-byte header")]
    TooShort { len: u64 },
    #[error("not a GoldSrc demo: bad magic")]
    BadMagic,
    #[error(
        "directory offset is zero: the recording is probably incomplete \
         (sequential recovery is not implemented)"
    )]
    NoDirectory,
    #[error("{what} out of range: {detail}")]
    Range { what: &'static str, detail: String },
    #[error("entry {entry}, frame {ordinal} at byte {offset}: {cause}")]
    Frame {
        entry: u32,
        ordinal: u64,
        offset: u64,
        cause: FrameCause,
    },
}

/// Bytes up to the first NUL, decoded lossily for display only
pub fn c_string_lossy(raw: &[u8]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).into_owned()
}

use crate::message::{Arg, Coded};

impl Coded for FrameCause {
    fn code(&self) -> &'static str {
        match self {
            FrameCause::Truncated { .. } => "container-frame-truncated",
            FrameCause::UnknownTag(_) => "container-frame-unknown-tag",
            FrameCause::NonFiniteTime(_) => "container-frame-bad-time",
            FrameCause::NegativeLength { .. } => "container-frame-negative-length",
            FrameCause::OverLimit { .. } => "container-frame-over-limit",
            FrameCause::MissingTerminal => "container-frame-missing-terminal",
        }
    }

    fn args(&self) -> Vec<(&'static str, Arg)> {
        match self {
            FrameCause::Truncated {
                field,
                needed,
                available,
            } => vec![
                ("field", (*field).into()),
                ("needed", (*needed).into()),
                ("available", (*available).into()),
            ],
            FrameCause::UnknownTag(tag) => vec![("tag", (*tag).into())],
            FrameCause::NonFiniteTime(_) | FrameCause::MissingTerminal => Vec::new(),
            FrameCause::NegativeLength { field, len } => {
                vec![("field", (*field).into()), ("len", (*len).into())]
            }
            FrameCause::OverLimit { field, len, limit } => vec![
                ("field", (*field).into()),
                ("len", (*len).into()),
                ("limit", (*limit).into()),
            ],
        }
    }
}

impl Coded for ContainerError {
    fn code(&self) -> &'static str {
        match self {
            ContainerError::Io { .. } => "container-io",
            ContainerError::TooShort { .. } => "container-too-short",
            ContainerError::BadMagic => "container-bad-magic",
            ContainerError::NoDirectory => "container-no-directory",
            ContainerError::Range { .. } => "container-range",
            ContainerError::Frame { cause, .. } => cause.code(),
        }
    }

    fn args(&self) -> Vec<(&'static str, Arg)> {
        match self {
            ContainerError::Io { offset, source } => vec![
                ("offset", (*offset).into()),
                ("detail", source.to_string().into()),
            ],
            ContainerError::TooShort { len } => vec![("len", (*len).into())],
            ContainerError::BadMagic | ContainerError::NoDirectory => Vec::new(),
            ContainerError::Range { what, detail } => {
                vec![("what", (*what).into()), ("detail", detail.clone().into())]
            }
            ContainerError::Frame {
                entry,
                ordinal,
                offset,
                cause,
            } => {
                let mut args = vec![
                    ("entry", (*entry).into()),
                    ("frame", (*ordinal).into()),
                    ("offset", (*offset).into()),
                ];
                args.extend(cause.args());
                args
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A playback entry with nothing else set
    fn playback() -> DirectoryEntry {
        let mut description_raw = [0u8; 64];
        description_raw[..8].copy_from_slice(b"Playback");
        DirectoryEntry {
            entry_type: 1,
            description_raw,
            flags: 0,
            cd_track: 0,
            track_time_bits: 0,
            frame_count: 0,
            offset: 0,
            length: 0,
        }
    }

    #[test]
    fn a_clip_names_its_demo_after_the_description() {
        let mut entry = playback();
        assert_eq!(entry.clip_of(), None);

        // The game still reads only the description
        entry.mark_clip("20210424_012456_crossfire");
        assert_eq!(entry.description(), "Playback");
        assert_eq!(
            entry.clip_of().as_deref(),
            Some("20210424_012456_crossfire")
        );
        assert_eq!(&entry.description_raw[9..12], b"LS#");
    }

    #[test]
    fn a_long_name_is_cut_between_letters_and_the_mark_ends() {
        // 51 bytes fit, so 52 are cut to 49 and `..`
        let mut entry = playback();
        entry.mark_clip(&"a".repeat(52));
        let kept = entry.clip_of().unwrap();
        assert_eq!(kept, format!("{}..", "a".repeat(49)));
        assert_eq!(entry.description_raw[63], 0);

        // Two-byte letters are never split
        let mut entry = playback();
        entry.mark_clip(&"ж".repeat(30));
        assert_eq!(entry.clip_of().unwrap(), format!("{}..", "ж".repeat(24)));

        // Exactly what fits stays whole
        let mut entry = playback();
        entry.mark_clip(&"b".repeat(51));
        assert_eq!(entry.clip_of().unwrap(), "b".repeat(51));
    }
}
