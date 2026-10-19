//! Maps and studio models read through the game's search path
//!
//! A map's textures come from the map itself or from the WADs its
//! `worldspawn` names, searched as the game searches them. A studio model
//! comes with its texture companion and its sequence groups. A player's
//! model is `models/player/<name>/<name>.mdl`, or the stock
//! `models/player.mdl` when that is missing

use lambda_slicer_assets::search::SearchPath;
use lambda_slicer_assets::sprite::{self, Sprite};
use lambda_slicer_assets::tga;
use lambda_slicer_assets::wad::Wad;
use lambda_slicer_assets::xenoxide::bsp30::BspDocument;
use lambda_slicer_assets::xenoxide::studio::StudioSequence;
use lambda_slicer_assets::xenoxide::studio::{StudioModel, StudioRenderAsset};
use lambda_slicer_assets::xenoxide::studio_pose::{
    BoneTransform, GaitState, Placement, PoseInputs, SequenceState, estimate_frame, evaluate_pose,
    player_blend,
};
use lambda_slicer_assets::{AssetError, Limits};

use crate::Rgba;
use crate::model::ModelMesh;
use crate::world::{self, WorldMesh};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(transparent)]
    Asset(#[from] AssetError),
    #[error("{name} is not a map this version reads ({reason})")]
    Map { name: String, reason: String },
    #[error("{name} is not a model this version reads ({reason})")]
    Model { name: String, reason: String },
    /// A sprite under a model's name, as AG ships some items, which the
    /// game draws as a sprite
    #[error("{name} is a sprite")]
    Sprite { name: String },
}

/// `maps/<name>.bsp` as a mesh, with the WADs that could not be read
pub fn map(
    search: &SearchPath,
    name: &str,
    limits: Limits,
) -> Result<(WorldMesh, Vec<String>), LoadError> {
    let path = format!("maps/{name}.bsp");
    let bytes = search.read(&search.find(&path)?)?;
    let bsp = BspDocument::parse(&bytes).map_err(|e| LoadError::Map {
        name: path.clone(),
        reason: format!("{e:?}"),
    })?;

    let mut wads = Vec::new();
    let mut unread = Vec::new();
    for wad in world::wad_names(&world::entities(bsp.entity_bytes())) {
        match search
            .find(&wad)
            .and_then(|found| search.read(&found))
            .and_then(|bytes| Wad::parse(bytes, limits))
        {
            Ok(w) => wads.push(w),
            Err(_) => unread.push(wad),
        }
    }
    let mut from_wads = |texture: &str| {
        wads.iter().find_map(|wad| {
            let t = wad.mip_texture(wad.find(texture)?).ok()?;
            let transparent = t.name.starts_with('{').then_some(255);
            Some(Rgba::from_palette(
                t.width,
                t.height,
                &t.indices,
                &t.palette,
                transparent,
            ))
        })
    };
    Ok((world::build(&bsp, &mut from_wads), unread))
}

/// A player's legs as playback keeps them
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaitPose {
    pub sequence: usize,
    /// The way the legs face, in degrees
    pub yaw: f32,
    /// The torso controller value, 0 to 255
    pub torso: f32,
    /// Units moved and seconds passed in the player's life
    pub distance: f64,
    pub elapsed: f64,
}

/// What an entity gives a studio model's pose
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityPose {
    pub sequence: usize,
    /// The entity's `frame`, 0 to 256 through the sequence
    pub cycle: f64,
    /// Seconds from the entity's `animtime` to the client's time
    pub elapsed: f64,
    pub framerate: f64,
    /// In byte units, 0 to 255
    pub controllers: [f64; 4],
    pub blending: [f64; 2],
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    /// A player, posed as `StudioDrawPlayer` poses one
    pub player: bool,
    pub gait: Option<GaitPose>,
}

/// An angle wrapped into -180 to 180
fn half_turn(x: f64) -> f64 {
    let x = x - (x / 360.).trunc() * 360.;
    if x > 180. {
        x - 360.
    } else if x < -180. {
        x + 360.
    } else {
        x
    }
}

/// The legs' frame in a gait sequence, from the distance moved through its
/// linear movement, or from the time passed in one that does not move, as
/// `StudioProcessGait` steps it
fn gait_frame(sequence: &StudioSequence, gait: &GaitPose) -> f64 {
    let frames = sequence.frames as f64;
    if frames <= 0. {
        return 0.;
    }
    let linear = f64::from(sequence.linear_movement[0]);
    let frame = if linear > 0. {
        gait.distance / linear * frames
    } else {
        f64::from(sequence.fps) * gait.elapsed
    };
    let frame = frame - (frame / frames).trunc() * frames;
    if frame < 0. { frame + frames } else { frame }
}

