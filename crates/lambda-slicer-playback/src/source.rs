//! A demo's records read in groups, one group at a time
//!
//! A group is the run of records the recording client wrote for one frame,
//! those sharing an entry and a frame number, as restarts count them. Every
//! entry is read in directory order, the LOADING entry first, and terminal
//! records only end a group. The reader keeps no more than one group and
//! one record past it in memory, and goes on from where it stopped

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use lambda_slicer_core::container::{
    DemoReader, FrameBody, FramePosition, Header, Limits, LocatedFrame,
};
use lambda_slicer_core::message::{Coded, Message, io_message};
use lambda_slicer_core::protocol::profile;
use lambda_slicer_core::state::FrameLocation;

use crate::time::SourceTime;

/// One record and where it came from
#[derive(Clone, Debug)]
pub struct Record {
    pub location: FrameLocation,
    pub body: FrameBody,
}

#[derive(Clone, Debug)]
pub struct Group {
    /// Counted from 0 across every entry
    pub number: u64,
    pub entry: u32,
    pub frame_number: i32,
    /// The first record's playback time
    pub time: SourceTime,
    pub records: Vec<Record>,
}

/// Records are read through a buffer this large, as a group is usually a
/// few kilobytes and every group starts with a seek
const BUFFER: usize = 64 * 1024;

/// Where a source goes on from, saved with a checkpoint
#[derive(Clone, Debug)]
pub struct Cursor {
    at: Option<FramePosition>,
    ahead: Option<(u32, LocatedFrame)>,
    next_number: u64,
}

pub struct Source {
    reader: DemoReader<BufReader<File>>,
    /// Where reading goes on, none once every entry is read
    at: Option<FramePosition>,
    /// The record read past the end of the last group, the next one's first
    ahead: Option<(u32, LocatedFrame)>,
    next_number: u64,
}

fn location(entry: u32, frame: &LocatedFrame) -> FrameLocation {
    FrameLocation {
        entry,
        ordinal: frame.ordinal,
        offset: frame.offset,
        time_bits: frame.frame.time_bits,
        frame_number: frame.frame.frame_number,
    }
}

impl Source {
    /// Open `path`, refusing a protocol this version does not decode
    pub fn open(path: &Path, limits: Limits) -> Result<Self, Message> {
        let file = File::open(path).map_err(|e| io_message(&e))?;
        let reader = DemoReader::new(BufReader::with_capacity(BUFFER, file), limits)
            .map_err(|e| e.message())?;
        profile::check(&reader.header).map_err(|e| e.message())?;

        let mut source = Self {
            reader,
            at: None,
            ahead: None,
            next_number: 0,
        };
        source.at = source.entry_start(0)?;
        Ok(source)
    }

    pub fn header(&self) -> &Header {
        &self.reader.header
    }

    pub fn cursor(&self) -> Cursor {
        Cursor {
            at: self.at,
            ahead: self.ahead.clone(),
            next_number: self.next_number,
        }
    }

    /// Go on from where `cursor` was taken, in this same demo
    pub fn restore(&mut self, cursor: &Cursor) {
        self.at = cursor.at;
        self.ahead = cursor.ahead.clone();
        self.next_number = cursor.next_number;
    }

    fn entry_start(&mut self, entry: u32) -> Result<Option<FramePosition>, Message> {
        if entry as usize >= self.reader.directory.len() {
            return Ok(None);
        }

        let frames = self.reader.frames(entry).map_err(|e| e.message())?;
        Ok(Some(frames.position()))
    }

    /// The next group, none after the last
    pub fn next_group(&mut self) -> Result<Option<Group>, Message> {
        let mut building = Building::default();
        if let Some((entry, frame)) = self.ahead.take() {
            building.add(entry, frame);
        }

        // One iterator per entry for the whole group, so the read buffer is
        // kept between records
        while let Some(at) = self.at {
            let mut frames = self.reader.frames_from(at).map_err(|e| e.message())?;
            let mut ended = true;
            for item in frames.by_ref() {
                let frame = item.map_err(|e| e.message())?;
                if let Some(past) = building.add(at.entry, frame) {
                    self.ahead = Some((at.entry, past));
                    ended = false;
                    break;
                }
            }
            let position = frames.position();

            if !ended {
                self.at = Some(position);
                break;
            }
            self.at = self.entry_start(at.entry + 1)?;
        }

        let Some((entry, frame_number)) = building.key else {
            return Ok(None);
        };
        let group = Group {
            number: self.next_number,
            entry,
            frame_number,
            time: building.time,
            records: building.records,
        };
        self.next_number += 1;
        Ok(Some(group))
    }
}

#[derive(Default)]
struct Building {
    key: Option<(u32, i32)>,
    time: SourceTime,
    records: Vec<Record>,
}

impl Building {
    /// Add a record of this group, or give it back when it starts the next
    fn add(&mut self, entry: u32, frame: LocatedFrame) -> Option<LocatedFrame> {
        let key = (entry, frame.frame.frame_number);
        if self.key.is_some_and(|k| k != key) {
            return Some(frame);
        }

        if matches!(frame.frame.body, FrameBody::NextSection) {
            return None;
        }
        let location = location(entry, &frame);
        if self.key.is_none() {
            self.key = Some(key);
            self.time = SourceTime(f64::from(location.playback_time()));
        }
        self.records.push(Record {
            location,
            body: frame.frame.body,
        });
        None
    }
}
