//! Decals laid on a BSP model's faces (implementation plan §13.6)
//!
//! A decal is its texture at one unit a texel, centred where it was aimed
//! and lined up with each face's texture axes, on every face whose plane
//! passes within 4 units of that point (`R_DecalShoot`, `DECAL_DISTANCE`).
//! Each such face is cut to the decal's square, and what is left is drawn
//! over the face, lit by its lightmap. Turbulent faces take none. How the
//! decal lines up follows Xash3D's restatement of the engine
//! (`R_DecalComputeBasis`), not yet compared with the game.
//!
//! A decal texture from `decals.wad` keeps its colour in its palette's last
//! entry, and each texel's index says how much of that colour shows

use std::collections::BTreeMap;
use std::sync::Arc;

use lambda_slicer_assets::wad::MipTexture;

use crate::Rgba;
use crate::world::{Surface, WorldMesh, WorldVertex};

/// How far from a face's plane a decal's point may be (`DECAL_DISTANCE`)
const DECAL_DISTANCE: f32 = 4.;

/// How far off its face a decal is drawn, so it never fights the face for
/// depth
const LIFT: f32 = 0.1;

/// A decal texture, its palette's last colour showing as much as each
/// texel's index says
pub fn decal_rgba(texture: &MipTexture) -> Rgba {
    let colour = texture
        .palette
        .get(255 * 3..256 * 3)
        .map_or([0; 3], |c| [c[0], c[1], c[2]]);
    Rgba {
        width: texture.width,
        height: texture.height,
        pixels: texture
            .indices
            .iter()
            .flat_map(|&i| [colour[0], colour[1], colour[2], i])
            .collect(),
    }
}

