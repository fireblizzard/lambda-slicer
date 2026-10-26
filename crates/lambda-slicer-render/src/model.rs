//! A studio model's triangles, skinned to an evaluated pose
//!
//! The chosen submodel of every body part is gathered into corners that
//! name their point, normal and bone, grouped by skin texture. Skinning
//! moves each corner by its bone's transform, on the CPU.
//!
//! A texture's flags change how its corners are drawn, as the engine's
//! studio renderer draws them (`R_StudioDrawPoints` in Xash3D's restatement).
//! A chrome texture's coordinates come from the normal against two vectors
//! square to the line from the eye to the bone (`R_StudioSetupChrome`). A
//! flat-shaded one is lit by the ambient light and most of the shade
//! whatever its normal. Additive textures come last, for the renderer to
//! add over the scene
//!
//! A glow shell is the model again, every point pushed out along the
//! average of its normals and every texture a chrome one, reflecting a
//! point that circles far around the world in place of the eye
//! (`R_StudioRenderModel`, `R_StudioBuildNormalTable`)

use lambda_slicer_assets::xenoxide::studio::StudioRenderAsset;
use lambda_slicer_assets::xenoxide::studio_pose::BoneTransform;

use crate::Rgba;
use crate::camera::angle_vectors;
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
    /// How much of it shows, 1 but for a translucent entity's
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Corner {
    point: usize,
    normal: usize,
    uv: [f32; 2],
    /// 32 over its texture's width and height, which scale chrome
    /// coordinates
    scale: [f32; 2],
    /// On a chrome texture
    chrome: bool,
    /// On a flat-shaded texture
    flat: bool,
}

/// A corner of a glow shell, pushed out from its model, and where it is
/// on the shell's texture
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

/// Where a model is seen from, which chrome textures reflect
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eye {
    pub origin: [f32; 3],
    /// The view's right
    pub right: [f32; 3],
}

impl Eye {
    pub fn new(origin: [f32; 3], angles: [f32; 3]) -> Self {
        let (_, right, _) = angle_vectors(angles);
        Self { origin, right }
    }

    /// What a glow shell reflects at client time `time` for a view whose
    /// right is `right`, a point 4000 units out that turns around the
    /// world, rising and falling a third as fast (`r_glowshellfreq`)
    pub fn shell(time: f64, right: [f32; 3]) -> Self {
        let turn = SHELL_FREQUENCY * time;
        let origin = [turn.cos(), turn.sin(), (turn * 0.33).cos()].map(|c| (c * 4000.) as f32);
        Self { origin, right }
    }
}

/// How fast a glow shell's reflection turns, the game's `r_glowshellfreq`
const SHELL_FREQUENCY: f64 = 2.2;

/// How far a glow shell stands out from its model for its entity's render
/// amount, 0 to 255, a 128th of a unit each and at least one
pub fn shell_distance(amount: f32) -> f32 {
    amount.max(1.) / 128.
}

