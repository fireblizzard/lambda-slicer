//! No-clobber publication of exported files. The source is first copied to a
//! private snapshot and hashed in the same pass, so the reported source hash
//! covers exactly the bytes the export reads. Output is written to an
//! unpredictable temporary file beside the destination, synced, validated,
//! and only then persisted under the final name without replacing anything

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tempfile::NamedTempFile;

use super::ExportError;
use crate::library::fingerprint::{SourceFingerprint, hash_file, quick_fingerprint};

#[derive(Clone, Debug, Serialize)]
pub struct Published {
    pub path: PathBuf,
    pub output_hash: String,
    pub source: SourceFingerprint,
}

/// Write a new file at `dest` derived from `source`
///
/// `write` reads the source snapshot, positioned at its start, and fills the
/// temporary output. It also gets the snapshot's path, for passes that open
/// the snapshot again. `validate` re-reads the output by path. A source that
/// changes while the snapshot is taken aborts the export. Nothing is left at
/// `dest` on failure
pub fn publish_new(
    source: &Path,
    dest: &Path,
    write: impl FnOnce(&mut File, &Path, &mut File) -> Result<(), ExportError>,
    validate: impl FnOnce(&Path) -> Result<(), ExportError>,
) -> Result<Published, ExportError> {
    let parent = match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    if !parent.is_dir() {
        return Err(ExportError::MissingParent(parent.to_path_buf()));
    }

    // Early, friendly rejection. The race-safe guarantee is persist_noclobber
    if dest.symlink_metadata().is_ok() {
        if same_file::is_same_file(source, dest).unwrap_or(false) {
            return Err(ExportError::SameAsSource(dest.to_path_buf()));
        }
        return Err(ExportError::DestinationExists(dest.to_path_buf()));
    }

    let (mut snapshot, source_fingerprint) = snapshot_source(source, parent)?;

    let mut tmp =
        NamedTempFile::with_prefix_in(".lambda-slicer-", parent).map_err(io_err(parent))?;
    let snapshot_path = snapshot.path().to_path_buf();
    write(snapshot.as_file_mut(), &snapshot_path, tmp.as_file_mut())?;
    tmp.as_file_mut().sync_all().map_err(io_err(tmp.path()))?;
    validate(tmp.path())?;
    let output_hash = hash_file(tmp.path()).map_err(io_err(tmp.path()))?;

    tmp.persist_noclobber(dest).map_err(|e| {
        if e.error.kind() == io::ErrorKind::AlreadyExists {
            ExportError::DestinationExists(dest.to_path_buf())
        } else {
            ExportError::Io {
                path: dest.to_path_buf(),
                source: e.error,
            }
        }
    })?;

    Ok(Published {
        path: dest.to_path_buf(),
        output_hash,
        source: source_fingerprint,
    })
}

/// Copy `source` into a private temporary file in `dir`, hashing the bytes as
/// they are copied. The fingerprint carries that hash, and the snapshot is
/// returned rewound to its start
fn snapshot_source(
    source: &Path,
    dir: &Path,
) -> Result<(NamedTempFile, SourceFingerprint), ExportError> {
    let before = quick_fingerprint(source).map_err(io_err(source))?;

    let mut snapshot =
        NamedTempFile::with_prefix_in(".lambda-slicer-src-", dir).map_err(io_err(dir))?;
    let src = File::open(source).map_err(io_err(source))?;
    let (copied, full_hash) = copy_hashed(src, snapshot.as_file_mut()).map_err(io_err(source))?;

    let after = quick_fingerprint(source).map_err(io_err(source))?;
    if !unchanged(&before, &after, copied) {
        return Err(ExportError::SourceChanged(source.to_path_buf()));
    }

    snapshot
        .as_file_mut()
        .seek(SeekFrom::Start(0))
        .map_err(io_err(snapshot.path()))?;

    let fingerprint = SourceFingerprint {
        full_hash: Some(full_hash),
        ..before
    };

    Ok((snapshot, fingerprint))
}