/// The sky box `name` from `gfx/env`, its faces in the order the renderer
/// draws them ([`crate::gpu::SKY_FACES`])
pub fn sky(search: &SearchPath, name: &str, limits: Limits) -> Result<[Rgba; 6], LoadError> {
    let mut faces = Vec::with_capacity(6);
    for suffix in crate::gpu::SKY_FACES {
        let path = format!("gfx/env/{name}{suffix}.tga");
        let image = tga::decode(&search.read(&search.find(&path)?)?, limits)?;
        faces.push(Rgba {
            width: image.width,
            height: image.height,
            pixels: image.rgba,
        });
    }
    Ok(faces.try_into().expect("six faces"))
}

/// The sprite at `path`, a `.spr` or one of AG's sprites named as a model
pub fn sprite(search: &SearchPath, path: &str, limits: Limits) -> Result<Sprite, LoadError> {
    Ok(sprite::parse(&search.read(&search.find(path)?)?, limits)?)
}

/// A studio model ready to pose and draw
pub struct LoadedModel {
    pub model: StudioModel,
    pub mesh: ModelMesh,
    /// What the mesh was built from, for building it again in a player's
    /// colours
    pub asset: StudioRenderAsset,
    /// The file it was read from
    pub path: String,
}

impl LoadedModel {
    /// The bones for an entity, its frame estimated from its cycle and the
    /// time since its animation time as the renderer does. A sequence the
    /// model lacks falls back to its first. None when the pose cannot be
    /// evaluated
    pub fn pose(&self, entity: &EntityPose) -> Option<Vec<BoneTransform>> {
        let (inputs, placement) = self.inputs(entity)?;
        evaluate_pose(&self.model, &inputs, &placement)
            .ok()
            .map(|pose| pose.bones().to_vec())
    }

    /// The pose inputs and placement for an entity. A player with a gait
    /// sequence aims through its first blend, faces the way its legs do with
    /// its torso turned to its view, and walks its legs. One without keeps
    /// its controllers centred
    fn inputs(&self, entity: &EntityPose) -> Option<(PoseInputs, Placement)> {
        let sequences = self.model.sequences();
        let sequence = if entity.sequence < sequences.len() {
            entity.sequence
        } else {
            0
        };
        let s = sequences.get(sequence)?;
        let advance = entity.elapsed.max(0.) * entity.framerate * f64::from(s.fps);

        let mut blend = entity.blending;
        let mut controllers = entity.controllers;
        let mut angles = entity.angles.map(f64::from);
        let mut gait = None;
        if entity.player {
            match entity.gait {
                Some(g) => {
                    let (aim, rest) = player_blend(s, half_turn(angles[0]));
                    blend[0] = aim;
                    angles[0] = rest;
                    controllers = [f64::from(g.torso); 4];
                    let yaw = f64::from(g.yaw);
                    angles[1] = if yaw < 0. { yaw + 360. } else { yaw };
                    gait = sequences.get(g.sequence).map(|legs| GaitState {
                        sequence: g.sequence,
                        frame: gait_frame(legs, &g),
                    });
                }
                None => controllers = [127.; 4],
            }
        }

        let inputs = PoseInputs {
            current: SequenceState {
                sequence,
                frame: estimate_frame(s, entity.cycle, advance),
                blend,
            },
            controllers,
            mouth: 0.,
            transition: None,
            gait,
        };
        let placement = Placement {
            origin: entity.origin.map(f64::from),
            angles,
        };
        Some((inputs, placement))
    }