/// The triangles of every decal, grouped by model and texture
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecalMesh {
    /// Grows whenever the decals change
    pub revision: u64,
    pub vertices: Vec<WorldVertex>,
    pub batches: Vec<DecalBatch>,
    /// Every decal texture the batches use, by its place in `decals.wad`
    pub textures: Arc<BTreeMap<u16, Rgba>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecalBatch {
    /// The BSP model the decals are on, 0 for the world
    pub model: usize,
    pub texture: u16,
    pub first: u32,
    pub count: u32,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot(v, v).sqrt();
    (length > 1e-6).then(|| v.map(|c| c / length))
}

/// A point of a decal's polygon, where it is and where in the decal
#[derive(Clone, Copy)]
struct Corner {
    position: [f32; 3],
    uv: [f32; 2],
}

/// Cut `polygon` to where `keep` of its decal coordinate along `axis` is
/// not negative
fn clip(polygon: &[Corner], axis: usize, keep: impl Fn(f32) -> f32) -> Vec<Corner> {
    let mut out = Vec::with_capacity(polygon.len() + 2);
    for (i, a) in polygon.iter().enumerate() {
        let b = polygon[(i + 1) % polygon.len()];
        let (da, db) = (keep(a.uv[axis]), keep(b.uv[axis]));
        if da >= 0. {
            out.push(*a);
        }
        if (da >= 0.) != (db >= 0.) {
            let f = da / (da - db);
            out.push(Corner {
                position: [0, 1, 2].map(|k| a.position[k] + (b.position[k] - a.position[k]) * f),
                uv: [0, 1].map(|k| a.uv[k] + (b.uv[k] - a.uv[k]) * f),
            });
        }
    }
    out
}

impl Surface {
    /// The decal of `size` aimed at `centre` cut to this face, as
    /// triangles, none when it does not reach the face
    fn decal(&self, centre: [f32; 3], size: [f32; 2], atlas: [f32; 2]) -> Vec<WorldVertex> {
        let n = self.normal;
        let plane = dot(n, self.points[0]);
        if (dot(n, centre) - plane).abs() > DECAL_DISTANCE {
            return Vec::new();
        }
        // The decal's right along the face's texture s axis, and its down
        // across it
        let s_axis = [self.axes[0][0], self.axes[0][1], self.axes[0][2]];
        let t_axis = [self.axes[1][0], self.axes[1][1], self.axes[1][2]];
        let Some(down) = normalize(cross(n, s_axis)) else {
            return Vec::new();
        };
        let down = if dot(down, t_axis) < 0. {
            down.map(|v| -v)
        } else {
            down
        };
        let Some(right) = normalize(cross(down, n)) else {
            return Vec::new();
        };
        let right = if dot(right, s_axis) < 0. {
            right.map(|v| -v)
        } else {
            right
        };

        let corners: Vec<Corner> = self
            .points
            .iter()
            .map(|p| {
                let d = [0, 1, 2].map(|k| p[k] - centre[k]);
                Corner {
                    position: *p,
                    uv: [dot(d, right) / size[0] + 0.5, dot(d, down) / size[1] + 0.5],
                }
            })
            .collect();
        let mut cut = corners;
        for axis in 0..2 {
            cut = clip(&cut, axis, |v| v);
            cut = clip(&cut, axis, |v| 1. - v);
            if cut.len() < 3 {
                return Vec::new();
            }
        }

        let vertex = |c: &Corner| {
            let p = c.position;
            let s = dot(p, s_axis) + self.axes[0][3];
            let t = dot(p, t_axis) + self.axes[1][3];
            let lightmap_uv = match self.lightmap {
                Some([x, y]) => [(x + s / 16.) / atlas[0], (y + t / 16.) / atlas[1]],
                None => [1.5 / atlas[0], 1.5 / atlas[1]],
            };
            WorldVertex {
                position: [0, 1, 2].map(|k| p[k] + n[k] * LIFT),
                uv: c.uv,
                lightmap_uv,
                style_step: self.style_step,
                styles: self.styles,
                flags: 0,
            }
        };
        let mut out = Vec::with_capacity((cut.len() - 2) * 3);
        for i in 1..cut.len() - 1 {
            out.extend([vertex(&cut[0]), vertex(&cut[i]), vertex(&cut[i + 1])]);
        }
        out
    }
}

impl WorldMesh {
    /// A decal of `size` units aimed at `centre`, in the model's own space,
    /// laid on model `model`'s faces as triangles
    pub fn decal(&self, model: usize, centre: [f32; 3], size: [f32; 2]) -> Vec<WorldVertex> {
        let atlas = [self.lightmap.width as f32, self.lightmap.height as f32];
        let reach = size[0].max(size[1]) * 0.5;
        let mut out = Vec::new();
        for surface in self.surfaces.get(model).into_iter().flatten() {
            // Faces nowhere near are passed over before any projecting
            let near = (0..3).all(|k| {
                let (lo, hi) = surface
                    .points
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                        (lo.min(p[k]), hi.max(p[k]))
                    });
                centre[k] >= lo - reach - DECAL_DISTANCE && centre[k] <= hi + reach + DECAL_DISTANCE
            });
            if near {
                out.extend(surface.decal(centre, size, atlas));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 64 by 64 floor facing up, its texture's s along x and t along -y
    fn floor() -> Surface {
        Surface {
            points: vec![[0., 0., 0.], [0., 64., 0.], [64., 64., 0.], [64., 0., 0.]],
            normal: [0., 0., 1.],
            axes: [[1., 0., 0., 0.], [0., -1., 0., 0.]],
            lightmap: Some([10., 20.]),
            style_step: 0.,
            styles: [0, 255, 255, 255],
        }
    }

    #[test]
    fn a_decal_is_cut_to_its_square_on_the_face() {
        let tris = floor().decal([32., 32., 2.], [16., 8.], [256., 256.]);
        assert!(!tris.is_empty());
        let xs = tris.iter().map(|v| v.position[0]);
        let ys = tris.iter().map(|v| v.position[1]);
        let (lo_x, hi_x) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let (lo_y, hi_y) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
        assert!((lo_x - 24.).abs() < 1e-4 && (hi_x - 40.).abs() < 1e-4);
        assert!((lo_y - 28.).abs() < 1e-4 && (hi_y - 36.).abs() < 1e-4);
        // Over the face, its texture's corners where the decal's are
        assert!(tris.iter().all(|v| (v.position[2] - LIFT).abs() < 1e-6));
        for v in &tris {
            let expect = [(v.position[0] - 24.) / 16., (36. - v.position[1]) / 8.];
            assert!((v.uv[0] - expect[0]).abs() < 1e-4 && (v.uv[1] - expect[1]).abs() < 1e-4);
        }
        // The lightmap read where the face's own is
        let v = tris[0];
        let t = -v.position[1];
        assert!((v.lightmap_uv[1] - (20. + t / 16.) / 256.).abs() < 1e-6);
    }

    #[test]
    fn a_decal_off_the_face_or_too_far_from_it_is_not_laid() {
        assert!(
            floor()
                .decal([32., 32., 5.], [16., 16.], [256., 256.])
                .is_empty()
        );
        assert!(
            floor()
                .decal([100., 32., 0.], [16., 16.], [256., 256.])
                .is_empty()
        );
        // Half over the edge, half of it is laid
        let tris = floor().decal([64., 32., 0.], [16., 16.], [256., 256.]);
        let hi_x = tris.iter().map(|v| v.position[0]).fold(f32::MIN, f32::max);
        assert!((hi_x - 64.).abs() < 1e-4);
    }

    #[test]
    fn a_decal_texture_shows_its_colour_by_index() {
        let mut palette = vec![255u8; 256 * 3];
        palette[255 * 3..].copy_from_slice(&[200, 10, 0]);
        let texture = MipTexture {
            name: "{blood1".into(),
            width: 2,
            height: 1,
            indices: vec![0, 254],
            palette,
        };
        assert_eq!(
            decal_rgba(&texture).pixels,
            [200, 10, 0, 0, 200, 10, 0, 254]
        );
    }
}
