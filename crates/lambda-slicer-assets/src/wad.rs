//! WAD3 archives, which hold map textures, decals, interface pictures and
//! fonts
//!
//! A 12-byte header (`WAD3`, entry count, directory offset) and a directory
//! of 32-byte entries (offset, size on disk, size, type, compression, a
//! 16-byte name). The whole directory is checked against the archive's
//! length before any entry is used.
//!
//! Mip texture decoding is adapted from `parse_wad3` in Xenoxide
//! (`crates/xenoxide-assets/src/lib.rs` at `33eac809`, see
//! docs/dependency-audit.md). The change is that the directory is read once
//! and kept with every entry's type, so decals and other entries can be
//! looked up by name, and a texture is decoded only when asked for

use crate::{AssetError, Limits};

const HEADER: usize = 12;
const ENTRY: usize = 32;
const MIP_HEADER: usize = 40;

/// Entry types. Decals and map textures share the mip texture layout
pub const DECAL: u8 = 0x40;
pub const PICTURE: u8 = 0x42;
pub const MIP_TEXTURE: u8 = 0x43;
pub const FONT: u8 = 0x46;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WadEntry {
    pub name: String,
    pub kind: u8,
    pub compression: u8,
    pub offset: usize,
    pub size: usize,
}

/// A mip texture's full-size level and its palette
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MipTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// One palette index per pixel, rows from the top
    pub indices: Vec<u8>,
    /// RGB triples
    pub palette: Vec<u8>,
}

