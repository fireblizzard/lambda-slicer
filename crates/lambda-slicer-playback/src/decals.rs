//! The decals the server asks the client to put on the map (implementation
//! plan §13.6)
//!
//! A decal is a texture from `decals.wad` stamped on the surfaces about a
//! point (`R_DecalShoot`). The server names it by its place in the wad's
//! directory, which is the same on both ends as both read the same file, so
//! no `svc_decalname` is sent. One on a brush entity, such as a door, keeps
//! its place on the brush as the brush moves. One aimed at any other entity
//! is dropped, as the engine puts decals only on brushes. The engine keeps
//! the newest 4096 (`MAX_DECALS`) and a new session clears them.
//!
//! Playback keeps where each decal is. Where its texture lands on the
//! surfaces is the renderer's to work out

use std::collections::VecDeque;
use std::sync::Arc;

use crate::scene::Scene;

/// As many decals as the engine keeps
pub const MAX_DECALS: usize = 4096;

/// `svc_temp_entity` kinds that make decals
mod kind {
    pub const BSP_DECAL: u8 = 13;
    pub const DECAL: u8 = 104;
    pub const GUNSHOT_DECAL: u8 = 109;
    pub const WORLD_DECAL: u8 = 116;
    pub const WORLD_DECAL_HIGH: u8 = 117;
    pub const DECAL_HIGH: u8 = 118;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decal {
    /// Unique within one playback, in the order they were made
    pub id: u64,
    /// Its place in `decals.wad`'s directory
    pub texture: u16,
    /// The brush model it is on, `*1` and on, none for the world
    pub model: Option<Arc<str>>,
    /// The entity carrying that brush, 0 for the world
    pub entity: u16,
    /// Where it was aimed, in the brush's own space
    pub origin: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub struct Decals {
    /// Shared until it changes, as every frame presents it
    list: Arc<VecDeque<Decal>>,
    next_id: u64,
    /// Grows whenever a decal is made or the list is cleared
    pub revision: u64,
}

fn short(b: &[u8], at: usize) -> i16 {
    i16::from_le_bytes([
        b.get(at).copied().unwrap_or(0),
        b.get(at + 1).copied().unwrap_or(0),
    ])
}

fn vector(b: &[u8]) -> [f32; 3] {
    [0, 2, 4].map(|at| f32::from(short(b, at)) / 8.)
}

impl Decals {
    pub fn list(&self) -> &Arc<VecDeque<Decal>> {
        &self.list
    }

    pub(crate) fn clear(&mut self) {
        Arc::make_mut(&mut self.list).clear();
        self.revision += 1;
    }

    /// A decal from an `svc_temp_entity` of `kind`, if it makes one.
    /// `model` names a precached model by its index
    pub(crate) fn message(
        &mut self,
        kind: u8,
        body: &[u8],
        scene: &Scene,
        model: &dyn Fn(i16) -> Option<Arc<str>>,
    ) {
        let byte = |at: usize| u16::from(body.get(at).copied().unwrap_or(0));
        let origin = vector(body);
        let (texture, entity, named) = match kind {
            kind::WORLD_DECAL => (byte(6), 0, None),
            kind::WORLD_DECAL_HIGH => (byte(6) + 256, 0, None),
            kind::DECAL => (byte(6), short(body, 7), None),
            kind::DECAL_HIGH => (byte(6) + 256, short(body, 7), None),
            kind::GUNSHOT_DECAL => (byte(8), short(body, 6), None),
            kind::BSP_DECAL => {
                let entity = short(body, 8);
                let named = (entity != 0).then(|| model(short(body, 10))).flatten();
                (short(body, 6) as u16, entity, named)
            }
            _ => return,
        };
        let entity = u16::try_from(entity).unwrap_or(0);
        let (model, origin) = if entity == 0 {
            (None, origin)
        } else {
            // On a brush entity, in the brush's own space
            let Some(e) = scene.entities.iter().find(|e| e.id.number == entity) else {
                return;
            };
            let Some(brush) = named
                .or_else(|| e.model.clone())
                .filter(|m| m.starts_with('*'))
            else {
                return;
            };
            (Some(brush), [0, 1, 2].map(|k| origin[k] - e.origin[k]))
        };

        let list = Arc::make_mut(&mut self.list);
        if list.len() >= MAX_DECALS {
            list.pop_front();
        }
        list.push_back(Decal {
            id: self.next_id,
            texture,
            model,
            entity,
            origin,
        });
        self.next_id += 1;
        self.revision += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(origin: [i16; 3], rest: &[u8]) -> Vec<u8> {
        let mut b: Vec<u8> = origin.iter().flat_map(|c| (c * 8).to_le_bytes()).collect();
        b.extend_from_slice(rest);
        b
    }

    #[test]
    fn world_and_gunshot_decals_keep_their_texture_and_place() {
        let mut decals = Decals::default();
        let scene = Scene::default();
        let none = |_| None;
        decals.message(kind::WORLD_DECAL, &body([1, 2, 3], &[27]), &scene, &none);
        decals.message(kind::WORLD_DECAL_HIGH, &body([0; 3], &[4]), &scene, &none);
        decals.message(
            kind::GUNSHOT_DECAL,
            &body([5, 6, 7], &[0, 0, 21]),
            &scene,
            &none,
        );
        let list: Vec<_> = decals
            .list()
            .iter()
            .map(|d| (d.texture, d.origin))
            .collect();
        assert_eq!(
            list,
            [(27, [1., 2., 3.]), (260, [0.; 3]), (21, [5., 6., 7.])]
        );
        // One aimed at an entity the scene does not hold is dropped
        decals.message(kind::DECAL, &body([0; 3], &[1, 9, 0]), &scene, &none);
        assert_eq!(decals.list().len(), 3);
    }

    #[test]
    fn the_oldest_decal_makes_room() {
        let mut decals = Decals::default();
        let scene = Scene::default();
        for _ in 0..MAX_DECALS + 2 {
            decals.message(kind::WORLD_DECAL, &body([0; 3], &[1]), &scene, &|_| None);
        }
        assert_eq!(decals.list().len(), MAX_DECALS);
        assert_eq!(decals.list()[0].id, 2);
    }
}