/// `STUDIO_NF_FLATSHADE`, lit the same whatever the normal
const FLATSHADE: i32 = 0x01;
/// `STUDIO_NF_CHROME`, coordinates from the view
const CHROME: i32 = 0x02;
/// `STUDIO_NF_ADDITIVE`, added over what is behind
const ADDITIVE: i32 = 0x20;
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
    /// Ranges of corners drawn with one texture, those of additive textures
    /// last
    pub batches: Vec<Batch>,
    pub textures: Vec<Rgba>,
    /// By texture, whether it is added over the scene
    additive: Vec<bool>,
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
        let flags: Vec<i32> = asset.textures().iter().map(|t| t.flags).collect();
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
                let flag = flags.get(texture).copied().unwrap_or(0);
                for triangle in &mesh.triangles {
                    for c in triangle {
                        per_texture[texture].push(Corner {
                            point: point_base + c.vertex,
                            normal: normal_base + c.normal,
                            uv: [f32::from(c.s) / w, f32::from(c.t) / h],
                            scale: [32. / w, 32. / h],
                            chrome: flag & CHROME != 0,
                            flat: flag & FLATSHADE != 0,
                        });
                    }
                }
            }
        }

        let additive: Vec<bool> = (0..per_texture.len())
            .map(|t| flags.get(t).is_some_and(|f| f & ADDITIVE != 0))
            .collect();
        let mut corners = Vec::new();
        let mut batches = Vec::new();
        // Solid textures first, then additive ones
        for last in [false, true] {
            for (texture, list) in per_texture.iter().enumerate() {
                if list.is_empty() || additive[texture] != last {
                    continue;
                }
                batches.push(Batch {
                    texture,
                    first: corners.len() as u32,
                    count: list.len() as u32,
                });
                corners.extend_from_slice(list);
            }
        }
        Self {
            corners,
            points,
            normals,
            batches,
            textures,
            additive,
        }
    }

    pub fn corner_count(&self) -> usize {
        self.corners.len()
    }

    /// Whether texture `texture` is added over the scene
    pub fn additive(&self, texture: usize) -> bool {
        self.additive.get(texture).copied().unwrap_or(false)
    }

    /// Every corner moved by its bone in `bones` and lit by `light`, chrome
    /// textures reflecting as seen from `eye`. A bone the pose lacks leaves
    /// its points where the model stores them
    pub fn skin(&self, bones: &[BoneTransform], light: &ModelLight, eye: &Eye) -> Vec<ModelVertex> {
        let mut chrome = ChromeAxes::new(bones, eye);
        self.corners
            .iter()
            .map(|c| {
                let (point, bone) = self.points.get(c.point).copied().unwrap_or(([0.; 3], 0));
                let (normal, normal_bone) = self
                    .normals
                    .get(c.normal)
                    .copied()
                    .unwrap_or(([0., 0., 1.], 0));
                let normal = normalize(transform(bones, normal, normal_bone, false));
                let uv = if c.chrome {
                    chrome.uv(normal_bone, normal, c.scale).unwrap_or(c.uv)
                } else {
                    c.uv
                };
                ModelVertex {
                    position: transform(bones, point, bone, true),
                    normal,
                    uv,
                    light: if c.flat {
                        light.flat()
                    } else {
                        light.vertex(normal)
                    },
                    alpha: 1.,
                }
            })
            .collect()
    }
}

impl ModelMesh {
    /// The model's glow shell posed by `bones`, every point `distance`
    /// units out along the average of its normals, so the shell holds
    /// together, and every texture a chrome one reflecting `eye`
    pub fn shell(&self, bones: &[BoneTransform], eye: &Eye, distance: f32) -> Vec<ShellVertex> {
        let mut smooth = vec![[0.; 3]; self.points.len()];
        for c in &self.corners {
            let (Some(&(normal, bone)), Some(sum)) =
                (self.normals.get(c.normal), smooth.get_mut(c.point))
            else {
                continue;
            };
            let n = normalize(transform(bones, normal, bone, false));
            for k in 0..3 {
                sum[k] += n[k];
            }
        }
        let smooth: Vec<[f32; 3]> = smooth.into_iter().map(normalize).collect();

        let mut chrome = ChromeAxes::new(bones, eye);
        self.corners
            .iter()
            .map(|c| {
                let (point, bone) = self.points.get(c.point).copied().unwrap_or(([0.; 3], 0));
                let normal = smooth.get(c.point).copied().unwrap_or([0., 0., 1.]);
                let at = transform(bones, point, bone, true);
                ShellVertex {
                    position: [0, 1, 2].map(|k| at[k] + normal[k] * distance),
                    uv: chrome.uv(bone, normal, c.scale).unwrap_or(c.uv),
                }
            })
            .collect()
    }
}

