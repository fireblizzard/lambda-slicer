//! The demo view's decals, laid on the map as playback makes them
//!
//! Each decal is laid on the map's faces once, when it first appears, and
//! kept by its id until playback drops it, so a new decal costs only its
//! own faces. They are drawn in the order they were made, and while decals
//! are only added the renderer is sent only the new ones. The textures come from `decals.wad`, read once, each decoded
//! when a decal first uses it
//!
//! Of the decals left in play only the newest are drawn, as many as the
//! settings keep, while the map's own always are

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
    /// The playback revision and the most decals the mesh was made for
    seen: Option<(u64, usize)>,
    mesh: Option<Arc<DecalMesh>>,
    revision: u64,
    /// The decals drawn, in the order they were made, their triangles,
    /// their batches and the textures they use
    order: Vec<u64>,
    vertices: Vec<WorldVertex>,
    batches: Vec<DecalBatch>,
    textures: BTreeMap<u16, Rgba>,
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
            order: Vec::new(),
            vertices: Vec::new(),
            batches: Vec::new(),
            textures: BTreeMap::new(),
        }
    }

    /// The map's world to lay decals on, every decal laid again on it
    pub fn set_world(&mut self, world: Arc<WorldMesh>) {
        self.world = Some(world);
        self.laid.clear();
        self.seen = None;
        self.mesh = None;
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

    /// Lay the decals playback holds at `revision`, every one the map placed
    /// and the newest `most` of the others
    pub fn update(&mut self, all: &VecDeque<Decal>, revision: u64, most: usize) {
        if self.seen == Some((revision, most)) {
            return;
        }
        let Some(world) = self.world.clone() else {
            return;
        };
        self.seen = Some((revision, most));
        let decals = newest(all, most);

        for &decal in &decals {
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

        // When the decals drawn so far are still the first ones, the new
        // ones go on the end and only they are sent to the GPU
        let ids: Vec<u64> = decals.iter().map(|d| d.id).collect();
        let only_added = ids.starts_with(&self.order) && self.mesh.is_some();
        if !only_added {
            self.order.clear();
            self.vertices.clear();
            self.batches.clear();
        }
        let kept = self.vertices.len();
        let previous = self
            .mesh
            .as_ref()
            .map(|m| m.revision)
            .filter(|_| only_added);
        for decal in decals.iter().skip(self.order.len()) {
            self.order.push(decal.id);
            let Some(laid) = self.laid.get(&decal.id).filter(|l| !l.triangles.is_empty()) else {
                continue;
            };
            let first = self.vertices.len() as u32;
            let count = laid.triangles.len() as u32;
            self.vertices.extend_from_slice(&laid.triangles);
            match self.batches.last_mut() {
                // A run of decals on one model in one texture is one batch
                Some(b) if b.model == laid.model && b.texture == laid.texture => b.count += count,
                _ => self.batches.push(DecalBatch {
                    model: laid.model,
                    texture: laid.texture,
                    first,
                    count,
                }),
            }
            if let Some(Some(image)) = self.images.get(&laid.texture) {
                self.textures
                    .entry(laid.texture)
                    .or_insert_with(|| image.clone());
            }
        }

        self.revision += 1;
        self.mesh = Some(Arc::new(DecalMesh {
            revision: self.revision,
            previous,
            kept,
            vertices: self.vertices.clone(),
            batches: self.batches.clone(),
            textures: Arc::new(self.textures.clone()),
        }));
    }
}

/// Every decal the map placed and the newest `most` of the others, oldest
/// first
fn newest(decals: &VecDeque<Decal>, most: usize) -> Vec<&Decal> {
    let mut left = most;
    let mut kept = Vec::new();
    for decal in decals.iter().rev() {
        if decal.map {
            kept.push(decal);
        } else if left > 0 {
            kept.push(decal);
            left -= 1;
        }
    }

    kept.reverse();
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decal(id: u64, map: bool) -> Decal {
        Decal {
            id,
            texture: 0,
            model: None,
            entity: 0,
            origin: [0.; 3],
            map,
        }
    }

    #[test]
    fn the_maps_decals_stay_and_only_the_newest_others_do() {
        let decals: VecDeque<Decal> = [
            decal(0, true),
            decal(1, false),
            decal(2, false),
            decal(3, true),
            decal(4, false),
        ]
        .into();

        let ids = |most| -> Vec<u64> { newest(&decals, most).iter().map(|d| d.id).collect() };
        assert_eq!(ids(2), [0, 2, 3, 4]);
        assert_eq!(ids(0), [0, 3]);
        assert_eq!(ids(10), [0, 1, 2, 3, 4]);
    }
}
