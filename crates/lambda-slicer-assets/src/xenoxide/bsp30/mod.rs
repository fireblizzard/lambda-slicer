//! Safe BSP30 parsing and validated source representations.

use std::fmt;
use std::ops::Range;

mod bspx;
mod texture;
mod validation;

pub use bspx::{BspxKind, BspxLump};
pub use texture::MipTexture;

pub const BSP_VERSION: i32 = 30;
pub const LUMP_COUNT: usize = 15;
pub const HEADER_SIZE: usize = 4 + LUMP_COUNT * 8;
pub const MAX_ENTITY_TEXT_SIZE: usize = 16 * 1024 * 1024;
pub const MAX_RECORDS_PER_LUMP: usize = 16 * 1024 * 1024;
pub const MAX_BSPX_LUMPS: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LumpKind {
    Entities = 0,
    Planes = 1,
    Textures = 2,
    Vertices = 3,
    Visibility = 4,
    Nodes = 5,
    TextureInfo = 6,
    Faces = 7,
    Lighting = 8,
    ClipNodes = 9,
    Leaves = 10,
    MarkSurfaces = 11,
    Edges = 12,
    SurfaceEdges = 13,
    Models = 14,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LumpLayout {
    Standard,
    BlueShift,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub lump: LumpKind,
    pub element: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceFloat {
    pub value: f32,
    pub original_bits: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub source: SourceLocation,
    pub normal: [SourceFloat; 3],
    pub distance: SourceFloat,
    pub plane_type: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub source: SourceLocation,
    pub point: [SourceFloat; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Edge {
    pub source: SourceLocation,
    pub vertices: [u16; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceEdge {
    pub source: SourceLocation,
    pub edge: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarkSurface {
    pub source: SourceLocation,
    pub face: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureInfo {
    pub source: SourceLocation,
    pub s_axis: [SourceFloat; 4],
    pub t_axis: [SourceFloat; 4],
    pub mip_texture: i32,
    pub flags: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Face {
    pub source: SourceLocation,
    pub plane: u16,
    pub plane_side: i16,
    pub first_surface_edge: i32,
    pub surface_edge_count: i16,
    pub texture_info: i16,
    pub styles: [u8; 4],
    pub lighting_offset: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Node {
    pub source: SourceLocation,
    pub plane: i32,
    pub children: [i16; 2],
    pub minimum: [i16; 3],
    pub maximum: [i16; 3],
    pub first_face: u16,
    pub face_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipNode {
    pub source: SourceLocation,
    pub plane: i32,
    pub children: [i16; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Leaf {
    pub source: SourceLocation,
    pub contents: i32,
    pub visibility_offset: i32,
    pub minimum: [i16; 3],
    pub maximum: [i16; 3],
    pub first_mark_surface: u16,
    pub mark_surface_count: u16,
    pub ambient_levels: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Model {
    pub source: SourceLocation,
    pub minimum: [SourceFloat; 3],
    pub maximum: [SourceFloat; 3],
    pub origin: [SourceFloat; 3],
    pub head_nodes: [i32; 4],
    pub visibility_leaf_count: i32,
    pub first_face: i32,
    pub face_count: i32,
}

impl LumpKind {
    pub const ALL: [Self; LUMP_COUNT] = [
        Self::Entities,
        Self::Planes,
        Self::Textures,
        Self::Vertices,
        Self::Visibility,
        Self::Nodes,
        Self::TextureInfo,
        Self::Faces,
        Self::Lighting,
        Self::ClipNodes,
        Self::Leaves,
        Self::MarkSurfaces,
        Self::Edges,
        Self::SurfaceEdges,
        Self::Models,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    TruncatedHeader {
        actual: usize,
    },
    UnsupportedVersion(i32),
    NegativeLumpRange {
        lump: LumpKind,
        offset: i32,
        length: i32,
    },
    LumpOutsideFile {
        lump: LumpKind,
        offset: usize,
        length: usize,
        file_length: usize,
    },
    MisalignedLump {
        lump: LumpKind,
        length: usize,
        stride: usize,
    },
    EntityTextTooLarge {
        length: usize,
        maximum: usize,
    },
    UnterminatedEntityText,
    NonFiniteCoordinate {
        lump: LumpKind,
        element: usize,
        field: usize,
        original_bits: u32,
    },
    RecordLimitExceeded {
        lump: LumpKind,
        count: usize,
        maximum: usize,
    },
    InvalidReference {
        source: SourceLocation,
        field: &'static str,
        value: i64,
        target: LumpKind,
        target_count: usize,
    },
    InvalidRecordRange {
        source: SourceLocation,
        field: &'static str,
        first: i64,
        count: i64,
        available: usize,
    },
    TopologyCycle {
        lump: LumpKind,
        element: usize,
    },
    InvalidValue {
        source: SourceLocation,
        field: &'static str,
        value: i64,
    },
    InvalidVisibility {
        leaf: usize,
        offset: usize,
    },
    BspxDirectoryTooLarge {
        count: usize,
        maximum: usize,
    },
    TruncatedBspxDirectory,
    InvalidBspxName {
        element: usize,
    },
    DuplicateBspxName(String),
    BspxLumpOutsideFile {
        element: usize,
        offset: usize,
        length: usize,
        file_length: usize,
    },
    InvalidTextureCount(i32),
    TruncatedTextureDirectory,
    InvalidTextureOffset {
        element: usize,
        offset: i32,
    },
    TruncatedTextureHeader {
        element: usize,
    },
    InvalidTextureDimensions {
        element: usize,
        width: u32,
        height: u32,
    },
    MixedTextureStorage {
        element: usize,
    },
    TextureMipOutsideLump {
        element: usize,
        level: usize,
        offset: usize,
        length: usize,
        lump_length: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { .. } => formatter.write_str("BSP30 header is truncated"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported BSP version {version}")
            }
            Self::NegativeLumpRange { lump, .. } => {
                write!(formatter, "{lump:?} lump has a negative range")
            }
            Self::LumpOutsideFile { lump, .. } => {
                write!(formatter, "{lump:?} lump extends outside the file")
            }
            Self::MisalignedLump { lump, .. } => {
                write!(formatter, "{lump:?} lump is not a whole number of records")
            }
            Self::EntityTextTooLarge { .. } => formatter.write_str("entity text exceeds its limit"),
            Self::UnterminatedEntityText => {
                formatter.write_str("entity text is not NUL-terminated")
            }
            Self::NonFiniteCoordinate { lump, element, .. } => {
                write!(
                    formatter,
                    "{lump:?} element {element} contains a non-finite coordinate"
                )
            }
            Self::RecordLimitExceeded { lump, .. } => {
                write!(formatter, "{lump:?} lump exceeds its record limit")
            }
            Self::InvalidReference { source, field, .. } => {
                write!(
                    formatter,
                    "{:?} element {} has an invalid {field}",
                    source.lump, source.element
                )
            }
            Self::InvalidRecordRange { source, field, .. } => {
                write!(
                    formatter,
                    "{:?} element {} has an invalid {field} range",
                    source.lump, source.element
                )
            }
            Self::TopologyCycle { lump, element } => {
                write!(
                    formatter,
                    "{lump:?} topology contains a cycle at element {element}"
                )
            }
            Self::InvalidValue { source, field, .. } => {
                write!(
                    formatter,
                    "{:?} element {} has an invalid {field}",
                    source.lump, source.element
                )
            }
            Self::InvalidVisibility { leaf, .. } => {
                write!(
                    formatter,
                    "leaf {leaf} has invalid compressed visibility data"
                )
            }
            Self::BspxDirectoryTooLarge { .. } => {
                formatter.write_str("BSPX directory exceeds its entry limit")
            }
            Self::TruncatedBspxDirectory => formatter.write_str("BSPX directory is truncated"),
            Self::InvalidBspxName { element } => {
                write!(formatter, "BSPX lump {element} has an invalid name")
            }
            Self::DuplicateBspxName(name) => write!(formatter, "BSPX repeats lump {name}"),
            Self::BspxLumpOutsideFile { element, .. } => {
                write!(formatter, "BSPX lump {element} extends outside the file")
            }
            Self::InvalidTextureCount(count) => write!(formatter, "invalid texture count {count}"),
            Self::TruncatedTextureDirectory => {
                formatter.write_str("texture directory is truncated")
            }
            Self::InvalidTextureOffset { element, .. } => {
                write!(formatter, "texture {element} has an invalid offset")
            }
            Self::TruncatedTextureHeader { element } => {
                write!(formatter, "texture {element} header is truncated")
            }
            Self::InvalidTextureDimensions { element, .. } => {
                write!(formatter, "texture {element} has invalid dimensions")
            }
            Self::MixedTextureStorage { element } => {
                write!(
                    formatter,
                    "texture {element} mixes external and embedded mip levels"
                )
            }
            Self::TextureMipOutsideLump { element, level, .. } => {
                write!(
                    formatter,
                    "texture {element} mip level {level} extends outside the lump"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug)]
pub struct BspDocument<'a> {
    bytes: &'a [u8],
    lumps: [Range<usize>; LUMP_COUNT],
    layout: LumpLayout,
    bspx_lumps: Vec<BspxLump<'a>>,
    textures: Vec<MipTexture<'a>>,
}

impl<'a> BspDocument<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ParseError> {
        if bytes.len() < HEADER_SIZE {
            return Err(ParseError::TruncatedHeader {
                actual: bytes.len(),
            });
        }
        let version = read_i32(bytes, 0);
        if version != BSP_VERSION {
            return Err(ParseError::UnsupportedVersion(version));
        }

        let mut lumps: [Range<usize>; LUMP_COUNT] = std::array::from_fn(|_| 0..0);
        for lump in LumpKind::ALL {
            let entry = 4 + lump.index() * 8;
            let file_offset = read_i32(bytes, entry);
            let file_length = read_i32(bytes, entry + 4);
            if file_offset < 0 || file_length < 0 {
                return Err(ParseError::NegativeLumpRange {
                    lump,
                    offset: file_offset,
                    length: file_length,
                });
            }
            let offset = file_offset as usize;
            let length = file_length as usize;
            let end = offset
                .checked_add(length)
                .filter(|end| *end <= bytes.len())
                .ok_or(ParseError::LumpOutsideFile {
                    lump,
                    offset,
                    length,
                    file_length: bytes.len(),
                })?;
            lumps[lump.index()] = offset..end;
        }

        let layout = if lumps[LumpKind::Planes.index()].len() % PLANE_SIZE != 0
            && lumps[LumpKind::Entities.index()].len() % PLANE_SIZE == 0
        {
            lumps.swap(LumpKind::Entities.index(), LumpKind::Planes.index());
            LumpLayout::BlueShift
        } else {
            LumpLayout::Standard
        };
        for lump in LumpKind::ALL {
            if let Some(stride) = record_stride(lump) {
                let length = lumps[lump.index()].len();
                if length % stride != 0 {
                    return Err(ParseError::MisalignedLump {
                        lump,
                        length,
                        stride,
                    });
                }
                let count = length / stride;
                if count > MAX_RECORDS_PER_LUMP {
                    return Err(ParseError::RecordLimitExceeded {
                        lump,
                        count,
                        maximum: MAX_RECORDS_PER_LUMP,
                    });
                }
            }
        }
        let entities = &bytes[lumps[LumpKind::Entities.index()].clone()];
        if entities.len() > MAX_ENTITY_TEXT_SIZE {
            return Err(ParseError::EntityTextTooLarge {
                length: entities.len(),
                maximum: MAX_ENTITY_TEXT_SIZE,
            });
        }
        if !entities.is_empty() && entities.last() != Some(&0) {
            return Err(ParseError::UnterminatedEntityText);
        }
        validate_finite_records(bytes, &lumps, LumpKind::Planes, PLANE_SIZE, &[0, 4, 8, 12])?;
        validate_finite_records(bytes, &lumps, LumpKind::Vertices, 12, &[0, 4, 8])?;
        validate_finite_records(
            bytes,
            &lumps,
            LumpKind::TextureInfo,
            40,
            &[0, 4, 8, 12, 16, 20, 24, 28],
        )?;
        validate_finite_records(
            bytes,
            &lumps,
            LumpKind::Models,
            64,
            &[0, 4, 8, 12, 16, 20, 24, 28, 32],
        )?;
        let textures = texture::parse(bytes, &lumps)?;
        validation::validate_topology(bytes, &lumps, textures.len())?;
        let bspx_lumps = bspx::parse(bytes, &lumps)?;

        Ok(Self {
            bytes,
            lumps,
            layout,
            bspx_lumps,
            textures,
        })
    }

    pub fn lump(&self, kind: LumpKind) -> &'a [u8] {
        &self.bytes[self.lumps[kind.index()].clone()]
    }

    pub fn lump_range(&self, kind: LumpKind) -> Range<usize> {
        self.lumps[kind.index()].clone()
    }

    pub fn layout(&self) -> LumpLayout {
        self.layout
    }

    pub fn entity_bytes(&self) -> &'a [u8] {
        self.lump(LumpKind::Entities)
            .strip_suffix(&[0])
            .unwrap_or_default()
    }

    pub fn planes(&self) -> impl ExactSizeIterator<Item = Plane> + '_ {
        self.lump(LumpKind::Planes)
            .chunks_exact(PLANE_SIZE)
            .enumerate()
            .map(|(element, bytes)| Plane {
                source: SourceLocation {
                    lump: LumpKind::Planes,
                    element,
                },
                normal: [
                    source_float(bytes, 0),
                    source_float(bytes, 4),
                    source_float(bytes, 8),
                ],
                distance: source_float(bytes, 12),
                plane_type: read_i32(bytes, 16),
            })
    }

    pub fn vertices(&self) -> impl ExactSizeIterator<Item = Vertex> + '_ {
        self.lump(LumpKind::Vertices)
            .chunks_exact(12)
            .enumerate()
            .map(|(element, bytes)| Vertex {
                source: SourceLocation {
                    lump: LumpKind::Vertices,
                    element,
                },
                point: [
                    source_float(bytes, 0),
                    source_float(bytes, 4),
                    source_float(bytes, 8),
                ],
            })
    }

    pub fn edges(&self) -> impl ExactSizeIterator<Item = Edge> + '_ {
        self.lump(LumpKind::Edges)
            .chunks_exact(4)
            .enumerate()
            .map(|(element, bytes)| Edge {
                source: SourceLocation {
                    lump: LumpKind::Edges,
                    element,
                },
                vertices: [read_u16(bytes, 0), read_u16(bytes, 2)],
            })
    }

    pub fn surface_edges(&self) -> impl ExactSizeIterator<Item = SurfaceEdge> + '_ {
        self.lump(LumpKind::SurfaceEdges)
            .chunks_exact(4)
            .enumerate()
            .map(|(element, bytes)| SurfaceEdge {
                source: SourceLocation {
                    lump: LumpKind::SurfaceEdges,
                    element,
                },
                edge: read_i32(bytes, 0),
            })
    }

    pub fn mark_surfaces(&self) -> impl ExactSizeIterator<Item = MarkSurface> + '_ {
        self.lump(LumpKind::MarkSurfaces)
            .chunks_exact(2)
            .enumerate()
            .map(|(element, bytes)| MarkSurface {
                source: SourceLocation {
                    lump: LumpKind::MarkSurfaces,
                    element,
                },
                face: read_u16(bytes, 0),
            })
    }

    pub fn texture_info(&self) -> impl ExactSizeIterator<Item = TextureInfo> + '_ {
        self.lump(LumpKind::TextureInfo)
            .chunks_exact(40)
            .enumerate()
            .map(|(element, bytes)| TextureInfo {
                source: SourceLocation {
                    lump: LumpKind::TextureInfo,
                    element,
                },
                s_axis: [
                    source_float(bytes, 0),
                    source_float(bytes, 4),
                    source_float(bytes, 8),
                    source_float(bytes, 12),
                ],
                t_axis: [
                    source_float(bytes, 16),
                    source_float(bytes, 20),
                    source_float(bytes, 24),
                    source_float(bytes, 28),
                ],
                mip_texture: read_i32(bytes, 32),
                flags: read_i32(bytes, 36),
            })
    }

    pub fn faces(&self) -> impl ExactSizeIterator<Item = Face> + '_ {
        self.lump(LumpKind::Faces)
            .chunks_exact(20)
            .enumerate()
            .map(|(element, bytes)| Face {
                source: SourceLocation {
                    lump: LumpKind::Faces,
                    element,
                },
                plane: read_u16(bytes, 0),
                plane_side: read_i16(bytes, 2),
                first_surface_edge: read_i32(bytes, 4),
                surface_edge_count: read_i16(bytes, 8),
                texture_info: read_i16(bytes, 10),
                styles: bytes[12..16].try_into().expect("face was sized"),
                lighting_offset: read_i32(bytes, 16),
            })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = Node> + '_ {
        self.lump(LumpKind::Nodes)
            .chunks_exact(24)
            .enumerate()
            .map(|(element, bytes)| Node {
                source: SourceLocation {
                    lump: LumpKind::Nodes,
                    element,
                },
                plane: read_i32(bytes, 0),
                children: [read_i16(bytes, 4), read_i16(bytes, 6)],
                minimum: [read_i16(bytes, 8), read_i16(bytes, 10), read_i16(bytes, 12)],
                maximum: [
                    read_i16(bytes, 14),
                    read_i16(bytes, 16),
                    read_i16(bytes, 18),
                ],
                first_face: read_u16(bytes, 20),
                face_count: read_u16(bytes, 22),
            })
    }

    pub fn clip_nodes(&self) -> impl ExactSizeIterator<Item = ClipNode> + '_ {
        self.lump(LumpKind::ClipNodes)
            .chunks_exact(8)
            .enumerate()
            .map(|(element, bytes)| ClipNode {
                source: SourceLocation {
                    lump: LumpKind::ClipNodes,
                    element,
                },
                plane: read_i32(bytes, 0),
                children: [read_i16(bytes, 4), read_i16(bytes, 6)],
            })
    }

    /// The leaves, an offset outside the visibility data read as none
    pub fn leaves(&self) -> impl ExactSizeIterator<Item = Leaf> + '_ {
        let visibility_length = self.lump(LumpKind::Visibility).len();
        self.lump(LumpKind::Leaves)
            .chunks_exact(28)
            .enumerate()
            .map(move |(element, bytes)| Leaf {
                source: SourceLocation {
                    lump: LumpKind::Leaves,
                    element,
                },
                contents: read_i32(bytes, 0),
                visibility_offset: validation::visibility_offset(
                    read_i32(bytes, 4),
                    visibility_length,
                ),
                minimum: [read_i16(bytes, 8), read_i16(bytes, 10), read_i16(bytes, 12)],
                maximum: [
                    read_i16(bytes, 14),
                    read_i16(bytes, 16),
                    read_i16(bytes, 18),
                ],
                first_mark_surface: read_u16(bytes, 20),
                mark_surface_count: read_u16(bytes, 22),
                ambient_levels: bytes[24..28].try_into().expect("leaf was sized"),
            })
    }

    pub fn models(&self) -> impl ExactSizeIterator<Item = Model> + '_ {
        self.lump(LumpKind::Models)
            .chunks_exact(64)
            .enumerate()
            .map(|(element, bytes)| Model {
                source: SourceLocation {
                    lump: LumpKind::Models,
                    element,
                },
                minimum: [
                    source_float(bytes, 0),
                    source_float(bytes, 4),
                    source_float(bytes, 8),
                ],
                maximum: [
                    source_float(bytes, 12),
                    source_float(bytes, 16),
                    source_float(bytes, 20),
                ],
                origin: [
                    source_float(bytes, 24),
                    source_float(bytes, 28),
                    source_float(bytes, 32),
                ],
                head_nodes: [
                    read_i32(bytes, 36),
                    read_i32(bytes, 40),
                    read_i32(bytes, 44),
                    read_i32(bytes, 48),
                ],
                visibility_leaf_count: read_i32(bytes, 52),
                first_face: read_i32(bytes, 56),
                face_count: read_i32(bytes, 60),
            })
    }

    pub fn bspx_lumps(&self) -> &[BspxLump<'a>] {
        &self.bspx_lumps
    }

    pub fn bspx_lump(&self, kind: BspxKind) -> Option<&BspxLump<'a>> {
        self.bspx_lumps.iter().find(|lump| lump.kind == kind)
    }

    pub fn textures(&self) -> &[MipTexture<'a>] {
        &self.textures
    }
}

const PLANE_SIZE: usize = 20;

const fn record_stride(lump: LumpKind) -> Option<usize> {
    match lump {
        LumpKind::Planes => Some(PLANE_SIZE),
        LumpKind::Vertices => Some(12),
        LumpKind::Nodes => Some(24),
        LumpKind::TextureInfo => Some(40),
        LumpKind::Faces => Some(20),
        LumpKind::ClipNodes => Some(8),
        LumpKind::Leaves => Some(28),
        LumpKind::MarkSurfaces => Some(2),
        LumpKind::Edges => Some(4),
        LumpKind::SurfaceEdges => Some(4),
        LumpKind::Models => Some(64),
        LumpKind::Lighting => Some(3),
        LumpKind::Entities | LumpKind::Textures | LumpKind::Visibility => None,
    }
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("header was sized"),
    )
}

