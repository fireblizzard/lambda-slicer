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
//! then the sky wherever nothing was drawn, then translucent brush entities
//! from the furthest in, as the game orders them. Each BSP model is drawn
//! with its entity's transform and render mode, every lightmap weighed by
//! its light style's brightness, and animated textures at their frame for
//! the time

use wgpu::util::DeviceExt;

use crate::Rgba;
use crate::camera::{Matrix, entity_matrix};
use crate::model::{ModelMesh, ModelVertex};
use crate::world::{Animation, Batch, FULL_BRIGHT, WorldMesh, WorldVertex};

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// A world vertex's position, texture and lightmap coordinates, style step,
/// styles and flags
const WORLD_ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
    0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Float32, 4 => Uint8x4, 5 => Uint32
];

/// Light style slots, the game's 64 and full brightness
const STYLE_SLOTS: usize = FULL_BRIGHT as usize + 1;

const SHADER: &str = r#"
struct Uniforms {
    view_projection: mat4x4<f32>,
    // x brightness, y time in seconds
    params: vec4<f32>,
    styles: array<vec4<f32>, 17>,
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

struct WorldOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) lightmap_uv: vec2<f32>,
    @location(2) @interpolate(flat) styles: vec4<u32>,
    @location(3) @interpolate(flat) flags: u32,
    @location(4) @interpolate(flat) style_step: f32,
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
    out.clip = u.view_projection * object.model * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.lightmap_uv = lightmap_uv;
    out.styles = styles;
    out.flags = flags;
    out.style_step = style_step;
    return out;
}

@fragment
fn world_fs(v: WorldOut) -> @location(0) vec4<f32> {
    // Turbulent faces carry texels, warped as the game's water is
    var uv = v.uv;
    if ((v.flags & 1u) != 0u) {
        let t = u.params.y;
        uv = vec2<f32>(
            v.uv.x + 8.0 * sin(v.uv.y * 0.125 + t),
            v.uv.y + 8.0 * sin(v.uv.x * 0.125 + t),
        ) / 64.0;
    }
    let a = textureSample(albedo, albedo_sampler, uv);
    let mode = u32(object.mode.x);
    if (mode == 1u) {
        return vec4<f32>(object.color.rgb, object.color.a);
    }
    if (mode == 3u || mode == 5u) {
        return vec4<f32>(a.rgb * object.color.a, 1.0);
    }
    if (a.a < 0.5) {
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
    let lit = min(a.rgb * light * u.params.x, vec3<f32>(1.0));
    if (mode == 2u) {
        return vec4<f32>(lit, object.color.a);
    }
    return vec4<f32>(lit, 1.0);
}

struct ModelOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec3<f32>,
}

@vertex
fn model_vs(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, @location(3) light: vec3<f32>) -> ModelOut {
    var out: ModelOut;
    out.clip = u.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.light = light;
    return out;
}

