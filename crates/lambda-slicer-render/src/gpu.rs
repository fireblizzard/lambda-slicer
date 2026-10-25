//! The world, brush entities, studio models and the sky drawn into an
//! offscreen texture with depth, and that texture shown in a window's render
//! pass
//!
//! Colours stay in the game's gamma space from texture to screen, as the
//! game blends them. [`Blit`] converts them only when the window's target
//! format expects linear values. The offscreen targets are made again only
//! when the size changes, and every buffer and texture made here is
//! counted in [`Scene::gpu_bytes`] for the backend measurement.
//!
//! A frame draws the opaque world and brush entities, then studio models,
//! then the sky wherever nothing was drawn, then translucent studio models,
//! the additive textures of solid ones, brush entities and sprites together
//! from the furthest in. The viewmodel
//! comes last, in a pass of its own with the depth cleared, so it never
//! sinks into a wall. Each BSP model is drawn
//! with its entity's transform and render mode, every lightmap weighed by
//! its light style's brightness, and animated textures at their frame for
//! the time

use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::Rgba;
use crate::decal::{DecalBatch, DecalMesh};
use crate::gamma::Gamma;
use lambda_slicer_assets::sprite::{self, Orientation};

use crate::camera::{Matrix, angle_vectors, entity_matrix};
use crate::model::{ModelMesh, ModelVertex};
use crate::world::{Animation, Batch, FULL_BRIGHT, WorldMesh, WorldVertex};

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// A world vertex's position, texture and lightmap coordinates, style step,
/// styles and flags
const WORLD_ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
    0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Float32, 4 => Uint8x4, 5 => Uint32
];

/// A model vertex's position, normal, texture coordinates, light and alpha
const MODEL_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x3, 4 => Float32
];

/// A sprite vertex's position, texture coordinates and colour
const SPRITE_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];

/// Light style slots, the game's 64 and full brightness
const STYLE_SLOTS: usize = FULL_BRIGHT as usize + 1;

/// The way round the world's and brushes' faces wind their corners as
/// they face the camera. The game draws none from behind, so a brush's far
/// side never shows through what is see-through on its near side, such as a
/// ladder's gaps or a pool of water (`R_DrawBrushModel`, `SURF_PLANEBACK`)
const WORLD_FRONT: wgpu::FrontFace = wgpu::FrontFace::Cw;

const SHADER: &str = r#"
// The faces silhouettes leave out, those the shader calls front, as the
// game's models wind their corners the other way round
const SILHOUETTE_BACK: bool = true;
struct Uniforms {
    view_projection: mat4x4<f32>,
    // y time in seconds, z 1 when everything is fully lit, w how many
    // dynamic lights there are
    params: vec4<f32>,
    styles: array<vec4<f32>, 17>,
    // The game's gamma (Gamma::uniform)
    gamma: array<vec4<f32>, 2>,
    // Each dynamic light's origin and radius, then its colour a 256th of
    // each channel's 0 to 255
    lights: array<vec4<f32>, 64>,
}
struct Object {
    model: mat4x4<f32>,
    // rgb the render colour, a the render amount, both 0 to 1
    color: vec4<f32>,
    // x the render mode
    mode: vec4<f32>,
}
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(1) @binding(0) var albedo: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(0) var lightmap: texture_2d<f32>;
@group(2) @binding(1) var lightmap_sampler: sampler;
@group(3) @binding(0) var<uniform> object: Object;

fn style_value(s: u32) -> f32 {
    return u.styles[s / 4u][s % 4u];
}

// A texture colour through the game's texture gamma table
fn tex_gamma(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(u.gamma[0].x));
}

// The game's light gamma table at x, 0 to 1 along it
fn light_table(x: vec3<f32>) -> vec3<f32> {
    var f = pow(max(x, vec3<f32>(0.0)), vec3<f32>(u.gamma[0].y));
    if (u.gamma[1].x > 1.0) {
        f = f * u.gamma[1].x;
    }
    let knee = u.gamma[0].w;
    let low = f / knee * 0.125;
    let high = (f - knee) / (1.0 - knee) * 0.875 + 0.125;
    f = select(high, low, f <= vec3<f32>(knee));
    return pow(max(f, vec3<f32>(0.0)), vec3<f32>(u.gamma[0].z));
}

// What a lightmap's light, 1 for 255 under a normal style, makes of a
// texture's colour, raised before the table or doubled after it
fn lightmap_light(l: vec3<f32>) -> vec3<f32> {
    let index = min(l * 1020.0 * u.gamma[1].y, vec3<f32>(1023.0));
    return light_table(index / 1023.0) * u.gamma[1].z;
}

// How far along its s axis a conveyor's texture has slid by the time, as a
// share of the texture's width. Its brush entity's render colour holds the
// speed, green and blue a 16th of a unit a second, and red set runs it the
// other way (`DrawGLPolyScroll`)
fn scroll_offset() -> f32 {
    let c = round(object.color.rgb * 255.0);
    var speed = (c.g * 256.0 + c.b) / 16.0;
    if (c.r != 0.0) {
        speed = -speed;
    }
    let rate = abs(speed) / f32(textureDimensions(albedo).x);
    let way = select(1.0, -1.0, speed >= 0.0);
    return fract(u.params.y * way * rate);
}

// Whether everything is fully lit, as the game's r_fullbright lights the
// world
fn fullbright() -> bool {
    return u.params.z > 0.5;
}

// The least light a model takes when everything is fully lit, which keeps
// some of its shading
const FULLBRIGHT_MODEL: f32 = 0.75;

// What a model's light, 0 to 1, makes of a texture's colour
fn model_light(l: vec3<f32>) -> vec3<f32> {
    return light_table(clamp(l, vec3<f32>(0.0), vec3<f32>(1.0)) * (1020.0 / 1023.0));
}

// What the dynamic lights add to a face's light at `world`, facing
// `normal`, as the game adds them to its lightmap. Each gives what is left
// of its radius after the way off the face's plane and the way across it,
// so a light lights its own wall brightest (`R_AddDynamicLights`)
fn dynamic_light(world: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    var sum = vec3<f32>(0.0);
    let count = u32(u.params.w);
    for (var k = 0u; k < count; k = k + 1u) {
        let at = u.lights[2u * k];
        let d = world - at.xyz;
        let off = dot(d, normal);
        let across = length(d - normal * off);
        let left = at.w - abs(off) - across;
        if (left > 0.0) {
            sum = sum + u.lights[2u * k + 1u].rgb * left / 255.0;
        }
    }
    return sum;
}

struct WorldOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) lightmap_uv: vec2<f32>,
    @location(2) @interpolate(flat) styles: vec4<u32>,
    @location(3) @interpolate(flat) flags: u32,
    @location(4) @interpolate(flat) style_step: f32,
    @location(5) world: vec3<f32>,
}

@vertex
fn world_vs(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) lightmap_uv: vec2<f32>,
    @location(3) style_step: f32,
    @location(4) styles: vec4<u32>,
    @location(5) flags: u32,
) -> WorldOut {
    var out: WorldOut;
    let world = object.model * vec4<f32>(position, 1.0);
    out.clip = u.view_projection * world;
    out.world = world.xyz;
    out.uv = uv;
    out.lightmap_uv = lightmap_uv;
    out.styles = styles;
    out.flags = flags;
    out.style_step = style_step;
    return out;
}

@fragment
fn world_fs(v: WorldOut) -> @location(0) vec4<f32> {
    // The face's plane, from how the position changes across the pixels
    let normal = normalize(cross(dpdx(v.world), dpdy(v.world)));

    // Turbulent faces carry texels, warped as the game's water is
    var uv = v.uv;
    if ((v.flags & 1u) != 0u) {
        let t = u.params.y;
        uv = vec2<f32>(
            v.uv.x + 8.0 * sin(v.uv.y * 0.125 + t),
            v.uv.y + 8.0 * sin(v.uv.x * 0.125 + t),
        ) / 64.0;
    }
    if ((v.flags & 2u) != 0u) {
        uv.x = uv.x + scroll_offset();
    }
    let a = textureSample(albedo, albedo_sampler, uv);
    let mode = u32(object.mode.x);
    if (mode == 1u) {
        return vec4<f32>(object.color.rgb, object.color.a);
    }
            if (mode == 3u || mode == 5u) {
        return vec4<f32>(tex_gamma(a.rgb) * object.color.a, 1.0);
    }
    // Decals show as much as their texels say, over the face
    let decal = mode == 6u;
    if (!decal && a.a < 0.5) {
        discard;
    }

    var light = vec3<f32>(0.0);
    for (var k = 0u; k < 4u; k = k + 1u) {
        let s = v.styles[k];
        if (s == 255u) {
            break;
        }
        let offset = vec2<f32>(f32(k) * v.style_step, 0.0);
        light = light + textureSample(lightmap, lightmap_sampler, v.lightmap_uv + offset).rgb * style_value(s);
    }
    light = light + dynamic_light(v.world, normal);
    // Fully lit, every face shows its texture's own colours
    if (fullbright()) {
        light = vec3<f32>(1.0);
    }
                // A decal's colour is its own, without the texture table
        let colour = select(tex_gamma(a.rgb), a.rgb, decal);
        let lit = min(colour * lightmap_light(light), vec3<f32>(1.0));
    if (decal) {
        return vec4<f32>(lit, a.a);
    }
    if (mode == 2u) {
        return vec4<f32>(lit, object.color.a);
    }
    return vec4<f32>(lit, 1.0);
}

