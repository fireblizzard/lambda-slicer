//! The folders and archives the game searches, in its order
//!
//! For the mod, then its `fallback_dir`, then `valve`, each present folder
//! of `<game>_addon`, `<game>_<language>`, `<game>_hd`, `<game>` and
//! `<game>_downloads`, the first three only when the matching option is on.
//! Within a folder its `pakN.pak` archives come first, the highest number
//! first, then its loose files (docs/viewer-profile.md, "Search order").
//! Further Half-Life folders can follow, each searched the same way once
//! the one before it has nothing

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::pak::Pak;
use crate::{AssetError, Limits, io_err};

/// The game's own content options, which add folders to the search
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// A language folder suffix such as `spanish`, none for English
    pub language: Option<String>,
    /// HD models (`_hd` folders)
    pub hd: bool,
    /// Custom addon content (`_addon` folders)
    pub addon: bool,
}

#[derive(Debug)]
pub enum SourceKind {
    Folder(PathBuf),
    Pak(Arc<Pak>),
}

/// One place searched, named as the game's folders are
#[derive(Debug)]
pub struct Source {
    /// `ag_downloads`, or `valve/pak0.pak` for an archive
    pub label: String,
    pub kind: SourceKind,
}

/// Where a name was found
#[derive(Clone, Debug)]
pub enum Location {
    File(PathBuf),
    Pak { archive: Arc<Pak>, entry: usize },
}

#[derive(Clone, Debug)]
pub struct Found {
    /// The name as asked for, with `/` separators
    pub name: String,
    /// The label of the source it came from
    pub source: String,
    pub location: Location,
}

pub struct SearchPath {
    sources: Vec<Source>,
    limits: Limits,
    /// Folder listings already read, by folder
    listings: Mutex<HashMap<PathBuf, Option<Arc<Vec<String>>>>>,
}

/// A name's components, or why it is refused. Names may use either slash
pub fn components(name: &str) -> Result<Vec<String>, AssetError> {
    let refuse = |reason| AssetError::Refused {
        name: name.to_string(),
        reason,
    };
    let name = name.replace('\\', "/");
    if name.is_empty() {
        return Err(refuse("an empty name"));
    }
    if name.starts_with('/') || name.contains(':') {
        return Err(refuse("an absolute name, outside the game folders"));
    }
    if name.contains('\0') {
        return Err(refuse("a name with a NUL character"));
    }
    name.split('/')
        .map(|c| match c {
            "" | "." | ".." => Err(refuse(
                "an empty, `.` or `..` component, which could leave the game folders",
            )),
            c => Ok(c.to_string()),
        })
        .collect()
}

/// A folder's entry names, sorted, `None` when it is not a folder
fn read_listing(dir: &Path) -> Result<Option<Vec<String>>, AssetError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            return Ok(None);
        }
        Err(e) => return Err(io_err(dir)(e)),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(io_err(dir))?;
        // Names that are not Unicode cannot be asked for by a demo
        if let Ok(name) = entry.file_name().into_string() {
            names.push(name);
        }
    }
    names.sort();
    Ok(Some(names))
}

/// The number of a `pakN.pak` name, matched ignoring case
fn pak_number(name: &str) -> Option<u32> {
    let lower = name.to_ascii_lowercase();
    lower
        .strip_prefix("pak")?
        .strip_suffix(".pak")?
        .parse()
        .ok()
}

