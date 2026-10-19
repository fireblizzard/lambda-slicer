//! The game's sounds, its models' sequences and its maps' worlds, read once
//! each through its search path
//!
//! A sample named `weapons/gauss2.wav` is `sound/weapons/gauss2.wav`. A
//! leading `*`, which marks a sound the game streams, is dropped. Sentences
//! (`!` names and sentence numbers) are not resolved to their words yet, so
//! they have no samples. A sound or model that cannot be read is remembered
//! as missing, so it is not read again. Of a model only its main file is
//! read, which describes every sequence and its events. Of a map only its
//! node tree and its faces' texture extents and materials are kept, for
//! tracing through, the materials from `sound/materials.txt`

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::search::SearchPath;
use lambda_slicer_assets::sprite;
use lambda_slicer_assets::wad::Wad;
use lambda_slicer_assets::wav::{self, Sound};
use lambda_slicer_assets::xenoxide::bsp30::{BspDocument, MipTexture};
use lambda_slicer_assets::xenoxide::studio::StudioModel;
use lambda_slicer_playback::GameFiles;
use lambda_slicer_playback::animation::{Sequence, StudioEvent};
use lambda_slicer_playback::audio::{SoundLength, SoundLengths, SoundName};
use lambda_slicer_playback::trace::{Child, Hull, HullFace, SOLID};

/// `STUDIO_LOOPING`
const STUDIO_LOOPING: i32 = 1;

pub struct Library {
    search: SearchPath,
    limits: Limits,
    sounds: Mutex<HashMap<Arc<str>, Option<Arc<Sound>>>>,
    /// The sound files that could not be read, and why
    missing_sounds: Mutex<std::collections::BTreeMap<String, String>>,
    sequences: Mutex<HashMap<String, Option<Arc<[Sequence]>>>>,
    hulls: Mutex<HashMap<String, Option<Arc<Hull>>>>,
    sprites: Mutex<HashMap<String, Option<u32>>>,
    materials: Mutex<Option<Arc<Materials>>>,
    /// `decals.wad`'s names in its order, read once
    decal_names: Mutex<Option<Arc<Vec<String>>>>,
}

impl Library {
    pub fn new(search: SearchPath, limits: Limits) -> Self {
        Self {
            search,
            limits,
            sounds: Mutex::new(HashMap::new()),
            missing_sounds: Mutex::new(std::collections::BTreeMap::new()),
            sequences: Mutex::new(HashMap::new()),
            hulls: Mutex::new(HashMap::new()),
            sprites: Mutex::new(HashMap::new()),
            materials: Mutex::new(None),
            decal_names: Mutex::new(None),
        }
    }

    /// The place of the decal `name` in `decals.wad`, ignoring case as the
    /// game does
    pub fn decal_index(&self, name: &str) -> Option<u16> {
        let mut known = self.decal_names.lock().unwrap_or_else(|e| e.into_inner());
        let names = known.get_or_insert_with(|| {
            let names = self
                .read("decals.wad")
                .and_then(|bytes| Wad::parse(bytes, self.limits).ok())
                .map(|wad| wad.entries().iter().map(|e| e.name.clone()).collect())
                .unwrap_or_default();
            Arc::new(names)
        });
        let at = names.iter().position(|n| n.eq_ignore_ascii_case(name))?;
        u16::try_from(at).ok()
    }

    /// The textures' materials, read once from `sound/materials.txt`
    fn materials(&self) -> Arc<Materials> {
        let mut known = self.materials.lock().unwrap_or_else(|e| e.into_inner());
        known
            .get_or_insert_with(|| {
                let text = self.read("sound/materials.txt").unwrap_or_default();
                Arc::new(Materials::parse(&String::from_utf8_lossy(&text)))
            })
            .clone()
    }

    /// The samples of `name`, none when it is a sentence or cannot be read
    pub fn sound(&self, name: &SoundName) -> Option<Arc<Sound>> {
        let SoundName::File(file) = name else {
            return None;
        };
        let mut sounds = self.sounds.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(known) = sounds.get(file) {
            return known.clone();
        }

        let path = format!("sound/{}", file.trim_start_matches('*'));
        let read = self
            .search
            .find(&path)
            .and_then(|found| self.search.read(&found))
            .and_then(|bytes| wav::parse(&bytes, self.limits));
        let sound = match read {
            Ok(sound) => Some(Arc::new(sound)),
            Err(e) => {
                let mut missing = self
                    .missing_sounds
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                missing.insert(path, e.to_string());
                None
            }
        };
        sounds.insert(file.clone(), sound.clone());
        sound
    }

    /// The sound files asked for so far that could not be read, and why
    pub fn missing_sounds(&self) -> Vec<(String, String)> {
        let missing = self
            .missing_sounds
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        missing
            .iter()
            .map(|(n, r)| (n.clone(), r.clone()))
            .collect()
    }