    /// The bones of a model a player holds, such as a `p_` weapon, as
    /// `StudioMergeBones` makes them. A bone named as one of the player's,
    /// ignoring case, takes that bone's transform. Any other is posed with
    /// the player's sequence and frame and goes on from its parent as merged,
    /// so a weapon's own bones follow the player's hand even where its arm's
    /// bones are named otherwise, as AG's `p_` models name them
    pub fn merged_pose(
        &self,
        entity: &EntityPose,
        holder: &LoadedModel,
        holder_bones: &[BoneTransform],
    ) -> Option<Vec<BoneTransform>> {
        let (_, placement) = holder.inputs(entity)?;
        let own = EntityPose {
            player: false,
            gait: None,
            angles: placement.angles.map(|a| a as f32),
            ..*entity
        };
        let own_bones = self.pose(&own)?;
        let names = holder.model.bones();

        // Bones come after their parents, so a parent is merged first
        let mut bones: Vec<BoneTransform> = Vec::with_capacity(own_bones.len());
        for (i, bone) in self.model.bones().iter().enumerate() {
            let matched = names
                .iter()
                .position(|b| b.name.eq_ignore_ascii_case(&bone.name))
                .and_then(|j| holder_bones.get(j));
            let own_transform = *own_bones.get(i)?;
            let transform = match (matched, bone.parent) {
                (Some(from), _) => *from,
                // Where the bone sits from its parent in its own pose,
                // carried by the parent as merged
                (None, Some(parent)) => {
                    let local = concat(&inverse(own_bones.get(parent)?), &own_transform);
                    concat(bones.get(parent)?, &local)
                }
                (None, None) => own_transform,
            };
            bones.push(transform);
        }
        Some(bones)
    }

    /// Where attachment `index` is on the model posed with `bones`, its
    /// point carried by its bone as `StudioCalcAttachments` carries it
    pub fn attachment(&self, bones: &[BoneTransform], index: usize) -> Option<[f32; 3]> {
        let a = self.model.attachments().get(index)?;
        let m = bones.get(a.bone)?;
        let o = a.origin.map(f64::from);
        Some([0, 1, 2].map(|r| (m[r][0] * o[0] + m[r][1] * o[1] + m[r][2] * o[2] + m[r][3]) as f32))
    }

    /// The sequence labelled `label`, ignoring case
    pub fn sequence(&self, label: &str) -> Option<usize> {
        self.model
            .sequences()
            .iter()
            .position(|s| s.label.eq_ignore_ascii_case(label))
    }

    /// The bones `seconds` into `sequence`, looped, for a model at `origin`
    /// turned `yaw` degrees. Controllers and blends sit in the middle of
    /// their ranges. None when the pose cannot be evaluated
    pub fn bones_at(
        &self,
        sequence: usize,
        seconds: f64,
        origin: [f32; 3],
        yaw: f32,
    ) -> Option<Vec<BoneTransform>> {
        let s = self.model.sequences().get(sequence)?;
        let last = s.frames.saturating_sub(1).max(1) as f64;
        let inputs = PoseInputs {
            current: SequenceState {
                sequence,
                frame: (seconds * f64::from(s.fps)).rem_euclid(last),
                blend: [127.5; 2],
            },
            controllers: [127.5; 4],
            mouth: 0.,
            transition: None,
            gait: None,
        };
        let placement = Placement {
            origin: origin.map(f64::from),
            angles: [0., f64::from(yaw), 0.],
        };
        evaluate_pose(&self.model, &inputs, &placement)
            .ok()
            .map(|pose| pose.bones().to_vec())
    }
}

/// The studio model at `path`, such as `models/w_gauss.mdl`
pub fn model(search: &SearchPath, path: &str) -> Result<LoadedModel, LoadError> {
    let invalid = |e: &dyn std::fmt::Debug| LoadError::Model {
        name: path.to_owned(),
        reason: format!("{e:?}"),
    };
    let main = search.read(&search.find(path)?)?;
    if main.starts_with(b"IDSP") {
        return Err(LoadError::Sprite {
            name: path.to_owned(),
        });
    }
    let unresolved = StudioModel::parse(&main).map_err(|e| invalid(&e))?;
    let mut groups = Vec::new();
    for d in unresolved.dependencies() {
        groups.push((d.index, search.read(&search.find(&d.name)?)?));
    }
    let files: Vec<(usize, &[u8])> = groups.iter().map(|(i, b)| (*i, b.as_slice())).collect();
    let model = unresolved.resolve(&files).map_err(|e| invalid(&e))?;
    let companion = match model.texture_companion() {
        true => {
            let name = format!("{}T.mdl", path.strip_suffix(".mdl").unwrap_or(path));
            Some(search.read(&search.find(&name)?)?)
        }
        false => None,
    };
    let asset =
        StudioRenderAsset::parse(&model, &main, companion.as_deref()).map_err(|e| invalid(&e))?;
    Ok(LoadedModel {
        mesh: ModelMesh::build(&asset, 0, 0),
        asset,
        model,
        path: path.to_owned(),
    })
}

