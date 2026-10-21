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
pub mod gamma;
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
        let mut image = Self {
            width,
            height,
            pixels,
        };
        if transparent.is_some() {
            image.fill_cut_out();
        }
        image
    }

    /// Give every texel drawn as nothing the average colour of the solid
    /// texels around it, wrapping round as the texture repeats. A filtered
    /// sample at a cut-out's edge then blends with the texture's own
    /// colours, not with the transparent index's, which is usually blue
    fn fill_cut_out(&mut self) {
        let (width, height) = (self.width as usize, self.height as usize);
        if width == 0 || height == 0 || self.pixels.len() < width * height * 4 {
            return;
        }

        let source = self.pixels.clone();
        for y in 0..height {
            for x in 0..width {
                let at = (y * width + x) * 4;
                if source[at + 3] != 0 {
                    continue;
                }

                let mut sum = [0u32; 3];
                let mut solid = 0;
                for dy in [height - 1, 0, 1] {
                    for dx in [width - 1, 0, 1] {
                        let near = ((y + dy) % height * width + (x + dx) % width) * 4;
                        if source[near + 3] == 0 {
                            continue;
                        }
                        for (k, total) in sum.iter_mut().enumerate() {
                            *total += u32::from(source[near + k]);
                        }
                        solid += 1;
                    }
                }

                // Black far from any solid texel, where nothing samples it
                let fill = if solid == 0 {
                    [0; 3]
                } else {
                    sum.map(|total| (total / solid) as u8)
                };
                self.pixels[at..at + 3].copy_from_slice(&fill);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn see_through_texels_take_the_colour_around_them() {
        // Red, then the blue transparent index, then red, and a solid blue
        // texture with no transparent index left as it is
        let mut palette = vec![0u8; 256 * 3];
        palette[3..6].copy_from_slice(&[200, 10, 10]);
        palette[255 * 3..].copy_from_slice(&[0, 0, 255]);
        let image = Rgba::from_palette(3, 1, &[1, 255, 1], &palette, Some(255));
        assert_eq!(&image.pixels[4..8], &[200, 10, 10, 0]);
        assert_eq!(&image.pixels[..4], &[200, 10, 10, 255]);

        let solid = Rgba::from_palette(1, 1, &[255], &palette, None);
        assert_eq!(solid.pixels, [0, 0, 255, 255]);

        // Nothing solid anywhere near leaves black, not blue
        let empty = Rgba::from_palette(2, 1, &[255, 255], &palette, Some(255));
        assert_eq!(empty.pixels, [0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
