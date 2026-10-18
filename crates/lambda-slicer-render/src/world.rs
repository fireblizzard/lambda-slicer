//! A BSP's models as textured, lightmapped triangles
//!
//! Each face of every BSP model, the world first and then the brush models
//! entities draw as `*N`, is fanned into triangles and grouped by model and
//! texture. Texture coordinates follow the face's texture axes. A face's
//! lightmaps, one for each of its up to four light styles, one luxel per 16
//! texture units, sit side by side in one atlas with a border of repeated
//! edge luxels so filtering never mixes two faces. Each vertex carries the
//! face's styles, so the renderer weighs every map by its style's current
//! brightness. A face without light data is drawn at full brightness, as
//! the game draws it.
//!
//! Sky faces are left out, as the game draws the sky behind everything.
//! Water and other turbulent faces (`!` names) keep their texture
//! coordinates in texels, for the renderer to warp, and take no lightmap.
//! Textures named `+0` to `+9` and `+a` to `+j` are animation frames,
//! gathered into chains for the renderer to step through

use std::collections::HashMap;

use lambda_slicer_assets::xenoxide::bsp30::{BspDocument, LumpKind, MipTexture};

use crate::Rgba;
use crate::light::LightTree;

/// The style slot that stands for full brightness, past the game's 64
pub const FULL_BRIGHT: u8 = 64;

/// [`WorldVertex::flags`]
pub mod flags {
    /// A turbulent face, its coordinates in texels
    pub const WARP: u32 = 1;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub lightmap_uv: [f32; 2],
    /// How far right in the atlas each further style's map is
    pub style_step: f32,
    /// The face's light styles, 255 after the last
    pub styles: [u8; 4],
    pub flags: u32,
}

/// Indices drawn with one texture
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Batch {
    pub texture: usize,
    pub first: u32,
    pub count: u32,
}

/// The frames a texture steps through, ten a second as the game does
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Animation {
    /// The chain the texture belongs to, by texture index
    pub frames: Vec<usize>,
    /// The other chain, `+0` to `+9` for a `+a` texture and the reverse,
    /// which a brush entity shows while its frame is not 0
    pub alternates: Vec<usize>,
}

/// One entity of the map's entity text, its keys in order
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entity {
    pub pairs: Vec<(String, String)>,
}

