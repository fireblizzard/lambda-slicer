use std::ops::Range;

use super::{LUMP_COUNT, LumpKind, MAX_RECORDS_PER_LUMP, ParseError, SourceLocation, read_i32};

const HEADER_SIZE: usize = 40;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MipTexture<'a> {
    Missing {
        source: SourceLocation,
    },
    External {
        source: SourceLocation,
        name: &'a [u8],
        width: u32,
        height: u32,
    },
    Embedded {
        source: SourceLocation,
        name: &'a [u8],
        width: u32,
        height: u32,
        mip_levels: [&'a [u8]; 4],
        palette: Option<&'a [u8]>,
    },
}

pub(super) fn parse<'a>(
    bytes: &'a [u8],
    lumps: &[Range<usize>; LUMP_COUNT],
) -> Result<Vec<MipTexture<'a>>, ParseError> {
    let lump = &bytes[lumps[LumpKind::Textures.index()].clone()];
    if lump.is_empty() {
        return Ok(Vec::new());
    }
    if lump.len() < 4 {
        return Err(ParseError::TruncatedTextureDirectory);
    }
    let raw_count = read_i32(lump, 0);
    if raw_count < 0 || raw_count as usize > MAX_RECORDS_PER_LUMP {
        return Err(ParseError::InvalidTextureCount(raw_count));
    }
    let count = raw_count as usize;
    let directory_end = count
        .checked_mul(4)
        .and_then(|length| 4_usize.checked_add(length))
        .filter(|end| *end <= lump.len())
        .ok_or(ParseError::TruncatedTextureDirectory)?;

    let mut textures = Vec::with_capacity(count);
    for element in 0..count {
        let source = SourceLocation {
            lump: LumpKind::Textures,
            element,
        };
        let offset = read_i32(lump, 4 + element * 4);
        if offset == -1 {
            textures.push(MipTexture::Missing { source });
            continue;
        }
        if offset < 0 || (offset as usize) < directory_end {
            return Err(ParseError::InvalidTextureOffset { element, offset });
        }
        let offset = offset as usize;
        let header_end = offset
            .checked_add(HEADER_SIZE)
            .filter(|end| *end <= lump.len())
            .ok_or(ParseError::TruncatedTextureHeader { element })?;
        let header = &lump[offset..header_end];
        let name_end = header[..16]
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(16);
        let name = &header[..name_end];
        let width = read_u32(header, 16);
        let height = read_u32(header, 20);
        if width == 0 || height == 0 {
            return Err(ParseError::InvalidTextureDimensions {
                element,
                width,
                height,
            });
        }
        let offsets = [
            read_u32(header, 24),
            read_u32(header, 28),
            read_u32(header, 32),
            read_u32(header, 36),
        ];
        if offsets.iter().all(|offset| *offset == 0) {
            textures.push(MipTexture::External {
                source,
                name,
                width,
                height,
            });
            continue;
        }
        if offsets.iter().any(|offset| *offset == 0) {
            return Err(ParseError::MixedTextureStorage { element });
        }

        let entry_end = (0..count)
            .filter_map(|other| {
                let other_offset = read_i32(lump, 4 + other * 4);
                (other_offset > offset as i32).then_some(other_offset as usize)
            })
            .min()
            .unwrap_or(lump.len());
        let mut levels = [&lump[0..0]; 4];
        let mut mip_end = offset;
        for level in 0..4 {
            let mip_width = (width >> level).max(1) as usize;
            let mip_height = (height >> level).max(1) as usize;
            let length =
                mip_width
                    .checked_mul(mip_height)
                    .ok_or(ParseError::TextureMipOutsideLump {
                        element,
                        level,
                        offset: offsets[level] as usize,
                        length: usize::MAX,
                        lump_length: lump.len(),
                    })?;
            let start = offset.checked_add(offsets[level] as usize).ok_or(
                ParseError::TextureMipOutsideLump {
                    element,
                    level,
                    offset: offsets[level] as usize,
                    length,
                    lump_length: lump.len(),
                },
            )?;
            let end = start
                .checked_add(length)
                .filter(|end| *end <= entry_end)
                .ok_or(ParseError::TextureMipOutsideLump {
                    element,
                    level,
                    offset: offsets[level] as usize,
                    length,
                    lump_length: lump.len(),
                })?;
            levels[level] = &lump[start..end];
            mip_end = mip_end.max(end);
        }
        let palette = mip_end
            .checked_add(2)
            .filter(|end| *end <= entry_end)
            .and_then(|count_end| {
                let color_count = u16::from_le_bytes(lump[mip_end..count_end].try_into().ok()?);
                let palette_end = usize::from(color_count)
                    .checked_mul(3)?
                    .checked_add(count_end)?;
                (color_count > 0 && palette_end <= entry_end)
                    .then_some(&lump[count_end..palette_end])
            });
        textures.push(MipTexture::Embedded {
            source,
            name,
            width,
            height,
            mip_levels: levels,
            palette,
        });
    }
    Ok(textures)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("texture field was sized"),
    )
}
