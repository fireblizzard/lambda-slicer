//! S0 control exports (implementation plan §6.1):
//!
//! - S0a: exact byte copy under another name.
//! - S0b: container rebuilt from decoded frames, payloads and clocks unchanged.
//! - S0c: end-only truncation `[0, end)` with a regenerated terminal record
//!   and directory.
//!
//! These qualify the container writer in the game before any arbitrary-start
//! transformation is attempted.

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

use serde::Serialize;

use super::ExportError;
use super::output::{Published, io_err, publish_new};
use crate::container::{
    DemoReader, Frame, FrameBody, HEADER_DIR_OFFSET_FIELD, Limits, encode_directory, encode_frame,
    encode_header, tag,
};
use crate::timeline::time::DemoTime;
use crate::validate::structural::{InspectStatus, inspect};

#[derive(Clone, Copy, Debug)]
pub enum Control {
    /// S0a
    ExactCopy,
    /// S0b
    Roundtrip,
    /// S0c: keep `[0, end)` of the playback entry.
    TruncateEnd(DemoTime),
}

impl Control {
    fn name(self) -> &'static str {
        match self {
            Control::ExactCopy => "s0a_exact_copy",
            Control::Roundtrip => "s0b_roundtrip",
            Control::TruncateEnd(_) => "s0c_truncate_end",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct WrittenEntry {
    pub index: u32,
    pub offset: i32,
    pub length: i32,
    pub frame_count: i32,
    pub track_time: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CutReport {
    pub requested_end: String,
    /// Time of the first excluded frame group, written as the terminal
    /// record time; the retained interval is `[0, effective_end)`.
    pub effective_end: String,
    pub effective_end_us: i64,
    pub retained_records: u64,
    pub retained_network_records: u64,
    pub terminal_frame_number: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlReport {
    pub control: &'static str,
    pub source: String,
    pub published: Published,
    /// S0b: whether the rebuilt file equals the source byte for byte.
    pub byte_identical: Option<bool>,
    pub entries: Vec<WrittenEntry>,
    pub cut: Option<CutReport>,
}

pub fn run_control(
    source: &Path,
    dest: &Path,
    control: Control,
    limits: Limits,
) -> Result<ControlReport, ExportError> {
    let mut rebuilt: Option<(Vec<WrittenEntry>, Option<CutReport>)> = None;
    let published = publish_new(
        source,
        dest,
        |out| match control {
            Control::ExactCopy => {
                let mut src = File::open(source).map_err(io_err(source))?;
                io::copy(&mut src, out).map_err(io_err(dest))?;
                Ok(())
            }
            Control::Roundtrip => {
                rebuilt = Some(rebuild(source, out, None, limits)?);
                Ok(())
            }
            Control::TruncateEnd(end) => {
                rebuilt = Some(rebuild(source, out, Some(end), limits)?);
                Ok(())
            }
        },
        |tmp| {
            let report = inspect(tmp, limits);
            if report.status != InspectStatus::Complete || report.has_errors() {
                let msgs: Vec<String> = report
                    .findings
                    .iter()
                    .map(|f| format!("{}: {}", f.code, f.message))
                    .collect();
                return Err(ExportError::Validation(msgs.join("; ")));
            }
            Ok(())
        },
    )?;
    let byte_identical = matches!(control, Control::Roundtrip)
        .then(|| published.source.full_hash.as_deref() == Some(published.output_hash.as_str()));
    let (entries, cut) = rebuilt.unwrap_or_default();
    Ok(ControlReport {
        control: control.name(),
        source: source.display().to_string(),
        published,
        byte_identical,
        entries,
        cut,
    })
}

/// Counts bytes written so entry offsets can be assigned while streaming.
struct Counting<W> {
    inner: W,
    pos: u64,
}

impl<W: Write> Counting<W> {
    fn put(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.inner.write_all(bytes)?;
        self.pos += bytes.len() as u64;
        Ok(())
    }
}

fn to_i32(v: u64, what: &str) -> Result<i32, ExportError> {
    i32::try_from(v).map_err(|_| ExportError::Unsupported(format!("{what} {v} exceeds i32")))
}

/// Re-encode every entry frame by frame. With `cut`, the playback entry stops
/// before the first frame group at or after `cut`.
fn rebuild(
    source: &Path,
    out: &mut File,
    cut: Option<DemoTime>,
    limits: Limits,
) -> Result<(Vec<WrittenEntry>, Option<CutReport>), ExportError> {
    let src = File::open(source).map_err(io_err(source))?;
    let mut reader = DemoReader::new(BufReader::new(src), limits)?;
    let mut header = reader.header.clone();
    let mut directory = reader.directory.clone();

    let cut_entry = match cut {
        None => None,
        Some(_) => {
            let types: Vec<i32> = directory.iter().map(|d| d.entry_type).collect();
            if types != [0, 1] {
                return Err(ExportError::Unsupported(format!(
                    "end-only truncation needs exactly LOADING(0) and Playback(1) entries, found types {types:?}"
                )));
            }
            Some(1u32)
        }
    };

    let dest_path = Path::new("<output>");
    let mut w = Counting {
        inner: BufWriter::new(&mut *out),
        pos: 0,
    };
    header.directory_offset = 0;
    w.put(&encode_header(&header)).map_err(io_err(dest_path))?;

    let mut written = Vec::new();
    let mut cut_report = None;
    let mut buf = Vec::with_capacity(64 * 1024);
    for i in 0..directory.len() as u32 {
        let start = w.pos;
        let truncate_at = if cut_entry == Some(i) { cut } else { None };
        let mut network = 0u64;
        let mut records = 0u64;
        let mut prev_frame_number: Option<i32> = None;
        let mut terminal: Option<Frame> = None;

        for item in reader.frames(i)? {
            let lf = item?;
            let f = lf.frame;
            if let Some(end) = truncate_at {
                if matches!(f.body, FrameBody::NextSection) {
                    let segment_end = DemoTime::from_f32(f.time()).unwrap_or(DemoTime::ZERO);
                    return Err(ExportError::RangeBeyondEnd { end, segment_end });
                }
                let t = DemoTime::from_f32(f.time()).unwrap_or(DemoTime::ZERO);
                let new_group = prev_frame_number != Some(f.frame_number);
                if new_group && t >= end {
                    terminal = Some(Frame {
                        tag: tag::NEXT_SECTION,
                        time_bits: f.time_bits,
                        frame_number: f.frame_number,
                        body: FrameBody::NextSection,
                    });
                    break;
                }
                prev_frame_number = Some(f.frame_number);
            }
            if matches!(f.body, FrameBody::Network(_)) {
                network += 1;
            }
            records += 1;
            buf.clear();
            encode_frame(&f, &mut buf);
            w.put(&buf).map_err(io_err(dest_path))?;
        }

        let d = &mut directory[i as usize];
        if let (Some(end), Some(term)) = (truncate_at, terminal.as_ref()) {
            if network == 0 {
                return Err(ExportError::EmptyRange);
            }
            buf.clear();
            encode_frame(term, &mut buf);
            w.put(&buf).map_err(io_err(dest_path))?;
            d.frame_count = to_i32(network, "network record count")?;
            d.track_time_bits = term.time_bits;
            let effective = DemoTime::from_f32(term.time()).unwrap_or(DemoTime::ZERO);
            cut_report = Some(CutReport {
                requested_end: end.to_string(),
                effective_end: effective.to_string(),
                effective_end_us: effective.micros(),
                retained_records: records,
                retained_network_records: network,
                terminal_frame_number: term.frame_number,
            });
        }
        d.offset = to_i32(start, "entry offset")?;
        d.length = to_i32(w.pos - start, "entry length")?;
        written.push(WrittenEntry {
            index: i,
            offset: d.offset,
            length: d.length,
            frame_count: d.frame_count,
            track_time: d.track_time(),
        });
    }

    let dir_offset = to_i32(w.pos, "directory offset")?;
    w.put(&encode_directory(&directory))
        .map_err(io_err(dest_path))?;
    w.inner.flush().map_err(io_err(dest_path))?;
    drop(w);
    out.seek(SeekFrom::Start(HEADER_DIR_OFFSET_FIELD as u64))
        .map_err(io_err(dest_path))?;
    out.write_all(&dir_offset.to_le_bytes())
        .map_err(io_err(dest_path))?;
    out.flush().map_err(io_err(dest_path))?;
    Ok((written, cut_report))
}
