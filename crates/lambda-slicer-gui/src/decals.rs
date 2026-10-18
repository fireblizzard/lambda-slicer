//! The demo view's decals, laid on the map as playback makes them
//!
//! Each decal is laid on the map's faces once, when it first appears, and
//! kept by its id until playback drops it, so a new decal costs only its
//! own faces. The textures come from `decals.wad`, read once, each decoded
//! when a decal first uses it

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

use lambda_slicer_assets::wad::Wad;
use lambda_slicer_playback::decals::Decal;
use lambda_slicer_render::Rgba;
use lambda_slicer_render::decal::{DecalBatch, DecalMesh, decal_rgba};
use lambda_slicer_render::world::{WorldMesh, WorldVertex};

/// One decal laid on its model
struct Laid {
    model: usize,
    texture: u16,
    triangles: Vec<WorldVertex>,
}

pub struct DecalCache {
    wad: Option<Wad>,
    images: HashMap<u16, Option<Rgba>>,
    laid: HashMap<u64, Laid>,
    world: Option<Arc<WorldMesh>>,
    /// The playback revision the mesh was made for
    seen: Option<u64>,
    mesh: Option<Arc<DecalMesh>>,
    revision: u64,
}

impl DecalCache {
    pub fn new(wad: Option<Wad>) -> Self {
        Self {
            wad,
            images: HashMap::new(),
            laid: HashMap::new(),
            world: None,
            seen: None,
            mesh: None,
            revision: 0,
        }
    }

    /// The map's world to lay decals on, every decal laid again on it
    pub fn set_world(&mut self, world: Arc<WorldMesh>) {
        self.world = Some(world);
        self.laid.clear();
        self.seen = None;
    }

    /// What the renderer draws
    pub fn mesh(&self) -> Option<Arc<DecalMesh>> {
        self.mesh.clone()
    }

    /// The image of the decal at `index` in `decals.wad`
    fn image(&mut self, index: u16) -> Option<&Rgba> {
        let wad = &self.wad;
        self.images
            .entry(index)
            .or_insert_with(|| {
                let wad = wad.as_ref()?;
                let texture = wad.mip_texture(usize::from(index)).ok()?;
                Some(decal_rgba(&texture))
            })
            .as_ref()
    }

    /// Lay the decals playback holds at `revision`
    pub fn update(&mut self, decals: &VecDeque<Decal>, revision: u64) {
        if self.seen == Some(revision) {
            return;
        }
        let Some(world) = self.world.clone() else {
            return;
        };
        self.seen = Some(revision);

        for decal in decals {
            if self.laid.contains_key(&decal.id) {
                continue;
            }
            let model = match decal.model.as_deref() {
                None => 0,
                Some(name) => match name.trim_start_matches('*').parse() {
                    Ok(n) => n,
                    Err(_) => continue,
                },
            };
            let triangles = match self.image(decal.texture) {
                Some(image) => {
                    let size = [image.width as f32, image.height as f32];
                    world.decal(model, decal.origin, size)
                }
                None => Vec::new(),
            };
            self.laid.insert(
                decal.id,
                Laid {
                    model,
                    texture: decal.texture,
                    triangles,
                },
            );
        }
        let alive: HashSet<u64> = decals.iter().map(|d| d.id).collect();
        self.laid.retain(|id, _| alive.contains(id));

        // Grouped by model and texture, each group in the order the decals
        // were made
        let mut groups: BTreeMap<(usize, u16), Vec<WorldVertex>> = BTreeMap::new();
        for decal in decals {
            if let Some(laid) = self.laid.get(&decal.id)
                && !laid.triangles.is_empty()
            {
                groups
                    .entry((laid.model, laid.texture))
                    .or_default()
                    .extend_from_slice(&laid.triangles);
            }
        }
        let mut vertices = Vec::new();
        let mut batches = Vec::new();
        let mut textures = BTreeMap::new();
        for ((model, texture), triangles) in groups {
            if let Some(Some(image)) = self.images.get(&texture) {
                textures.insert(texture, image.clone());
            }
            batches.push(DecalBatch {
                model,
                texture,
                first: vertices.len() as u32,
                count: triangles.len() as u32,
            });
            vertices.extend(triangles);
        }
        self.revision += 1;
        self.mesh = Some(Arc::new(DecalMesh {
            revision: self.revision,
            vertices,
            batches,
            textures: Arc::new(textures),
        }));
    }
}
