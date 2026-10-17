//! A studio model's rendering data: body parts, meshes and textures.
//!
//! Read from the same main file as the pose and hitbox data ([`StudioModel`]), with
//! the textures and skin families from the main file or, when it has none, from its
//! texture companion (`<name>T.mdl`). The companion is a rendering dependency only, so
//! this asset has its own identity, distinct from the pose and hitbox identity.
//!
//! Layouts, from the same pinned `engine/studio.h`: body parts (`mstudiobodyparts_t`,
//! 76 bytes), models (`mstudiomodel_t`, 112), meshes (`mstudiomesh_t`, 20), textures
//! (`mstudiotexture_t`, 80, each followed in the file by its 8-bit pixels and a 768-byte
//! palette), and the skin table (`short[families][references]`). A mesh's triangles are
//! commands of 16-bit values: a count, positive for a strip and negative for a fan, then
//! that many corners of four values (vertex, normal, s, t), until a zero count. Strips
//! and fans are decoded in OpenGL's order. Every stock mesh's decoded triangle count
//! equals the count it declares, and so must every mesh read here.

use sha2::{Digest, Sha256};

use super::{
    HEADER_SIZE, STUDIO_HEADER_ID, StudioDigest, StudioError, StudioModel, check_file,
    finite_array, i32_at, name_at, table,
};

/// `MAXSTUDIOBODYPARTS`
pub const MAX_STUDIO_BODY_PARTS: usize = 32;
/// `MAXSTUDIOMODELS`, per body part
pub const MAX_STUDIO_MODELS: usize = 32;
/// `MAXSTUDIOMESHES`, per model
pub const MAX_STUDIO_MESHES: usize = 256;
/// `MAXSTUDIOVERTS`, vertices and normals each, per model
pub const MAX_STUDIO_VERTICES: usize = 2048;
/// `MAXSTUDIOTRIANGLES`, per model
pub const MAX_STUDIO_TRIANGLES: usize = 20_000;
/// `MAXSTUDIOSKINS`, textures and skin references each
pub const MAX_STUDIO_TEXTURES: usize = 100;
/// No SDK limit; ours (stock models have at most 6)
pub const MAX_STUDIO_SKIN_FAMILIES: usize = 100;
/// No SDK limit; ours (stock textures are at most 512)
pub const MAX_STUDIO_TEXTURE_DIMENSION: usize = 1024;

/// Triangles in the whole asset, across every body part and model (ours; the stock
/// maximum is 3423, `loader.mdl`). Each becomes three 40-byte corners on the GPU.
pub const MAX_STUDIO_ASSET_TRIANGLES: usize = 32_768;
/// Vertices and normals in the whole asset (ours; the stock maximum is 4518)
pub const MAX_STUDIO_ASSET_POINTS: usize = 65_536;
/// Texture pixels in the whole asset (ours; the stock maximum is 1195448,
/// `lada5.mdl`). Each becomes four bytes in the atlas.
pub const MAX_STUDIO_ASSET_TEXTURE_PIXELS: usize = 4 * 1024 * 1024;
const BODY_PART_SIZE: usize = 76;
const SUBMODEL_SIZE: usize = 112;
const MESH_SIZE: usize = 20;
const TEXTURE_SIZE: usize = 80;
const PALETTE_SIZE: usize = 768;

/// What a model's rendering depends on: its pose and hitbox identity, and the texture
/// companion when its textures are there
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioRenderIdentity {
    /// `StudioModelIdentity::combined`
    pub model: StudioDigest,
    pub texture_companion: Option<StudioDigest>,
    /// SHA-256 over a fixed tag, the model identity, and a byte saying whether a
    /// companion follows, then its digest
    pub combined: StudioDigest,
}

impl StudioRenderIdentity {
    fn new(model: StudioDigest, texture_companion: Option<StudioDigest>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"xenoxide studio render identity v1\0");
        hasher.update(model.0);
        match texture_companion {
            Some(companion) => {
                hasher.update([1]);
                hasher.update(companion.0);
            }
            None => hasher.update([0]),
        }
        Self {
            model,
            texture_companion,
            combined: StudioDigest(hasher.finalize().into()),
        }
    }
}

