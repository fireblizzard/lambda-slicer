//! Bounded, streaming container reader. Every length or offset derived from
//! file contents is checked against the containing region before use

use std::io::{self, Read, Seek, SeekFrom};

use super::frame::{
    CLIENT_DATA_LEN, CONSOLE_COMMAND_LEN, DEMO_INFO_LEN, EVENT_ARGS_LEN, EventRecord, Frame,
    FrameBody, NetworkFrame, SoundRecord, tag,
};
use super::{ContainerError, DIR_ENTRY_LEN, DirectoryEntry, FrameCause, HEADER_LEN, Header, MAGIC};

/// Resource limits for lengths read from the file. Defaults are generous
/// parser caps, not the engine's encoded packet limits
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_directory_entries: u32,
    pub max_network_message: u64,
    pub max_demo_buffer: u64,
    pub max_sound_sample: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_directory_entries: 1024,
            max_network_message: 4 << 20,
            max_demo_buffer: 4 << 20,
            max_sound_sample: 4096,
        }
    }
}

fn io_at(offset: u64) -> impl FnOnce(io::Error) -> ContainerError {
    move |source| ContainerError::Io { offset, source }
}

pub fn read_header<R: Read + Seek>(r: &mut R, file_len: u64) -> Result<Header, ContainerError> {
    if file_len < HEADER_LEN {
        return Err(ContainerError::TooShort { len: file_len });
    }

    let mut buf = [0u8; HEADER_LEN as usize];
    r.seek(SeekFrom::Start(0)).map_err(io_at(0))?;
    r.read_exact(&mut buf).map_err(io_at(0))?;
    if &buf[0..8] != MAGIC {
        return Err(ContainerError::BadMagic);
    }

    let i32_at = |at: usize| i32::from_le_bytes(buf[at..at + 4].try_into().unwrap());
    let mut map_name_raw = Box::new([0u8; 260]);
    map_name_raw.copy_from_slice(&buf[16..276]);
    let mut game_dir_raw = Box::new([0u8; 260]);
    game_dir_raw.copy_from_slice(&buf[276..536]);

    Ok(Header {
        demo_protocol: i32_at(8),
        network_protocol: i32_at(12),
        map_name_raw,
        game_dir_raw,
        map_checksum: u32::from_le_bytes(buf[536..540].try_into().unwrap()),
        directory_offset: i32_at(540),
    })
}

pub fn read_directory<R: Read + Seek>(
    r: &mut R,
    header: &Header,
    file_len: u64,
    limits: &Limits,
) -> Result<Vec<DirectoryEntry>, ContainerError> {
    if header.directory_offset == 0 {
        return Err(ContainerError::NoDirectory);
    }

    let offset = u64::try_from(header.directory_offset).map_err(|_| ContainerError::Range {
        what: "directory offset",
        detail: format!("negative offset {}", header.directory_offset),
    })?;
    if offset < HEADER_LEN || offset.saturating_add(4) > file_len {
        return Err(ContainerError::Range {
            what: "directory offset",
            detail: format!("{offset} is outside {HEADER_LEN}..{file_len}"),
        });
    }

    let mut count_buf = [0u8; 4];
    r.seek(SeekFrom::Start(offset)).map_err(io_at(offset))?;
    r.read_exact(&mut count_buf).map_err(io_at(offset))?;
    let count = i32::from_le_bytes(count_buf);
    let count = u32::try_from(count)
        .ok()
        .filter(|&c| c <= limits.max_directory_entries)
        .ok_or_else(|| ContainerError::Range {
            what: "directory entry count",
            detail: format!("{count} (limit {})", limits.max_directory_entries),
        })?;

    let table_end = offset + 4 + u64::from(count) * DIR_ENTRY_LEN;
    if table_end > file_len {
        return Err(ContainerError::Range {
            what: "directory table",
            detail: format!("ends at {table_end}, file is {file_len} bytes"),
        });
    }

    let mut entries = Vec::with_capacity(count as usize);
    for i in 0..u64::from(count) {
        let at = offset + 4 + i * DIR_ENTRY_LEN;
        let mut b = [0u8; DIR_ENTRY_LEN as usize];
        r.read_exact(&mut b).map_err(io_at(at))?;

        let i32_at = |o: usize| i32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let mut description_raw = [0u8; 64];
        description_raw.copy_from_slice(&b[4..68]);
        entries.push(DirectoryEntry {
            entry_type: i32_at(0),
            description_raw,
            flags: i32_at(68),
            cd_track: i32_at(72),
            track_time_bits: u32::from_le_bytes(b[76..80].try_into().unwrap()),
            frame_count: i32_at(80),
            offset: i32_at(84),
            length: i32_at(88),
        });
    }

    Ok(entries)
}