#[derive(Debug)]
pub struct Wad {
    bytes: Vec<u8>,
    entries: Vec<WadEntry>,
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn usize_at(bytes: &[u8], at: usize, what: &str) -> Result<usize, AssetError> {
    let v = i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    usize::try_from(v).map_err(|_| AssetError::Wad(format!("negative {what}")))
}

/// A fixed-size name field up to its first NUL
fn name_of(field: &[u8]) -> String {
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

impl Wad {
    pub fn parse(bytes: Vec<u8>, limits: Limits) -> Result<Self, AssetError> {
        let bad = |what: &str| AssetError::Wad(what.to_string());
        if bytes.len() < HEADER {
            return Err(bad("shorter than its header"));
        }
        if &bytes[..4] != b"WAD3" {
            return Err(bad("no WAD3 signature"));
        }
        let count = usize_at(&bytes, 4, "entry count")?;
        if count > limits.max_entries {
            return Err(bad("more entries than the limit"));
        }
        let directory = usize_at(&bytes, 8, "directory offset")?;
        // The complete directory is checked before any entry offset is trusted
        count
            .checked_mul(ENTRY)
            .and_then(|length| directory.checked_add(length))
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| bad("directory runs past the end"))?;

        let entries = (0..count)
            .map(|i| {
                let e = &bytes[directory + i * ENTRY..directory + (i + 1) * ENTRY];
                let offset = usize_at(e, 0, "entry offset")?;
                let size = usize_at(e, 4, "entry size")?;
                offset
                    .checked_add(size)
                    .filter(|end| *end <= bytes.len())
                    .ok_or_else(|| AssetError::Wad(format!("entry {i} runs past the end")))?;
                Ok(WadEntry {
                    name: name_of(&e[16..32]),
                    kind: e[12],
                    compression: e[13],
                    offset,
                    size,
                })
            })
            .collect::<Result<Vec<_>, AssetError>>()?;

        Ok(Self { bytes, entries })
    }

    pub fn entries(&self) -> &[WadEntry] {
        &self.entries
    }

    /// The first entry with this name, ignoring ASCII case as the game does
    pub fn find(&self, name: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// A map texture or decal entry decoded
    pub fn mip_texture(&self, index: usize) -> Result<MipTexture, AssetError> {
        let entry = &self.entries[index];
        let bad = |what: &str| AssetError::Wad(format!("entry {index} ({}) {what}", entry.name));
        if !matches!(entry.kind, MIP_TEXTURE | DECAL) {
            return Err(bad("is not a texture"));
        }
        if entry.compression != 0 {
            return Err(bad("uses an unsupported compression"));
        }
        let bytes = &self.bytes;
        let (start, end) = (entry.offset, entry.offset + entry.size);
        if start + MIP_HEADER > end {
            return Err(bad("is shorter than a texture header"));
        }
        let header = &bytes[start..start + MIP_HEADER];
        let width = u32_at(header, 16);
        let height = u32_at(header, 20);
        let pixels = (width as usize)
            .checked_mul(height as usize)
            .filter(|n| *n > 0)
            .ok_or_else(|| bad("has no pixels"))?;
        let within = |offset: u32, length: usize| {
            (offset as usize)
                .checked_add(start)
                .and_then(|at| at.checked_add(length).map(|e| (at, e)))
                .filter(|(_, e)| *e <= end)
        };
        let (first, first_end) = within(u32_at(header, 24), pixels)
            .ok_or_else(|| bad("has a level outside the entry"))?;
        // The palette follows the smallest of the four levels
        let smallest = ((width >> 3).max(1) * (height >> 3).max(1)) as usize;
        let (_, after) = within(u32_at(header, 36), smallest)
            .ok_or_else(|| bad("has a level outside the entry"))?;
        if after + 2 > end {
            return Err(bad("has no palette"));
        }
        let colours = usize::from(u16::from_le_bytes([bytes[after], bytes[after + 1]]));
        let palette_end = after + 2 + colours * 3;
        if palette_end > end {
            return Err(bad("has a palette outside the entry"));
        }

        Ok(MipTexture {
            name: name_of(&header[..16]),
            width,
            height,
            indices: bytes[first..first_end].to_vec(),
            palette: bytes[after + 2..palette_end].to_vec(),
        })
    }
}

/// A WAD3 archive's bytes holding one mip texture per `(name, width,
/// height, kind)`, every pixel set to its own index modulo 256, for tests
#[cfg(test)]
pub(crate) fn build(textures: &[(&str, u32, u32, u8)]) -> Vec<u8> {
    let mut data = Vec::new();
    let mut directory = Vec::new();
    for (name, width, height, kind) in textures {
        let offset = HEADER + data.len();
        let mut lump = vec![0u8; MIP_HEADER];
        lump[..name.len()].copy_from_slice(name.as_bytes());
        lump[16..20].copy_from_slice(&width.to_le_bytes());
        lump[20..24].copy_from_slice(&height.to_le_bytes());
        let mut at = MIP_HEADER as u32;
        for level in 0..4 {
            lump[24 + level * 4..28 + level * 4].copy_from_slice(&at.to_le_bytes());
            let n = ((width >> level).max(1) * (height >> level).max(1)) as usize;
            lump.extend((0..n).map(|i| i as u8));
            at += n as u32;
        }
        lump.extend_from_slice(&256u16.to_le_bytes());
        lump.extend((0..256 * 3).map(|i| (i / 3) as u8));
        lump.extend_from_slice(&[0, 0]);

        let mut entry = [0u8; ENTRY];
        entry[0..4].copy_from_slice(&(offset as i32).to_le_bytes());
        entry[4..8].copy_from_slice(&(lump.len() as i32).to_le_bytes());
        entry[8..12].copy_from_slice(&(lump.len() as i32).to_le_bytes());
        entry[12] = *kind;
        entry[16..16 + name.len()].copy_from_slice(name.as_bytes());
        directory.extend_from_slice(&entry);
        data.extend(lump);
    }
    let mut out = b"WAD3".to_vec();
    out.extend_from_slice(&(textures.len() as i32).to_le_bytes());
    out.extend_from_slice(&((HEADER + data.len()) as i32).to_le_bytes());
    out.extend(data);
    out.extend(directory);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_textures_and_decals_found_ignoring_case() {
        let bytes = build(&[("BRICK1", 16, 8, MIP_TEXTURE), ("{blood1", 8, 8, DECAL)]);
        let wad = Wad::parse(bytes, Limits::default()).unwrap();
        assert_eq!(wad.entries().len(), 2);
        let t = wad.mip_texture(wad.find("brick1").unwrap()).unwrap();
        assert_eq!((t.name.as_str(), t.width, t.height), ("BRICK1", 16, 8));
        assert_eq!(t.indices.len(), 128);
        assert_eq!(t.indices[5], 5);
        assert_eq!(t.palette.len(), 768);
        assert_eq!(&t.palette[3..6], &[1, 1, 1]);
        assert!(wad.mip_texture(wad.find("{BLOOD1").unwrap()).is_ok());
        assert_eq!(wad.find("none"), None);
    }

    #[test]
    fn refuses_bad_archives_and_entries() {
        let good = build(&[("A", 8, 8, MIP_TEXTURE)]);
        assert!(Wad::parse(good[..8].to_vec(), Limits::default()).is_err());
        let mut no_magic = good.clone();
        no_magic[0] = b'X';
        assert!(Wad::parse(no_magic, Limits::default()).is_err());

        // An entry whose size runs past the archive
        let mut long = good.clone();
        let at = long.len() - ENTRY + 4;
        long[at..at + 4].copy_from_slice(&1_000_000i32.to_le_bytes());
        assert!(Wad::parse(long, Limits::default()).is_err());

        // A texture claiming more pixels than its entry holds
        let mut wide = good.clone();
        wide[HEADER + 16..HEADER + 20].copy_from_slice(&4096u32.to_le_bytes());
        let wad = Wad::parse(wide, Limits::default()).unwrap();
        assert!(wad.mip_texture(0).is_err());

        let few = Limits {
            max_entries: 0,
            ..Limits::default()
        };
        assert!(Wad::parse(good, few).is_err());
    }
}