/// A vertex or normal of a model, in the space of the bone it moves with
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StudioPoint {
    pub position: [f32; 3],
    pub bone: usize,
}

/// A triangle corner: a vertex and a normal of its model, and its texel coordinates
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StudioCorner {
    pub vertex: usize,
    pub normal: usize,
    pub s: i16,
    pub t: i16,
}

/// `mstudiomesh_t`, its commands decoded to triangles. In stock models a triangle's
/// corners wind clockwise seen from outside, as the world's faces do.
#[derive(Clone, Debug, PartialEq)]
pub struct StudioMesh {
    /// Which skin reference textures it; the skin family maps it to a texture
    pub skin_reference: usize,
    pub triangles: Vec<[StudioCorner; 3]>,
}

/// `mstudiomodel_t`: one choice of a body part
#[derive(Clone, Debug, PartialEq)]
pub struct StudioSubmodel {
    pub name: String,
    pub vertices: Vec<StudioPoint>,
    pub normals: Vec<StudioPoint>,
    pub meshes: Vec<StudioMesh>,
}

/// `mstudiobodyparts_t`
#[derive(Clone, Debug, PartialEq)]
pub struct StudioBodyPart {
    pub name: String,
    /// The body value's divisor for this part ([`StudioRenderAsset::submodel`])
    pub base: usize,
    pub models: Vec<StudioSubmodel>,
}

/// `mstudiotexture_t` and its 8-bit pixels and palette
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioTexture {
    pub name: String,
    /// `STUDIO_NF_*` lighting and blending flags, as authored
    pub flags: i32,
    pub width: usize,
    pub height: usize,
    /// One palette index per texel, row by row
    pub pixels: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
}

/// A studio model's rendering data, read and validated against its pose and hitbox
/// data
#[derive(Clone, Debug, PartialEq)]
pub struct StudioRenderAsset {
    identity: StudioRenderIdentity,
    body_parts: Vec<StudioBodyPart>,
    textures: Vec<StudioTexture>,
    skin_families: Vec<Vec<usize>>,
}

impl StudioRenderAsset {
    /// Reads the rendering data of `main`, the file `model` was read from, with its
    /// textures from `main` or, when it has none, from `companion`. A companion must be
    /// supplied exactly when the main file has no textures.
    pub fn parse(
        model: &StudioModel,
        main: &[u8],
        companion: Option<&[u8]>,
    ) -> Result<Self, StudioError> {
        if StudioDigest::of(main) != model.identity().main {
            return Err(StudioError::RenderModelMismatch);
        }
        let field = |bytes: &[u8], index: usize| i32_at(bytes, 140 + index * 4);
        let textures_in_main = field(main, 10) != 0;
        let (texture_bytes, companion_digest) = match (textures_in_main, companion) {
            (true, None) => (main, None),
            (true, Some(_)) => return Err(StudioError::UnexpectedTextureCompanion),
            (false, None) => return Err(StudioError::MissingTextureCompanion),
            (false, Some(companion)) => {
                check_file(
                    companion,
                    "texture companion",
                    STUDIO_HEADER_ID,
                    HEADER_SIZE,
                )?;
                if field(companion, 10) == 0 {
                    return Err(StudioError::InvalidTexture {
                        index: 0,
                        reason: "the texture companion has no textures",
                    });
                }
                (companion, Some(StudioDigest::of(companion)))
            }
        };
        let textures = parse_textures(
            texture_bytes,
            field(texture_bytes, 10),
            field(texture_bytes, 11),
        )?;
        let (skin_references, skin_families) = parse_skins(
            texture_bytes,
            field(texture_bytes, 13),
            field(texture_bytes, 14),
            field(texture_bytes, 15),
            textures.len(),
        )?;
        let body_parts = parse_body_parts(
            main,
            field(main, 16),
            field(main, 17),
            model.bones().len(),
            skin_references,
            &mut AssetBudget {
                triangles: MAX_STUDIO_ASSET_TRIANGLES,
                points: MAX_STUDIO_ASSET_POINTS,
            },
        )?;
        Ok(Self {
            identity: StudioRenderIdentity::new(model.identity().combined, companion_digest),
            body_parts,
            textures,
            skin_families,
        })
    }

