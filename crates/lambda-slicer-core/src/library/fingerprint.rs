//! Source identity, made of metadata, a bounded quick hash, and a full BLAKE3
//! hash

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::Serialize;

/// Bytes hashed from each of the start and end of a file for the quick hash
const QUICK_REGION: u64 = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceFingerprint {
    pub size: u64,
    pub modified_ns: Option<i128>,
    /// BLAKE3 of size, first and last 64 KiB. A hint, not proof of identity
    pub quick_hash: String,
    pub full_hash: Option<String>,
}

impl SourceFingerprint {
    /// True when both fingerprints carry full hashes and they match, along
    /// with size
    pub fn same_content(&self, other: &Self) -> bool {
        self.size == other.size && self.full_hash.is_some() && self.full_hash == other.full_hash
    }
}

fn modified_ns(meta: &std::fs::Metadata) -> Option<i128> {
    let t = meta.modified().ok()?;

    Some(match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i128,
        Err(e) => -(e.duration().as_nanos() as i128),
    })
}

pub fn quick_fingerprint(path: &Path) -> io::Result<SourceFingerprint> {
    let mut f = File::open(path)?;
    let meta = f.metadata()?;
    let size = meta.len();

    let mut h = blake3::Hasher::new();
    h.update(&size.to_le_bytes());

    let mut buf = vec![0u8; QUICK_REGION.min(size) as usize];
    f.read_exact(&mut buf)?;
    h.update(&buf);

    if size > QUICK_REGION {
        let tail_start = size.saturating_sub(QUICK_REGION).max(QUICK_REGION);
        f.seek(SeekFrom::Start(tail_start))?;
        let mut tail = Vec::new();
        f.read_to_end(&mut tail)?;
        h.update(&tail);
    }

    Ok(SourceFingerprint {
        size,
        modified_ns: modified_ns(&meta),
        quick_hash: h.finalize().to_hex().to_string(),
        full_hash: None,
    })
}

/// Quick fingerprint plus a streamed full-content BLAKE3 hash
pub fn full_fingerprint(path: &Path) -> io::Result<SourceFingerprint> {
    let mut fp = quick_fingerprint(path)?;
    fp.full_hash = Some(hash_file(path)?);
    Ok(fp)
}

pub fn hash_file(path: &Path) -> io::Result<String> {
    let mut h = blake3::Hasher::new();
    h.update_reader(File::open(path)?)?;
    Ok(h.finalize().to_hex().to_string())
}
