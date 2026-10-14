//! No-clobber publication of exported files. Output is written to an
//! unpredictable temporary file beside the destination, synced, validated,
//! and only then persisted under the final name without replacing anything.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tempfile::NamedTempFile;

use super::ExportError;
use crate::library::fingerprint::{SourceFingerprint, full_fingerprint, hash_file};

#[derive(Clone, Debug, Serialize)]
pub struct Published {
    pub path: PathBuf,
    pub output_hash: String,
    pub source: SourceFingerprint,
}

/// Write a new file at `dest` derived from `source`.
///
/// `write` fills the temporary file; `validate` re-reads it by path. The
/// source's full hash is checked before and after; any change aborts.
/// Nothing is left at `dest` on failure.
pub fn publish_new(
    source: &Path,
    dest: &Path,
    write: impl FnOnce(&mut File) -> Result<(), ExportError>,
    validate: impl FnOnce(&Path) -> Result<(), ExportError>,
) -> Result<Published, ExportError> {
    let parent = match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    if !parent.is_dir() {
        return Err(ExportError::MissingParent(parent.to_path_buf()));
    }
    // Early, friendly rejection. The race-safe guarantee is persist_noclobber.
    if dest.symlink_metadata().is_ok() {
        if same_file::is_same_file(source, dest).unwrap_or(false) {
            return Err(ExportError::SameAsSource(dest.to_path_buf()));
        }
        return Err(ExportError::DestinationExists(dest.to_path_buf()));
    }

    let before = full_fingerprint(source).map_err(io_err(source))?;
    let mut tmp =
        NamedTempFile::with_prefix_in(".lambda-slicer-", parent).map_err(io_err(parent))?;
    write(tmp.as_file_mut())?;
    tmp.as_file_mut().sync_all().map_err(io_err(tmp.path()))?;
    validate(tmp.path())?;
    let output_hash = hash_file(tmp.path()).map_err(io_err(tmp.path()))?;

    let after = full_fingerprint(source).map_err(io_err(source))?;
    if !before.same_content(&after) || before.modified_ns != after.modified_ns {
        return Err(ExportError::SourceChanged(source.to_path_buf()));
    }

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
        source: before,
    })
}

pub(crate) fn io_err(path: &Path) -> impl FnOnce(io::Error) -> ExportError + '_ {
    move |source| ExportError::Io {
        path: path.to_path_buf(),
        source,
    }
}