    pub fn identity(&self) -> &StudioRenderIdentity {
        &self.identity
    }

    pub fn body_parts(&self) -> &[StudioBodyPart] {
        &self.body_parts
    }

    pub fn textures(&self) -> &[StudioTexture] {
        &self.textures
    }

    /// `[family][skin reference]`: the texture of each skin reference in each family
    pub fn skin_families(&self) -> &[Vec<usize>] {
        &self.skin_families
    }

    /// Which model of body part `part` the entity's `body` value selects:
    /// `(body / base) % models`, as the renderer's `StudioSetupModel` does
    pub fn submodel(&self, part: usize, body: u32) -> usize {
        let part = &self.body_parts[part];
        (body as usize / part.base) % part.models.len()
    }
}

fn parse_textures(
    bytes: &[u8],
    count: i32,
    offset: i32,
) -> Result<Vec<StudioTexture>, StudioError> {
    let (count, start) = table(
        bytes,
        "texture",
        count,
        offset,
        TEXTURE_SIZE,
        MAX_STUDIO_TEXTURES,
    )?;
    let mut pixels_left = MAX_STUDIO_ASSET_TEXTURE_PIXELS;
    (0..count)
        .map(|index| {
            let at = start + index * TEXTURE_SIZE;
            let invalid = |reason| StudioError::InvalidTexture { index, reason };
            let name = name_at(bytes, at, 64, "texture", index)?;
            let dimension = |at| {
                usize::try_from(i32_at(bytes, at))
                    .ok()
                    .filter(|value| (1..=MAX_STUDIO_TEXTURE_DIMENSION).contains(value))
                    .ok_or(invalid("dimensions"))
            };
            let (width, height) = (dimension(at + 68)?, dimension(at + 72)?);
            pixels_left = pixels_left.checked_sub(width * height).ok_or(
                StudioError::RenderBudgetExceeded {
                    what: "texture pixels",
                    limit: MAX_STUDIO_ASSET_TEXTURE_PIXELS,
                },
            )?;
            let pixels_at = usize::try_from(i32_at(bytes, at + 76))
                .map_err(|_| invalid("data lies outside the file"))?;
            let end = pixels_at
                .checked_add(width * height + PALETTE_SIZE)
                .filter(|end| *end <= bytes.len())
                .ok_or(invalid("data lies outside the file"))?;
            let palette_at = end - PALETTE_SIZE;
            Ok(StudioTexture {
                name,
                flags: i32_at(bytes, at + 64),
                width,
                height,
                pixels: bytes[pixels_at..palette_at].to_vec(),
                palette: bytes[palette_at..end]
                    .chunks_exact(3)
                    .map(|rgb| [rgb[0], rgb[1], rgb[2]])
                    .collect(),
            })
        })
        .collect()
}

/// The skin table: the number of skin references, and each family's texture per
/// reference
fn parse_skins(
    bytes: &[u8],
    references: i32,
    families: i32,
    offset: i32,
    textures: usize,
) -> Result<(usize, Vec<Vec<usize>>), StudioError> {
    let references = usize::try_from(references).map_err(|_| StudioError::InvalidCount {
        what: "skin reference",
        count: references,
    })?;
    if references > MAX_STUDIO_TEXTURES {
        return Err(StudioError::CountOverLimit {
            what: "skin reference",
            count: references,
            limit: MAX_STUDIO_TEXTURES,
        });
    }
    if references == 0 {
        return Ok((0, Vec::new()));
    }
    let families_count = usize::try_from(families).map_err(|_| StudioError::InvalidCount {
        what: "skin family",
        count: families,
    })?;
    if families_count == 0 {
        return Err(StudioError::InvalidCount {
            what: "skin family",
            count: families,
        });
    }
    let (families, start) = table(
        bytes,
        "skin family",
        families,
        offset,
        references * 2,
        MAX_STUDIO_SKIN_FAMILIES,
    )?;
    (0..families)
        .map(|family| {
            (0..references)
                .map(|reference| {
                    let at = start + (family * references + reference) * 2;
                    let texture = i16::from_le_bytes([bytes[at], bytes[at + 1]]);
                    usize::try_from(texture)
                        .ok()
                        .filter(|texture| *texture < textures)
                        .ok_or(StudioError::InvalidSkin { family, reference })
                })
                .collect()
        })
        .collect::<Result<_, _>>()
        .map(|families| (references, families))
}

