use std::collections::BTreeSet;
use std::ops::Range;

use super::{LUMP_COUNT, MAX_BSPX_LUMPS, ParseError};

const HEADER_SIZE: usize = 8;
const ENTRY_SIZE: usize = 32;
const NAME_SIZE: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BspxKind {
    BrushList,
    DecoupledLightmaps,
    FaceNormals,
    LightGridOctree,
    LightGrids,
    LightingDirection,
    LightingE5Bgr9,
    LightmapOffset,
    LightmapShift,
    RgbLighting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BspxLump<'a> {
    pub name: &'a str,
    pub kind: BspxKind,
    pub data: &'a [u8],
}

pub(super) fn parse<'a>(
    bytes: &'a [u8],
    lumps: &[Range<usize>; LUMP_COUNT],
) -> Result<Vec<BspxLump<'a>>, ParseError> {
    let official_end = lumps.iter().map(|range| range.end).max().unwrap_or(0);
    let Some(directory_offset) = official_end.checked_add(3).map(|offset| offset & !3) else {
        return Ok(Vec::new());
    };
    if bytes.get(directory_offset..directory_offset + 4) != Some(b"BSPX") {
        return Ok(Vec::new());
    }
    let header_end = directory_offset
        .checked_add(HEADER_SIZE)
        .filter(|end| *end <= bytes.len())
        .ok_or(ParseError::TruncatedBspxDirectory)?;
    let count = read_u32(bytes, directory_offset + 4) as usize;
    if count > MAX_BSPX_LUMPS {
        return Err(ParseError::BspxDirectoryTooLarge {
            count,
            maximum: MAX_BSPX_LUMPS,
        });
    }
    let directory_end = count
        .checked_mul(ENTRY_SIZE)
        .and_then(|length| header_end.checked_add(length))
        .filter(|end| *end <= bytes.len())
        .ok_or(ParseError::TruncatedBspxDirectory)?;

    let mut names = BTreeSet::new();
    let mut result = Vec::new();
    for element in 0..count {
        let entry = header_end + element * ENTRY_SIZE;
        let name_bytes = &bytes[entry..entry + NAME_SIZE];
        let Some(name_end) = name_bytes.iter().position(|byte| *byte == 0) else {
            return Err(ParseError::InvalidBspxName { element });
        };
        let name = std::str::from_utf8(&name_bytes[..name_end])
            .ok()
            .filter(|name| !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_graphic()))
            .ok_or(ParseError::InvalidBspxName { element })?;
        if !names.insert(name) {
            return Err(ParseError::DuplicateBspxName(name.into()));
        }
        let offset = read_u32(bytes, entry + NAME_SIZE) as usize;
        let length = read_u32(bytes, entry + NAME_SIZE + 4) as usize;
        let end = offset
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or(ParseError::BspxLumpOutsideFile {
                element,
                offset,
                length,
                file_length: bytes.len(),
            })?;
        if let Some(kind) = recognized_kind(name) {
            result.push(BspxLump {
                name,
                kind,
                data: &bytes[offset..end],
            });
        }
    }

    // The directory may precede its data, but it must itself be fully readable
    let _ = directory_end;
    Ok(result)
}

fn recognized_kind(name: &str) -> Option<BspxKind> {
    match name {
        "BRUSHLIST" => Some(BspxKind::BrushList),
        "DECOUPLED_LM" => Some(BspxKind::DecoupledLightmaps),
        "FACENORMALS" => Some(BspxKind::FaceNormals),
        "LIGHTGRID_OCTREE" => Some(BspxKind::LightGridOctree),
        "LIGHTGRIDS" => Some(BspxKind::LightGrids),
        "LIGHTINGDIR" => Some(BspxKind::LightingDirection),
        "LIGHTING_E5BGR9" => Some(BspxKind::LightingE5Bgr9),
        "LMOFFSET" => Some(BspxKind::LightmapOffset),
        "LMSHIFT" => Some(BspxKind::LightmapShift),
        "RGBLIGHTING" => Some(BspxKind::RgbLighting),
        _ => None,
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("BSPX field was sized"),
    )
}
