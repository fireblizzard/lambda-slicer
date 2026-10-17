//! A map and a player model read through the game's search path
//!
//! A map's textures come from the map itself or from the WADs its
//! `worldspawn` names, searched as the game searches them. A player model
//! is `models/player/<name>/<name>.mdl` with its texture companion and its
//! sequence groups, or the stock `models/player.mdl` when that is missing

use lambda_slicer_assets::search::SearchPath;
use lambda_slicer_assets::wad::Wad;
use lambda_slicer_assets::xenoxide::bsp30::BspDocument;
use lambda_slicer_assets::xenoxide::studio::{StudioModel, StudioRenderAsset};
use lambda_slicer_assets::xenoxide::studio_pose::{
    BoneTransform, Placement, PoseInputs, SequenceState, evaluate_pose,
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

/// A player model ready to pose and draw
pub struct PlayerModel {
    pub model: StudioModel,
    pub mesh: ModelMesh,
    /// The file it was read from
    pub path: String,
}

impl PlayerModel {
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

fn studio(search: &SearchPath, path: &str) -> Result<PlayerModel, LoadError> {
    let invalid = |e: &dyn std::fmt::Debug| LoadError::Model {
        name: path.to_owned(),
        reason: format!("{e:?}"),
    };
    let main = search.read(&search.find(path)?)?;
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
    Ok(PlayerModel {
        mesh: ModelMesh::build(&asset, 0, 0),
        model,
        path: path.to_owned(),
    })
}

/// The player model `name`, or the stock one when it cannot be read
pub fn player(search: &SearchPath, name: &str) -> Result<PlayerModel, LoadError> {
    studio(search, &format!("models/player/{name}/{name}.mdl"))
        .or_else(|_| studio(search, "models/player.mdl"))
}