    /// How long `name` plays, as its file says
    pub fn length(&self, name: &SoundName) -> SoundLength {
        match self.sound(name) {
            Some(s) if s.loop_start.is_some() => SoundLength::Looping,
            Some(s) => SoundLength::Once(s.seconds()),
            None => SoundLength::Unknown,
        }
    }

    /// [`Library::length`] for playback
    pub fn lengths(self: &Arc<Self>) -> SoundLengths {
        let library = self.clone();
        Arc::new(move |name| library.length(name))
    }

    /// The sequences of the studio model at `path`, such as
    /// `models/v_9mmar.mdl`, none when it is not a studio model or cannot
    /// be read
    pub fn sequences(&self, path: &str) -> Option<Arc<[Sequence]>> {
        let mut known = self.sequences.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(sequences) = known.get(path) {
            return sequences.clone();
        }

        let read = self
            .search
            .find(path)
            .and_then(|found| self.search.read(&found))
            .ok();
        let sequences = read
            .filter(|bytes| bytes.starts_with(b"IDST"))
            .and_then(|bytes| StudioModel::parse(&bytes).ok())
            .map(|model| {
                model
                    .sequences()
                    .iter()
                    .map(|s| Sequence {
                        fps: s.fps,
                        frames: u32::try_from(s.frames).unwrap_or(u32::MAX),
                        looping: s.flags & STUDIO_LOOPING != 0,
                        events: s
                            .events
                            .iter()
                            .map(|e| StudioEvent {
                                frame: e.frame,
                                event: e.event,
                                options: Arc::from(e.options.as_str()),
                            })
                            .collect(),
                    })
                    .collect()
            });
        known.insert(path.to_owned(), sequences.clone());
        sequences
    }

    /// The world's point hull of the map at `path`, such as
    /// `maps/crossfire.bsp`, none when it cannot be read
    pub fn hull(&self, path: &str) -> Option<Arc<Hull>> {
        let mut known = self.hulls.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(hull) = known.get(path) {
            return hull.clone();
        }
        let hull = self
            .search
            .find(path)
            .and_then(|found| self.search.read(&found))
            .ok()
            .and_then(|bytes| world_hull(&bytes, &self.materials()))
            .map(Arc::new);
        known.insert(path.to_owned(), hull.clone());
        hull
    }

    /// The file at `path` through the search path, none when it is not
    /// there
    pub fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.search
            .find(path)
            .and_then(|found| self.search.read(&found))
            .ok()
    }

    /// The game's palette (`gfx/palette.lmp`), empty when it cannot be read
    pub fn palette(&self) -> Vec<[u8; 3]> {
        let bytes = self
            .search
            .find("gfx/palette.lmp")
            .and_then(|found| self.search.read(&found))
            .unwrap_or_default();
        bytes.as_chunks::<3>().0.iter().take(256).copied().collect()
    }

    /// How many frames the sprite at `path` has, none when it cannot be
    /// read
    pub fn sprite_frames(&self, path: &str) -> Option<u32> {
        let mut known = self.sprites.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(frames) = known.get(path) {
            return *frames;
        }
        let frames = self
            .search
            .find(path)
            .and_then(|found| self.search.read(&found))
            .and_then(|bytes| sprite::parse(&bytes, self.limits))
            .ok()
            .map(|s| u32::try_from(s.frames.len()).unwrap_or(u32::MAX));
        known.insert(path.to_owned(), frames);
        frames
    }

    /// The sound lengths, model sequences, map worlds and sprite frames
    /// playback reads
    pub fn game_files(self: &Arc<Self>) -> GameFiles {
        let (models, maps, sprites) = (self.clone(), self.clone(), self.clone());
        let decals = self.clone();
        GameFiles {
            lengths: self.lengths(),
            sequences: Arc::new(move |path| models.sequences(path)),
            hulls: Arc::new(move |path| maps.hull(path)),
            sprite_frames: Arc::new(move |path| sprites.sprite_frames(path)),
            decal_index: Arc::new(move |name| decals.decal_index(name)),
        }
    }
}