/// Copy `src` to `dst` and return the byte count with the BLAKE3 hash of
/// exactly those bytes
fn copy_hashed(mut src: impl Read, mut dst: impl Write) -> io::Result<(u64, String)> {
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 256 * 1024];
    let mut copied = 0u64;
    loop {
        let n = match src.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };

        hasher.update(&buf[..n]);
        dst.write_all(&buf[..n])?;
        copied += n as u64;
    }

    dst.flush()?;

    Ok((copied, hasher.finalize().to_hex().to_string()))
}

/// Whether the source kept its size, modification time and quick hash while
/// it was copied, and the copy read exactly that many bytes
fn unchanged(before: &SourceFingerprint, after: &SourceFingerprint, copied: u64) -> bool {
    before.size == after.size
        && before.modified_ns == after.modified_ns
        && before.quick_hash == after.quick_hash
        && copied == before.size
}

/// Counts bytes written so entry offsets can be assigned while streaming
pub(super) struct Counting<W> {
    pub inner: W,
    pub pos: u64,
}

impl<W: Write> Counting<W> {
    pub fn put(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.inner.write_all(bytes)?;
        self.pos += bytes.len() as u64;
        Ok(())
    }
}

pub(super) fn to_i32(v: u64, what: &str) -> Result<i32, ExportError> {
    i32::try_from(v).map_err(|_| ExportError::Unsupported(format!("{what} {v} exceeds i32")))
}

pub(crate) fn io_err(path: &Path) -> impl FnOnce(io::Error) -> ExportError + '_ {
    move |source| ExportError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fingerprint(size: u64, modified_ns: i128, quick_hash: &str) -> SourceFingerprint {
        SourceFingerprint {
            size,
            modified_ns: Some(modified_ns),
            quick_hash: quick_hash.to_string(),
            full_hash: None,
        }
    }

    #[test]
    fn copy_hash_covers_the_copied_bytes() {
        let data: Vec<u8> = (0..700_000u32).map(|i| (i % 251) as u8).collect();
        let mut out = Vec::new();

        let (copied, hash) = copy_hashed(&data[..], &mut out).unwrap();

        assert_eq!(copied, data.len() as u64);
        assert_eq!(out, data);
        assert_eq!(hash, blake3::hash(&data).to_hex().to_string());
    }

    #[test]
    fn any_metadata_change_or_short_copy_counts_as_a_change() {
        let base = fingerprint(100, 5, "q");

        assert!(unchanged(&base, &base.clone(), 100));
        assert!(!unchanged(&base, &fingerprint(101, 5, "q"), 100));
        assert!(!unchanged(&base, &fingerprint(100, 6, "q"), 100));
        assert!(!unchanged(&base, &fingerprint(100, 5, "r"), 100));
        assert!(!unchanged(&base, &base.clone(), 99));
    }

    #[test]
    fn export_reads_the_hashed_snapshot_even_if_the_source_changes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("src.bin");
        let dest = dir.path().join("out.bin");
        let original = b"original bytes".to_vec();
        std::fs::write(&source, &original).unwrap();

        let published = publish_new(
            &source,
            &dest,
            |snapshot, _, out| {
                std::fs::write(&source, b"replaced after the snapshot").unwrap();
                io::copy(snapshot, out).map_err(io_err(&source))?;
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();

        let original_hash = blake3::hash(&original).to_hex().to_string();
        assert_eq!(std::fs::read(&dest).unwrap(), original);
        assert_eq!(
            published.source.full_hash.as_deref(),
            Some(original_hash.as_str())
        );
        assert_eq!(published.output_hash, original_hash);
        assert_eq!(published.source.size, original.len() as u64);
    }

    #[test]
    fn snapshots_are_removed_after_a_failed_export() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("src.bin");
        std::fs::write(&source, b"data").unwrap();

        let failed = publish_new(
            &source,
            &dir.path().join("out.bin"),
            |_, _, _| Err(ExportError::EmptyRange),
            |_| Ok(()),
        );
        assert!(matches!(failed, Err(ExportError::EmptyRange)));

        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from("src.bin")]);
    }
}