struct ModelOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec3<f32>,
    @location(2) alpha: f32,
}

@vertex
fn model_vs(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, @location(3) light: vec3<f32>, @location(4) alpha: f32) -> ModelOut {
    var out: ModelOut;
    out.clip = u.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.light = light;
    out.alpha = alpha;
    return out;
}

@fragment
fn model_fs(v: ModelOut) -> @location(0) vec4<f32> {
    let a = textureSample(albedo, albedo_sampler, v.uv);
    if (a.a < 0.5) {
        discard;
    }
    var light = v.light;
    if (fullbright()) {
        light = max(light, vec3<f32>(FULLBRIGHT_MODEL));
    }
    return vec4<f32>(min(tex_gamma(a.rgb) * model_light(light), vec3<f32>(1.0)), v.alpha);
}

// A model behind something nearer, in one colour, its texture's cut-outs
// kept. Faces turned away are left out, so a model in view is not tinted by
// its own far side
@fragment
fn silhouette_fs(v: ModelOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    if (front == SILHOUETTE_BACK) {
        discard;
    }
    let a = textureSample(albedo, albedo_sampler, v.uv);
    if (a.a < 0.5) {
        discard;
    }
    return vec4<f32>(v.light, v.alpha);
}

struct SkyOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// A direction with no translation, at the far plane
@vertex
fn sky_vs(@location(0) direction: vec3<f32>, @location(1) uv: vec2<f32>) -> SkyOut {
    var out: SkyOut;
    let clip = u.view_projection * vec4<f32>(direction, 0.0);
    out.clip = vec4<f32>(clip.xy, clip.w, clip.w);
    out.uv = uv;
    return out;
}

@fragment
fn sky_fs(v: SkyOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(albedo, albedo_sampler, v.uv).rgb, 1.0);
}

struct SpriteOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn sprite_vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) color: vec4<f32>) -> SpriteOut {
    var out: SpriteOut;
    out.clip = u.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.color = color;
    return out;
}

@fragment
fn sprite_fs(v: SpriteOut) -> @location(0) vec4<f32> {
    let a = textureSample(albedo, albedo_sampler, v.uv);
    let c = vec4<f32>(a.rgb * v.color.rgb, a.a * v.color.a);
    // A cut-out sprite's colour is 1 in alpha, which marks it
    if (v.color.a > 1.5 && a.a < 0.5) {
        discard;
    }
    return vec4<f32>(c.rgb, min(c.a, 1.0));
}
"#;

const BLIT_SHADER: &str = r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) i: u32) -> Out {
    let xy = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var out: Out;
    out.clip = vec4<f32>(xy * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(xy.x, 1.0 - xy.y);
    return out;
}

@fragment
fn fs_gamma(v: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(image, image_sampler, v.uv).rgb, 1.0);
}

// The target encodes linear values to sRGB, so gamma values are decoded first
@fragment
fn fs_linear(v: Out) -> @location(0) vec4<f32> {
    let c = textureSample(image, image_sampler, v.uv).rgb;
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return vec4<f32>(select(high, low, c <= vec3<f32>(0.04045)), 1.0);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_projection: Matrix,
    params: [f32; 4],
    styles: [f32; 68],
    gamma: [[f32; 4]; 2],
    lights: [[f32; 4]; 2 * MOST_LIGHTS],
}

/// The most dynamic lights a frame lights the world with, the game's
/// `MAX_DLIGHTS`
pub const MOST_LIGHTS: usize = 32;

/// A light that brightens what is near it for a moment, such as a muzzle
/// flash or an explosion (`dlight_t`)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldLight {
    pub origin: [f32; 3],
    /// How far it reaches, in units
    pub radius: f32,
    /// 0 to 255 for each channel
    pub color: [f32; 3],
}

/// One brush model's draw, at a dynamic offset in the object buffer
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ObjectUniform {
    model: Matrix,
    color: [f32; 4],
    mode: [f32; 4],
}

/// Room for one object, as uniform offsets must be multiples of 256 bytes
const OBJECT_STRIDE: u64 = 256;

/// How an entity is drawn, the game's `kRender` modes
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderMode {
    #[default]
    Normal,
    /// A flat colour, blended
    Color,
    /// Lit and blended by the render amount
    Texture,
    /// Added, as glowing sprites are
    Glow,
    /// Lit, with transparent texels cut out
    Alpha,
    /// Added by the render amount, unlit
    Additive,
}

impl RenderMode {
    /// The game's number for the mode, normal for an unknown one
    pub fn from_game(mode: i64) -> Self {
        match mode {
            1 => RenderMode::Color,
            2 => RenderMode::Texture,
            3 => RenderMode::Glow,
            4 => RenderMode::Alpha,
            5 => RenderMode::Additive,
            _ => RenderMode::Normal,
        }
    }

    fn code(self) -> f32 {
        match self {
            RenderMode::Normal => 0.,
            RenderMode::Color => 1.,
            RenderMode::Texture => 2.,
            RenderMode::Glow => 3.,
            RenderMode::Alpha => 4.,
            RenderMode::Additive => 5.,
        }
    }

    fn pass(self) -> Pass {
        match self {
            RenderMode::Normal | RenderMode::Alpha => Pass::Opaque,
            RenderMode::Color | RenderMode::Texture => Pass::Blend,
            RenderMode::Glow | RenderMode::Additive => Pass::Add,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
    Opaque,
    Blend,
    Add,
}

/// A BSP model drawn this frame, the world or a brush entity's `*N`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brush {
    pub model: usize,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub mode: RenderMode,
    /// 0 to 1
    pub amount: f32,
    pub color: [f32; 3],
    /// The entity's frame is not 0, which shows `+a` textures for `+0` ones
    pub alternate: bool,
}

impl Brush {
    /// The world as the map stores it
    pub fn world() -> Self {
        Self {
            model: 0,
            origin: [0.; 3],
            angles: [0.; 3],
            mode: RenderMode::Normal,
            amount: 1.,
            color: [1.; 3],
            alternate: false,
        }
    }
}

/// What a frame is seen from and when
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub view_projection: Matrix,
    /// The camera's position, for ordering translucent brushes and sprites
    pub eye: [f32; 3],
    /// The camera's angles, which most sprites face
    pub angles: [f32; 3],
    /// The client's time in seconds, for animations, light styles and water
    pub time: f32,
    /// Each light style's brightness, 1 for the game's `m`
    pub light_styles: [f32; 64],
    /// The game's gamma settings
    pub gamma: Gamma,
    /// Every face fully lit and every model lit well, to see into dark
    /// places
    pub fullbright: bool,
    /// The dynamic lights that light the world, the first `light_count`
    pub lights: [WorldLight; MOST_LIGHTS],
    pub light_count: usize,
}

/// The render mode the world's shader takes for decals
const DECAL_MODE: f32 = 6.;

struct DecalGpu {
    revision: u64,
    vertices: wgpu::Buffer,
    /// Vertices the buffer has room for
    capacity: usize,
    batches: Vec<DecalBatch>,
    bytes: u64,
}

struct WorldGpu {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    models: Vec<Vec<Batch>>,
    /// Each model's centre, for ordering translucent brushes
    centres: Vec<[f32; 3]>,
    textures: Vec<wgpu::BindGroup>,
    animations: Vec<Option<Animation>>,
    lightmap: wgpu::BindGroup,
}