/// An open demo. Header and directory are validated eagerly, and frames are
/// streamed per entry on demand
pub struct DemoReader<R> {
    inner: R,
    pub file_len: u64,
    pub header: Header,
    pub directory: Vec<DirectoryEntry>,
    pub limits: Limits,
}

impl<R: Read + Seek> DemoReader<R> {
    pub fn new(mut inner: R, limits: Limits) -> Result<Self, ContainerError> {
        let file_len = inner.seek(SeekFrom::End(0)).map_err(io_at(0))?;
        let header = read_header(&mut inner, file_len)?;
        let directory = read_directory(&mut inner, &header, file_len, &limits)?;

        Ok(Self {
            inner,
            file_len,
            header,
            directory,
            limits,
        })
    }

    /// Stream frames of directory entry `entry`. The iterator yields at most
    /// one error, after which it ends
    pub fn frames(&mut self, entry: u32) -> Result<FrameIter<'_, R>, ContainerError> {
        let dir = self
            .directory
            .get(entry as usize)
            .ok_or(ContainerError::Range {
                what: "directory entry index",
                detail: format!("{entry} of {}", self.directory.len()),
            })?;
        let (start, end) = dir.region(self.file_len)?;
        self.inner
            .seek(SeekFrom::Start(start))
            .map_err(io_at(start))?;

        Ok(FrameIter {
            r: &mut self.inner,
            limits: self.limits,
            entry,
            pos: start,
            end,
            ordinal: 0,
            done: false,
            terminals: 0,
            after_terminal: 0,
        })
    }
}

/// A frame plus its location in the source
#[derive(Clone, Debug)]
pub struct LocatedFrame {
    pub ordinal: u64,
    pub offset: u64,
    pub len: u64,
    pub frame: Frame,
}

pub struct FrameIter<'a, R> {
    r: &'a mut R,
    limits: Limits,
    entry: u32,
    pos: u64,
    end: u64,
    ordinal: u64,
    done: bool,
    terminals: u64,
    after_terminal: u64,
}

impl<R> FrameIter<'_, R> {
    /// NextSection records read so far
    pub fn terminal_records(&self) -> u64 {
        self.terminals
    }

    /// Non-NextSection records read after the first NextSection
    pub fn records_after_terminal(&self) -> u64 {
        self.after_terminal
    }
}

enum Fail {
    Cause(FrameCause),
    Io(io::Error),
}

impl From<FrameCause> for Fail {
    fn from(c: FrameCause) -> Self {
        Fail::Cause(c)
    }
}

