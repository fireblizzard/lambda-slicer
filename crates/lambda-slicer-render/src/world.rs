//! A BSP's world as textured, lightmapped triangles
//!
//! Each face of the world model is fanned into triangles and grouped by
//! texture. Texture coordinates follow the face's texture axes, and the
//! face's style 0 lightmap, one luxel per 16 texture units, is packed into
//! one atlas with a border of repeated edge luxels so filtering never mixes
//! two faces. Sky faces are left out, as the game draws the sky instead.
//! Only the static lightmap is drawn, light styles and moving brush models
//! come with M9

use lambda_slicer_assets::xenoxide::bsp30::{BspDocument, LumpKind, MipTexture};

use crate::Rgba;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub lightmap_uv: [f32; 2],
}

/// Indices drawn with one texture
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Batch {
    pub texture: usize,
    pub first: u32,
    pub count: u32,
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldMesh {
    pub vertices: Vec<WorldVertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
    pub textures: Vec<Rgba>,
    /// Textures neither the map nor its WADs hold, drawn as a checker
    pub missing_textures: Vec<String>,
    pub lightmap: Rgba,
    pub entities: Vec<Entity>,
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

/// The WAD files a map's `worldspawn` names, by file name, in order
pub fn wad_names(entities: &[Entity]) -> Vec<String> {
    let Some(world) = entities
        .iter()
        .find(|e| e.get("classname") == Some("worldspawn"))
    else {
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

struct FaceLight {
    /// The first luxel's texture-space corner divided by 16
    mins: [f32; 2],
    size: [u32; 2],
    /// Bytes of style 0, or none for a face drawn at full brightness
    data: Option<std::ops::Range<usize>>,
    at: (u32, u32),
}

/// A world face with its texture, texture axes and lightmap
struct Polygon {
    texture: usize,
    points: Vec<[f32; 3]>,
    light: FaceLight,
    axes: [[f32; 4]; 2],
}

/// The world model's faces as a mesh. `wad_texture` gives a texture the map
/// names but does not hold, from the map's WADs
pub fn build(bsp: &BspDocument, wad_texture: &mut dyn FnMut(&str) -> Option<Rgba>) -> WorldMesh {
    let vertices: Vec<[f32; 3]> = bsp.vertices().map(|v| v.point.map(|f| f.value)).collect();
    let edges: Vec<[u16; 2]> = bsp.edges().map(|e| e.vertices).collect();
    let surface_edges: Vec<i32> = bsp.surface_edges().map(|e| e.edge).collect();
    let texinfo: Vec<_> = bsp.texture_info().collect();
    let faces: Vec<_> = bsp.faces().collect();
    let lighting = bsp.lump(LumpKind::Lighting);
    let entities = entities(bsp.entity_bytes());

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

    // The world model's faces, sky left out, with their lightmaps placed
    let Some(world) = bsp.models().next() else {
        return WorldMesh {
            entities,
            ..WorldMesh::default()
        };
    };
    let first = world.first_face.max(0) as usize;
    let range = first..first + world.face_count.max(0) as usize;
    let mut shelves = Shelves {
        width: ATLAS_WIDTH,
        x: 0,
        y: 0,
        row: 0,
    };
    // Luxel 0, 0 is white, for faces without light
    shelves.place(3, 3);

    let mut polygons: Vec<Polygon> = Vec::new();
    for face in faces.get(range).unwrap_or_default() {
        let Some(info) = texinfo.get(face.texture_info.max(0) as usize) else {
            continue;
        };
        let texture = info.mip_texture.max(0) as usize;
        if names
            .get(texture)
            .is_some_and(|n| n.to_ascii_lowercase().starts_with("sky"))
        {
            continue;
        }
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
        let bytes = (size[0] * size[1] * 3) as usize;
        let data = (face.styles[0] != 255 && face.lighting_offset >= 0)
            .then(|| face.lighting_offset as usize..face.lighting_offset as usize + bytes)
            .filter(|r| r.end <= lighting.len() && size[0] <= 64 && size[1] <= 64);
        let at = match data {
            Some(_) => shelves.place(size[0] + 2, size[1] + 2),
            None => (0, 0),
        };
        polygons.push(Polygon {
            texture,
            points,
            light: FaceLight {
                mins,
                size,
                data,
                at,
            },
            axes,
        });
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
        let luxels = &lighting[range.clone()];
        let [w, h] = light.size;
        for y in 0..h + 2 {
            for x in 0..w + 2 {
                let sx = x.saturating_sub(1).min(w - 1);
                let sy = y.saturating_sub(1).min(h - 1);
                let i = ((sy * w + sx) * 3) as usize;
                put(
                    light.at.0 + x,
                    light.at.1 + y,
                    [luxels[i], luxels[i + 1], luxels[i + 2]],
                );
            }
        }
    }

    // Triangles grouped by texture
    let mut out_vertices = Vec::new();
    let mut per_texture: Vec<Vec<u32>> = vec![Vec::new(); textures.len()];
    for Polygon {
        texture,
        points,
        light,
        axes,
    } in &polygons
    {
        let base = out_vertices.len() as u32;
        let [tw, th] = sizes.get(*texture).copied().unwrap_or([1., 1.]);
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
                uv: [s / tw, t / th],
                lightmap_uv,
            });
        }
        if let Some(list) = per_texture.get_mut(*texture) {
            for i in 1..points.len() as u32 - 1 {
                list.extend_from_slice(&[base, base + i, base + i + 1]);
            }
        }
    }
    let mut indices = Vec::new();
    let mut batches = Vec::new();
    for (texture, list) in per_texture.into_iter().enumerate() {
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

    missing_textures.sort();
    missing_textures.dedup();
    WorldMesh {
        vertices: out_vertices,
        indices,
        batches,
        textures,
        missing_textures,
        lightmap: Rgba {
            width: ATLAS_WIDTH,
            height,
            pixels: atlas,
        },
        entities,
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
}
