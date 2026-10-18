//! The light under a point, as the game samples it to light a model
//! (`R_LightPoint`)
//!
//! A line from the point 2048 units down is followed through the world's
//! BSP tree to the first face it crosses, and that face's lightmaps are read
//! at the crossing, each weighed by its light style's brightness. Sky and
//! turbulent faces are passed through. A lit face without light data gives
//! black, as the game's does.
//!
//! [`ModelLight`] turns that colour into the ambient and directional parts
//! the renderer lights a model's vertices with. That step follows Quake's
//! model lighting, which the game's builds on, and is not yet compared with
//! the game (9e)

use lambda_slicer_assets::xenoxide::bsp30::{BspDocument, LumpKind, MipTexture};

/// How far down the line goes
const REACH: f32 = 2048.;

#[derive(Clone, Copy, Debug)]
struct Node {
    plane: usize,
    /// A node index, or a leaf when negative
    children: [i32; 2],
    first_face: usize,
    face_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct LitFace {
    axes: [[f32; 4]; 2],
    /// The first luxel's corner and the faces's extent, in texels
    mins: [i32; 2],
    extents: [i32; 2],
    styles: [u8; 4],
    /// Where its lightmaps start in the lighting lump
    data: Option<usize>,
}

/// The world's BSP tree with what each face's lightmaps need
#[derive(Clone, Debug, Default)]
pub struct LightTree {
    planes: Vec<([f32; 3], f32)>,
    nodes: Vec<Node>,
    /// By face index, none for a face the line passes through
    faces: Vec<Option<LitFace>>,
    lighting: Vec<u8>,
}

impl LightTree {
    pub fn build(bsp: &BspDocument) -> Self {
        let planes = bsp
            .planes()
            .map(|p| (p.normal.map(|f| f.value), p.distance.value))
            .collect();
        let nodes = bsp
            .nodes()
            .map(|n| Node {
                plane: n.plane.max(0) as usize,
                children: n.children.map(i32::from),
                first_face: usize::from(n.first_face),
                face_count: usize::from(n.face_count),
            })
            .collect();

        let vertices: Vec<[f32; 3]> = bsp.vertices().map(|v| v.point.map(|f| f.value)).collect();
        let edges: Vec<[u16; 2]> = bsp.edges().map(|e| e.vertices).collect();
        let surface_edges: Vec<i32> = bsp.surface_edges().map(|e| e.edge).collect();
        let texinfo: Vec<_> = bsp.texture_info().collect();
        let names: Vec<String> = bsp
            .textures()
            .iter()
            .map(|t| match t {
                MipTexture::Missing { .. } => String::new(),
                MipTexture::External { name, .. } | MipTexture::Embedded { name, .. } => {
                    String::from_utf8_lossy(name).to_ascii_lowercase()
                }
            })
            .collect();

        let faces = bsp
            .faces()
            .map(|face| {
                let info = texinfo.get(face.texture_info.max(0) as usize)?;
                let name = names
                    .get(info.mip_texture.max(0) as usize)
                    .map_or("", String::as_str);
                if name.starts_with("sky") || name.starts_with('!') || info.flags & 1 != 0 {
                    return None;
                }
                let axes = [info.s_axis.map(|f| f.value), info.t_axis.map(|f| f.value)];

                // The game's surface extents, from the face's corners
                let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
                for i in 0..face.surface_edge_count.max(0) as usize {
                    let e = *surface_edges.get(face.first_surface_edge.max(0) as usize + i)?;
                    let v = if e >= 0 {
                        edges.get(e as usize)?[0]
                    } else {
                        edges.get(e.unsigned_abs() as usize)?[1]
                    };
                    let p = vertices.get(usize::from(v))?;
                    for k in 0..2 {
                        let a = axes[k];
                        let st = p[0] * a[0] + p[1] * a[1] + p[2] * a[2] + a[3];
                        lo[k] = lo[k].min(st);
                        hi[k] = hi[k].max(st);
                    }
                }
                let low = lo.map(|v| (v / 16.).floor() as i32);
                let high = hi.map(|v| (v / 16.).ceil() as i32);
                Some(LitFace {
                    axes,
                    mins: low.map(|v| v * 16),
                    extents: [(high[0] - low[0]) * 16, (high[1] - low[1]) * 16],
                    styles: face.styles,
                    data: (face.lighting_offset >= 0).then_some(face.lighting_offset as usize),
                })
            })
            .collect();

        Self {
            planes,
            nodes,
            faces,
            lighting: bsp.lump(LumpKind::Lighting).to_vec(),
        }
    }

    /// The colour under `point`, each channel 0 to about 255 and more where
    /// styles brighten it, with `styles` the brightness of each light style.
    /// None when nothing is under it
    pub fn at(&self, point: [f32; 3], styles: &[f32; 64]) -> Option<[f32; 3]> {
        if self.nodes.is_empty() {
            return None;
        }
        let end = [point[0], point[1], point[2] - REACH];
        self.trace(0, point, end, styles)
    }