/// What the rest of the asset may still use
struct AssetBudget {
    triangles: usize,
    points: usize,
}

fn parse_body_parts(
    bytes: &[u8],
    count: i32,
    offset: i32,
    bones: usize,
    skin_references: usize,
    budget: &mut AssetBudget,
) -> Result<Vec<StudioBodyPart>, StudioError> {
    let (count, start) = table(
        bytes,
        "body part",
        count,
        offset,
        BODY_PART_SIZE,
        MAX_STUDIO_BODY_PARTS,
    )?;
    let mut parts = Vec::with_capacity(count);
    for index in 0..count {
        parts.push({
            let at = start + index * BODY_PART_SIZE;
            let name = name_at(bytes, at, 64, "body part", index)?;
            let models = i32_at(bytes, at + 64);
            if models < 1 {
                return Err(StudioError::InvalidBodyPart {
                    index,
                    reason: "no models",
                });
            }
            let base = usize::try_from(i32_at(bytes, at + 68))
                .ok()
                .filter(|base| *base >= 1)
                .ok_or(StudioError::InvalidBodyPart {
                    index,
                    reason: "base",
                })?;
            let (models, models_at) = table(
                bytes,
                "model",
                models,
                i32_at(bytes, at + 72),
                SUBMODEL_SIZE,
                MAX_STUDIO_MODELS,
            )?;
            let models = (0..models)
                .map(|model| {
                    parse_submodel(
                        bytes,
                        models_at + model * SUBMODEL_SIZE,
                        index,
                        model,
                        bones,
                        skin_references,
                        budget,
                    )
                })
                .collect::<Result<_, _>>()?;
            StudioBodyPart { name, base, models }
        });
    }
    Ok(parts)
}

