//! Drawing a demo's world and models with `wgpu` (implementation plan §13)
//!
//! The M8 backend spike. [`world`] turns a BSP into textured, lightmapped
//! triangles, [`model`] turns a studio model and an evaluated pose into
//! skinned triangles, and [`gpu`] draws both into an offscreen texture with
//! depth, which a window shows through [`gpu::Blit`]. Everything here takes
//! the device and queue it is given, so it shares the window's GPU device
//! instead of opening one of its own. Nothing here depends on egui
//! (docs/viewer-backend.md)

pub mod camera;
pub mod decal;
pub mod gpu;
pub mod light;
pub mod load;
pub mod model;
pub mod world;

/// An image in 8-bit RGBA, rows from the top
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Rgba {
    /// A palette image, `transparent` naming the index drawn as nothing
    pub fn from_palette(
        width: u32,
        height: u32,
        indices: &[u8],
        palette: &[u8],
        transparent: Option<u8>,
    ) -> Self {
        let colour = |i: u8| {
            let at = usize::from(i) * 3;
            palette
                .get(at..at + 3)
                .map_or([255, 0, 255], |c| [c[0], c[1], c[2]])
        };
        let pixels = indices
            .iter()
            .flat_map(|&i| {
                let [r, g, b] = colour(i);
                let a = if Some(i) == transparent { 0 } else { 255 };
                [r, g, b, a]
            })
            .collect();
        Self {
            width,
            height,
            pixels,
        }
    }

    /// A magenta and black checker for an image that could not be found,
    /// so a missing texture is visible and never passes for the real one
    pub fn missing() -> Self {
        let pixels = (0..64)
            .flat_map(|i| {
                if (i % 8 < 4) ^ (i / 8 < 4) {
                    [255, 0, 255, 255]
                } else {
                    [0, 0, 0, 255]
                }
            })
            .collect();
        Self {
            width: 8,
            height: 8,
            pixels,
        }
    }
}