impl WorldGpu {
    /// The texture a batch shows at `time`
    fn frame(&self, texture: usize, time: f32, alternate: bool) -> usize {
        let Some(animation) = self.animations.get(texture).and_then(Option::as_ref) else {
            return texture;
        };
        let chain = if alternate && !animation.alternates.is_empty() {
            &animation.alternates
        } else {
            &animation.frames
        };
        if chain.is_empty() {
            return texture;
        }
        chain[(time.max(0.) * 10.) as usize % chain.len()]
    }
}

struct ModelGpu {
    batches: Vec<Batch>,
    /// By batch, whether its texture is added over the scene
    additive: Vec<bool>,
    textures: Vec<wgpu::BindGroup>,
    corners: usize,
}

/// Which of an instance's batches to draw
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    /// Those of textures drawn solid
    Solid,
    /// Those of additive textures
    Additive,
    All,
}

/// A model uploaded once with [`Scene::add_model`], drawn as any number of
/// instances
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModelHandle(usize);

/// One posed copy of a model, its corners skinned in the model's order
#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    pub model: ModelHandle,
    pub vertices: Vec<ModelVertex>,
    /// Its entity's render mode and amount, 0 to 1. A model takes no render
    /// colour, as the game's studio drawing takes none
    pub mode: RenderMode,
    pub amount: f32,
}

impl Instance {
    /// How the game draws a studio model in its entity's render mode,
    /// blended by the amount in the colour, texture and alpha modes and
    /// added by it in the glow and additive modes
    fn pass(&self) -> Pass {
        match self.mode {
            RenderMode::Normal => Pass::Opaque,
            RenderMode::Color | RenderMode::Texture | RenderMode::Alpha => Pass::Blend,
            RenderMode::Glow | RenderMode::Additive => Pass::Add,
        }
    }

    /// The middle of its corners, for drawing translucent ones from the
    /// furthest in
    fn centre(&self) -> [f32; 3] {
        let n = self.vertices.len().max(1) as f32;
        let sum = self.vertices.iter().fold([0.; 3], |s, v| {
            [
                s[0] + v.position[0],
                s[1] + v.position[1],
                s[2] + v.position[2],
            ]
        });
        sum.map(|c| c / n)
    }
}

struct SpriteFrame {
    texture: wgpu::BindGroup,
    origin: [i32; 2],
    size: [u32; 2],
}

struct SpriteGpu {
    orientation: Orientation,
    format: sprite::Format,
    frames: Vec<SpriteFrame>,
}

/// A sprite uploaded once with [`Scene::add_sprite`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpriteHandle(usize);

/// One sprite drawn this frame
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteInstance {
    pub sprite: SpriteHandle,
    /// Wrapped to the sprite's frames
    pub frame: usize,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    /// 0 for the sprite's own size
    pub scale: f32,
    pub mode: RenderMode,
    pub amount: f32,
    /// Black for the sprite's own colours
    pub color: [f32; 3],
    /// A glow keeping its brightness at any distance
    /// (`kRenderFxNoDissipation`)
    pub steady: bool,
}

/// Everything drawn in one frame besides the sky
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawList {
    /// The world first, then brush entities
    pub brushes: Vec<Brush>,
    pub instances: Vec<Instance>,
    pub sprites: Vec<SpriteInstance>,
    /// The first-person weapon, drawn over everything
    pub viewmodel: Vec<Instance>,
    /// Models drawn again where something nearer hides them, each vertex's
    /// light its colour and its alpha how much it shows
    pub silhouettes: Vec<Instance>,
    /// The decals on the world and brush entities, uploaded again when
    /// their revision changes
    pub decals: Option<Arc<DecalMesh>>,
    /// Beams, trails and tracers, added over everything else translucent
    pub streaks: Vec<Streak>,
}

/// A beam, trail or tracer drawn this frame, a strip along its points
/// turned to face the view
#[derive(Clone, Debug, PartialEq)]
pub struct Streak {
    /// The sprite and frame it is textured with, none for a tracer's soft
    /// line
    pub sprite: Option<(SpriteHandle, usize)>,
    pub points: Vec<StreakPoint>,
    /// Units either side of the line
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreakPoint {
    pub position: [f32; 3],
    /// How far along the texture, a whole texture each 1
    pub v: f32,
    /// Added, 0 to 1
    pub color: [f32; 3],
}

/// The quads of `streak` facing the eye at `eye`, two triangles each
fn streak_quads(streak: &Streak, eye: [f32; 3]) -> Vec<SpriteVertex> {
    let mut out = Vec::new();
    for pair in streak.points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let along = [0, 1, 2].map(|k| b.position[k] - a.position[k]);
        let to_eye = [0, 1, 2].map(|k| eye[k] - a.position[k]);
        let side = [
            along[1] * to_eye[2] - along[2] * to_eye[1],
            along[2] * to_eye[0] - along[0] * to_eye[2],
            along[0] * to_eye[1] - along[1] * to_eye[0],
        ];
        let length = (side[0] * side[0] + side[1] * side[1] + side[2] * side[2]).sqrt();
        if length < 1e-6 {
            continue;
        }
        let side = side.map(|v| v / length * streak.width);
        let corner = |p: StreakPoint, sign: f32, u: f32| SpriteVertex {
            position: [0, 1, 2].map(|k| p.position[k] + side[k] * sign),
            uv: [u, p.v],
            color: [p.color[0], p.color[1], p.color[2], 1.],
        };
        let (a0, a1, b1, b0) = (
            corner(a, 1., 0.),
            corner(a, -1., 1.),
            corner(b, -1., 1.),
            corner(b, 1., 0.),
        );
        out.extend([a0, a1, b1, a0, b1, b0]);
    }
    out
}

/// A soft white dot, clear at its edges, for tracers
fn dot() -> Rgba {
    const SIZE: u32 = 8;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let d = |c: u32| (c as f32 + 0.5) / SIZE as f32 * 2. - 1.;
            let r = (d(x) * d(x) + d(y) * d(y)).sqrt();
            pixels.extend([255, 255, 255, ((1. - r).clamp(0., 1.) * 255.) as u8]);
        }
    }
    Rgba {
        width: SIZE,
        height: SIZE,
        pixels,
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteVertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
}

/// The pipeline a sprite draws with
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpritePass {
    Cut,
    Blend,
    Add,
    Glow,
}

/// A translucent thing to draw, by its place in its own list
#[derive(Clone, Copy, Debug, PartialEq)]
enum Translucent {
    Model(usize),
    /// The additive textures of a model drawn solid
    Additive(usize),
    Brush(usize),
    Sprite(usize),
}

/// Translucent models, brushes and sprites, each with its squared distance
/// from the eye, in the order they are drawn, the furthest first. Equally
/// far ones keep the order given
fn translucent_order(items: impl Iterator<Item = (f32, Translucent)>) -> Vec<Translucent> {
    let mut items: Vec<(f32, Translucent)> = items.collect();
    items.sort_by(|a, b| b.0.total_cmp(&a.0));
    items.into_iter().map(|(_, item)| item).collect()
}

/// How bright a glowing sprite is at `distance`, as the game fades them
/// (`GlowBlend`, without its test for what hides the sprite)
fn glow_brightness(distance: f32) -> f32 {
    (19000. / (distance * distance).max(1.)).clamp(0.05, 1.)
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0. {
        v.map(|c| c / length)
    } else {
        v
    }
}

