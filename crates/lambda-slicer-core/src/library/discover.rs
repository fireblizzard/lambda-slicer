//! Streaming discovery of demo files under library roots
//!
//! Paths are handed out as they are found, so results can start before the
//! walk ends. Directory symlinks and junctions are not followed, roots inside
//! other roots are dropped, and unreadable directories are reported without
//! stopping the walk

use std::path::{Path, PathBuf};

/// One discovery result
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    Demo(PathBuf),
    /// A directory or entry that could not be read
    Error {
        path: PathBuf,
        error: String,
    },
    /// A symlink or junction that was not followed
    Skipped(PathBuf),
}

fn is_demo(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("dem"))
}

/// The roots to walk, absolute, with any root inside another removed.
/// Unreadable roots come back as errors
pub fn roots(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<Found>) {
    let mut errors = Vec::new();
    let mut resolved: Vec<(PathBuf, PathBuf)> = Vec::new();
    for p in paths {
        let absolute = std::path::absolute(p).unwrap_or_else(|_| p.clone());
        match std::fs::canonicalize(p) {
            Ok(canonical) => {
                if !resolved.iter().any(|(c, _)| *c == canonical) {
                    resolved.push((canonical, absolute));
                }
            }
            Err(e) => errors.push(Found::Error {
                path: absolute,
                error: e.to_string(),
            }),
        }
    }

    let kept = resolved
        .iter()
        .filter(|(c, _)| {
            !resolved
                .iter()
                .any(|(other, _)| other != c && c.starts_with(other))
        })
        .map(|(_, a)| a.clone())
        .collect();

    (kept, errors)
}

/// Walk `roots` depth first in name order, calling `found` for each result
/// until it returns false
pub fn walk(roots: &[PathBuf], mut found: impl FnMut(Found) -> bool) {
    for root in roots {
        match std::fs::symlink_metadata(root) {
            Ok(m) if m.is_file() => {
                if !found(Found::Demo(root.clone())) {
                    return;
                }
                continue;
            }
            Ok(_) => {}
            Err(e) => {
                if !found(Found::Error {
                    path: root.clone(),
                    error: e.to_string(),
                }) {
                    return;
                }
                continue;
            }
        }

        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let entries = match std::fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(e) => {
                    if !found(Found::Error {
                        path: dir,
                        error: e.to_string(),
                    }) {
                        return;
                    }
                    continue;
                }
            };

            let mut files = Vec::new();
            let mut dirs = Vec::new();
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        if !found(Found::Error {
                            path: dir.clone(),
                            error: e.to_string(),
                        }) {
                            return;
                        }
                        continue;
                    }
                };
                let path = entry.path();
                // A junction reports as a symlink on Windows, so neither is
                // followed
                match entry.file_type() {
                    Ok(t) if t.is_symlink() => {
                        if !found(Found::Skipped(path)) {
                            return;
                        }
                    }
                    Ok(t) if t.is_dir() => dirs.push(path),
                    Ok(_) if is_demo(&path) => files.push(path),
                    Ok(_) => {}
                    Err(e) => {
                        if !found(Found::Error {
                            path,
                            error: e.to_string(),
                        }) {
                            return;
                        }
                    }
                }
            }

            files.sort();
            for f in files {
                if !found(Found::Demo(f)) {
                    return;
                }
            }
            dirs.sort();
            stack.extend(dirs.into_iter().rev());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walked(roots: &[PathBuf]) -> Vec<Found> {
        let mut out = Vec::new();
        walk(roots, |f| {
            out.push(f);
            true
        });
        out
    }

    #[test]
    fn walks_demos_in_name_order_and_drops_nested_roots() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = a.join("b");
        std::fs::create_dir_all(&b).unwrap();
        for p in [
            a.join("2.dem"),
            a.join("1.DEM"),
            a.join("notes.txt"),
            b.join("3.dem"),
        ] {
            std::fs::write(p, b"x").unwrap();
        }

        let (roots, errors) = roots(&[b.clone(), a.clone(), dir.path().join("missing")]);
        assert_eq!(roots.len(), 1);
        assert_eq!(errors.len(), 1);

        let names: Vec<String> = walked(&roots)
            .into_iter()
            .map(|f| match f {
                Found::Demo(p) => p.file_name().unwrap().to_string_lossy().into_owned(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(names, ["1.DEM", "2.dem", "3.dem"]);
    }

    #[test]
    fn stops_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        for n in ["1.dem", "2.dem"] {
            std::fs::write(dir.path().join(n), b"x").unwrap();
        }

        let mut seen = 0;
        walk(&[dir.path().to_path_buf()], |_| {
            seen += 1;
            false
        });
        assert_eq!(seen, 1);
    }
}
