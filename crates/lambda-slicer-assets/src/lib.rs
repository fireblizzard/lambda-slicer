//! The installed game's content, found the way the game finds it
//! (implementation plan §13.3)
//!
//! A [`SearchPath`] lists the game folders and archives to search, in the
//! game's order. It finds a name such as `models/player.mdl` in the first
//! that has it, matching letter case the way Windows does, and refuses names
//! that would leave the game folders. [`pak`] and [`wad`] read the archives,
//! and [`resource`] turns a demo session's resource list into lookups and a
//! report of what is missing. Nothing here depends on the demo decoder, a
//! GPU or a sound device, and no game content is bundled

pub mod pak;
pub mod resource;
pub mod search;
pub mod wad;
pub mod xenoxide;

use std::path::PathBuf;

/// Bounds on what is read, checked before anything is allocated
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// The largest file or archive entry read into memory
    pub max_file: u64,
    /// The most entries a PAK or WAD directory may list
    pub max_entries: usize,
}

impl Default for Limits {
    /// Larger than any map, model, sound or WAD in the surveyed install
    fn default() -> Self {
        Self {
            max_file: 256 << 20,
            max_entries: 1 << 18,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("{name}: {reason}")]
    Refused { name: String, reason: &'static str },
    #[error("{name} is in none of {}", .searched.join(", "))]
    Missing { name: String, searched: Vec<String> },
    #[error("{name} matches more than one entry ignoring case: {}", .candidates.join(", "))]
    Ambiguous {
        name: String,
        candidates: Vec<String>,
    },
    #[error("{path} is {size} bytes, over the limit of {limit}")]
    TooLarge { path: String, size: u64, limit: u64 },
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path} is not a valid PAK archive: {detail}")]
    Pak { path: PathBuf, detail: String },
    #[error("not a valid WAD3 archive: {0}")]
    Wad(String),
}

fn io_err(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> AssetError {
    let path = path.into();
    move |source| AssetError::Io { path, source }
}