/// A sprite's six corners as two triangles, its pipeline and its distance
/// from the camera, turned as its orientation turns it and drawn as its
/// format and render mode say (`R_DrawSpriteModel`)
fn sprite_quad(
    (orientation, format): (Orientation, sprite::Format),
    frame_origin: [i32; 2],
    frame_size: [u32; 2],
    s: &SpriteInstance,
    view: &View,
) -> ([SpriteVertex; 6], SpritePass, f32) {
    let (forward, camera_right, camera_up) = angle_vectors(view.angles);
    let to = [0, 1, 2].map(|k| s.origin[k] - view.eye[k]);
    let (right, up) = match orientation {
        Orientation::Parallel => (camera_right, camera_up),
        Orientation::ParallelUpright => (normalize([forward[1], -forward[0], 0.]), [0., 0., 1.]),
        Orientation::FacingUpright => (normalize([to[1], -to[0], 0.]), [0., 0., 1.]),
        Orientation::Oriented => {
            let (_, r, u) = angle_vectors(s.angles);
            (r, u)
        }
        Orientation::ParallelOriented => {
            let (sr, cr) = s.angles[2].to_radians().sin_cos();
            (
                [0, 1, 2].map(|k| camera_right[k] * cr + camera_up[k] * sr),
                [0, 1, 2].map(|k| -camera_right[k] * sr + camera_up[k] * cr),
            )
        }
    };

    let distance = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
    let mut scale = if s.scale > 0. { s.scale } else { 1. };
    if s.mode == RenderMode::Glow && !s.steady {
        // A glow keeps its size on screen, growing with its distance
        // (`R_SpriteGlowBlend`)
        scale *= distance / 200.;
    }
    let down = (frame_origin[1] - frame_size[1] as i32) as f32 * scale;
    let top = frame_origin[1] as f32 * scale;
    let left = frame_origin[0] as f32 * scale;
    let far_right = (frame_origin[0] + frame_size[0] as i32) as f32 * scale;

    // Every mode takes the render colour, black meaning none
    let tint = if s.color == [0.; 3] { [1.; 3] } else { s.color };
    let (pass, color) = match s.mode {
        // A sprite whose texels carry their alpha in their index blends by
        // it even drawn normally, as AG's battery does, a sprite in place
        // of the model
        RenderMode::Normal if format == sprite::Format::IndexAlpha => {
            (SpritePass::Blend, [tint[0], tint[1], tint[2], 1.])
        }
        RenderMode::Normal => (SpritePass::Cut, [tint[0], tint[1], tint[2], 2.]),
        // A sprite blends by its texels' own alpha and the render amount,
        // as smoke's soft edges need, rather than cutting them out as a
        // brush or model in this mode does
        RenderMode::Alpha => (SpritePass::Blend, [tint[0], tint[1], tint[2], s.amount]),
        RenderMode::Color => (
            SpritePass::Blend,
            [s.color[0], s.color[1], s.color[2], s.amount],
        ),
        RenderMode::Texture => (SpritePass::Blend, [tint[0], tint[1], tint[2], s.amount]),
        RenderMode::Additive => (SpritePass::Add, [tint[0], tint[1], tint[2], s.amount]),
        RenderMode::Glow => {
            let amount = if s.steady {
                s.amount
            } else {
                s.amount * glow_brightness(distance)
            };
            (SpritePass::Glow, [tint[0], tint[1], tint[2], amount])
        }
    };
    let corner = |v: f32, h: f32, uv: [f32; 2]| SpriteVertex {
        position: [0, 1, 2].map(|k| s.origin[k] + up[k] * v + right[k] * h),
        uv,
        color,
    };
    let a = corner(down, left, [0., 1.]);
    let b = corner(top, left, [0., 0.]);
    let c = corner(top, far_right, [1., 0.]);
    let d = corner(down, far_right, [1., 1.]);
    ([a, b, c, a, c, d], pass, distance)
}

struct SkyGpu {
    vertices: wgpu::Buffer,
    faces: Vec<wgpu::BindGroup>,
}

struct Targets {
    size: [u32; 2],
    color_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
}

struct Pipelines {
    opaque: wgpu::RenderPipeline,
    blend: wgpu::RenderPipeline,
    add: wgpu::RenderPipeline,
    model: wgpu::RenderPipeline,
    model_blend: wgpu::RenderPipeline,
    model_add: wgpu::RenderPipeline,
    silhouette: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    sprite_cut: wgpu::RenderPipeline,
    sprite_blend: wgpu::RenderPipeline,
    sprite_add: wgpu::RenderPipeline,
    sprite_glow: wgpu::RenderPipeline,
}

pub struct Scene {
    uniforms: wgpu::Buffer,
    uniform_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    object_layout: wgpu::BindGroupLayout,
    objects: wgpu::Buffer,
    object_group: wgpu::BindGroup,
    object_capacity: u64,
    repeat: wgpu::Sampler,
    clamp: wgpu::Sampler,
    pipelines: Pipelines,
    world: Option<WorldGpu>,
    world_bytes: u64,
    decals: Option<DecalGpu>,
    /// Decal textures by their place in `decals.wad`
    decal_textures: HashMap<u16, wgpu::BindGroup>,
    /// The tracers' soft line, made on the first frame that has one
    dot: Option<wgpu::BindGroup>,
    sky: Option<SkyGpu>,
    sky_bytes: u64,
    models: Vec<ModelGpu>,
    sprites: Vec<SpriteGpu>,
    /// Every sprite's corners for the frame, grown as needed
    sprite_vertices: Option<wgpu::Buffer>,
    sprite_capacity: u64,
    /// Every instance's corners for the frame, grown as needed
    instances: Option<wgpu::Buffer>,
    instance_capacity: u64,
    targets: Option<Targets>,
    /// Bytes of every buffer and texture made here, targets included
    gpu_bytes: u64,
    target_bytes: u64,
}

fn texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("viewer texture"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn uniform_layout(device: &wgpu::Device, label: &str, dynamic: bool) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn sampler(device: &wgpu::Device, mode: wgpu::AddressMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("viewer sampler"),
        address_mode_u: mode,
        address_mode_v: mode,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

fn object_buffer(device: &wgpu::Device, capacity: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("viewer objects"),
        size: capacity * OBJECT_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn object_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("viewer objects"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<ObjectUniform>() as u64),
            }),
        }],
    })
}

/// The six sky faces as the game's sky box lays them out, each a quad of
/// directions and texture coordinates
fn sky_vertices() -> Vec<[f32; 5]> {
    // Each axis's direction components from (s, t, 1), and the image of
    // `rt`, `bk`, `lf`, `ft`, `up`, `dn` it shows
    const AXES: [[i32; 3]; 6] = [
        [3, -1, 2],
        [-3, 1, 2],
        [1, 3, 2],
        [-1, -3, 2],
        [-2, -1, 3],
        [2, -1, -3],
    ];
    let mut out = Vec::with_capacity(36);
    for axis in AXES {
        let corner = |s: f32, t: f32| {
            let b = [s, t, 1.];
            let d = axis.map(|k| {
                if k < 0 {
                    -b[(-k - 1) as usize]
                } else {
                    b[(k - 1) as usize]
                }
            });
            [d[0], d[1], d[2], (s + 1.) / 2., 1. - (t + 1.) / 2.]
        };
        let quad = [
            corner(-1., -1.),
            corner(-1., 1.),
            corner(1., 1.),
            corner(1., -1.),
        ];
        out.extend([quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]]);
    }
    out
}