    fn trace(
        &self,
        node: i32,
        start: [f32; 3],
        end: [f32; 3],
        styles: &[f32; 64],
    ) -> Option<[f32; 3]> {
        let Ok(index) = usize::try_from(node) else {
            return None;
        };
        let node = self.nodes.get(index)?;
        let (normal, distance) = *self.planes.get(node.plane)?;
        let dot = |p: [f32; 3]| p[0] * normal[0] + p[1] * normal[1] + p[2] * normal[2] - distance;
        let (front, back) = (dot(start), dot(end));
        let side = usize::from(front < 0.);

        if (back < 0.) == (front < 0.) {
            return self.trace(node.children[side], start, end, styles);
        }
        let fraction = front / (front - back);
        let mid = [0, 1, 2].map(|k| start[k] + (end[k] - start[k]) * fraction);

        if let Some(colour) = self.trace(node.children[side], start, mid, styles) {
            return Some(colour);
        }

        // The line crosses this node's plane at `mid`, so one of its faces
        // may hold it
        for face in self
            .faces
            .iter()
            .skip(node.first_face)
            .take(node.face_count)
        {
            let Some(face) = face else {
                continue;
            };
            let st = |a: [f32; 4]| mid[0] * a[0] + mid[1] * a[1] + mid[2] * a[2] + a[3];
            let ds = st(face.axes[0]) as i32 - face.mins[0];
            let dt = st(face.axes[1]) as i32 - face.mins[1];
            if ds < 0 || dt < 0 || ds > face.extents[0] || dt > face.extents[1] {
                continue;
            }
            let Some(data) = face.data else {
                return Some([0.; 3]);
            };

            let width = (face.extents[0] >> 4) as usize + 1;
            let height = (face.extents[1] >> 4) as usize + 1;
            let mut at = data + ((dt >> 4) as usize * width + (ds >> 4) as usize) * 3;
            let mut colour = [0.; 3];
            for &style in face.styles.iter().take_while(|s| **s != 255) {
                let scale = styles.get(usize::from(style)).copied().unwrap_or(1.);
                let luxel = self.lighting.get(at..at + 3)?;
                for k in 0..3 {
                    colour[k] += f32::from(luxel[k]) * scale;
                }
                at += width * height * 3;
            }
            return Some(colour);
        }

        self.trace(node.children[1 - side], mid, end, styles)
    }
}

/// How a model is lit, from the colour under it
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelLight {
    /// Light every vertex gets, 0 to 255
    pub ambient: f32,
    /// Light from above, 0 to 255
    pub shade: f32,
    /// The light's colour, its brightest channel 1
    pub color: [f32; 3],
}

impl ModelLight {
    /// Lighting where nothing is under a model
    pub const UNLIT: Self = Self {
        ambient: 128.,
        shade: 64.,
        color: [1.; 3],
    };

    /// The lighting a model gets from the colour under it, its brightest
    /// channel as the ambient level up to 128 and the shade up to 192
    pub fn from_point(colour: [f32; 3]) -> Self {
        let brightest = colour[0].max(colour[1]).max(colour[2]);
        if brightest <= 0. {
            return Self {
                ambient: 0.,
                shade: 0.,
                color: [1.; 3],
            };
        }
        Self {
            ambient: brightest.min(128.),
            shade: brightest.min(192.),
            color: colour.map(|c| c / brightest),
        }
    }

    /// A vertex's light for its normal, the shade weakened on faces turned
    /// from a light shining down, softened as the game's lambert of 1.5
    /// softens it
    pub fn vertex(&self, normal: [f32; 3]) -> [f32; 3] {
        // The normal against the light's direction, straight down
        let facing = (-normal[2]).min(1.);
        let lambert = 1.5;
        let facing = (facing + (lambert - 1.)) / lambert;
        let mut illum = self.ambient + self.shade;
        if facing > 0. {
            illum -= self.shade * facing;
        }
        let level = illum.clamp(0., 255.) / 255.;
        self.color.map(|c| c * level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_are_lit_from_above_and_coloured_by_the_light() {
        let light = ModelLight::from_point([200., 100., 50.]);
        assert_eq!(light.ambient, 128.);
        assert_eq!(light.shade, 192.);
        assert_eq!(light.color, [1., 0.5, 0.25]);

        // Facing up gets ambient and shade, facing down the ambient alone
        let up = light.vertex([0., 0., 1.]);
        let down = light.vertex([0., 0., -1.]);
        assert_eq!(up[0], 1.);
        assert!((down[0] - 128. / 255.).abs() < 1e-6);
        assert!(up[1] > down[1]);

        assert_eq!(
            ModelLight::from_point([0.; 3]).vertex([0., 0., 1.]),
            [0.; 3]
        );
    }
}