/// The `fallback_dir` a `liblist.gam` names, if any
fn fallback_dir(game_folder: &Path) -> Option<String> {
    let text = std::fs::read(game_folder.join("liblist.gam")).ok()?;
    let text = String::from_utf8_lossy(&text);
    text.lines().find_map(|line| {
        let line = line.split("//").next()?.trim();
        let rest = line.strip_prefix("fallback_dir")?.trim();
        let value = rest.trim_matches('"').trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

/// A folder name of one component, or why it is refused
fn one(name: &str) -> Result<String, AssetError> {
    match components(name)?.as_slice() {
        [single] => Ok(single.clone()),
        _ => Err(AssetError::Refused {
            name: name.to_string(),
            reason: "a game folder name with more than one component",
        }),
    }
}

/// The places searched for `game` in the installation folder `install`,
/// first first, each labelled after `label` and its own name
fn sources_in(
    install: &Path,
    game: &str,
    options: &SearchOptions,
    limits: Limits,
    label: impl Fn(&str) -> String,
) -> Result<Vec<Source>, AssetError> {
    let mut games = vec![one(game)?];
    if let Some(f) = fallback_dir(&install.join(game))
        && let Ok(f) = one(&f)
        && !games.iter().any(|g| g.eq_ignore_ascii_case(&f))
    {
        games.push(f);
    }
    if !games.iter().any(|g| g.eq_ignore_ascii_case("valve")) {
        games.push("valve".to_string());
    }

    let mut sources = Vec::new();
    for g in &games {
        let mut folders = Vec::new();
        if options.addon {
            folders.push(format!("{g}_addon"));
        }
        if let Some(language) = &options.language {
            folders.push(format!("{g}_{}", one(language)?));
        }
        if options.hd {
            folders.push(format!("{g}_hd"));
        }
        folders.push(g.clone());
        folders.push(format!("{g}_downloads"));

        for name in folders {
            let folder = install.join(&name);
            let Some(names) = read_listing(&folder)? else {
                continue;
            };
            let mut paks: Vec<(u32, &String)> = names
                .iter()
                .filter_map(|n| pak_number(n).map(|k| (k, n)))
                .collect();
            paks.sort_by_key(|p| std::cmp::Reverse(p.0));
            for (_, pak_name) in paks {
                let pak = Pak::open(&folder.join(pak_name), limits)?;
                sources.push(Source {
                    label: label(&format!("{name}/{pak_name}")),
                    kind: SourceKind::Pak(Arc::new(pak)),
                });
            }
            sources.push(Source {
                label: label(&name),
                kind: SourceKind::Folder(folder),
            });
        }
    }
    Ok(sources)
}

impl SearchPath {
    /// The search for `game` in the installation folder `install`, such as
    /// `ag` in the folder that holds `valve`. Only folders that exist are
    /// searched, and every PAK archive in them is opened now
    pub fn new(
        install: &Path,
        game: &str,
        options: &SearchOptions,
        limits: Limits,
    ) -> Result<Self, AssetError> {
        Self::across(&[(install.to_path_buf(), options.clone())], game, limits)
    }

    /// The search for `game` in each of `installs` in turn, each with its
    /// own content options, the first of which must be readable. A later
    /// one that cannot be read, or holds an archive that cannot be opened,
    /// is left out. Its places are labelled with its folder, as the first
    /// one's own names are not
    pub fn across(
        installs: &[(PathBuf, SearchOptions)],
        game: &str,
        limits: Limits,
    ) -> Result<Self, AssetError> {
        let mut sources = Vec::new();
        for (k, (install, options)) in installs.iter().enumerate() {
            let label = |name: &str| match k {
                0 => name.to_string(),
                _ => install.join(name).display().to_string(),
            };
            match sources_in(install, game, options, limits, label) {
                Ok(found) => sources.extend(found),
                Err(e) if k == 0 => return Err(e),
                Err(_) => {}
            }
        }

        Ok(Self {
            sources,
            limits,
            listings: Mutex::new(HashMap::new()),
        })
    }

    /// Every place searched, first first
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }

    fn listing(&self, dir: &Path) -> Result<Option<Arc<Vec<String>>>, AssetError> {
        if let Some(l) = self.listings.lock().unwrap().get(dir) {
            return Ok(l.clone());
        }
        let listing = read_listing(dir)?.map(Arc::new);
        self.listings
            .lock()
            .unwrap()
            .insert(dir.to_path_buf(), listing.clone());
        Ok(listing)
    }

    /// The file `components` name under `root`, each matched exactly first,
    /// then ignoring ASCII case
    fn in_folder(
        &self,
        root: &Path,
        name: &str,
        components: &[String],
    ) -> Result<Option<PathBuf>, AssetError> {
        let mut path = root.to_path_buf();
        for c in components {
            let Some(listing) = self.listing(&path)? else {
                return Ok(None);
            };
            let next = if listing.contains(c) {
                c.clone()
            } else {
                let matches: Vec<&String> = listing
                    .iter()
                    .filter(|e| e.eq_ignore_ascii_case(c))
                    .collect();
                match matches.as_slice() {
                    [] => return Ok(None),
                    [one] => (*one).clone(),
                    _ => {
                        return Err(AssetError::Ambiguous {
                            name: name.to_string(),
                            candidates: matches
                                .iter()
                                .map(|m| path.join(m).display().to_string())
                                .collect(),
                        });
                    }
                }
            };
            path.push(next);
        }
        Ok(path.is_file().then_some(path))
    }

    /// The first source that has `name`
    pub fn find(&self, name: &str) -> Result<Found, AssetError> {
        let parts = components(name)?;
        let name = parts.join("/");
        for source in &self.sources {
            let location = match &source.kind {
                SourceKind::Folder(root) => {
                    self.in_folder(root, &name, &parts)?.map(Location::File)
                }
                SourceKind::Pak(pak) => pak.find(&parts)?.map(|entry| Location::Pak {
                    archive: pak.clone(),
                    entry,
                }),
            };
            if let Some(location) = location {
                return Ok(Found {
                    name,
                    source: source.label.clone(),
                    location,
                });
            }
        }
        Err(AssetError::Missing {
            name,
            searched: self.sources.iter().map(|s| s.label.clone()).collect(),
        })
    }

    /// A found file's bytes, within the size limit
    pub fn read(&self, found: &Found) -> Result<Vec<u8>, AssetError> {
        match &found.location {
            Location::File(path) => {
                let size = std::fs::metadata(path).map_err(io_err(path))?.len();
                if size > self.limits.max_file {
                    return Err(AssetError::TooLarge {
                        path: path.display().to_string(),
                        size,
                        limit: self.limits.max_file,
                    });
                }
                std::fs::read(path).map_err(io_err(path))
            }
            Location::Pak { archive, entry } => archive.read(*entry, self.limits),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(root: &Path, name: &str, contents: &[u8]) {
        let path = root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn found(search: &SearchPath, name: &str) -> (String, Vec<u8>) {
        let f = search.find(name).unwrap();
        (f.source.clone(), search.read(&f).unwrap())
    }

    #[test]
    fn a_later_install_is_searched_once_the_first_has_nothing() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        file(first.path(), "valve/models/player.mdl", b"first");
        file(first.path(), "ag/sprites/a.spr", b"first ag");
        file(second.path(), "valve/models/player.mdl", b"second");
        file(second.path(), "ag_downloads/maps/x.bsp", b"second map");

        let installs = [
            (first.path().to_path_buf(), SearchOptions::default()),
            (second.path().to_path_buf(), SearchOptions::default()),
        ];
        let search = SearchPath::across(&installs, "ag", Limits::default()).unwrap();

        // The first folder's valve comes before the second's ag
        assert_eq!(found(&search, "models/player.mdl").1, b"first");
        assert_eq!(found(&search, "sprites/a.spr").1, b"first ag");
        let (source, bytes) = found(&search, "maps/x.bsp");
        assert_eq!(bytes, b"second map");
        assert_eq!(
            source,
            second.path().join("ag_downloads").display().to_string()
        );
    }

    #[test]
    fn folders_are_searched_in_the_games_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(root, "valve/models/player.mdl", b"valve");
        file(root, "valve_hd/models/player.mdl", b"valve hd");
        file(root, "ag/models/player.mdl", b"ag");
        file(root, "ag_downloads/maps/x.bsp", b"download");
        file(root, "valve/maps/x.bsp", b"valve map");
        file(root, "valve/sound/a.wav", b"sound");

        let plain =
            SearchPath::new(root, "ag", &SearchOptions::default(), Limits::default()).unwrap();
        let labels: Vec<&str> = plain.sources().iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, ["ag", "ag_downloads", "valve"]);
        assert_eq!(
            found(&plain, "models/player.mdl"),
            ("ag".into(), b"ag".to_vec())
        );
        assert_eq!(
            found(&plain, "maps/x.bsp"),
            ("ag_downloads".into(), b"download".to_vec())
        );
        assert_eq!(found(&plain, "sound/a.wav").0, "valve");

        // HD models count only when chosen, and for valve after AG's own
        let hd = SearchOptions {
            hd: true,
            ..SearchOptions::default()
        };
        let hd = SearchPath::new(root, "ag", &hd, Limits::default()).unwrap();
        assert_eq!(found(&hd, "models/player.mdl").0, "ag");
        let valve_hd = SearchOptions {
            hd: true,
            ..SearchOptions::default()
        };
        let valve = SearchPath::new(root, "valve", &valve_hd, Limits::default()).unwrap();
        assert_eq!(found(&valve, "models/player.mdl").0, "valve_hd");

        assert!(matches!(
            plain.find("models/none.mdl"),
            Err(AssetError::Missing { .. })
        ));
    }

    #[test]
    fn a_fallback_dir_comes_before_valve() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(
            root,
            "mymod/liblist.gam",
            b"game \"My mod\"\nfallback_dir \"gearbox\" // shared\n",
        );
        file(root, "gearbox/models/a.mdl", b"gearbox");
        file(root, "valve/models/a.mdl", b"valve");
        let s =
            SearchPath::new(root, "mymod", &SearchOptions::default(), Limits::default()).unwrap();
        assert_eq!(found(&s, "models/a.mdl").0, "gearbox");
    }

    #[test]
    fn archives_come_before_loose_files_highest_first() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(root, "valve/sound/a.wav", b"loose");
        file(
            root,
            "valve/pak0.pak",
            &crate::pak::build(&[("sound/a.wav", b"pak0"), ("sound/b.wav", b"b")]),
        );
        file(
            root,
            "valve/PAK1.PAK",
            &crate::pak::build(&[("sound/a.wav", b"pak1")]),
        );
        let s =
            SearchPath::new(root, "valve", &SearchOptions::default(), Limits::default()).unwrap();
        assert_eq!(
            found(&s, "sound/a.wav"),
            ("valve/PAK1.PAK".into(), b"pak1".to_vec())
        );
        assert_eq!(found(&s, "sound/b.wav").0, "valve/pak0.pak");
    }

    #[test]
    fn names_match_ignoring_case_with_either_slash() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(root, "valve/Models/Player/Gordon.mdl", b"g");
        let s =
            SearchPath::new(root, "valve", &SearchOptions::default(), Limits::default()).unwrap();
        assert_eq!(found(&s, "models\\player\\gordon.MDL").1, b"g");
    }

    #[test]
    fn names_that_leave_the_game_folders_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(root, "valve/a.txt", b"a");
        file(root, "secret.txt", b"no");
        let s =
            SearchPath::new(root, "valve", &SearchOptions::default(), Limits::default()).unwrap();
        for name in [
            "../secret.txt",
            "maps/../../secret.txt",
            "/etc/passwd",
            "C:\\secret.txt",
            "a//b",
            "",
            "./a.txt",
        ] {
            assert!(
                matches!(s.find(name), Err(AssetError::Refused { .. })),
                "{name}"
            );
        }
        assert!(
            SearchPath::new(root, "../x", &SearchOptions::default(), Limits::default()).is_err()
        );
    }

    #[test]
    fn files_over_the_limit_are_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(root, "valve/big.bin", &[0u8; 100]);
        let small = Limits {
            max_file: 10,
            ..Limits::default()
        };
        let s = SearchPath::new(root, "valve", &SearchOptions::default(), small).unwrap();
        let f = s.find("big.bin").unwrap();
        assert!(matches!(s.read(&f), Err(AssetError::TooLarge { .. })));
    }

    // Only a case-sensitive file system can hold names that differ in case
    #[cfg(target_os = "linux")]
    #[test]
    fn names_differing_only_in_case_are_ambiguous() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        file(root, "valve/sound/A.wav", b"1");
        file(root, "valve/sound/a.WAV", b"2");
        let s =
            SearchPath::new(root, "valve", &SearchOptions::default(), Limits::default()).unwrap();
        assert!(matches!(
            s.find("sound/a.wav"),
            Err(AssetError::Ambiguous { .. })
        ));
        assert_eq!(found(&s, "sound/A.wav").1, b"1");
    }
}