/// The sky images in the order of [`sky_vertices`]' axes
pub const SKY_FACES: [&str; 6] = ["rt", "lf", "bk", "ft", "up", "dn"];

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewer"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let uniforms_layout = uniform_layout(device, "viewer uniforms", false);
        let object_layout = uniform_layout(device, "viewer objects", true);
        let texture_layout = texture_layout(device);
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewer uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer uniforms"),
            layout: &uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });
        let objects = object_buffer(device, 16);
        let object_group = object_group(device, &object_layout, &objects);

        let pipeline = |name: &str,
                        entry: &str,
                        layouts: &[Option<&wgpu::BindGroupLayout>],
                        buffer: wgpu::VertexBufferLayout<'_>,
                        blend: Option<wgpu::BlendState>,
                        depth_write: bool,
                        depth_compare: wgpu::CompareFunction,
                        cull: Option<wgpu::Face>| {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(name),
                bind_group_layouts: layouts,
                immediate_size: 0,
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some(&format!("{entry}_vs")),
                    compilation_options: Default::default(),
                    buffers: &[Some(buffer)],
                },
                primitive: wgpu::PrimitiveState {
                    front_face: WORLD_FRONT,
                    cull_mode: cull,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(depth_compare),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(&format!("{entry}_fs")),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: COLOR_FORMAT,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let world_layout = || wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<WorldVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &WORLD_ATTRIBUTES,
        };
        let world_groups = [
            Some(&uniforms_layout),
            Some(&texture_layout),
            Some(&texture_layout),
            Some(&object_layout),
        ];
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        };
        let less = wgpu::CompareFunction::Less;
        let added_by_alpha = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        };
        let sprite_pipeline = |name: &str, blend, write, compare| {
            pipeline(
                name,
                "sprite",
                &[Some(&uniforms_layout), Some(&texture_layout)],
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<SpriteVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &SPRITE_ATTRIBUTES,
                },
                blend,
                write,
                compare,
                None,
            )
        };
        let model_pipeline = |name: &str, blend, write| {
            pipeline(
                name,
                "model",
                &[Some(&uniforms_layout), Some(&texture_layout)],
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ModelVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &MODEL_ATTRIBUTES,
                },
                blend,
                write,
                less,
                None,
            )
        };
        // Only where something nearer was drawn
        let silhouette_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("silhouette"),
                bind_group_layouts: &[Some(&uniforms_layout), Some(&texture_layout)],
                immediate_size: 0,
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("silhouette"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("model_vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ModelVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &MODEL_ATTRIBUTES,
                    })],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Greater),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("silhouette_fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: COLOR_FORMAT,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipelines = Pipelines {
            opaque: pipeline(
                "world opaque",
                "world",
                &world_groups,
                world_layout(),
                None,
                true,
                less,
                Some(wgpu::Face::Back),
            ),
            blend: pipeline(
                "world blend",
                "world",
                &world_groups,
                world_layout(),
                Some(wgpu::BlendState::ALPHA_BLENDING),
                false,
                less,
                Some(wgpu::Face::Back),
            ),
            add: pipeline(
                "world add",
                "world",
                &world_groups,
                world_layout(),
                Some(additive),
                false,
                less,
                Some(wgpu::Face::Back),
            ),
            model: model_pipeline("model", None, true),
            model_blend: model_pipeline(
                "model blend",
                Some(wgpu::BlendState::ALPHA_BLENDING),
                false,
            ),
            model_add: model_pipeline("model add", Some(added_by_alpha), false),
            silhouette: silhouette_pipeline,
            sky: pipeline(
                "sky",
                "sky",
                &[Some(&uniforms_layout), Some(&texture_layout)],
                wgpu::VertexBufferLayout {
                    array_stride: 20,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                },
                None,
                false,
                wgpu::CompareFunction::LessEqual,
                None,
            ),
            sprite_cut: sprite_pipeline("sprite cut", None, true, less),
            sprite_blend: sprite_pipeline(
                "sprite blend",
                Some(wgpu::BlendState::ALPHA_BLENDING),
                false,
                less,
            ),
            sprite_add: sprite_pipeline("sprite add", Some(added_by_alpha), false, less),
            sprite_glow: sprite_pipeline(
                "sprite glow",
                Some(added_by_alpha),
                false,
                wgpu::CompareFunction::Always,
            ),
        };

        Self {
            gpu_bytes: std::mem::size_of::<Uniforms>() as u64 + 16 * OBJECT_STRIDE,
            uniforms,
            uniform_group,
            repeat: sampler(device, wgpu::AddressMode::Repeat),
            clamp: sampler(device, wgpu::AddressMode::ClampToEdge),
            texture_layout,
            object_layout,
            objects,
            object_group,
            object_capacity: 16,
            pipelines,
            world: None,
            world_bytes: 0,
            decals: None,
            decal_textures: HashMap::new(),
            dot: None,
            sky: None,
            sky_bytes: 0,
            models: Vec::new(),
            sprites: Vec::new(),
            sprite_vertices: None,
            sprite_capacity: 0,
            instances: None,
            instance_capacity: 0,
            targets: None,
            target_bytes: 0,
        }
    }

    /// Bytes of buffers and textures this scene holds on the GPU
    pub fn gpu_bytes(&self) -> u64 {
        self.gpu_bytes + self.target_bytes
    }

    fn texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &Rgba,
        repeat: bool,
    ) -> wgpu::BindGroup {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("viewer texture"),
                size: wgpu::Extent3d {
                    width: image.width,
                    height: image.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &image.pixels,
        );
        self.gpu_bytes += image.pixels.len() as u64;
        let view = texture.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer texture"),
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(if repeat {
                        &self.repeat
                    } else {
                        &self.clamp
                    }),
                },
            ],
        })
    }

    /// Draw `mesh` as the world and brush models from now on, in place of
    /// any other
    pub fn set_world(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &WorldMesh) {
        self.world = None;
        self.gpu_bytes -= self.world_bytes;
        if let Some(decals) = self.decals.take() {
            self.gpu_bytes -= decals.bytes;
        }
        let before = self.gpu_bytes;

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world vertices"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world indices"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.gpu_bytes +=
            (std::mem::size_of_val(mesh.vertices.as_slice()) + mesh.indices.len() * 4) as u64;
        let textures = mesh
            .textures
            .iter()
            .map(|t| self.texture(device, queue, t, true))
            .collect();
        let lightmap = self.texture(device, queue, &mesh.lightmap, false);

        let centres = mesh
            .models
            .iter()
            .map(|batches| {
                let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                for b in batches {
                    for &i in &mesh.indices[b.first as usize..(b.first + b.count) as usize] {
                        let p = mesh.vertices[i as usize].position;
                        for k in 0..3 {
                            lo[k] = lo[k].min(p[k]);
                            hi[k] = hi[k].max(p[k]);
                        }
                    }
                }
                [0, 1, 2].map(|k| {
                    if lo[k] <= hi[k] {
                        (lo[k] + hi[k]) / 2.
                    } else {
                        0.
                    }
                })
            })
            .collect();

        self.world = Some(WorldGpu {
            vertices,
            indices,
            models: mesh.models.clone(),
            centres,
            textures,
            animations: mesh.animations.clone(),
            lightmap,
        });
        self.world_bytes = self.gpu_bytes - before;
    }

    /// Draw these six images, in [`SKY_FACES`] order, behind everything.
    /// None draws no sky
    pub fn set_sky(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        faces: Option<&[Rgba; 6]>,
    ) {
        self.sky = None;
        self.gpu_bytes -= self.sky_bytes;
        let Some(faces) = faces else {
            self.sky_bytes = 0;
            return;
        };
        let before = self.gpu_bytes;

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sky vertices"),
            contents: bytemuck::cast_slice(&sky_vertices()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.gpu_bytes += 36 * 20;
        let faces = faces
            .iter()
            .map(|f| self.texture(device, queue, f, false))
            .collect();
        self.sky = Some(SkyGpu { vertices, faces });
        self.sky_bytes = self.gpu_bytes - before;
    }

    /// Upload a model's textures, for instances of it to be drawn
    pub fn add_model(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: &ModelMesh,
    ) -> ModelHandle {
        let textures = mesh
            .textures
            .iter()
            .map(|t| self.texture(device, queue, t, true))
            .collect();
        self.models.push(ModelGpu {
            batches: mesh.batches.clone(),
            additive: mesh
                .batches
                .iter()
                .map(|b| mesh.additive(b.texture))
                .collect(),
            textures,
            corners: mesh.corner_count(),
        });
        ModelHandle(self.models.len() - 1)
    }

    /// Upload a sprite's frames, for instances of it to be drawn
    pub fn add_sprite(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sprite: &sprite::Sprite,
    ) -> SpriteHandle {
        let frames = sprite
            .frames
            .iter()
            .map(|f| {
                let image = Rgba {
                    width: f.width.max(1),
                    height: f.height.max(1),
                    pixels: if f.width * f.height > 0 {
                        sprite.rgba(f)
                    } else {
                        vec![0; 4]
                    },
                };
                SpriteFrame {
                    texture: self.texture(device, queue, &image, false),
                    origin: f.origin,
                    size: [f.width, f.height],
                }
            })
            .collect();
        self.sprites.push(SpriteGpu {
            orientation: sprite.orientation,
            format: sprite.format,
            frames,
        });
        SpriteHandle(self.sprites.len() - 1)
    }

    /// Every sprite's corners in one buffer, each sprite's pipeline and
    /// place in it, the furthest first
    /// Every sprite's quad from the furthest in, then every streak's, and
    /// where each begins in the buffer
    #[allow(clippy::type_complexity)]
    fn upload_sprites(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &View,
        sprites: &[SpriteInstance],
        streaks: &[Streak],
    ) -> (Vec<(usize, SpritePass, u32)>, Vec<(usize, u32, u32)>) {
        let mut quads: Vec<(f32, usize, SpritePass, [SpriteVertex; 6])> = sprites
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let gpu = self.sprites.get(s.sprite.0)?;
                let frame = gpu.frames.get(s.frame % gpu.frames.len().max(1))?;
                let (quad, pass, distance) = sprite_quad(
                    (gpu.orientation, gpu.format),
                    frame.origin,
                    frame.size,
                    s,
                    view,
                );
                Some((distance, i, pass, quad))
            })
            .collect();
        quads.sort_by(|a, b| b.0.total_cmp(&a.0));

        let mut bytes = Vec::with_capacity(quads.len() * 6 * std::mem::size_of::<SpriteVertex>());
        let mut order = Vec::with_capacity(quads.len());
        for (k, (_, i, pass, quad)) in quads.iter().enumerate() {
            bytes.extend_from_slice(bytemuck::cast_slice(quad));
            order.push((*i, *pass, k as u32 * 6));
        }
        let mut strips = Vec::with_capacity(streaks.len());
        let mut first = quads.len() as u32 * 6;
        for (i, streak) in streaks.iter().enumerate() {
            let vertices = streak_quads(streak, view.eye);
            if vertices.is_empty() {
                continue;
            }
            bytes.extend_from_slice(bytemuck::cast_slice(&vertices));
            strips.push((i, first, vertices.len() as u32));
            first += vertices.len() as u32;
        }
        if bytes.is_empty() {
            return (order, strips);
        }
        if bytes.len() as u64 > self.sprite_capacity {
            let capacity = (bytes.len() as u64).next_power_of_two();
            self.gpu_bytes = self.gpu_bytes - self.sprite_capacity + capacity;
            self.sprite_capacity = capacity;
            self.sprite_vertices = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite vertices"),
                size: capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(buffer) = &self.sprite_vertices {
            queue.write_buffer(buffer, 0, &bytes);
        }
        (order, strips)
    }

    /// The last frame drawn, read back from the GPU, waiting for it. For
    /// captures and tests, never every frame
    pub fn read_color(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Rgba> {
        let targets = self.targets.as_ref()?;
        let texture = targets.color_view.texture();
        let [width, height] = targets.size;
        let row = width * 4;
        let padded =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewer readback"),
            size: u64::from(padded * height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: None,
                },
            },
            texture.size(),
        );
        queue.submit([encoder.finish()]);
        buffer.map_async(wgpu::MapMode::Read, .., |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let data = buffer.get_mapped_range(..).ok()?;
        let pixels = data
            .chunks(padded as usize)
            .flat_map(|r| &r[..row as usize])
            .copied()
            .collect();
        Some(Rgba {
            width,
            height,
            pixels,
        })
    }

    /// The offscreen colour target, once a frame was drawn
    pub fn color_view(&self) -> Option<&wgpu::TextureView> {
        self.targets.as_ref().map(|t| &t.color_view)
    }

    fn ensure_targets(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        let size = [size[0].max(1), size[1].max(1)];
        if self.targets.as_ref().is_some_and(|t| t.size == size) {
            return;
        }
        let target = |format, usage| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("viewer target"),
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        self.targets = Some(Targets {
            size,
            color_view: target(
                COLOR_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            ),
            depth_view: target(DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT),
        });
        self.target_bytes = u64::from(size[0]) * u64::from(size[1]) * 8;
    }

    /// Every instance's corners in one buffer, and where each instance
    /// starts in it. An instance whose corner count does not match its model
    /// is left out
    fn upload_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Instance],
    ) -> Vec<Option<u32>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut starts = Vec::with_capacity(instances.len());
        let mut corners = 0u32;
        for instance in instances {
            let fits = self
                .models
                .get(instance.model.0)
                .is_some_and(|m| m.corners == instance.vertices.len());
            if !fits {
                starts.push(None);
                continue;
            }
            starts.push(Some(corners));
            if instance.pass() == Pass::Opaque {
                bytes.extend_from_slice(bytemuck::cast_slice(&instance.vertices));
            } else {
                let faded: Vec<ModelVertex> = instance
                    .vertices
                    .iter()
                    .map(|v| ModelVertex {
                        alpha: instance.amount,
                        ..*v
                    })
                    .collect();
                bytes.extend_from_slice(bytemuck::cast_slice(&faded));
            }
            corners += instance.vertices.len() as u32;
        }
        if bytes.is_empty() {
            return starts;
        }

        if bytes.len() as u64 > self.instance_capacity {
            let capacity = (bytes.len() as u64).next_power_of_two();
            self.gpu_bytes = self.gpu_bytes - self.instance_capacity + capacity;
            self.instance_capacity = capacity;
            self.instances = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("model instances"),
                size: capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(buffer) = &self.instances {
            queue.write_buffer(buffer, 0, &bytes);
        }
        starts
    }

    /// Each brush's object uniform at its own offset, the buffer grown as
    /// needed
    /// Each brush's object, then each brush's again for its decals
    fn upload_objects(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, brushes: &[Brush]) {
        let needed = (brushes.len() * 2).max(1) as u64;
        if needed > self.object_capacity {
            let capacity = needed.next_power_of_two();
            self.gpu_bytes =
                self.gpu_bytes - self.object_capacity * OBJECT_STRIDE + capacity * OBJECT_STRIDE;
            self.objects = object_buffer(device, capacity);
            self.object_group = object_group(device, &self.object_layout, &self.objects);
            self.object_capacity = capacity;
        }

        let mut bytes = vec![0u8; brushes.len() * 2 * OBJECT_STRIDE as usize];
        let decals = brushes.iter().map(|brush| ObjectUniform {
            model: entity_matrix(brush.origin, brush.angles),
            color: [1.; 4],
            mode: [DECAL_MODE, 0., 0., 0.],
        });
        let objects = brushes
            .iter()
            .map(|brush| ObjectUniform {
                model: entity_matrix(brush.origin, brush.angles),
                color: [brush.color[0], brush.color[1], brush.color[2], brush.amount],
                mode: [brush.mode.code(), 0., 0., 0.],
            })
            .chain(decals);
        for (i, object) in objects.enumerate() {
            let at = i * OBJECT_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<ObjectUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&object));
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.objects, 0, &bytes);
        }
    }

    /// The decals' triangles and any textures they bring, when they changed
    fn upload_decals(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: Option<&DecalMesh>,
    ) {
        let Some(mesh) = mesh else {
            if let Some(old) = self.decals.take() {
                self.gpu_bytes -= old.bytes;
            }
            return;
        };
        if self
            .decals
            .as_ref()
            .is_some_and(|d| d.revision == mesh.revision)
        {
            return;
        }
        for (index, image) in mesh.textures.iter() {
            if !self.decal_textures.contains_key(index) {
                let group = self.texture(device, queue, image, false);
                self.decal_textures.insert(*index, group);
            }
        }
        // Only the decals made since are sent when the buffer has room
        if let Some(gpu) = &mut self.decals
            && mesh.previous == Some(gpu.revision)
            && mesh.vertices.len() <= gpu.capacity
        {
            let stride = std::mem::size_of::<WorldVertex>();
            let added = &mesh.vertices[mesh.kept.min(mesh.vertices.len())..];
            if !added.is_empty() {
                let offset = (mesh.kept * stride) as u64;
                queue.write_buffer(&gpu.vertices, offset, bytemuck::cast_slice(added));
            }
            gpu.revision = mesh.revision;
            gpu.batches = mesh.batches.clone();
            return;
        }
        if let Some(old) = self.decals.take() {
            self.gpu_bytes -= old.bytes;
        }
        if mesh.vertices.is_empty() {
            return;
        }
        // Room to grow, so the decals made next go in without a new buffer
        let capacity = mesh.vertices.len().next_power_of_two().max(1024);
        let bytes = (capacity * std::mem::size_of::<WorldVertex>()) as u64;
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("decal vertices"),
            size: bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertices, 0, bytemuck::cast_slice(&mesh.vertices));
        self.gpu_bytes += bytes;
        self.decals = Some(DecalGpu {
            revision: mesh.revision,
            vertices,
            capacity,
            batches: mesh.batches.clone(),
            bytes,
        });
    }

    /// One frame at `size` of everything in `draw`. Returns whether the
    /// targets were made again, for a resize, and the commands
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        view: &View,
        draw: &DrawList,
    ) -> (bool, wgpu::CommandBuffer) {
        let brushes = &draw.brushes;
        let before = self.targets.as_ref().map(|t| t.size);
        self.ensure_targets(device, size);
        let resized = before != self.targets.as_ref().map(|t| t.size);

        let mut styles = [0f32; 68];
        styles[..64].copy_from_slice(&view.light_styles);
        styles[STYLE_SLOTS - 1] = 1.;
        let count = view.light_count.min(MOST_LIGHTS);
        let mut lights = [[0f32; 4]; 2 * MOST_LIGHTS];
        for (k, light) in view.lights[..count].iter().enumerate() {
            let [x, y, z] = light.origin;
            let [r, g, b] = light.color.map(|c| c / 256.);
            lights[2 * k] = [x, y, z, light.radius];
            lights[2 * k + 1] = [r, g, b, 0.];
        }
        let uniforms = Uniforms {
            view_projection: view.view_projection,
            params: [
                0.,
                view.time,
                f32::from(u8::from(view.fullbright)),
                count as f32,
            ],
            styles,
            gamma: view.gamma.uniform(),
            lights,
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));
        let models: Vec<Instance> = draw
            .instances
            .iter()
            .chain(&draw.viewmodel)
            .chain(&draw.silhouettes)
            .cloned()
            .collect();
        let starts = self.upload_instances(device, queue, &models);
        let (starts, rest) = starts.split_at(draw.instances.len());
        let (viewmodel_starts, silhouette_starts) = rest.split_at(draw.viewmodel.len());
        let instances = &draw.instances;
        if self.dot.is_none() && draw.streaks.iter().any(|s| s.sprite.is_none()) {
            self.dot = Some(self.texture(device, queue, &dot(), false));
        }
        let (sprites, streaks) =
            self.upload_sprites(device, queue, view, &draw.sprites, &draw.streaks);
        self.upload_objects(device, queue, brushes);
        self.upload_decals(device, queue, draw.decals.as_deref());

        // Translucent models, brushes and sprites together, from the
        // furthest in, as the game sorts its translucent entities
        let distance = |b: &Brush| {
            let centre = self
                .world
                .as_ref()
                .and_then(|w| w.centres.get(b.model))
                .copied()
                .unwrap_or([0.; 3]);
            let p = [0, 1, 2].map(|k| centre[k] + b.origin[k] - view.eye[k]);
            p[0] * p[0] + p[1] * p[1] + p[2] * p[2]
        };
        let from_eye = |p: [f32; 3]| {
            let d = [0, 1, 2].map(|k| p[k] - view.eye[k]);
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
        };
        // A solid model's additive textures are added with the translucent
        // things, as they write no depth
        let models = (0..instances.len()).filter_map(|i| {
            let item = if instances[i].pass() != Pass::Opaque {
                Translucent::Model(i)
            } else if self.has_additive(&instances[i]) {
                Translucent::Additive(i)
            } else {
                return None;
            };
            Some((from_eye(instances[i].centre()), item))
        });
        let translucent_brushes = (0..brushes.len())
            .filter(|&i| brushes[i].mode.pass() != Pass::Opaque)
            .map(|i| (distance(&brushes[i]), Translucent::Brush(i)));
        let sprite_items = sprites
            .iter()
            .enumerate()
            .map(|(k, &(i, ..))| (from_eye(draw.sprites[i].origin), Translucent::Sprite(k)));
        let translucent = translucent_order(models.chain(translucent_brushes).chain(sprite_items));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("viewer"),
        });
        let targets = self.targets.as_ref().expect("targets were made");
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewer"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.1,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.uniform_group, &[]);

            let draw_brush = |pass: &mut wgpu::RenderPass<'_>, world: &WorldGpu, i: usize| {
                let brush = &brushes[i];
                let Some(batches) = world.models.get(brush.model) else {
                    return;
                };
                let offset = (i as u64 * OBJECT_STRIDE) as u32;
                pass.set_bind_group(3, &self.object_group, &[offset]);
                for b in batches {
                    let texture = world.frame(b.texture, view.time, brush.alternate);
                    pass.set_bind_group(1, &world.textures[texture], &[]);
                    pass.draw_indexed(b.first..b.first + b.count, 0, 0..1);
                }
            };
            let bind_world = |pass: &mut wgpu::RenderPass<'_>, world: &WorldGpu| {
                pass.set_vertex_buffer(0, world.vertices.slice(..));
                pass.set_index_buffer(world.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.set_bind_group(2, &world.lightmap, &[]);
            };

            if let Some(world) = &self.world {
                pass.set_pipeline(&self.pipelines.opaque);
                bind_world(&mut pass, world);
                for (i, brush) in brushes.iter().enumerate() {
                    if brush.mode.pass() == Pass::Opaque {
                        draw_brush(&mut pass, world, i);
                    }
                }
            }

            // Decals over the world and the brushes that carry them
            if let (Some(world), Some(decals)) = (&self.world, &self.decals) {
                pass.set_pipeline(&self.pipelines.blend);
                pass.set_vertex_buffer(0, decals.vertices.slice(..));
                pass.set_bind_group(2, &world.lightmap, &[]);
                for (i, brush) in brushes.iter().enumerate() {
                    let offset = ((brushes.len() + i) as u64 * OBJECT_STRIDE) as u32;
                    pass.set_bind_group(3, &self.object_group, &[offset]);
                    for b in decals.batches.iter().filter(|b| b.model == brush.model) {
                        if let Some(texture) = self.decal_textures.get(&b.texture) {
                            pass.set_bind_group(1, texture, &[]);
                            pass.draw(b.first..b.first + b.count, 0..1);
                        }
                    }
                }
            }

            if let Some(buffer) = &self.instances {
                pass.set_pipeline(&self.pipelines.model);
                pass.set_vertex_buffer(0, buffer.slice(..));
                for (instance, start) in instances.iter().zip(starts) {
                    if instance.pass() == Pass::Opaque {
                        self.draw_instance(&mut pass, instance, *start, Part::Solid);
                    }
                }
            }

            if let Some(sky) = &self.sky {
                pass.set_pipeline(&self.pipelines.sky);
                pass.set_vertex_buffer(0, sky.vertices.slice(..));
                for (face, group) in sky.faces.iter().enumerate() {
                    pass.set_bind_group(1, group, &[]);
                    let first = face as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
            }

            // Each kind's vertices are bound again when the kind changes
            let mut bound = None;
            for &item in &translucent {
                let kind = std::mem::discriminant(&item);
                let rebind = bound != Some(kind);
                bound = Some(kind);
                match item {
                    Translucent::Model(i) => {
                        let Some(buffer) = &self.instances else {
                            continue;
                        };
                        if rebind {
                            pass.set_vertex_buffer(0, buffer.slice(..));
                        }
                        let instance = &instances[i];
                        pass.set_pipeline(match instance.pass() {
                            Pass::Add => &self.pipelines.model_add,
                            _ => &self.pipelines.model_blend,
                        });
                        self.draw_instance(&mut pass, instance, starts[i], Part::All);
                    }
                    Translucent::Additive(i) => {
                        let Some(buffer) = &self.instances else {
                            continue;
                        };
                        if rebind {
                            pass.set_vertex_buffer(0, buffer.slice(..));
                        }
                        pass.set_pipeline(&self.pipelines.model_add);
                        self.draw_instance(&mut pass, &instances[i], starts[i], Part::Additive);
                    }
                    Translucent::Brush(i) => {
                        let Some(world) = &self.world else {
                            continue;
                        };
                        if rebind {
                            bind_world(&mut pass, world);
                        }
                        pass.set_pipeline(match brushes[i].mode.pass() {
                            Pass::Add => &self.pipelines.add,
                            _ => &self.pipelines.blend,
                        });
                        draw_brush(&mut pass, world, i);
                    }
                    Translucent::Sprite(k) => {
                        let Some(buffer) = &self.sprite_vertices else {
                            continue;
                        };
                        if rebind {
                            pass.set_vertex_buffer(0, buffer.slice(..));
                        }
                        let (i, kind, first) = sprites[k];
                        let s = &draw.sprites[i];
                        let gpu = &self.sprites[s.sprite.0];
                        let frame = &gpu.frames[s.frame % gpu.frames.len()];
                        pass.set_pipeline(match kind {
                            SpritePass::Cut => &self.pipelines.sprite_cut,
                            SpritePass::Blend => &self.pipelines.sprite_blend,
                            SpritePass::Add => &self.pipelines.sprite_add,
                            SpritePass::Glow => &self.pipelines.sprite_glow,
                        });
                        pass.set_bind_group(1, &frame.texture, &[]);
                        pass.draw(first..first + 6, 0..1);
                    }
                }
            }
            if let Some(buffer) = &self.sprite_vertices
                && !streaks.is_empty()
            {
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.set_pipeline(&self.pipelines.sprite_add);
                for &(i, first, count) in &streaks {
                    let texture = match draw.streaks[i].sprite {
                        Some((sprite, frame)) => self
                            .sprites
                            .get(sprite.0)
                            .and_then(|gpu| gpu.frames.get(frame % gpu.frames.len().max(1)))
                            .map(|f| &f.texture),
                        None => self.dot.as_ref(),
                    };
                    if let Some(texture) = texture {
                        pass.set_bind_group(1, texture, &[]);
                        pass.draw(first..first + count, 0..1);
                    }
                }
            }
        }

        // Silhouettes over everything but the viewmodel, where hidden
        if let Some(buffer) = &self.instances
            && silhouette_starts.iter().any(Option::is_some)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("silhouettes"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.uniform_group, &[]);
            pass.set_pipeline(&self.pipelines.silhouette);
            pass.set_vertex_buffer(0, buffer.slice(..));
            for (instance, start) in draw.silhouettes.iter().zip(silhouette_starts) {
                self.draw_instance(&mut pass, instance, *start, Part::All);
            }
        }

        // The viewmodel over everything, against a cleared depth
        if let Some(buffer) = &self.instances
            && viewmodel_starts.iter().any(Option::is_some)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewmodel"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.uniform_group, &[]);
            pass.set_vertex_buffer(0, buffer.slice(..));
            for (instance, start) in draw.viewmodel.iter().zip(viewmodel_starts) {
                match instance.pass() {
                    Pass::Opaque => {
                        pass.set_pipeline(&self.pipelines.model);
                        self.draw_instance(&mut pass, instance, *start, Part::Solid);
                        pass.set_pipeline(&self.pipelines.model_add);
                        self.draw_instance(&mut pass, instance, *start, Part::Additive);
                    }
                    Pass::Blend => {
                        pass.set_pipeline(&self.pipelines.model_blend);
                        self.draw_instance(&mut pass, instance, *start, Part::All);
                    }
                    Pass::Add => {
                        pass.set_pipeline(&self.pipelines.model_add);
                        self.draw_instance(&mut pass, instance, *start, Part::All);
                    }
                }
            }
        }
        (resized, encoder.finish())
    }

    /// Whether any of an instance's textures is additive
    fn has_additive(&self, instance: &Instance) -> bool {
        self.models
            .get(instance.model.0)
            .is_some_and(|m| m.additive.contains(&true))
    }

    /// One uploaded instance's batches of `part`, the pipeline and its
    /// vertex buffer already set
    fn draw_instance(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        instance: &Instance,
        start: Option<u32>,
        part: Part,
    ) {
        let Some(start) = start else {
            return;
        };
        let model = &self.models[instance.model.0];
        for (b, &additive) in model.batches.iter().zip(&model.additive) {
            let wanted = match part {
                Part::Solid => !additive,
                Part::Additive => additive,
                Part::All => true,
            };
            if !wanted {
                continue;
            }
            if let Some(texture) = model.textures.get(b.texture) {
                pass.set_bind_group(1, texture, &[]);
                pass.draw(start + b.first..start + b.first + b.count, 0..1);
            }
        }
    }
}

