//! The game's sprites (`IDSP` version 2), single frames and frame groups
//!
//! A sprite holds its orientation, its texture format, a palette and its
//! frames, each a block of palette indices with its offset from the
//! sprite's origin. Grouped frames are flattened in order, each keeping
//! its group's interval. Sizes are checked before anything is allocated

use crate::{AssetError, Limits};

const IDENT: &[u8; 4] = b"IDSP";
const VERSION: i32 = 2;
const HEADER: usize = 40;
/// The most frames one sprite may hold
const MAX_FRAMES: usize = 4096;

/// How a sprite faces the camera (`SPR_*`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    /// Facing the view, kept upright
    ParallelUpright,
    /// Facing the camera's position, kept upright
    FacingUpright,
    /// Facing the view
    Parallel,
    /// Turned by its entity's angles
    Oriented,
    /// Facing the view, rolled by its entity
    ParallelOriented,
}

/// How a sprite's palette indices become colours (`SPR_*`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Normal,
    Additive,
    /// The last palette colour, the index giving its alpha
    IndexAlpha,
    /// Index 255 transparent
    AlphaTest,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// The left and top edges relative to the sprite's origin, in texels
    pub origin: [i32; 2],
    pub width: u32,
    pub height: u32,
    /// Palette indices, rows from the top
    pub indices: Vec<u8>,
    /// Seconds the frame shows within its group, none for a single frame
    pub interval: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    pub orientation: Orientation,
    pub format: Format,
    pub radius: f32,
    /// 256 colours
    pub palette: Vec<[u8; 3]>,
    pub frames: Vec<Frame>,
}

impl Sprite {
    /// A frame as RGBA, as the sprite's format colours it
    pub fn rgba(&self, frame: &Frame) -> Vec<u8> {
        let last = self.palette.last().copied().unwrap_or([255; 3]);
        frame
            .indices
            .iter()
            .flat_map(|&i| {
                let colour = self.palette.get(usize::from(i)).copied().unwrap_or([0; 3]);
                match self.format {
                    Format::IndexAlpha => [last[0], last[1], last[2], i],
                    Format::AlphaTest if i == 255 => [0, 0, 0, 0],
                    _ => [colour[0], colour[1], colour[2], 255],
                }
            })
            .collect()
    }
}

fn invalid(reason: impl Into<String>) -> AssetError {
    AssetError::Image(reason.into())
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], AssetError> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.bytes.len());
        let end = end.ok_or_else(|| invalid("the sprite ends early"))?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn i32(&mut self) -> Result<i32, AssetError> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    fn f32(&mut self) -> Result<f32, AssetError> {
        Ok(f32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    fn frame(&mut self, interval: Option<f32>, limits: Limits) -> Result<Frame, AssetError> {
        let origin = [self.i32()?, self.i32()?];
        let (width, height) = (self.i32()?, self.i32()?);
        let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
            return Err(invalid("a frame has a negative size"));
        };
        let texels = u64::from(width) * u64::from(height);
        if texels * 4 > limits.max_file {
            return Err(AssetError::TooLarge {
                path: "sprite frame".into(),
                size: texels * 4,
                limit: limits.max_file,
            });
        }
        let indices = self.take(texels as usize)?.to_vec();
        Ok(Frame {
            origin,
            width,
            height,
            indices,
            interval,
        })
    }
}

