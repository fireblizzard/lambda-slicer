//! A studio model's triangles, skinned to an evaluated pose
//!
//! The chosen submodel of every body part is gathered into corners that
//! name their point, normal and bone, grouped by skin texture. Skinning
//! moves each corner by its bone's transform, on the CPU for the spike.
//! Chrome, additive and flat-shaded textures are drawn as plain ones for now

use lambda_slicer_assets::xenoxide::studio::StudioRenderAsset;
use lambda_slicer_assets::xenoxide::studio_pose::BoneTransform;

use crate::Rgba;
use crate::light::ModelLight;
use crate::world::Batch;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// The vertex's light, from [`ModelLight::vertex`]
    pub light: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Corner {
    point: usize,
    normal: usize,
    uv: [f32; 2],
}

/// `STUDIO_NF_MASKED`, palette index 255 drawn as nothing
const MASKED: i32 = 0x40;

/// The palette ranges a player's top and bottom colours recolour
const PLATE: std::ops::RangeInclusive<usize> = 160..=191;
const SUIT: std::ops::RangeInclusive<usize> = 80..=111;

/// Give a range of palette colours the hue `colour`, 0 to 255 around the
/// colour wheel, keeping each one's brightness and saturation, as the
/// game's `PaletteHueReplace` does
fn replace_hue(palette: &mut [[u8; 3]], colour: u8, range: std::ops::RangeInclusive<usize>) {
    let hue = f32::from(colour) * (360. / 255.);
    for entry in palette
        .iter_mut()
        .take(range.end() + 1)
        .skip(*range.start())
    {
        let [r, g, b] = entry.map(f32::from);
        let max = r.max(g).max(b) / 255.;
        let min = r.min(g).min(b) / 255.;
        if max == 0. {
            continue;
        }
        let value = max;
        let saturation = (max - min) / max;
        let low = value * (1. - saturation);

        let (r, g, b) = if hue <= 120. {
            if hue < 60. {
                (value, low + hue * (value - low) / (120. - hue), low)
            } else {
                (low + (120. - hue) * (value - low) / hue, value, low)
            }
        } else if hue <= 240. {
            if hue < 180. {
                (
                    low,
                    value,
                    low + (hue - 120.) * (value - low) / (240. - hue),
                )
            } else {
                (
                    low,
                    low + (240. - hue) * (value - low) / (hue - 120.),
                    value,
                )
            }
        } else if hue < 300. {
            (
                low + (hue - 240.) * (value - low) / (360. - hue),
                low,
                value,
            )
        } else {
            (
                value,
                low,
                low + (360. - hue) * (value - low) / (hue - 240.),
            )
        };
        *entry = [r, g, b].map(|c| (c * 255.) as u8);
    }
}

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
        Self::build_coloured(asset, body, skin, None)
    }

    /// [`ModelMesh::build`] with a player's top and bottom colours on its
    /// `DM_Base` textures, as the game colours them
    pub fn build_coloured(
        asset: &StudioRenderAsset,
        body: u32,
        skin: usize,
        colours: Option<[u8; 2]>,
    ) -> Self {
        let textures: Vec<Rgba> = asset
            .textures()
            .iter()
            .map(|t| {
                let mut palette = t.palette.clone();
                if let Some([top, bottom]) = colours
                    && t.name.to_ascii_lowercase().starts_with("dm_base")
                {
                    replace_hue(&mut palette, top, PLATE);
                    replace_hue(&mut palette, bottom, SUIT);
                }
                let palette: Vec<u8> = palette.iter().flatten().copied().collect();
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

    /// Every corner moved by its bone in `bones` and lit by `light`. A bone
    /// the pose lacks leaves its points where the model stores them
    pub fn skin(&self, bones: &[BoneTransform], light: &ModelLight) -> Vec<ModelVertex> {
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
                let normal = n.map(|v| v / length);
                ModelVertex {
                    position: apply(point, bone, true),
                    normal,
                    uv: c.uv,
                    light: light.vertex(normal),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hues_turn_around_the_wheel_keeping_brightness() {
        let mut palette = vec![[0u8; 3]; 256];
        palette[160] = [200, 100, 100];
        palette[161] = [0, 0, 0];
        palette[90] = [255, 255, 255];

        // A hue of 0 is red, 85 green, 170 blue
        replace_hue(&mut palette, 0, PLATE);
        assert_eq!(palette[160], [200, 100, 100]);
        replace_hue(&mut palette, 85, PLATE);
        assert_eq!(palette[160], [100, 200, 100]);
        replace_hue(&mut palette, 170, PLATE);
        assert_eq!(palette[160], [100, 100, 200]);
        assert_eq!(palette[161], [0, 0, 0]);

        // Grey keeps no hue, outside the range nothing changes
        replace_hue(&mut palette, 170, SUIT);
        assert_eq!(palette[90], [255, 255, 255]);
        assert_eq!(palette[0], [0, 0, 0]);
    }
}