/// Draws an offscreen image over a render pass's viewport
pub struct Blit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Blit {
    pub fn new(device: &wgpu::Device, target: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewer blit"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });
        let layout = texture_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewer blit"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("viewer blit"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(if target.is_srgb() {
                    "fs_linear"
                } else {
                    "fs_gamma"
                }),
                compilation_options: Default::default(),
                targets: &[Some(target.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            layout,
            sampler: sampler(device, wgpu::AddressMode::ClampToEdge),
        }
    }

    pub fn bind_group(&self, device: &wgpu::Device, view: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer blit"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    pub fn paint(&self, pass: &mut wgpu::RenderPass<'_>, group: &wgpu::BindGroup) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn translucent_models_brushes_and_sprites_are_drawn_together_furthest_first() {
        // A glass gib in front of a window, with a sprite between them and
        // another behind both
        let items = [
            (100., Translucent::Model(0)),
            (900., Translucent::Brush(3)),
            (400., Translucent::Sprite(1)),
            (1600., Translucent::Sprite(0)),
        ];
        assert_eq!(
            translucent_order(items.into_iter()),
            [
                Translucent::Sprite(0),
                Translucent::Brush(3),
                Translucent::Sprite(1),
                Translucent::Model(0),
            ]
        );
    }

    use super::*;

    fn view() -> View {
        View {
            view_projection: [[0.; 4]; 4],
            eye: [0.; 3],
            angles: [0.; 3],
            time: 0.,
            light_styles: [1.; 64],
            gamma: Gamma::default(),
            fullbright: false,
            lights: [WorldLight::default(); MOST_LIGHTS],
            light_count: 0,
        }
    }

    fn sprite(mode: RenderMode) -> SpriteInstance {
        SpriteInstance {
            sprite: SpriteHandle(0),
            frame: 0,
            origin: [100., 0., 0.],
            angles: [0.; 3],
            scale: 2.,
            mode,
            amount: 0.5,
            color: [0.; 3],
            steady: false,
        }
    }

    #[test]
    fn a_streak_is_a_strip_across_the_view() {
        // Along x, seen from above, so its width runs along y
        let point = |x: f32, v: f32| StreakPoint {
            position: [x, 0., 0.],
            v,
            color: [1., 0.5, 0.],
        };
        let streak = Streak {
            sprite: None,
            points: vec![point(0., 0.), point(10., 1.), point(30., 2.)],
            width: 2.,
        };
        let quads = streak_quads(&streak, [5., 0., 100.]);
        assert_eq!(quads.len(), 12);
        for v in &quads {
            assert!((v.position[1].abs() - 2.).abs() < 1e-5);
            assert_eq!(v.position[2], 0.);
        }
        // Its texture runs along it by each point's v, across it from 0 to 1
        assert_eq!(quads[0].uv, [0., 0.]);
        assert_eq!(quads[2].uv, [1., 1.]);
        assert_eq!(quads[11].uv, [0., 2.]);
        // A streak pointing at the eye has no width to draw
        let end_on = streak_quads(&streak, [100., 0., 0.]);
        assert!(end_on.is_empty());
    }

    #[test]
    fn a_translucent_sprite_blends_by_its_own_alpha_and_tint() {
        // Smoke, dark grey and soft at its edges rather than cut out
        let smoke = SpriteInstance {
            color: [0.1; 3],
            amount: 1.,
            ..sprite(RenderMode::Alpha)
        };
        let (quad, pass, _) = sprite_quad(
            (Orientation::Parallel, sprite::Format::Normal),
            [-1, 1],
            [2, 2],
            &smoke,
            &view(),
        );
        assert_eq!(pass, SpritePass::Blend);
        assert_eq!(quad[0].color, [0.1, 0.1, 0.1, 1.]);

        let (_, pass, _) = sprite_quad(
            (Orientation::Parallel, sprite::Format::Normal),
            [-1, 1],
            [2, 2],
            &sprite(RenderMode::Normal),
            &view(),
        );
        assert_eq!(pass, SpritePass::Cut);

        // Unless its texels carry their own alpha, as AG's battery's do
        let (quad, pass, _) = sprite_quad(
            (Orientation::Parallel, sprite::Format::IndexAlpha),
            [-1, 1],
            [2, 2],
            &sprite(RenderMode::Normal),
            &view(),
        );
        assert_eq!(pass, SpritePass::Blend);
        assert_eq!(quad[0].color[3], 1.);
    }

    #[test]
    fn sprites_face_the_view_and_take_their_frame_offset() {
        // Looking along x, right is -y and up is z. A 2 by 2 frame whose
        // top left is 1 left and 1 up of the origin, at scale 2
        let (quad, pass, distance) = sprite_quad(
            (Orientation::Parallel, sprite::Format::Normal),
            [-1, 1],
            [2, 2],
            &sprite(RenderMode::Normal),
            &view(),
        );
        assert_eq!(pass, SpritePass::Cut);
        assert_eq!(distance, 100.);
        assert_eq!(quad[0].position, [100., 2., -2.]);
        assert_eq!(quad[1].position, [100., 2., 2.]);
        assert_eq!(quad[2].position, [100., -2., 2.]);
        assert_eq!(quad[1].uv, [0., 0.]);

        // An upright sprite facing the camera's position stays vertical
        let mut v = view();
        v.angles = [45., 0., 0.];
        let (quad, _, _) = sprite_quad(
            (Orientation::FacingUpright, sprite::Format::Normal),
            [-1, 1],
            [2, 2],
            &sprite(RenderMode::Additive),
            &v,
        );
        assert_eq!(quad[0].position[0], quad[1].position[0]);

        // Additive sprites without a render colour are white, glow fades far
        let (quad, pass, _) = sprite_quad(
            (Orientation::Parallel, sprite::Format::Normal),
            [0, 0],
            [1, 1],
            &sprite(RenderMode::Additive),
            &view(),
        );
        assert_eq!((pass, quad[0].color), (SpritePass::Add, [1., 1., 1., 0.5]));
        assert!(glow_brightness(1000.) < glow_brightness(100.));
    }
}
