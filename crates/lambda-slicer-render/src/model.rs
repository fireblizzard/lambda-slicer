//! A studio model's triangles, skinned to an evaluated pose
//!
//! The chosen submodel of every body part is gathered into corners that
//! name their point, normal and bone, grouped by skin texture. Skinning
//! moves each corner by its bone's transform, on the CPU for the spike.
//! Chrome, additive and flat-shaded textures are drawn as plain ones until
//! M9

use lambda_slicer_assets::xenoxide::studio::StudioRenderAsset;
use lambda_slicer_assets::xenoxide::studio_pose::BoneTransform;

use crate::Rgba;
use crate::world::Batch;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Corner {
    point: usize,
    normal: usize,
    uv: [f32; 2],
}

/// `STUDIO_NF_MASKED`, palette index 255 drawn as nothing
const MASKED: i32 = 0x40;

#[derive(Clone, Debug, PartialEq)]
pub struct ModelMesh {
    corners: Vec<Corner>,
    /// Points and normals with their bones, across the chosen submodels
    points: Vec<([f32; 3], usize)>,
    normals: Vec<([f32; 3], usize)>,
    /// Ranges of corners drawn with one texture
    pub batches: Vec<Batch>,
    pub textures: Vec<Rgba>,
}

impl ModelMesh {
    /// The submodels `body` chooses, with skin family `skin`
    pub fn build(asset: &StudioRenderAsset, body: u32, skin: usize) -> Self {
        let textures: Vec<Rgba> = asset
            .textures()
            .iter()
            .map(|t| {
                let palette: Vec<u8> = t.palette.iter().flatten().copied().collect();
                let transparent = (t.flags & MASKED != 0).then_some(255);
                Rgba::from_palette(
                    t.width as u32,
                    t.height as u32,
                    &t.pixels,
                    &palette,
                    transparent,
                )
            })
            .collect();
        let family = asset
            .skin_families()
            .get(skin)
            .or(asset.skin_families().first());

        let mut points = Vec::new();
        let mut normals = Vec::new();
        let mut per_texture: Vec<Vec<Corner>> = vec![Vec::new(); textures.len().max(1)];
        for (p, part) in asset.body_parts().iter().enumerate() {
            let Some(sub) = part.models.get(asset.submodel(p, body)) else {
                continue;
            };
            let (point_base, normal_base) = (points.len(), normals.len());
            points.extend(sub.vertices.iter().map(|v| (v.position, v.bone)));
            normals.extend(sub.normals.iter().map(|n| (n.position, n.bone)));
            for mesh in &sub.meshes {
                let texture = family
                    .and_then(|f| f.get(mesh.skin_reference))
                    .copied()
                    .unwrap_or(mesh.skin_reference)
                    .min(per_texture.len() - 1);
                let (w, h) = textures.get(texture).map_or((1., 1.), |t| {
                    (t.width.max(1) as f32, t.height.max(1) as f32)
                });
                for triangle in &mesh.triangles {
                    for c in triangle {
                        per_texture[texture].push(Corner {
                            point: point_base + c.vertex,
                            normal: normal_base + c.normal,
                            uv: [f32::from(c.s) / w, f32::from(c.t) / h],
                        });
                    }
                }
            }
        }

        let mut corners = Vec::new();
        let mut batches = Vec::new();
        for (texture, list) in per_texture.into_iter().enumerate() {
            if list.is_empty() {
                continue;
            }
            batches.push(Batch {
                texture,
                first: corners.len() as u32,
                count: list.len() as u32,
            });
            corners.extend(list);
        }
        Self {
            corners,
            points,
            normals,
            batches,
            textures,
        }
    }

    pub fn corner_count(&self) -> usize {
        self.corners.len()
    }

    /// Every corner moved by its bone in `bones`. A bone the pose lacks
    /// leaves its points where the model stores them
    pub fn skin(&self, bones: &[BoneTransform]) -> Vec<ModelVertex> {
        let apply = |p: [f32; 3], bone: usize, translate: bool| -> [f32; 3] {
            let Some(m) = bones.get(bone) else {
                return p;
            };
            let p = p.map(f64::from);
            let row = |r: &[f64; 4]| {
                r[0] * p[0] + r[1] * p[1] + r[2] * p[2] + if translate { r[3] } else { 0. }
            };
            [row(&m[0]) as f32, row(&m[1]) as f32, row(&m[2]) as f32]
        };
        self.corners
            .iter()
            .map(|c| {
                let (point, bone) = self.points.get(c.point).copied().unwrap_or(([0.; 3], 0));
                let (normal, normal_bone) = self
                    .normals
                    .get(c.normal)
                    .copied()
                    .unwrap_or(([0., 0., 1.], 0));
                let n = apply(normal, normal_bone, false);
                let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                ModelVertex {
                    position: apply(point, bone, true),
                    normal: n.map(|v| v / length),
                    uv: c.uv,
                }
            })
            .collect()
    }
}