impl<R: Read> FrameIter<'_, R> {
    fn fail(&mut self, offset: u64, fail: Fail) -> ContainerError {
        self.done = true;
        match fail {
            Fail::Cause(cause) => ContainerError::Frame {
                entry: self.entry,
                ordinal: self.ordinal,
                offset,
                cause,
            },
            Fail::Io(source) => ContainerError::Io {
                offset: self.pos,
                source,
            },
        }
    }

    fn bytes(&mut self, field: &'static str, buf: &mut [u8]) -> Result<(), Fail> {
        let needed = buf.len() as u64;
        let available = self.end - self.pos;
        if needed > available {
            return Err(FrameCause::Truncated {
                field,
                needed,
                available,
            }
            .into());
        }

        self.r.read_exact(buf).map_err(Fail::Io)?;
        self.pos += needed;
        Ok(())
    }

    fn arr<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N], Fail> {
        let mut b = [0u8; N];
        self.bytes(field, &mut b)?;
        Ok(b)
    }

    fn i32(&mut self, field: &'static str) -> Result<i32, Fail> {
        Ok(i32::from_le_bytes(self.arr::<4>(field)?))
    }

    fn u32(&mut self, field: &'static str) -> Result<u32, Fail> {
        Ok(u32::from_le_bytes(self.arr::<4>(field)?))
    }

    /// Length-prefixed byte string with sign and limit checks
    fn sized(&mut self, field: &'static str, limit: u64) -> Result<Vec<u8>, Fail> {
        let len = self.i32(field)?;
        let len = u64::try_from(len).map_err(|_| FrameCause::NegativeLength {
            field,
            len: len.into(),
        })?;
        if len > limit {
            return Err(FrameCause::OverLimit { field, len, limit }.into());
        }

        let available = self.end - self.pos;
        if len > available {
            return Err(FrameCause::Truncated {
                field,
                needed: len,
                available,
            }
            .into());
        }

        let mut v = vec![0u8; len as usize];
        self.bytes(field, &mut v)?;
        Ok(v)
    }

    fn read_frame(&mut self) -> Result<Frame, Fail> {
        let [t] = self.arr::<1>("frame tag")?;
        let time_bits = self.u32("frame time")?;
        let frame_number = self.i32("frame number")?;
        if !f32::from_bits(time_bits).is_finite() {
            return Err(FrameCause::NonFiniteTime(time_bits).into());
        }

        let body = match t {
            tag::NETWORK_START | tag::NETWORK => {
                let demo_info = Box::new(self.arr::<DEMO_INFO_LEN>("demo info")?);
                let mut sequence = [0i32; 7];
                for s in &mut sequence {
                    *s = self.i32("sequence info")?;
                }
                let message = self.sized("network message", self.limits.max_network_message)?;

                FrameBody::Network(Box::new(NetworkFrame {
                    demo_info,
                    sequence,
                    message,
                }))
            }
            tag::DEMO_START => FrameBody::DemoStart,
            tag::CONSOLE_COMMAND => {
                FrameBody::ConsoleCommand(Box::new(self.arr::<CONSOLE_COMMAND_LEN>("command")?))
            }
            tag::CLIENT_DATA => FrameBody::ClientData(self.arr::<CLIENT_DATA_LEN>("client data")?),
            tag::NEXT_SECTION => FrameBody::NextSection,
            tag::EVENT => FrameBody::Event(EventRecord {
                flags: self.i32("event flags")?,
                index: self.i32("event index")?,
                delay_bits: self.u32("event delay")?,
                args: self.arr::<EVENT_ARGS_LEN>("event args")?,
            }),
            tag::WEAPON_ANIM => FrameBody::WeaponAnim {
                sequence: self.i32("weapon anim sequence")?,
                body: self.i32("weapon anim body")?,
            },
            tag::SOUND => FrameBody::Sound(SoundRecord {
                channel: self.i32("sound channel")?,
                sample: self.sized("sound sample", self.limits.max_sound_sample)?,
                attenuation_bits: self.u32("sound attenuation")?,
                volume_bits: self.u32("sound volume")?,
                flags: self.i32("sound flags")?,
                pitch: self.i32("sound pitch")?,
            }),
            tag::DEMO_BUFFER => {
                FrameBody::DemoBuffer(self.sized("demo buffer", self.limits.max_demo_buffer)?)
            }
            other => return Err(FrameCause::UnknownTag(other).into()),
        };

        Ok(Frame {
            tag: t,
            time_bits,
            frame_number,
            body,
        })
    }
}

impl<R: Read> Iterator for FrameIter<'_, R> {
    type Item = Result<LocatedFrame, ContainerError>;

    /// Reads the whole entry region. Recordings can contain several
    /// NextSection records in a row, and all of them are yielded. An entry that
    /// ends without any NextSection is an error
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let offset = self.pos;
        if offset == self.end {
            self.done = true;
            if self.terminals == 0 {
                return Some(Err(self.fail(offset, FrameCause::MissingTerminal.into())));
            }
            return None;
        }

        match self.read_frame() {
            Ok(frame) => {
                if matches!(frame.body, FrameBody::NextSection) {
                    self.terminals += 1;
                } else if self.terminals > 0 {
                    self.after_terminal += 1;
                }

                let item = LocatedFrame {
                    ordinal: self.ordinal,
                    offset,
                    len: self.pos - offset,
                    frame,
                };
                self.ordinal += 1;
                Some(Ok(item))
            }
            Err(f) => Some(Err(self.fail(offset, f))),
        }
    }
}