impl Entity {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn vector(&self, key: &str) -> Option<[f32; 3]> {
        let mut parts = self
            .get(key)?
            .split_whitespace()
            .map(|p| p.parse::<f32>().ok());
        Some([parts.next()??, parts.next()??, parts.next()??])
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorldMesh {
    pub vertices: Vec<WorldVertex>,
    pub indices: Vec<u32>,
    /// Each BSP model's batches, the world's first
    pub models: Vec<Vec<Batch>>,
    pub textures: Vec<Rgba>,
    /// Each texture's animation, if it has one
    pub animations: Vec<Option<Animation>>,
    /// Textures neither the map nor its WADs hold, drawn as a checker
    pub missing_textures: Vec<String>,
    pub lightmap: Rgba,
    pub entities: Vec<Entity>,
    /// The `worldspawn` sky name, `desert` when it names none
    pub sky: String,
    /// What models are lit by
    pub light: LightTree,
    /// Each BSP model's faces that take decals, the world's first
    pub surfaces: Vec<Vec<Surface>>,
}

/// A face as decals see it, flat, with its texture axes and where its
/// lightmap sits in the atlas
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    pub points: Vec<[f32; 3]>,
    /// Facing the side the face is drawn on
    pub normal: [f32; 3],
    pub axes: [[f32; 4]; 2],
    /// The atlas coordinates are `(lightmap[0] + s / 16, lightmap[1] + t /
    /// 16)` over the atlas's size, none for a face at full brightness
    pub lightmap: Option<[f32; 2]>,
    pub style_step: f32,
    pub styles: [u8; 4],
}

/// The map's entity text read into entities. Text outside quotes other than
/// braces is ignored, as the game's parser does
pub fn entities(text: &[u8]) -> Vec<Entity> {
    let text = String::from_utf8_lossy(text);
    let mut out = Vec::new();
    let mut current: Option<Entity> = None;
    let mut key: Option<String> = None;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '{' => current = Some(Entity::default()),
            '}' => {
                if let Some(e) = current.take() {
                    out.push(e);
                }
                key = None;
            }
            '"' => {
                let token: String = chars.by_ref().take_while(|c| *c != '"').collect();
                if let Some(entity) = current.as_mut() {
                    match key.take() {
                        Some(k) => entity.pairs.push((k, token)),
                        None => key = Some(token),
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Where a player first appears, the first deathmatch spawn or else the
/// single-player start, and the way it faces in degrees
pub fn spawn(entities: &[Entity]) -> ([f32; 3], f32) {
    ["info_player_deathmatch", "info_player_start"]
        .iter()
        .find_map(|class| entities.iter().find(|e| e.get("classname") == Some(class)))
        .map_or(([0.; 3], 0.), |e| {
            let yaw = e
                .vector("angles")
                .map(|a| a[1])
                .or_else(|| e.get("angle")?.trim().parse().ok())
                .unwrap_or(0.);
            (e.vector("origin").unwrap_or([0.; 3]), yaw)
        })
}

fn worldspawn(entities: &[Entity]) -> Option<&Entity> {
    entities
        .iter()
        .find(|e| e.get("classname") == Some("worldspawn"))
}

/// A light style's brightness at `time`, about 1 for its usual `m`. The game
/// steps a pattern ten times a second and maps `a` to `z` to 0 to 550 on a
/// scale where 256 is the lightmap as stored. An empty pattern is that 256
pub fn style_brightness(pattern: &[u8], time: f64) -> f32 {
    if pattern.is_empty() {
        return 1.;
    }
    let c = pattern[(time.max(0.) * 10.) as usize % pattern.len()];
    let step = c.to_ascii_lowercase().clamp(b'a', b'z') - b'a';
    f32::from(step) * 22. / 256.
}

/// The WAD files a map's `worldspawn` names, by file name, in order
pub fn wad_names(entities: &[Entity]) -> Vec<String> {
    let Some(world) = worldspawn(entities) else {
        return Vec::new();
    };
    world
        .get("wad")
        .unwrap_or("")
        .split(';')
        .filter_map(|path| {
            let name = path.trim().rsplit(['/', '\\']).next()?;
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

/// Rectangles packed into rows of a fixed width
struct Shelves {
    width: u32,
    x: u32,
    y: u32,
    row: u32,
}

impl Shelves {
    fn place(&mut self, w: u32, h: u32) -> (u32, u32) {
        if self.x + w > self.width {
            self.x = 0;
            self.y += self.row;
            self.row = 0;
        }
        let at = (self.x, self.y);
        self.x += w;
        self.row = self.row.max(h);
        at
    }

    fn height(&self) -> u32 {
        self.y + self.row
    }
}

const ATLAS_WIDTH: u32 = 2048;
const LUXEL: f32 = 16.;
/// The game's largest lightmap side, in luxels
const MAX_LUXELS: u32 = 64;

struct FaceLight {
    /// The first luxel's texture-space corner divided by 16
    mins: [f32; 2],
    size: [u32; 2],
    /// The bytes of every style's map, none for a face at full brightness
    data: Option<std::ops::Range<usize>>,
    styles: [u8; 4],
    at: (u32, u32),
}

/// A face with its model, texture, texture axes and lightmaps
struct Polygon {
    model: usize,
    texture: usize,
    points: Vec<[f32; 3]>,
    light: FaceLight,
    axes: [[f32; 4]; 2],
    /// Facing the side the face is drawn on
    normal: [f32; 3],
    warp: bool,
}

/// The animation chains of `+` textures, by texture index
fn animations(names: &[String]) -> Vec<Option<Animation>> {
    // Frames by the name after their two-character prefix, each as its
    // place in the chain and its texture, digits first and letters second
    type Frames = Vec<(u8, usize)>;
    let mut chains: HashMap<String, (Frames, Frames)> = HashMap::new();
    for (i, name) in names.iter().enumerate() {
        let bytes = name.as_bytes();
        if bytes.len() < 3 || bytes[0] != b'+' {
            continue;
        }
        let key = name[2..].to_ascii_lowercase();
        let chain = chains.entry(key).or_default();
        match bytes[1].to_ascii_lowercase() {
            c @ b'0'..=b'9' => chain.0.push((c - b'0', i)),
            c @ b'a'..=b'j' => chain.1.push((c - b'a', i)),
            _ => {}
        }
    }

    let order = |mut list: Vec<(u8, usize)>| {
        list.sort();
        list.into_iter().map(|(_, i)| i).collect::<Vec<_>>()
    };
    let mut out = vec![None; names.len()];
    for (digits, letters) in chains.into_values() {
        let (digits, letters) = (order(digits), order(letters));
        for &i in &digits {
            out[i] = Some(Animation {
                frames: digits.clone(),
                alternates: letters.clone(),
            });
        }
        for &i in &letters {
            out[i] = Some(Animation {
                frames: letters.clone(),
                alternates: digits.clone(),
            });
        }
    }
    out
}

/// Every model's faces as one mesh. `wad_texture` gives a texture the map
/// names but does not hold, from the map's WADs
pub fn build(bsp: &BspDocument, wad_texture: &mut dyn FnMut(&str) -> Option<Rgba>) -> WorldMesh {
    let vertices: Vec<[f32; 3]> = bsp.vertices().map(|v| v.point.map(|f| f.value)).collect();
    let edges: Vec<[u16; 2]> = bsp.edges().map(|e| e.vertices).collect();
    let surface_edges: Vec<i32> = bsp.surface_edges().map(|e| e.edge).collect();
    let texinfo: Vec<_> = bsp.texture_info().collect();
    let faces: Vec<_> = bsp.faces().collect();
    let planes: Vec<[f32; 3]> = bsp.planes().map(|p| p.normal.map(|f| f.value)).collect();
    let lighting = bsp.lump(LumpKind::Lighting);
    let entities = entities(bsp.entity_bytes());
    let sky = worldspawn(&entities)
        .and_then(|w| w.get("skyname"))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("desert")
        .to_owned();

    // Every texture, embedded or from a WAD
    let mut missing_textures = Vec::new();
    let mut names = Vec::new();
    let mut sizes = Vec::new();
    let textures: Vec<Rgba> = bsp
        .textures()
        .iter()
        .map(|t| {
            let (name, width, height) = match t {
                MipTexture::Missing { .. } => (String::new(), 1, 1),
                MipTexture::External {
                    name,
                    width,
                    height,
                    ..
                }
                | MipTexture::Embedded {
                    name,
                    width,
                    height,
                    ..
                } => (String::from_utf8_lossy(name).into_owned(), *width, *height),
            };
            names.push(name.clone());
            sizes.push([width.max(1) as f32, height.max(1) as f32]);
            let transparent = name.starts_with('{').then_some(255);
            let image = match t {
                MipTexture::Embedded {
                    mip_levels,
                    palette: Some(palette),
                    width,
                    height,
                    ..
                } => Some(Rgba::from_palette(
                    *width,
                    *height,
                    mip_levels[0],
                    palette,
                    transparent,
                )),
                _ => wad_texture(&name),
            };
            image.unwrap_or_else(|| {
                missing_textures.push(name);
                Rgba::missing()
            })
        })
        .collect();

    let mut shelves = Shelves {
        width: ATLAS_WIDTH,
        x: 0,
        y: 0,
        row: 0,
    };
    // Luxel 0, 0 is white, for faces at full brightness
    shelves.place(3, 3);

    // Every model's faces, sky left out, with their lightmaps placed
    let models: Vec<_> = bsp.models().collect();
    let mut polygons: Vec<Polygon> = Vec::new();
    for (model_index, model) in models.iter().enumerate() {
        let first = model.first_face.max(0) as usize;
        let range = first..first + model.face_count.max(0) as usize;
        for face in faces.get(range).unwrap_or_default() {
            let Some(info) = texinfo.get(face.texture_info.max(0) as usize) else {
                continue;
            };
            let texture = info.mip_texture.max(0) as usize;
            let name = names.get(texture).map(String::as_str).unwrap_or("");
            if name.to_ascii_lowercase().starts_with("sky") {
                continue;
            }
            let warp = name.starts_with('!') || info.flags & 1 != 0;

            let points: Vec<[f32; 3]> = (0..face.surface_edge_count.max(0) as usize)
                .filter_map(|i| {
                    let e = *surface_edges.get(face.first_surface_edge.max(0) as usize + i)?;
                    let v = if e >= 0 {
                        edges.get(e as usize)?[0]
                    } else {
                        edges.get(e.unsigned_abs() as usize)?[1]
                    };
                    vertices.get(usize::from(v)).copied()
                })
                .collect();
            if points.len() < 3 {
                continue;
            }

            let axes = [info.s_axis.map(|f| f.value), info.t_axis.map(|f| f.value)];
            let st = |p: [f32; 3], a: [f32; 4]| p[0] * a[0] + p[1] * a[1] + p[2] * a[2] + a[3];
            let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
            for p in &points {
                for k in 0..2 {
                    let v = st(*p, axes[k]);
                    lo[k] = lo[k].min(v);
                    hi[k] = hi[k].max(v);
                }
            }
            let mins = lo.map(|v| (v / LUXEL).floor());
            let maxs = hi.map(|v| (v / LUXEL).ceil());
            let size = [
                (maxs[0] - mins[0]) as u32 + 1,
                (maxs[1] - mins[1]) as u32 + 1,
            ];

            let style_count = face.styles.iter().take_while(|s| **s != 255).count() as u32;
            let bytes = (size[0] * size[1] * 3 * style_count) as usize;
            let data = (!warp && style_count > 0 && face.lighting_offset >= 0)
                .then(|| face.lighting_offset as usize..face.lighting_offset as usize + bytes)
                .filter(|r| {
                    r.end <= lighting.len() && size[0] <= MAX_LUXELS && size[1] <= MAX_LUXELS
                });
            let (styles, at) = match data {
                Some(_) => (
                    face.styles,
                    shelves.place((size[0] + 2) * style_count, size[1] + 2),
                ),
                None => ([FULL_BRIGHT, 255, 255, 255], (0, 0)),
            };
            polygons.push(Polygon {
                model: model_index,
                texture,
                points,
                light: FaceLight {
                    mins,
                    size,
                    data,
                    styles,
                    at,
                },
                axes,
                normal: planes.get(usize::from(face.plane)).map_or([0.; 3], |n| {
                    if face.plane_side != 0 {
                        n.map(|v| -v)
                    } else {
                        *n
                    }
                }),
                warp,
            });
        }
    }

    // The atlas, each lightmap with its edge luxels repeated around it
    let height = shelves.height().next_power_of_two().max(4);
    let mut atlas = vec![0u8; (ATLAS_WIDTH * height * 4) as usize];
    let mut put = |x: u32, y: u32, rgb: [u8; 3]| {
        let at = ((y * ATLAS_WIDTH + x) * 4) as usize;
        atlas[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    };
    for y in 0..3 {
        for x in 0..3 {
            put(x, y, [255; 3]);
        }
    }
    for Polygon { light, .. } in &polygons {
        let Some(range) = &light.data else {
            continue;
        };
        let [w, h] = light.size;
        let per_style = (w * h * 3) as usize;
        for (k, luxels) in lighting[range.clone()].chunks(per_style).enumerate() {
            let left = light.at.0 + k as u32 * (w + 2);
            for y in 0..h + 2 {
                for x in 0..w + 2 {
                    let sx = x.saturating_sub(1).min(w - 1);
                    let sy = y.saturating_sub(1).min(h - 1);
                    let i = ((sy * w + sx) * 3) as usize;
                    put(
                        left + x,
                        light.at.1 + y,
                        [luxels[i], luxels[i + 1], luxels[i + 2]],
                    );
                }
            }
        }
    }

    // Triangles grouped by model and texture
    let mut out_vertices = Vec::new();
    let mut surfaces: Vec<Vec<Surface>> = vec![Vec::new(); models.len()];
    let mut per_texture: Vec<Vec<Vec<u32>>> = vec![vec![Vec::new(); textures.len()]; models.len()];
    for polygon in &polygons {
        let Polygon {
            model,
            texture,
            points,
            light,
            axes,
            normal,
            warp,
        } = polygon;
        let base = out_vertices.len() as u32;
        let [tw, th] = sizes.get(*texture).copied().unwrap_or([1., 1.]);
        let style_step = (light.size[0] + 2) as f32 / ATLAS_WIDTH as f32;
        // Turbulent faces take no decals, as the game's do not
        if !*warp && let Some(list) = surfaces.get_mut(*model) {
            list.push(Surface {
                points: points.clone(),
                normal: *normal,
                axes: *axes,
                lightmap: light.data.as_ref().map(|_| {
                    [
                        light.at.0 as f32 + 1. - light.mins[0] + 0.5,
                        light.at.1 as f32 + 1. - light.mins[1] + 0.5,
                    ]
                }),
                style_step,
                styles: light.styles,
            });
        }
        for p in points {
            let s = p[0] * axes[0][0] + p[1] * axes[0][1] + p[2] * axes[0][2] + axes[0][3];
            let t = p[0] * axes[1][0] + p[1] * axes[1][1] + p[2] * axes[1][2] + axes[1][3];
            let lightmap_uv = match light.data {
                Some(_) => [
                    (light.at.0 as f32 + 1. + s / LUXEL - light.mins[0] + 0.5) / ATLAS_WIDTH as f32,
                    (light.at.1 as f32 + 1. + t / LUXEL - light.mins[1] + 0.5) / height as f32,
                ],
                None => [1.5 / ATLAS_WIDTH as f32, 1.5 / height as f32],
            };
            out_vertices.push(WorldVertex {
                position: *p,
                uv: if *warp { [s, t] } else { [s / tw, t / th] },
                lightmap_uv,
                style_step,
                styles: light.styles,
                flags: if *warp { flags::WARP } else { 0 },
            });
        }
        if let Some(list) = per_texture[*model].get_mut(*texture) {
            for i in 1..points.len() as u32 - 1 {
                list.extend_from_slice(&[base, base + i, base + i + 1]);
            }
        }
    }

    let mut indices = Vec::new();
    let mut batches_by_model = Vec::with_capacity(models.len());
    for lists in per_texture {
        let mut batches = Vec::new();
        for (texture, list) in lists.into_iter().enumerate() {
            if list.is_empty() {
                continue;
            }
            batches.push(Batch {
                texture,
                first: indices.len() as u32,
                count: list.len() as u32,
            });
            indices.extend(list);
        }
        batches_by_model.push(batches);
    }

    missing_textures.sort();
    missing_textures.dedup();
    WorldMesh {
        vertices: out_vertices,
        indices,
        models: batches_by_model,
        animations: animations(&names),
        textures,
        missing_textures,
        lightmap: Rgba {
            width: ATLAS_WIDTH,
            height,
            pixels: atlas,
        },
        entities,
        sky,
        light: LightTree::build(bsp),
        surfaces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_and_wads_read_from_the_entity_text() {
        let text = br#"{
"classname" "worldspawn"
"wad" "\half-life\valve\halflife.wad;\sierra\ag\ag.wad;"
}
{
"origin" "16 -32 64"
"classname" "info_player_deathmatch"
}"#;
        let e = entities(text);
        assert_eq!(e.len(), 2);
        assert_eq!(wad_names(&e), ["halflife.wad", "ag.wad"]);
        assert_eq!(e[1].vector("origin"), Some([16., -32., 64.]));
        assert_eq!(e[1].get("CLASSNAME"), Some("info_player_deathmatch"));
    }

    #[test]
    fn light_styles_step_ten_times_a_second() {
        assert_eq!(style_brightness(b"", 3.), 1.);
        assert_eq!(style_brightness(b"m", 3.), 264. / 256.);
        assert_eq!(style_brightness(b"az", 0.05), 0.);
        assert_eq!(style_brightness(b"az", 0.15), 550. / 256.);
        assert_eq!(style_brightness(b"az", 0.25), 0.);
    }

    #[test]
    fn animated_textures_chain_with_their_alternates() {
        let names: Vec<String> = [
            "+1lab", "wall", "+0LAB", "+alab", "+2lab", "+bLab", "+0solo",
        ]
        .map(String::from)
        .into();
        let a = animations(&names);
        let digits = Animation {
            frames: vec![2, 0, 4],
            alternates: vec![3, 5],
        };
        assert_eq!(a[0].as_ref(), Some(&digits));
        assert_eq!(a[4].as_ref(), Some(&digits));
        assert_eq!(a[1], None);
        assert_eq!(
            a[5],
            Some(Animation {
                frames: vec![3, 5],
                alternates: vec![2, 0, 4],
            })
        );
        assert_eq!(a[6].as_ref().map(|c| c.frames.len()), Some(1));
    }
}