/// `first` applied after `second`, as the engine's `ConcatTransforms`
fn concat(first: &BoneTransform, second: &BoneTransform) -> BoneTransform {
    let mut out = [[0.; 4]; 3];
    for row in 0..3 {
        for column in 0..4 {
            let carried = first[row][0] * second[0][column]
                + first[row][1] * second[1][column]
                + first[row][2] * second[2][column];
            out[row][column] = if column == 3 {
                carried + first[row][3]
            } else {
                carried
            };
        }
    }
    out
}

/// The transform undoing `m`, a bone's rotation and translation, whose
/// rotation's inverse is its transpose
fn inverse(m: &BoneTransform) -> BoneTransform {
    let mut out = [[0.; 4]; 3];
    for row in 0..3 {
        for column in 0..3 {
            out[row][column] = m[column][row];
        }
    }
    // The translation undone through the inverse rotation
    for row in &mut out {
        row[3] = -(row[0] * m[0][3] + row[1] * m[1][3] + row[2] * m[2][3]);
    }
    out
}

/// The player model `name`, or the stock one when it cannot be read
pub fn player(search: &SearchPath, name: &str) -> Result<LoadedModel, LoadError> {
    model(search, &format!("models/player/{name}/{name}.mdl"))
        .or_else(|_| model(search, "models/player.mdl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use lambda_slicer_assets::xenoxide::studio::fixture::{
        BodyPart, Fixture, Mesh, Submodel, Texture,
    };

    /// The standard test model, a root, a spine and a head, with bones
    /// renamed by `rename` and the spine `spine` units above the root
    fn loaded(
        rename: impl Fn(&mut Vec<lambda_slicer_assets::xenoxide::studio::fixture::Bone>),
        spine: f32,
    ) -> LoadedModel {
        let mut fixture = Fixture::standard();
        fixture.bones[1].value[2] = spine;
        // A texture and a triangle, so the model has what drawing it needs
        fixture.texture_data = vec![Texture::solid("plain.bmp", 4, 4, 1, [128, 128, 128])];
        fixture.skins = vec![vec![0]];
        fixture.body_parts = vec![BodyPart {
            name: "body",
            base: 1,
            models: vec![Submodel {
                name: "plate",
                vertices: vec![([0., 0., 0.], 2), ([0., 1., 0.], 2), ([0., 0., 1.], 2)],
                normals: vec![([-1., 0., 0.], 2)],
                meshes: vec![Mesh {
                    skin_reference: 0,
                    commands: vec![(3, vec![[0, 0, 0, 0], [1, 0, 4, 0], [2, 0, 0, 4]])],
                    declared: None,
                }],
            }],
        }];
        rename(&mut fixture.bones);
        let built = fixture.build();
        let model = StudioModel::parse(&built.bytes)
            .unwrap()
            .resolve(&[])
            .unwrap();
        let asset = StudioRenderAsset::parse(&model, &built.bytes, None).unwrap();
        LoadedModel {
            mesh: ModelMesh::build(&asset, 0, 0),
            asset,
            model,
            path: "fixture.mdl".to_owned(),
        }
    }

    #[test]
    fn a_held_models_own_bones_follow_its_holders_matching_bones() {
        // The holder's spine stands 40 units above its root, the weapon's
        // 10, and the weapon names its spine in capitals and has a mesh bone
        // of its own where the holder has a head
        let holder = loaded(|_| {}, 40.);
        let weapon = loaded(
            |bones| {
                bones[1].name = "BIP01 SPINE";
                bones[2].name = "Weapon Mesh";
            },
            10.,
        );
        let entity = EntityPose {
            sequence: 0,
            cycle: 0.,
            elapsed: 0.,
            framerate: 1.,
            controllers: [127.5; 4],
            blending: [0.; 2],
            origin: [0.; 3],
            angles: [0.; 3],
            player: false,
            gait: None,
        };
        let holder_bones = holder.pose(&entity).unwrap();
        let own = weapon.pose(&entity).unwrap();
        let merged = weapon.merged_pose(&entity, &holder, &holder_bones).unwrap();

        // The spine is the holder's, matched whatever the case
        assert_eq!(merged[1], holder_bones[1]);

        // The mesh bone keeps its place from the spine, so it moves with the
        // holder's spine
        let offset = |bones: &[BoneTransform]| [0, 1, 2].map(|k| bones[2][k][3] - bones[1][k][3]);
        let (want, got) = (offset(&own), offset(&merged));
        for k in 0..3 {
            assert!((want[k] - got[k]).abs() < 1e-6, "{want:?} against {got:?}");
        }
        assert!((merged[2][2][3] - own[2][2][3] - 30.).abs() < 1e-6);
    }
}
