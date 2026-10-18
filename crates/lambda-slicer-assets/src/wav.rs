//! The game's sounds, PCM WAV files, with the loop the game reads from them
//!
//! 8-bit unsigned and 16-bit signed PCM are read, mono or stereo, as 16-bit
//! samples. A sound loops when it has a `cue ` chunk, from the sample offset
//! of its first cue point, and a following `LIST` chunk whose label is a
//! `mark` gives the loop's length in samples. That is how Quake's
//! `GetWavinfo` reads them, which the game kept, so a loop is found where
//! the game finds it even in files other tools would read differently

use crate::{AssetError, Limits};

#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub rate: u32,
    pub channels: u16,
    /// Interleaved when stereo
    pub samples: Vec<i16>,
    /// The first sample of the loop, none for a sound played once
    pub loop_start: Option<u32>,
    /// Sample frames the sound plays to, the loop's end when it loops
    pub frames: u32,
}

impl Sound {
    /// Seconds the sound plays once at its own rate, to its end or its
    /// loop's end
    pub fn seconds(&self) -> f64 {
        f64::from(self.frames) / f64::from(self.rate.max(1))
    }
}

fn invalid(reason: impl Into<String>) -> AssetError {
    AssetError::Sound(reason.into())
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// Each chunk's name, where it starts and its data
fn chunks(bytes: &[u8]) -> Vec<(&[u8], usize, &[u8])> {
    let mut out = Vec::new();
    let mut at = 12;
    while let (Some(name), Some(size)) = (bytes.get(at..at + 4), u32_at(bytes, at + 4)) {
        let start = at + 8;
        let end = start.saturating_add(size as usize).min(bytes.len());
        out.push((name, at, &bytes[start..end]));
        // Chunks are padded to an even length
        at = end + (size as usize & 1);
    }
    out
}

pub fn parse(bytes: &[u8], limits: Limits) -> Result<Sound, AssetError> {
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err(invalid("not a WAV file"));
    }
    let chunks = chunks(bytes);
    let find = |name: &[u8]| chunks.iter().position(|(n, _, _)| *n == name);

    let format = find(b"fmt ")
        .map(|i| chunks[i].2)
        .ok_or_else(|| invalid("no format"))?;
    let (Some(tag), Some(channels), Some(rate), Some(bits)) = (
        u16_at(format, 0),
        u16_at(format, 2),
        u32_at(format, 4),
        u16_at(format, 14),
    ) else {
        return Err(invalid("the format is short"));
    };
    if tag != 1 {
        return Err(invalid(format!("format {tag} is not PCM")));
    }
    if !(1..=2).contains(&channels) || rate == 0 {
        return Err(invalid(format!("{channels} channels at {rate} Hz")));
    }
    let width = match bits {
        8 => 1,
        16 => 2,
        _ => return Err(invalid(format!("{bits} bits a sample are not read"))),
    };

    let data = find(b"data")
        .map(|i| chunks[i].2)
        .ok_or_else(|| invalid("no data"))?;
    if data.len() as u64 * 2 > limits.max_file {
        return Err(AssetError::TooLarge {
            path: "sound".into(),
            size: data.len() as u64 * 2,
            limit: limits.max_file,
        });
    }
    let samples: Vec<i16> = match width {
        1 => data.iter().map(|&b| (i16::from(b) - 128) << 8).collect(),
        _ => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes(*b))
            .collect(),
    };
    let stored = (samples.len() / usize::from(channels)) as u32;

    // GetWavinfo's loop, 32 bytes past the start of the cue chunk's header,
    // and the length a mark label gives
    let mut loop_start = None;
    let mut frames = stored;
    if let Some(cue) = find(b"cue ") {
        let start = u32_at(bytes, chunks[cue].1 + 32).unwrap_or(0);
        loop_start = Some(start.min(stored));
        let list = chunks[cue + 1..].iter().find(|(n, _, _)| *n == b"LIST");
        if let Some(&(_, at, _)) = list
            && bytes.get(at + 28..at + 32) == Some(b"mark")
            && let Some(length) = u32_at(bytes, at + 24)
        {
            frames = start.saturating_add(length).min(stored);
        }
    }

    Ok(Sound {
        rate,
        channels,
        samples,
        loop_start,
        frames,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(bits: u16, data: &[u8], extra: &[u8]) -> Vec<u8> {
        let mut b = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(22050u32.to_le_bytes());
        b.extend((22050 * u32::from(bits / 8)).to_le_bytes());
        b.extend((bits / 8).to_le_bytes());
        b.extend(bits.to_le_bytes());
        b.extend(b"data");
        b.extend((data.len() as u32).to_le_bytes());
        b.extend(data);
        b.extend(extra);
        b
    }

    /// A cue chunk with one point whose sample offset is `start`, and a
    /// mark label of `length` samples
    fn cue(start: u32, length: Option<u32>) -> Vec<u8> {
        let mut b = b"cue ".to_vec();
        b.extend(28u32.to_le_bytes());
        for v in [1u32, 1, 0] {
            b.extend(v.to_le_bytes());
        }
        b.extend(b"data");
        for v in [0u32, 0, start] {
            b.extend(v.to_le_bytes());
        }
        if let Some(length) = length {
            b.extend(b"LIST");
            b.extend(28u32.to_le_bytes());
            b.extend(b"adtlltxt");
            b.extend(16u32.to_le_bytes());
            b.extend(1u32.to_le_bytes());
            b.extend(length.to_le_bytes());
            b.extend(b"mark");
            b.extend([0u8; 4]);
        }
        b
    }

    #[test]
    fn samples_widen_and_loops_come_from_the_cue_chunk() {
        let once = parse(&wav(8, &[128, 255, 0, 128], &[]), Limits::default()).unwrap();
        assert_eq!(once.samples, [0, 127 << 8, -128 << 8, 0]);
        assert_eq!((once.loop_start, once.frames), (None, 4));
        assert!((once.seconds() - 4. / 22050.).abs() < 1e-9);

        let looped = parse(&wav(8, &[128; 10], &cue(2, None)), Limits::default()).unwrap();
        assert_eq!((looped.loop_start, looped.frames), (Some(2), 10));

        let marked = parse(&wav(8, &[128; 10], &cue(2, Some(5))), Limits::default()).unwrap();
        assert_eq!((marked.loop_start, marked.frames), (Some(2), 7));

        let wide = parse(&wav(16, &[0x34, 0x12, 0xff, 0xff], &[]), Limits::default()).unwrap();
        assert_eq!(wide.samples, [0x1234, -1]);

        assert!(parse(b"RIFF\0\0\0\0WAVEjunk", Limits::default()).is_err());
    }
}