fn parse_submodel(
    bytes: &[u8],
    at: usize,
    part: usize,
    model: usize,
    bones: usize,
    skin_references: usize,
    budget: &mut AssetBudget,
) -> Result<StudioSubmodel, StudioError> {
    let invalid = |reason| StudioError::InvalidSubmodel {
        part,
        model,
        reason,
    };
    let name = name_at(bytes, at, 64, "model", model)?;
    let mut points =
        |count: i32, info: i32, data: i32, what| -> Result<Vec<StudioPoint>, StudioError> {
            let (count, info_at) = table(bytes, what, count, info, 1, MAX_STUDIO_VERTICES)?;
            budget.points =
                budget
                    .points
                    .checked_sub(count)
                    .ok_or(StudioError::RenderBudgetExceeded {
                        what: "vertices and normals",
                        limit: MAX_STUDIO_ASSET_POINTS,
                    })?;
            let (_, data_at) = table(bytes, what, count as i32, data, 12, MAX_STUDIO_VERTICES)?;
            (0..count)
                .map(|index| {
                    let bone = bytes[info_at + index] as usize;
                    if bone >= bones {
                        return Err(invalid("a point names a bone the model does not have"));
                    }
                    Ok(StudioPoint {
                        position: finite_array(bytes, data_at + index * 12, what, index)?,
                        bone,
                    })
                })
                .collect()
        };
    let vertices = points(
        i32_at(bytes, at + 80),
        i32_at(bytes, at + 84),
        i32_at(bytes, at + 88),
        "vertex",
    )?;
    let normals = points(
        i32_at(bytes, at + 92),
        i32_at(bytes, at + 96),
        i32_at(bytes, at + 100),
        "normal",
    )?;
    let (meshes, meshes_at) = table(
        bytes,
        "mesh",
        i32_at(bytes, at + 72),
        i32_at(bytes, at + 76),
        MESH_SIZE,
        MAX_STUDIO_MESHES,
    )?;
    let mut triangles_left = MAX_STUDIO_TRIANGLES;
    let meshes = (0..meshes)
        .map(|mesh| {
            let mesh_at = meshes_at + mesh * MESH_SIZE;
            let invalid = |reason| StudioError::InvalidMesh {
                part,
                model,
                mesh,
                reason,
            };
            let declared =
                usize::try_from(i32_at(bytes, mesh_at)).map_err(|_| invalid("triangle count"))?;
            let skin_reference = usize::try_from(i32_at(bytes, mesh_at + 8))
                .ok()
                .filter(|reference| *reference < skin_references)
                .ok_or(invalid("skin reference"))?;
            let commands = usize::try_from(i32_at(bytes, mesh_at + 4))
                .map_err(|_| invalid("commands lie outside the file"))?;
            let triangles = decode_commands(
                bytes,
                commands,
                vertices.len(),
                normals.len(),
                &mut triangles_left,
                &mut budget.triangles,
            )
            .map_err(|failure| match failure {
                DecodeFailure::Malformed(reason) => invalid(reason),
                DecodeFailure::AssetBudget => StudioError::RenderBudgetExceeded {
                    what: "triangles",
                    limit: MAX_STUDIO_ASSET_TRIANGLES,
                },
            })?;
            if triangles.len() != declared {
                return Err(invalid("decoded triangles differ from the declared count"));
            }
            Ok(StudioMesh {
                skin_reference,
                triangles,
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(StudioSubmodel {
        name,
        vertices,
        normals,
        meshes,
    })
}

enum DecodeFailure {
    Malformed(&'static str),
    /// More triangles than the rest of the asset may have
    AssetBudget,
}

impl From<&'static str> for DecodeFailure {
    fn from(reason: &'static str) -> Self {
        Self::Malformed(reason)
    }
}

/// Triangle commands, until a zero count: a strip (positive count) gives corners
/// `(k, k+1, k+2)` for even `k` and `(k+1, k, k+2)` for odd, a fan (negative)
/// `(0, k+1, k+2)`. A command of fewer than three corners, a corner outside the model's
/// vertices or normals, commands reaching past the end of the file, or more triangles
/// than the model may have, are malformed. Each command's triangles are charged to the
/// model's and the asset's budgets before they are made.
fn decode_commands(
    bytes: &[u8],
    start: usize,
    vertices: usize,
    normals: usize,
    left: &mut usize,
    asset_left: &mut usize,
) -> Result<Vec<[StudioCorner; 3]>, DecodeFailure> {
    let value = |at: usize| -> Result<i16, &'static str> {
        bytes
            .get(at..at + 2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .ok_or("commands lie outside the file")
    };
    let mut triangles = Vec::new();
    let mut at = start;
    loop {
        let count = value(at)?;
        at += 2;
        if count == 0 {
            return Ok(triangles);
        }
        let corners = count.unsigned_abs() as usize;
        if corners < 3 {
            return Err("a command has fewer than three corners".into());
        }
        let mut decoded = Vec::with_capacity(corners);
        for _ in 0..corners {
            let corner = StudioCorner {
                vertex: value(at)? as u16 as usize,
                normal: value(at + 2)? as u16 as usize,
                s: value(at + 4)?,
                t: value(at + 6)?,
            };
            if value(at)? < 0
                || corner.vertex >= vertices
                || value(at + 2)? < 0
                || corner.normal >= normals
            {
                return Err("a corner lies outside the model's vertices or normals".into());
            }
            decoded.push(corner);
            at += 8;
        }
        *left = left
            .checked_sub(corners - 2)
            .ok_or("more triangles than the model may have")?;
        *asset_left = asset_left
            .checked_sub(corners - 2)
            .ok_or(DecodeFailure::AssetBudget)?;
        for k in 0..corners - 2 {
            triangles.push(if count < 0 {
                [decoded[0], decoded[k + 1], decoded[k + 2]]
            } else if k % 2 == 0 {
                [decoded[k], decoded[k + 1], decoded[k + 2]]
            } else {
                [decoded[k + 1], decoded[k], decoded[k + 2]]
            });
        }
    }
}

#[cfg(test)]
mod tests;