@fragment
fn model_fs(v: ModelOut) -> @location(0) vec4<f32> {
    let a = textureSample(albedo, albedo_sampler, v.uv);
    if (a.a < 0.5) {
        discard;
    }
        return vec4<f32>(min(a.rgb * v.light * u.params.x, vec3<f32>(1.0)), 1.0);
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
    /// The camera's position, for ordering translucent brushes
    pub eye: [f32; 3],
    /// The client's time in seconds, for animations, light styles and water
    pub time: f32,
    /// Each light style's brightness, 1 for the game's `m`
    pub light_styles: [f32; 64],
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
    textures: Vec<wgpu::BindGroup>,
    corners: usize,
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
    sky: wgpu::RenderPipeline,
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
    sky: Option<SkyGpu>,
    sky_bytes: u64,
    models: Vec<ModelGpu>,
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
                        depth_compare: wgpu::CompareFunction| {
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
                primitive: wgpu::PrimitiveState::default(),
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
        let pipelines = Pipelines {
            opaque: pipeline(
                "world opaque",
                "world",
                &world_groups,
                world_layout(),
                None,
                true,
                less,
            ),
            blend: pipeline(
                "world blend",
                "world",
                &world_groups,
                world_layout(),
                Some(wgpu::BlendState::ALPHA_BLENDING),
                false,
                less,
            ),
            add: pipeline(
                "world add",
                "world",
                &world_groups,
                world_layout(),
                Some(additive),
                false,
                less,
            ),
            model: pipeline(
                "model",
                "model",
                &[Some(&uniforms_layout), Some(&texture_layout)],
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ModelVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x3],
                },
                None,
                true,
                less,
            ),
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
            sky: None,
            sky_bytes: 0,
            models: Vec::new(),
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
            textures,
            corners: mesh.corner_count(),
        });
        ModelHandle(self.models.len() - 1)
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
            bytes.extend_from_slice(bytemuck::cast_slice(&instance.vertices));
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
    fn upload_objects(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, brushes: &[Brush]) {
        let needed = brushes.len().max(1) as u64;
        if needed > self.object_capacity {
            let capacity = needed.next_power_of_two();
            self.gpu_bytes =
                self.gpu_bytes - self.object_capacity * OBJECT_STRIDE + capacity * OBJECT_STRIDE;
            self.objects = object_buffer(device, capacity);
            self.object_group = object_group(device, &self.object_layout, &self.objects);
            self.object_capacity = capacity;
        }

        let mut bytes = vec![0u8; brushes.len() * OBJECT_STRIDE as usize];
        for (i, brush) in brushes.iter().enumerate() {
            let object = ObjectUniform {
                model: entity_matrix(brush.origin, brush.angles),
                color: [brush.color[0], brush.color[1], brush.color[2], brush.amount],
                mode: [brush.mode.code(), 0., 0., 0.],
            };
            let at = i * OBJECT_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<ObjectUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&object));
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.objects, 0, &bytes);
        }
    }

    /// One frame at `size` with `brushes`, the world among them, and
    /// `instances`. Returns whether the targets were made again, for a
    /// resize, and the commands
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        view: &View,
        brushes: &[Brush],
        instances: &[Instance],
    ) -> (bool, wgpu::CommandBuffer) {
        let before = self.targets.as_ref().map(|t| t.size);
        self.ensure_targets(device, size);
        let resized = before != self.targets.as_ref().map(|t| t.size);

        let mut styles = [0f32; 68];
        styles[..64].copy_from_slice(&view.light_styles);
        styles[STYLE_SLOTS - 1] = 1.;
        let uniforms = Uniforms {
            view_projection: view.view_projection,
            params: [1.6, view.time, 0., 0.],
            styles,
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));
        let starts = self.upload_instances(device, queue, instances);
        self.upload_objects(device, queue, brushes);

        // Translucent brushes from the furthest in
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
        let mut translucent: Vec<usize> = (0..brushes.len())
            .filter(|&i| brushes[i].mode.pass() != Pass::Opaque)
            .collect();
        translucent.sort_by(|&a, &b| distance(&brushes[b]).total_cmp(&distance(&brushes[a])));

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
                        store: wgpu::StoreOp::Discard,
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

            if let Some(buffer) = &self.instances {
                pass.set_pipeline(&self.pipelines.model);
                pass.set_vertex_buffer(0, buffer.slice(..));
                for (instance, start) in instances.iter().zip(&starts) {
                    let Some(start) = *start else {
                        continue;
                    };
                    let model = &self.models[instance.model.0];
                    for b in &model.batches {
                        if let Some(texture) = model.textures.get(b.texture) {
                            pass.set_bind_group(1, texture, &[]);
                            pass.draw(start + b.first..start + b.first + b.count, 0..1);
                        }
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

            if let Some(world) = &self.world {
                bind_world(&mut pass, world);
                for &i in &translucent {
                    let pipeline = match brushes[i].mode.pass() {
                        Pass::Add => &self.pipelines.add,
                        _ => &self.pipelines.blend,
                    };
                    pass.set_pipeline(pipeline);
                    draw_brush(&mut pass, world, i);
                }
            }
        }
        (resized, encoder.finish())
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
