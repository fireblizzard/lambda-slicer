//! The decals the server asks the client to put on the map
//!
//! A decal is a texture from `decals.wad` stamped on the surfaces about a
//! point (`R_DecalShoot`). The server numbers it by its own list of decals,
//! which can differ from the client's, as the engine merges the game's and
//! the mod's `decals.wad` and a server may carry decals the client lacks.
//! The session's resource list names each number, and the client finds the
//! decal by that name in its own files (`Draw_DecalSetName`). One on a
//! brush entity, such as a door, keeps
//! its place on the brush as the brush moves. One aimed at any other entity
//! is dropped, as the engine puts decals only on brushes. The engine keeps
//! the newest 4096 (`MAX_DECALS`) and a new session clears them.
//!
//! The map's own decals, its `infodecal` entities, come as `TE_BSPDECAL`
//! (`CDecal` in HLSDK `dlls/effects.cpp`), so they can be told apart from
//! the marks left in play.
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
    /// Placed by the map itself rather than left in play
    pub map: bool,
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
    /// `model` names a precached model by its index, and `texture` finds
    /// the client's place in `decals.wad` of the decal the server numbers,
    /// none for one the client lacks, which is left out
    pub(crate) fn message(
        &mut self,
        kind: u8,
        body: &[u8],
        scene: &Scene,
        model: &dyn Fn(i16) -> Option<Arc<str>>,
        texture: &dyn Fn(u16) -> Option<u16>,
    ) {
        let byte = |at: usize| u16::from(body.get(at).copied().unwrap_or(0));
        let origin = vector(body);
        let (number, entity, named) = match kind {
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
        let Some(texture) = texture(number) else {
            return;
        };

        let entity = u16::try_from(entity).unwrap_or(0);
        let map = kind == kind::BSP_DECAL;
        self.shoot(texture, entity, origin, named, map, scene);
    }

    /// A decal of texture `texture` aimed at `origin` on entity `entity`,
    /// the world when 0, its brush named by `named` or by the entity. `map`
    /// says the map placed it
    pub(crate) fn shoot(
        &mut self,
        texture: u16,
        entity: u16,
        origin: [f32; 3],
        named: Option<Arc<str>>,
        map: bool,
        scene: &Scene,
    ) {
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
            map,
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
        let same = |n| Some(n);
        decals.message(
            kind::WORLD_DECAL,
            &body([1, 2, 3], &[27]),
            &scene,
            &none,
            &same,
        );
        decals.message(
            kind::WORLD_DECAL_HIGH,
            &body([0; 3], &[4]),
            &scene,
            &none,
            &same,
        );
        decals.message(
            kind::GUNSHOT_DECAL,
            &body([5, 6, 7], &[0, 0, 21]),
            &scene,
            &none,
            &same,
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
        decals.message(kind::DECAL, &body([0; 3], &[1, 9, 0]), &scene, &none, &same);
        assert_eq!(decals.list().len(), 3);
    }

    #[test]
    fn only_bsp_decals_are_the_maps() {
        let mut decals = Decals::default();
        let scene = Scene::default();
        decals.message(
            kind::BSP_DECAL,
            &body([0; 3], &[5, 0, 0, 0]),
            &scene,
            &|_| None,
            &Some,
        );
        decals.message(
            kind::WORLD_DECAL,
            &body([0; 3], &[6]),
            &scene,
            &|_| None,
            &Some,
        );

        let made: Vec<(u16, bool)> = decals.list().iter().map(|d| (d.texture, d.map)).collect();
        assert_eq!(made, [(5, true), (6, false)]);
    }

    #[test]
    fn a_decal_is_found_by_the_name_the_server_gives_its_number() {
        let mut decals = Decals::default();
        let scene = Scene::default();

        // The server's 192 is the client's 32, and its 7 the client lacks
        let texture = |n: u16| match n {
            192 => Some(32),
            _ => None,
        };
        decals.message(
            kind::WORLD_DECAL,
            &body([0; 3], &[192]),
            &scene,
            &|_| None,
            &texture,
        );
        decals.message(
            kind::WORLD_DECAL,
            &body([0; 3], &[7]),
            &scene,
            &|_| None,
            &texture,
        );

        let textures: Vec<u16> = decals.list().iter().map(|d| d.texture).collect();
        assert_eq!(textures, [32]);
    }

    #[test]
    fn the_oldest_decal_makes_room() {
        let mut decals = Decals::default();
        let scene = Scene::default();
        for _ in 0..MAX_DECALS + 2 {
            decals.message(
                kind::WORLD_DECAL,
                &body([0; 3], &[1]),
                &scene,
                &|_| None,
                &Some,
            );
        }
        assert_eq!(decals.list().len(), MAX_DECALS);
        assert_eq!(decals.list()[0].id, 2);
    }
}
