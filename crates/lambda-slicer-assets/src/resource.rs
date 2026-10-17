//! A demo session's resource list as lookups in the game folders
//!
//! The server lists every sound, model, sprite and other file a session
//! uses (`svc_resourcelist`). Sounds are named under `sound/`, `*1` and on
//! name the map's own brush models, names starting with `!` are sentences
//! from `sound/sentences.txt`, and decals are entries of `decals.wad`.
//! Servers that check clients' files list each file as a decal named in
//! the consistency list, such as `../opengl32.dll`. Those are checks, not
//! content (docs/viewer-profile.md, "Resources")

use crate::search::{Found, SearchPath};
use crate::wad::Wad;
use crate::{AssetError, Limits};

/// The resource types of `svc_resourcelist`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Sound,
    Skin,
    Model,
    Decal,
    Generic,
    EventScript,
    World,
    Unknown(u8),
}

impl From<u8> for Kind {
    fn from(v: u8) -> Self {
        match v {
            0 => Kind::Sound,
            1 => Kind::Skin,
            2 => Kind::Model,
            3 => Kind::Decal,
            4 => Kind::Generic,
            5 => Kind::EventScript,
            6 => Kind::World,
            other => Kind::Unknown(other),
        }
    }
}

/// One entry of a resource list
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Listed<'a> {
    pub kind: u8,
    pub name: &'a [u8],
    /// Named in the list's consistency list
    pub checked: bool,
}

/// Where a resource lives
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A file in the game folders
    File(String),
    /// A brush model inside the map's own file
    MapModel,
    /// A sentence, resolved with the sentence list when sounds are loaded
    Sentence(String),
    /// An image in `decals.wad`
    Decal(String),
    /// A file the server checks on the client, not content to load
    Check,
}

pub fn target(kind: Kind, name: &str, checked: bool) -> Target {
    match kind {
        Kind::Decal if checked => Target::Check,
        Kind::Sound => match name.strip_prefix('!') {
            Some(sentence) => Target::Sentence(sentence.to_string()),
            None => Target::File(format!("sound/{name}")),
        },
        Kind::Model if name.starts_with('*') => Target::MapModel,
        Kind::Decal => Target::Decal(name.to_string()),
        _ => Target::File(name.to_string()),
    }
}

#[derive(Debug)]
pub enum Outcome {
    Found(Found),
    /// A decal found in the `decals.wad` that was found
    InDecals(Found),
    MapModel,
    Sentence,
    Check,
    Missing(AssetError),
}

#[derive(Debug)]
pub struct Resolved {
    pub kind: Kind,
    pub name: String,
    pub outcome: Outcome,
}

/// Look up every entry of a resource list. `decals.wad` is read
/// once, and only when a decal is listed
pub fn resolve_all<'a>(
    search: &SearchPath,
    resources: impl IntoIterator<Item = Listed<'a>>,
    limits: Limits,
) -> Vec<Resolved> {
    let mut decals: Option<Result<(Found, Wad), String>> = None;
    resources
        .into_iter()
        .map(
            |Listed {
                 kind,
                 name,
                 checked,
             }| {
                let kind = Kind::from(kind);
                let Ok(name) = std::str::from_utf8(name) else {
                    let name = String::from_utf8_lossy(name).into_owned();
                    return Resolved {
                        outcome: Outcome::Missing(AssetError::Refused {
                            name: name.clone(),
                            reason: "a name that is not text",
                        }),
                        kind,
                        name,
                    };
                };
                let outcome = match target(kind, name, checked) {
                    Target::File(path) => match search.find(&path) {
                        Ok(f) => Outcome::Found(f),
                        Err(e) => Outcome::Missing(e),
                    },
                    Target::MapModel => Outcome::MapModel,
                    Target::Sentence(_) => Outcome::Sentence,
                    Target::Check => Outcome::Check,
                    Target::Decal(decal) => {
                        let wad = decals.get_or_insert_with(|| {
                            let found = search.find("decals.wad").map_err(|e| e.to_string())?;
                            let bytes = search.read(&found).map_err(|e| e.to_string())?;
                            let wad = Wad::parse(bytes, limits).map_err(|e| e.to_string())?;
                            Ok((found, wad))
                        });
                        match wad {
                            Ok((found, wad)) if wad.find(&decal).is_some() => {
                                Outcome::InDecals(found.clone())
                            }
                            Ok(_) => Outcome::Missing(AssetError::Missing {
                                name: format!("decals.wad/{decal}"),
                                searched: vec!["decals.wad".to_string()],
                            }),
                            Err(detail) => Outcome::Missing(AssetError::Wad(detail.clone())),
                        }
                    }
                };
                Resolved {
                    kind,
                    name: name.to_string(),
                    outcome,
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchOptions;

    #[test]
    fn targets_follow_the_resource_kind() {
        let t = |kind, name| target(kind, name, false);
        let file = |name: &str| Target::File(name.into());
        assert_eq!(
            t(Kind::Sound, "weapons/gauss2.wav"),
            file("sound/weapons/gauss2.wav")
        );
        assert_eq!(
            t(Kind::Sound, "!HG_ALERT0"),
            Target::Sentence("HG_ALERT0".into())
        );
        assert_eq!(t(Kind::Model, "*12"), Target::MapModel);
        assert_eq!(
            t(Kind::Model, "models/player.mdl"),
            file("models/player.mdl")
        );
        assert_eq!(t(Kind::Decal, "{blood1"), Target::Decal("{blood1".into()));
        assert_eq!(t(Kind::World, "maps/c1a0.bsp"), file("maps/c1a0.bsp"));
        // A decal in the consistency list is a file check, and a checked
        // sound is still a sound to load
        assert_eq!(target(Kind::Decal, "../opengl32.dll", true), Target::Check);
        assert_eq!(target(Kind::Sound, "a.wav", true), file("sound/a.wav"));
    }

    #[test]
    fn a_resource_list_resolves_with_what_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("valve/sound/weapons")).unwrap();
        std::fs::create_dir_all(root.join("valve/models")).unwrap();
        std::fs::write(root.join("valve/sound/weapons/gauss2.wav"), b"RIFF").unwrap();
        std::fs::write(root.join("valve/models/player.mdl"), b"IDST").unwrap();
        std::fs::write(
            root.join("valve/decals.wad"),
            crate::wad::build(&[("{blood1", 8, 8, crate::wad::DECAL)]),
        )
        .unwrap();
        let search =
            SearchPath::new(root, "valve", &SearchOptions::default(), Limits::default()).unwrap();

        let list: [(u8, &[u8], bool); 8] = [
            (0, b"weapons/gauss2.wav", false),
            (0, b"!HG_ALERT0", false),
            (2, b"models/player.mdl", false),
            (2, b"*3", false),
            (2, b"models/none.mdl", false),
            (3, b"{blood1", false),
            (3, b"{nothing", false),
            (3, b"../opengl32.dll", true),
        ];
        let listed = list.map(|(kind, name, checked)| Listed {
            kind,
            name,
            checked,
        });
        let resolved = resolve_all(&search, listed, Limits::default());
        let shape: Vec<&str> = resolved
            .iter()
            .map(|r| match r.outcome {
                Outcome::Found(_) => "found",
                Outcome::InDecals(_) => "decal",
                Outcome::MapModel => "map",
                Outcome::Sentence => "sentence",
                Outcome::Check => "check",
                Outcome::Missing(_) => "missing",
            })
            .collect();
        assert_eq!(
            shape,
            [
                "found", "sentence", "found", "map", "missing", "decal", "missing", "check"
            ]
        );
    }
}