/// `p` moved by bone `bone` of `bones`, turned only unless `translate`. A
/// bone the pose lacks leaves it where the model stores it
fn transform(bones: &[BoneTransform], p: [f32; 3], bone: usize, translate: bool) -> [f32; 3] {
    let Some(m) = bones.get(bone) else {
        return p;
    };
    let p = p.map(f64::from);
    let row = |r: &[f64; 4]| {
        let shift = if translate { r[3] } else { 0. };
        r[0] * p[0] + r[1] * p[1] + r[2] * p[2] + shift
    };
    [row(&m[0]) as f32, row(&m[1]) as f32, row(&m[2]) as f32]
}

/// Each bone's chrome vectors as seen from an eye, made once a bone
struct ChromeAxes<'a> {
    bones: &'a [BoneTransform],
    eye: &'a Eye,
    made: Vec<Option<([f32; 3], [f32; 3])>>,
}

impl<'a> ChromeAxes<'a> {
    fn new(bones: &'a [BoneTransform], eye: &'a Eye) -> Self {
        Self {
            bones,
            eye,
            made: vec![None; bones.len()],
        }
    }

    /// The chrome coordinates of `normal` on bone `bone`, scaled by
    /// `scale`. None for a bone the pose lacks
    fn uv(&mut self, bone: usize, normal: [f32; 3], scale: [f32; 2]) -> Option<[f32; 2]> {
        let m = self.bones.get(bone)?;
        let eye = self.eye;
        let origin = [m[0][3], m[1][3], m[2][3]].map(|v| v as f32);
        let (right, up) = *self.made[bone].get_or_insert_with(|| chrome_axes(origin, eye));
        let dot = |a: [f32; 3]| a[0] * normal[0] + a[1] * normal[1] + a[2] * normal[2];
        Some([(dot(right) + 1.) * scale[0], (dot(up) + 1.) * scale[1]])
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 1e-6 {
        v.map(|c| c / length)
    } else {
        v
    }
}

/// The world vectors a chrome texture's s and t coordinates follow on a
/// bone at `bone`, square to the line from the eye to it, the t one also
/// square to the view's right
fn chrome_axes(bone: [f32; 3], eye: &Eye) -> ([f32; 3], [f32; 3]) {
    let towards = normalize([0, 1, 2].map(|k| bone[k] - eye.origin[k]));
    let up = normalize(cross(towards, eye.right));
    let right = normalize(cross(towards, up));
    (right, up)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model of two meshes facing back along x, the first on a chrome
    /// additive 64 by 64 texture and the second on a flat-shaded one
    fn flagged() -> ModelMesh {
        use lambda_slicer_assets::xenoxide::studio::StudioModel;
        use lambda_slicer_assets::xenoxide::studio::fixture::{
            BodyPart, Fixture, Mesh, Submodel, Texture,
        };
        let mut fixture = Fixture::standard();
        let mut shiny = Texture::solid("shiny.bmp", 64, 64, 1, [255, 0, 255]);
        shiny.flags = CHROME | ADDITIVE;
        let mut flat = Texture::solid("flat.bmp", 4, 2, 2, [0, 0, 255]);
        flat.flags = FLATSHADE;
        fixture.texture_data = vec![shiny, flat];
        fixture.skins = vec![vec![0, 1]];
        let triangle = |skin_reference| Mesh {
            skin_reference,
            commands: vec![(3, vec![[0, 0, 0, 0], [1, 0, 4, 0], [2, 0, 0, 2]])],
            declared: None,
        };
        fixture.body_parts = vec![BodyPart {
            name: "body",
            base: 1,
            models: vec![Submodel {
                name: "plate",
                vertices: vec![([0., 0., 0.], 0), ([0., 1., 0.], 0), ([0., 0., 1.], 0)],
                normals: vec![([-1., 0., 0.], 0)],
                meshes: vec![triangle(0), triangle(1)],
            }],
        }];
        let built = fixture.build();
        let model = StudioModel::parse(&built.bytes)
            .unwrap()
            .resolve(&[])
            .unwrap();
        let asset = StudioRenderAsset::parse(&model, &built.bytes, None).unwrap();
        ModelMesh::build(&asset, 0, 0)
    }

    #[test]
    fn additive_textures_come_last_chrome_follows_the_eye_and_flat_ignores_the_normal() {
        let mesh = flagged();
        // The flat-shaded texture's batch first, the additive one's after
        let order: Vec<usize> = mesh.batches.iter().map(|b| b.texture).collect();
        assert_eq!(order, [1, 0]);
        assert!(mesh.additive(0) && !mesh.additive(1));

        // The bone 100 units ahead of an eye at the origin looking along x,
        // the normal facing the eye
        let bone: BoneTransform = [[1., 0., 0., 100.], [0., 1., 0., 0.], [0., 0., 1., 0.]];
        let bones = vec![bone; 4];
        let light = ModelLight::UNLIT;
        let vertices = mesh.skin(&bones, &light, &Eye::new([0.; 3], [0.; 3]));
        let additive = &vertices[3..];
        assert!(additive.iter().all(|v| v.uv == [0.5, 0.5]));
        let flat = &vertices[..3];
        assert!(flat.iter().all(|v| v.light == light.flat()));
        assert_eq!(flat[1].uv, [1., 0.]);
    }

    #[test]
    fn chrome_follows_the_view_and_faces_turned_to_it_take_the_middle() {
        // Looking along x at a bone ahead, the view's right is -y
        let eye = Eye::new([0.; 3], [0.; 3]);
        assert_eq!(eye.right.map(f32::round), [0., -1., 0.]);
        let (right, up) = chrome_axes([100., 0., 0.], &eye);
        // Down and to the left, as the engine's two cross products give them
        assert_eq!(up.map(f32::round), [0., 0., -1.]);
        assert_eq!(right.map(f32::round), [0., 1., 0.]);
        // A normal facing the eye is square to both, the middle of a 64 by
        // 64 texture, (0 + 1) * 32 / 64
        let normal = [-1., 0., 0.];
        let dot = |a: [f32; 3]| a[0] * normal[0] + a[1] * normal[1] + a[2] * normal[2];
        assert_eq!((dot(right) + 1.) * 32. / 64., 0.5);
        assert_eq!((dot(up) + 1.) * 32. / 64., 0.5);
    }

    #[test]
    fn a_shell_stands_out_along_the_normals_and_reflects_on_every_texture() {
        let mesh = flagged();
        let bone: BoneTransform = [[1., 0., 0., 100.], [0., 1., 0., 0.], [0., 0., 1., 0.]];
        let bones = vec![bone; 4];
        let eye = Eye::new([0.; 3], [0.; 3]);
        let skinned = mesh.skin(&bones, &ModelLight::UNLIT, &eye);
        let shell = mesh.shell(&bones, &eye, shell_distance(50.));
        assert_eq!(shell.len(), skinned.len());

        // Pushed back toward the eye along the plate's normal, -x
        let out = skinned[0].position[0] - shell[0].position[0];
        assert!((out - 50. / 128.).abs() < 1e-4);
        assert_eq!(shell[0].position[1..], skinned[0].position[1..]);

        // The flat texture reflects too, the middle of its 4 by 2 texture
        // scaled by 32 over its size
        assert_eq!(shell[0].uv, [8., 16.]);
        assert_eq!(shell_distance(0.), 1. / 128.);
    }

    #[test]
    fn the_shell_reflects_a_point_circling_far_around_the_world() {
        let start = Eye::shell(0., [0., -1., 0.]);
        assert_eq!(start.origin, [4000., 0., 4000.]);
        let later = Eye::shell(std::f64::consts::FRAC_PI_2 / SHELL_FREQUENCY, [0., -1., 0.]);
        assert!(later.origin[0].abs() < 1e-3 && (later.origin[1] - 4000.).abs() < 1e-3);
    }

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