/// The map's node tree, its leaves by their contents, every BSP model's
/// first node, and its faces with their textures' materials
fn world_hull(bytes: &[u8], materials: &Materials) -> Option<Hull> {
    let bsp = BspDocument::parse(bytes).ok()?;
    let contents: Vec<i32> = bsp.leaves().map(|l| l.contents).collect();
    let child = |c: i32| {
        if c >= 0 {
            Child::Node(c as usize)
        } else {
            let leaf = (-1 - c) as usize;
            Child::Contents(contents.get(leaf).copied().unwrap_or(SOLID))
        }
    };
    let planes = bsp
        .planes()
        .map(|p| (p.normal.map(|f| f.value), p.distance.value))
        .collect();
    let nodes: Vec<_> = bsp.nodes().collect();
    let node_faces = nodes
        .iter()
        .map(|n| {
            let first = usize::from(n.first_face);
            first..first + usize::from(n.face_count)
        })
        .collect();

    let vertices: Vec<[f32; 3]> = bsp.vertices().map(|v| v.point.map(|f| f.value)).collect();
    let edges: Vec<[u16; 2]> = bsp.edges().map(|e| e.vertices).collect();
    let surface_edges: Vec<i32> = bsp.surface_edges().map(|e| e.edge).collect();
    let texinfo: Vec<_> = bsp.texture_info().collect();
    let names: Vec<String> = bsp
        .textures()
        .iter()
        .map(|t| match t {
            MipTexture::Missing { .. } => String::new(),
            MipTexture::External { name, .. } | MipTexture::Embedded { name, .. } => {
                String::from_utf8_lossy(name).into_owned()
            }
        })
        .collect();
    let faces = bsp
        .faces()
        .map(|face| {
            let info = texinfo.get(face.texture_info.max(0) as usize);
            let axes = info.map_or([[0.; 4]; 2], |i| {
                [i.s_axis.map(|f| f.value), i.t_axis.map(|f| f.value)]
            });
            // The texture extent in 16-texel steps (`CalcSurfaceExtents`)
            let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
            let first = face.first_surface_edge.max(0) as usize;
            for k in 0..face.surface_edge_count.max(0) as usize {
                let Some(&e) = surface_edges.get(first + k) else {
                    continue;
                };
                let v = if e >= 0 {
                    edges.get(e as usize).map(|e| e[0])
                } else {
                    edges.get(e.unsigned_abs() as usize).map(|e| e[1])
                };
                let Some(p) = v.and_then(|v| vertices.get(usize::from(v))) else {
                    continue;
                };
                for (k, a) in axes.iter().enumerate() {
                    let st = p[0] * a[0] + p[1] * a[1] + p[2] * a[2] + a[3];
                    lo[k] = lo[k].min(st);
                    hi[k] = hi[k].max(st);
                }
            }
            let mins = lo.map(|v| (v / 16.).floor() * 16.);
            let maxs = hi.map(|v| (v / 16.).ceil() * 16.);
            let name = info
                .and_then(|i| names.get(i.mip_texture.max(0) as usize))
                .map_or("", String::as_str);
            HullFace {
                axes,
                mins,
                extents: [maxs[0] - mins[0], maxs[1] - mins[1]],
                material: materials.of(name),
            }
        })
        .collect();

    Some(Hull {
        planes,
        nodes: nodes
            .iter()
            .map(|n| {
                let children = n.children.map(|c| child(i32::from(c)));
                (n.plane.max(0) as usize, children)
            })
            .collect(),
        models: bsp.models().map(|m| child(m.head_nodes[0])).collect(),
        node_faces,
        faces,
    })
}

/// The textures `sound/materials.txt` names, each with its material
#[derive(Debug, Default)]
pub struct Materials(HashMap<String, u8>);

/// How much of a texture's name the materials file looks at
/// (`CBTEXTURENAMEMAX` less its terminator)
const MATERIAL_NAME: usize = 12;

impl Materials {
    /// Read from the file's lines, a material letter and a texture name
    /// each, `//` starting a comment
    pub fn parse(text: &str) -> Self {
        let mut map = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("//") {
                continue;
            }
            let mut parts = line.split_whitespace();
            let (Some(kind), Some(name)) = (parts.next(), parts.next()) else {
                continue;
            };
            let [kind] = kind.as_bytes() else {
                continue;
            };
            map.insert(material_key(name), kind.to_ascii_uppercase());
        }
        Self(map)
    }

    /// The material of the texture `name`, concrete when the file names
    /// none. A leading `-0` or `+0`, and then one of `{`, `!`, `~` and a
    /// space, are not part of it, as the client's
    /// `EV_HLDM_PlayTextureSound` strips them
    pub fn of(&self, name: &str) -> u8 {
        let mut name = name;
        if name.starts_with(['-', '+']) {
            name = name.get(2..).unwrap_or("");
        }
        if name.starts_with(['{', '!', '~', ' ']) {
            name = &name[1..];
        }
        self.0.get(&material_key(name)).copied().unwrap_or(b'C')
    }
}

fn material_key(name: &str) -> String {
    name.chars()
        .take(MATERIAL_NAME)
        .collect::<String>()
        .to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materials_match_the_first_twelve_letters_of_a_bare_name() {
        let m = Materials::parse("// comment\nV DUCT_FLR01\n//D OUT_RK1\nM METAL_SHEET_LONG\n");
        assert_eq!(m.of("duct_flr01"), b'V');
        assert_eq!(m.of("+0duct_flr01"), b'V');
        assert_eq!(m.of("{duct_flr01"), b'V');
        assert_eq!(m.of("METAL_SHEET_LOOP"), b'M');
        assert_eq!(m.of("OUT_RK1"), b'C');
        assert_eq!(m.of(""), b'C');
    }
}
