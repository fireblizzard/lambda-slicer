//! Quake PAK archives, which the game reads like a folder of files
//!
//! A 12-byte header (`PACK`, directory offset, directory length) and a
//! directory of 64-byte entries (a 56-byte name, offset and size). The whole
//! directory is checked against the file's length before any entry is used

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::{AssetError, Limits, io_err};

const HEADER: usize = 12;
const ENTRY: usize = 64;
const NAME: usize = 56;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PakEntry {
    /// The stored name with `/` separators
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug)]
pub struct Pak {
    path: PathBuf,
    entries: Vec<PakEntry>,
}

fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

impl Pak {
    pub fn open(path: &Path, limits: Limits) -> Result<Self, AssetError> {
        let bad = |detail: &str| AssetError::Pak {
            path: path.to_path_buf(),
            detail: detail.to_string(),
        };
        let mut file = File::open(path).map_err(io_err(path))?;
        let length = file.metadata().map_err(io_err(path))?.len();
        let mut header = [0u8; HEADER];
        file.read_exact(&mut header)
            .map_err(|_| bad("shorter than its header"))?;
        if &header[..4] != b"PACK" {
            return Err(bad("no PACK signature"));
        }
        let offset =
            u64::try_from(i32_at(&header, 4)).map_err(|_| bad("negative directory offset"))?;
        let size =
            usize::try_from(i32_at(&header, 8)).map_err(|_| bad("negative directory length"))?;
        if size % ENTRY != 0 || size / ENTRY > limits.max_entries {
            return Err(bad(
                "directory length is not a whole number of entries within the limit",
            ));
        }
        if offset
            .checked_add(size as u64)
            .is_none_or(|end| end > length)
        {
            return Err(bad("directory runs past the end of the file"));
        }

        let mut directory = vec![0u8; size];
        file.seek(SeekFrom::Start(offset)).map_err(io_err(path))?;
        file.read_exact(&mut directory).map_err(io_err(path))?;
        let entries = directory
            .as_chunks::<ENTRY>()
            .0
            .iter()
            .map(|e| {
                let end = e[..NAME].iter().position(|b| *b == 0).unwrap_or(NAME);
                let name = std::str::from_utf8(&e[..end])
                    .map_err(|_| bad("an entry name is not text"))?
                    .replace('\\', "/");
                let offset =
                    u64::try_from(i32_at(e, NAME)).map_err(|_| bad("negative entry offset"))?;
                let size =
                    u64::try_from(i32_at(e, NAME + 4)).map_err(|_| bad("negative entry size"))?;
                if offset.checked_add(size).is_none_or(|end| end > length) {
                    return Err(bad("an entry runs past the end of the file"));
                }
                Ok(PakEntry { name, offset, size })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            path: path.to_path_buf(),
            entries,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn entries(&self) -> &[PakEntry] {
        &self.entries
    }

    /// The entry named by `components`, matched exactly first, then
    /// ignoring ASCII case. Two entries equal ignoring case are ambiguous
    pub fn find(&self, components: &[String]) -> Result<Option<usize>, AssetError> {
        let name = components.join("/");
        if let Some(i) = self.entries.iter().position(|e| e.name == name) {
            return Ok(Some(i));
        }
        let matches: Vec<usize> = (0..self.entries.len())
            .filter(|&i| self.entries[i].name.eq_ignore_ascii_case(&name))
            .collect();
        match matches.as_slice() {
            [] => Ok(None),
            [i] => Ok(Some(*i)),
            _ => Err(AssetError::Ambiguous {
                name,
                candidates: matches
                    .iter()
                    .map(|&i| self.entries[i].name.clone())
                    .collect(),
            }),
        }
    }

    /// An entry's bytes, within the size limit
    pub fn read(&self, index: usize, limits: Limits) -> Result<Vec<u8>, AssetError> {
        let entry = &self.entries[index];
        if entry.size > limits.max_file {
            return Err(AssetError::TooLarge {
                path: format!("{}/{}", self.path.display(), entry.name),
                size: entry.size,
                limit: limits.max_file,
            });
        }
        let mut file = File::open(&self.path).map_err(io_err(&self.path))?;
        file.seek(SeekFrom::Start(entry.offset))
            .map_err(io_err(&self.path))?;
        let mut bytes = vec![0u8; entry.size as usize];
        file.read_exact(&mut bytes).map_err(io_err(&self.path))?;
        Ok(bytes)
    }
}

/// A PAK archive's bytes from `(name, contents)` pairs, for tests
#[cfg(test)]
pub(crate) fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut data = Vec::new();
    let mut directory = Vec::new();
    for (name, contents) in files {
        let offset = HEADER + data.len();
        data.extend_from_slice(contents);
        let mut entry = [0u8; ENTRY];
        entry[..name.len()].copy_from_slice(name.as_bytes());
        entry[NAME..NAME + 4].copy_from_slice(&(offset as i32).to_le_bytes());
        entry[NAME + 4..].copy_from_slice(&(contents.len() as i32).to_le_bytes());
        directory.extend_from_slice(&entry);
    }
    let mut out = b"PACK".to_vec();
    out.extend_from_slice(&((HEADER + data.len()) as i32).to_le_bytes());
    out.extend_from_slice(&(directory.len() as i32).to_le_bytes());
    out.extend(data);
    out.extend(directory);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pak0.pak");
        std::fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    fn components(name: &str) -> Vec<String> {
        name.split('/').map(String::from).collect()
    }

    #[test]
    fn reads_entries_and_finds_them_ignoring_case() {
        let bytes = build(&[("sound/a.wav", b"one"), ("models/Player.mdl", b"two")]);
        let (_dir, path) = write(&bytes);
        let pak = Pak::open(&path, Limits::default()).unwrap();
        assert_eq!(pak.entries().len(), 2);
        let i = pak.find(&components("models/player.mdl")).unwrap().unwrap();
        assert_eq!(pak.read(i, Limits::default()).unwrap(), b"two");
        assert_eq!(pak.find(&components("models/x.mdl")).unwrap(), None);
    }

    #[test]
    fn refuses_bad_archives_before_reading_them() {
        let mut bytes = build(&[("a", b"x")]);
        let (_dir, path) = write(&bytes[..8]);
        assert!(Pak::open(&path, Limits::default()).is_err());

        // An entry that claims more bytes than the file has
        let at = bytes.len() - ENTRY + NAME + 4;
        bytes[at..at + 4].copy_from_slice(&1_000_000i32.to_le_bytes());
        let (_dir, path) = write(&bytes);
        assert!(matches!(
            Pak::open(&path, Limits::default()),
            Err(AssetError::Pak { .. })
        ));

        let bytes = build(&[("a", b"x"), ("b", b"y")]);
        let (_dir, path) = write(&bytes);
        let few = Limits {
            max_entries: 1,
            ..Limits::default()
        };
        assert!(Pak::open(&path, few).is_err());
    }

    #[test]
    fn names_equal_ignoring_case_are_ambiguous() {
        let bytes = build(&[("A.wav", b"1"), ("a.WAV", b"2")]);
        let (_dir, path) = write(&bytes);
        let pak = Pak::open(&path, Limits::default()).unwrap();
        assert!(matches!(
            pak.find(&components("a.wav")),
            Err(AssetError::Ambiguous { .. })
        ));
        assert_eq!(pak.find(&components("A.wav")).unwrap(), Some(0));
    }
}