fn source_float(bytes: &[u8], offset: usize) -> SourceFloat {
    let original_bits = u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("record was sized"),
    );
    SourceFloat {
        value: f32::from_bits(original_bits),
        original_bits,
    }
}

fn validate_finite_records(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    lump: LumpKind,
    stride: usize,
    fields: &[usize],
) -> Result<(), ParseError> {
    for (element, record) in bytes[lumps[lump.index()].clone()]
        .chunks_exact(stride)
        .enumerate()
    {
        for (field, offset) in fields.iter().copied().enumerate() {
            let value = source_float(record, offset);
            if !value.value.is_finite() {
                return Err(ParseError::NonFiniteCoordinate {
                    lump,
                    element,
                    field,
                    original_bits: value.original_bits,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with_lump(kind: LumpKind, data: &[u8]) -> Vec<u8> {
        bsp_with_lumps(&[(kind, data)])
    }

    fn bsp_with_lumps(lumps: &[(LumpKind, &[u8])]) -> Vec<u8> {
        let mut bytes = vec![0_u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&BSP_VERSION.to_le_bytes());
        for &(kind, data) in lumps {
            let entry = 4 + kind.index() * 8;
            let offset = bytes.len() as i32;
            bytes[entry..entry + 4].copy_from_slice(&offset.to_le_bytes());
            bytes[entry + 4..entry + 8].copy_from_slice(&(data.len() as i32).to_le_bytes());
            bytes.extend_from_slice(data);
        }
        bytes
    }

    fn append_bspx(bytes: &mut Vec<u8>, entries: &[(&str, &[u8])]) {
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        let directory = bytes.len();
        let data_start = directory + 8 + entries.len() * 32;
        bytes.extend_from_slice(b"BSPX");
        bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        let mut data_offset = data_start;
        for &(name, data) in entries {
            let mut encoded_name = [0_u8; 24];
            encoded_name[..name.len()].copy_from_slice(name.as_bytes());
            bytes.extend_from_slice(&encoded_name);
            bytes.extend_from_slice(&(data_offset as u32).to_le_bytes());
            bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
            data_offset += data.len();
        }
        for &(_, data) in entries {
            bytes.extend_from_slice(data);
        }
    }

    fn texture_lump(embedded: bool) -> Vec<u8> {
        let mut lump = Vec::new();
        lump.extend_from_slice(&1_i32.to_le_bytes());
        lump.extend_from_slice(&8_i32.to_le_bytes());
        let mut header = [0_u8; 40];
        header[..4].copy_from_slice(b"TEST");
        header[16..20].copy_from_slice(&16_u32.to_le_bytes());
        header[20..24].copy_from_slice(&16_u32.to_le_bytes());
        if embedded {
            for (level, offset) in [40_u32, 296, 360, 376].into_iter().enumerate() {
                let field = 24 + level * 4;
                header[field..field + 4].copy_from_slice(&offset.to_le_bytes());
            }
        }
        lump.extend_from_slice(&header);
        if embedded {
            lump.extend_from_slice(&vec![7_u8; 340]);
            lump.extend_from_slice(&256_u16.to_le_bytes());
            lump.extend_from_slice(&vec![11_u8; 256 * 3]);
        }
        lump
    }

    #[test]
    fn parses_checked_lump_ranges() {
        let bytes = header_with_lump(LumpKind::Entities, b"{\n}\0");
        let document = BspDocument::parse(&bytes).unwrap();

        assert_eq!(document.lump(LumpKind::Entities), b"{\n}\0");
        assert_eq!(
            document.lump_range(LumpKind::Entities),
            HEADER_SIZE..bytes.len()
        );
        assert!(document.lump(LumpKind::Planes).is_empty());
    }

    #[test]
    fn rejects_truncated_header_and_wrong_version() {
        assert_eq!(
            BspDocument::parse(&[0; HEADER_SIZE - 1]).unwrap_err(),
            ParseError::TruncatedHeader {
                actual: HEADER_SIZE - 1
            }
        );

        let mut bytes = header_with_lump(LumpKind::Entities, &[]);
        bytes[0..4].copy_from_slice(&29_i32.to_le_bytes());
        assert_eq!(
            BspDocument::parse(&bytes).unwrap_err(),
            ParseError::UnsupportedVersion(29)
        );
    }

    #[test]
    fn rejects_negative_and_out_of_file_ranges() {
        let mut negative = header_with_lump(LumpKind::Planes, &[]);
        let planes = 4 + LumpKind::Planes.index() * 8;
        negative[planes..planes + 4].copy_from_slice(&(-1_i32).to_le_bytes());
        assert!(matches!(
            BspDocument::parse(&negative),
            Err(ParseError::NegativeLumpRange {
                lump: LumpKind::Planes,
                ..
            })
        ));

        let mut outside = header_with_lump(LumpKind::Vertices, &[]);
        let vertices = 4 + LumpKind::Vertices.index() * 8;
        outside[vertices..vertices + 4].copy_from_slice(&(HEADER_SIZE as i32).to_le_bytes());
        outside[vertices + 4..vertices + 8].copy_from_slice(&1_i32.to_le_bytes());
        assert!(matches!(
            BspDocument::parse(&outside),
            Err(ParseError::LumpOutsideFile {
                lump: LumpKind::Vertices,
                ..
            })
        ));
    }

    #[test]
    fn normalizes_blue_shift_entity_plane_permutation() {
        let entity_text = b"{\n\"classname\" \"worldspawn\"\n}\0";
        let plane = [0_u8; PLANE_SIZE];
        let mut bytes = vec![0_u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&BSP_VERSION.to_le_bytes());
        let entities_entry = 4 + LumpKind::Entities.index() * 8;
        let planes_entry = 4 + LumpKind::Planes.index() * 8;
        bytes[entities_entry..entities_entry + 4]
            .copy_from_slice(&(HEADER_SIZE as i32).to_le_bytes());
        bytes[entities_entry + 4..entities_entry + 8]
            .copy_from_slice(&(plane.len() as i32).to_le_bytes());
        bytes[planes_entry..planes_entry + 4]
            .copy_from_slice(&((HEADER_SIZE + plane.len()) as i32).to_le_bytes());
        bytes[planes_entry + 4..planes_entry + 8]
            .copy_from_slice(&(entity_text.len() as i32).to_le_bytes());
        bytes.extend_from_slice(&plane);
        bytes.extend_from_slice(entity_text);

        let document = BspDocument::parse(&bytes).unwrap();
        assert_eq!(document.layout(), LumpLayout::BlueShift);
        assert_eq!(document.lump(LumpKind::Entities), entity_text);
        assert_eq!(document.lump(LumpKind::Planes), plane);
    }

    #[test]
    fn rejects_partial_fixed_size_records() {
        let bytes = header_with_lump(LumpKind::Faces, &[0; 19]);
        assert_eq!(
            BspDocument::parse(&bytes).unwrap_err(),
            ParseError::MisalignedLump {
                lump: LumpKind::Faces,
                length: 19,
                stride: 20,
            }
        );
    }

    #[test]
    fn vertices_retain_source_bits_and_location() {
        let coordinates = [1.25_f32, -2.5, 3.75];
        let mut record = Vec::new();
        for coordinate in coordinates {
            record.extend_from_slice(&coordinate.to_bits().to_le_bytes());
        }
        let bytes = header_with_lump(LumpKind::Vertices, &record);
        let vertex = BspDocument::parse(&bytes)
            .unwrap()
            .vertices()
            .next()
            .unwrap();

        assert_eq!(
            vertex.source,
            SourceLocation {
                lump: LumpKind::Vertices,
                element: 0,
            }
        );
        assert_eq!(vertex.point[1].value, -2.5);
        assert_eq!(vertex.point[1].original_bits, (-2.5_f32).to_bits());
    }

    #[test]
    fn accepts_extended_world_bound_coordinates() {
        let mut vertex = Vec::new();
        for coordinate in [262_144.0_f32, -262_144.0, 0.0] {
            vertex.extend_from_slice(&coordinate.to_le_bytes());
        }
        let mut plane = [0_u8; PLANE_SIZE];
        plane[0..4].copy_from_slice(&1.0_f32.to_le_bytes());
        plane[12..16].copy_from_slice(&262_144.0_f32.to_le_bytes());
        let bytes = bsp_with_lumps(&[(LumpKind::Planes, &plane), (LumpKind::Vertices, &vertex)]);
        let document = BspDocument::parse(&bytes).unwrap();
        let point = document.vertices().next().unwrap().point;

        assert_eq!(point[0].value, 262_144.0);
        assert_eq!(point[1].value, -262_144.0);
        assert_eq!(document.planes().next().unwrap().distance.value, 262_144.0);
    }

    #[test]
    fn rejects_non_finite_geometry() {
        let mut plane = [0_u8; PLANE_SIZE];
        plane[4..8].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
        let bytes = header_with_lump(LumpKind::Planes, &plane);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::NonFiniteCoordinate {
                lump: LumpKind::Planes,
                element: 0,
                field: 1,
                ..
            })
        ));
    }

    #[test]
    fn rejects_invalid_plane_type() {
        let mut plane = [0_u8; PLANE_SIZE];
        plane[16..20].copy_from_slice(&6_i32.to_le_bytes());
        let bytes = header_with_lump(LumpKind::Planes, &plane);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::InvalidValue {
                source: SourceLocation {
                    lump: LumpKind::Planes,
                    element: 0,
                },
                field: "plane type",
                value: 6,
            })
        ));
    }

    #[test]
    fn rejects_unterminated_entity_text() {
        let bytes = header_with_lump(LumpKind::Entities, b"{\n}\n");
        assert_eq!(
            BspDocument::parse(&bytes).unwrap_err(),
            ParseError::UnterminatedEntityText
        );
    }

    #[test]
    fn rejects_edge_with_missing_vertex() {
        let edge = [0_u8; 4];
        let bytes = header_with_lump(LumpKind::Edges, &edge);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::InvalidReference {
                source: SourceLocation {
                    lump: LumpKind::Edges,
                    element: 0,
                },
                target: LumpKind::Vertices,
                target_count: 0,
                ..
            })
        ));
    }

    #[test]
    fn rejects_unrepresentable_surface_edge_reference() {
        let surface_edge = i32::MIN.to_le_bytes();
        let bytes = header_with_lump(LumpKind::SurfaceEdges, &surface_edge);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::InvalidReference {
                source: SourceLocation {
                    lump: LumpKind::SurfaceEdges,
                    element: 0,
                },
                field: "edge",
                value: i64::MAX,
                ..
            })
        ));
    }

    #[test]
    fn rejects_cyclic_node_topology() {
        let plane = [0_u8; PLANE_SIZE];
        let mut leaf = [0_u8; 28];
        leaf[4..8].copy_from_slice(&(-1_i32).to_le_bytes());
        let mut node = [0_u8; 24];
        node[4..6].copy_from_slice(&0_i16.to_le_bytes());
        node[6..8].copy_from_slice(&(-1_i16).to_le_bytes());
        let bytes = bsp_with_lumps(&[
            (LumpKind::Planes, &plane),
            (LumpKind::Nodes, &node),
            (LumpKind::Leaves, &leaf),
        ]);

        assert_eq!(
            BspDocument::parse(&bytes).unwrap_err(),
            ParseError::TopologyCycle {
                lump: LumpKind::Nodes,
                element: 0,
            }
        );
    }

    #[test]
    fn rejects_zero_length_visibility_run() {
        let mut leaf = [0_u8; 28];
        leaf[4..8].copy_from_slice(&0_i32.to_le_bytes());
        let mut leaves = Vec::from(leaf);
        leaves.extend_from_slice(&leaf);
        let bytes = bsp_with_lumps(&[(LumpKind::Visibility, &[0, 0]), (LumpKind::Leaves, &leaves)]);

        assert_eq!(
            BspDocument::parse(&bytes).unwrap_err(),
            ParseError::InvalidVisibility { leaf: 0, offset: 0 }
        );
    }

    #[test]
    fn rejects_invalid_face_side() {
        let plane = [0_u8; PLANE_SIZE];
        let mut face = [0_u8; 20];
        face[2..4].copy_from_slice(&2_i16.to_le_bytes());
        let texture_info = [0_u8; 40];
        let texture = texture_lump(false);
        let bytes = bsp_with_lumps(&[
            (LumpKind::Planes, &plane),
            (LumpKind::Textures, &texture),
            (LumpKind::TextureInfo, &texture_info),
            (LumpKind::Faces, &face),
        ]);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::InvalidValue {
                field: "plane side",
                value: 2,
                ..
            })
        ));
    }

    #[test]
    fn rejects_partial_rgb_lighting_sample() {
        let bytes = header_with_lump(LumpKind::Lighting, &[1, 2]);

        assert_eq!(
            BspDocument::parse(&bytes).unwrap_err(),
            ParseError::MisalignedLump {
                lump: LumpKind::Lighting,
                length: 2,
                stride: 3,
            }
        );
    }

    #[test]
    fn rejects_face_lighting_offset_outside_lump() {
        let plane = [0_u8; PLANE_SIZE];
        let mut face = [0_u8; 20];
        face[16..20].copy_from_slice(&3_i32.to_le_bytes());
        let texture_info = [0_u8; 40];
        let texture = texture_lump(false);
        let bytes = bsp_with_lumps(&[
            (LumpKind::Planes, &plane),
            (LumpKind::Textures, &texture),
            (LumpKind::TextureInfo, &texture_info),
            (LumpKind::Faces, &face),
            (LumpKind::Lighting, &[1, 2, 3]),
        ]);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::InvalidRecordRange {
                field: "lighting",
                first: 3,
                available: 3,
                ..
            })
        ));
    }

    #[test]
    fn rejects_model_visibility_leaf_count_outside_lump() {
        let mut model = [0_u8; 64];
        model[52..56].copy_from_slice(&1_i32.to_le_bytes());
        let bytes = header_with_lump(LumpKind::Models, &model);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::InvalidRecordRange {
                source: SourceLocation {
                    lump: LumpKind::Models,
                    element: 0,
                },
                field: "visibility leaf count",
                count: 1,
                available: 0,
                ..
            })
        ));
    }

    #[test]
    fn preserves_recognized_bspx_and_skips_unknown_entries() {
        let mut bytes = header_with_lump(LumpKind::Entities, b"{}\0");
        append_bspx(
            &mut bytes,
            &[("RGBLIGHTING", &[1, 2, 3]), ("FUTURE_DATA", &[4, 5])],
        );

        let document = BspDocument::parse(&bytes).unwrap();
        assert_eq!(document.bspx_lumps().len(), 1);
        assert_eq!(
            document.bspx_lump(BspxKind::RgbLighting).unwrap().data,
            [1, 2, 3]
        );
    }

    #[test]
    fn rejects_bspx_lump_outside_file() {
        let mut bytes = header_with_lump(LumpKind::Entities, b"{}\0");
        append_bspx(&mut bytes, &[("LMSHIFT", &[4])]);
        let entry_offset_field = bytes
            .windows(4)
            .position(|window| window == b"BSPX")
            .unwrap()
            + 8
            + 24;
        bytes[entry_offset_field..entry_offset_field + 4].copy_from_slice(&u32::MAX.to_le_bytes());

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::BspxLumpOutsideFile { element: 0, .. })
        ));
    }

    #[test]
    fn parses_external_and_embedded_textures() {
        let external = texture_lump(false);
        let bytes = header_with_lump(LumpKind::Textures, &external);
        assert!(matches!(
            BspDocument::parse(&bytes).unwrap().textures(),
            [MipTexture::External {
                name: b"TEST",
                width: 16,
                height: 16,
                ..
            }]
        ));

        let embedded = texture_lump(true);
        let bytes = header_with_lump(LumpKind::Textures, &embedded);
        let document = BspDocument::parse(&bytes).unwrap();
        let MipTexture::Embedded {
            mip_levels,
            palette,
            ..
        } = &document.textures()[0]
        else {
            panic!("expected embedded texture")
        };
        assert_eq!(
            mip_levels
                .iter()
                .map(|level| level.len())
                .collect::<Vec<_>>(),
            [256, 64, 16, 4]
        );
        assert_eq!(palette.unwrap().len(), 256 * 3);
    }

    #[test]
    fn rejects_truncated_embedded_texture() {
        let mut texture = texture_lump(true);
        texture.truncate(8 + 40 + 339);
        let bytes = header_with_lump(LumpKind::Textures, &texture);

        assert!(matches!(
            BspDocument::parse(&bytes),
            Err(ParseError::TextureMipOutsideLump {
                element: 0,
                level: 3,
                ..
            })
        ));
    }
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("record was sized"),
    )
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("record was sized"),
    )
}