pub fn parse(bytes: &[u8], limits: Limits) -> Result<Sprite, AssetError> {
    if bytes.len() < HEADER || &bytes[..4] != IDENT {
        return Err(invalid("not a sprite"));
    }
    let mut r = Reader { bytes, at: 4 };
    if r.i32()? != VERSION {
        return Err(invalid("not a version 2 sprite"));
    }
    let orientation = match r.i32()? {
        0 => Orientation::ParallelUpright,
        1 => Orientation::FacingUpright,
        2 => Orientation::Parallel,
        3 => Orientation::Oriented,
        4 => Orientation::ParallelOriented,
        other => return Err(invalid(format!("orientation {other} is unknown"))),
    };
    let format = match r.i32()? {
        0 => Format::Normal,
        1 => Format::Additive,
        2 => Format::IndexAlpha,
        3 => Format::AlphaTest,
        other => return Err(invalid(format!("texture format {other} is unknown"))),
    };
    let radius = r.f32()?;
    let (_max_width, _max_height) = (r.i32()?, r.i32()?);
    let count = usize::try_from(r.i32()?).map_err(|_| invalid("a negative frame count"))?;
    if count > MAX_FRAMES {
        return Err(invalid(format!("{count} frames, over {MAX_FRAMES}")));
    }
    let (_beam_length, _sync) = (r.f32()?, r.i32()?);

    let colours = usize::from(u16::from_le_bytes(
        r.take(2)?.try_into().expect("two bytes"),
    ));
    let mut palette: Vec<[u8; 3]> = r.take(colours * 3)?.as_chunks::<3>().0.to_vec();
    palette.resize(256, [0; 3]);

    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        match r.i32()? {
            0 => frames.push(r.frame(None, limits)?),
            _ => {
                let grouped = usize::try_from(r.i32()?).map_err(|_| invalid("a negative group"))?;
                if frames.len() + grouped > MAX_FRAMES {
                    return Err(invalid(format!("over {MAX_FRAMES} frames")));
                }
                let mut intervals = Vec::with_capacity(grouped);
                for _ in 0..grouped {
                    intervals.push(r.f32()?);
                }
                for interval in intervals {
                    frames.push(r.frame(Some(interval), limits)?);
                }
            }
        }
    }
    Ok(Sprite {
        orientation,
        format,
        radius,
        palette,
        frames,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite(format: i32, frames: &[u8]) -> Vec<u8> {
        let mut b = IDENT.to_vec();
        for v in [VERSION, 2, format] {
            b.extend(v.to_le_bytes());
        }
        b.extend(10f32.to_le_bytes());
        for v in [2i32, 1, 2] {
            b.extend(v.to_le_bytes());
        }
        b.extend(0f32.to_le_bytes());
        b.extend(0i32.to_le_bytes());
        b.extend(256u16.to_le_bytes());
        for i in 0..=255u8 {
            b.extend([i, 0, 255 - i]);
        }
        b.extend(frames);
        b
    }

    fn frame(origin: [i32; 2], indices: [u8; 2]) -> Vec<u8> {
        let mut b = Vec::new();
        for v in [origin[0], origin[1], 2, 1] {
            b.extend(v.to_le_bytes());
        }
        b.extend(indices);
        b
    }

    #[test]
    fn single_and_grouped_frames_read_in_order() {
        let mut frames = 0i32.to_le_bytes().to_vec();
        frames.extend(frame([-1, 1], [3, 255]));
        frames.extend(1i32.to_le_bytes());
        frames.extend(1i32.to_le_bytes());
        frames.extend(0.1f32.to_le_bytes());
        frames.extend(frame([0, 0], [7, 8]));

        let s = parse(&sprite(3, &frames), Limits::default()).unwrap();
        assert_eq!(s.orientation, Orientation::Parallel);
        assert_eq!(s.format, Format::AlphaTest);
        assert_eq!(s.frames.len(), 2);
        assert_eq!(s.frames[0].origin, [-1, 1]);
        assert_eq!(s.frames[1].interval, Some(0.1));
        // Index 255 is cut out in an alpha-tested sprite
        assert_eq!(s.rgba(&s.frames[0]), [3, 0, 252, 255, 0, 0, 0, 0]);

        let s = parse(&sprite(2, &frames), Limits::default()).unwrap();
        // Index alpha takes the last colour, the index as alpha
        assert_eq!(s.rgba(&s.frames[1]), [255, 0, 0, 7, 255, 0, 0, 8]);

        let short = sprite(0, &frames[..20]);
        assert!(parse(&short, Limits::default()).is_err());
    }
}
