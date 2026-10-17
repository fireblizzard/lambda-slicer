//! The world and model drawn into an offscreen texture with depth, and that
//! texture shown in a window's render pass
//!
//! Colours stay in the game's gamma space from texture to screen, as the
//! game blends them. [`Blit`] converts them only when the window's target
//! format expects linear values. The offscreen targets are made again only
//! when the size changes, and every buffer and texture made here is
//! counted in [`Scene::gpu_bytes`] for the backend measurement

use wgpu::util::DeviceExt;

use crate::Rgba;
use crate::camera::Matrix;
use crate::model::{ModelMesh, ModelVertex};
use crate::world::{Batch, WorldMesh, WorldVertex};

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

const SHADER: &str = r#"
struct Uniforms {
    view_projection: mat4x4<f32>,
    params: vec4<f32>,
}
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(1) @binding(0) var albedo: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(0) var lightmap: texture_2d<f32>;
@group(2) @binding(1) var lightmap_sampler: sampler;

struct WorldOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) lightmap_uv: vec2<f32>,
}

@vertex
fn world_vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) lightmap_uv: vec2<f32>) -> WorldOut {
    var out: WorldOut;
    out.clip = u.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.lightmap_uv = lightmap_uv;
    return out;
}

@fragment
fn world_fs(v: WorldOut) -> @location(0) vec4<f32> {
    let a = textureSample(albedo, albedo_sampler, v.uv);
    if (a.a < 0.5) {
        discard;
    }
    let light = textureSample(lightmap, lightmap_sampler, v.lightmap_uv).rgb;
    return vec4<f32>(min(a.rgb * light * u.params.x, vec3<f32>(1.0)), 1.0);
}

struct ModelOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn model_vs(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>) -> ModelOut {
    var out: ModelOut;
    out.clip = u.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.normal = normal;
    return out;
}

@fragment
fn model_fs(v: ModelOut) -> @location(0) vec4<f32> {
    let a = textureSample(albedo, albedo_sampler, v.uv);
    if (a.a < 0.5) {
        discard;
    }
    let shade = 0.55 + 0.45 * max(dot(normalize(v.normal), normalize(vec3<f32>(0.4, 0.3, 0.85))), 0.0);
    return vec4<f32>(a.rgb * shade, 1.0);
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
}

struct WorldGpu {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    batches: Vec<Batch>,
    textures: Vec<wgpu::BindGroup>,
    lightmap: wgpu::BindGroup,
}

struct ModelGpu {
    mesh_batches: Vec<Batch>,
    textures: Vec<wgpu::BindGroup>,
    vertices: wgpu::Buffer,
    capacity: u64,
}

struct Targets {
    size: [u32; 2],
    color_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
}

pub struct Scene {
    uniforms: wgpu::Buffer,
    uniform_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    repeat: wgpu::Sampler,
    clamp: wgpu::Sampler,
    world_pipeline: wgpu::RenderPipeline,
    model_pipeline: wgpu::RenderPipeline,
    world: Option<WorldGpu>,
    model: Option<ModelGpu>,
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

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewer"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewer uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let texture_layout = texture_layout(device);
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewer uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer uniforms"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let pipeline = |name: &str,
                        layouts: &[Option<&wgpu::BindGroupLayout>],
                        buffer: wgpu::VertexBufferLayout<'_>| {
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
                    entry_point: Some(&format!("{name}_vs")),
                    compilation_options: Default::default(),
                    buffers: &[Some(buffer)],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(&format!("{name}_fs")),
                    compilation_options: Default::default(),
                    targets: &[Some(COLOR_FORMAT.into())],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let world_pipeline = pipeline(
            "world",
            &[
                Some(&uniform_layout),
                Some(&texture_layout),
                Some(&texture_layout),
            ],
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<WorldVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x2],
            },
        );
        let model_pipeline = pipeline(
            "model",
            &[Some(&uniform_layout), Some(&texture_layout)],
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ModelVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
            },
        );

        Self {
            uniforms,
            uniform_group,
            repeat: sampler(device, wgpu::AddressMode::Repeat),
            clamp: sampler(device, wgpu::AddressMode::ClampToEdge),
            texture_layout,
            world_pipeline,
            model_pipeline,
            world: None,
            model: None,
            targets: None,
            gpu_bytes: std::mem::size_of::<Uniforms>() as u64,
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
        sampler: bool,
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
                    resource: wgpu::BindingResource::Sampler(if sampler {
                        &self.repeat
                    } else {
                        &self.clamp
                    }),
                },
            ],
        })
    }

    pub fn set_world(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &WorldMesh) {
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
        self.world = Some(WorldGpu {
            vertices,
            indices,
            batches: mesh.batches.clone(),
            textures,
            lightmap,
        });
    }

    pub fn set_model(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &ModelMesh) {
        let capacity = (mesh.corner_count().max(1) * std::mem::size_of::<ModelVertex>()) as u64;
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model vertices"),
            size: capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.gpu_bytes += capacity;
        let textures = mesh
            .textures
            .iter()
            .map(|t| self.texture(device, queue, t, true))
            .collect();
        self.model = Some(ModelGpu {
            mesh_batches: mesh.batches.clone(),
            textures,
            vertices,
            capacity,
        });
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

    /// One frame at `size`, the model drawn from `model_vertices`. Returns
    /// whether the targets were made again, for a resize, and the commands
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        view_projection: Matrix,
        model_vertices: &[ModelVertex],
    ) -> (bool, wgpu::CommandBuffer) {
        let before = self.targets.as_ref().map(|t| t.size);
        self.ensure_targets(device, size);
        let resized = before != self.targets.as_ref().map(|t| t.size);
        let uniforms = Uniforms {
            view_projection,
            params: [1.6, 0., 0., 0.],
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));
        if let Some(model) = &self.model {
            let bytes: &[u8] = bytemuck::cast_slice(model_vertices);
            if (bytes.len() as u64) <= model.capacity {
                queue.write_buffer(&model.vertices, 0, bytes);
            }
        }

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
            if let Some(world) = &self.world {
                pass.set_pipeline(&self.world_pipeline);
                pass.set_vertex_buffer(0, world.vertices.slice(..));
                pass.set_index_buffer(world.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.set_bind_group(2, &world.lightmap, &[]);
                for b in &world.batches {
                    pass.set_bind_group(1, &world.textures[b.texture], &[]);
                    pass.draw_indexed(b.first..b.first + b.count, 0, 0..1);
                }
            }
            if let Some(model) = &self.model
                && !model_vertices.is_empty()
            {
                pass.set_pipeline(&self.model_pipeline);
                pass.set_vertex_buffer(0, model.vertices.slice(..));
                for b in &model.mesh_batches {
                    if let Some(texture) = model.textures.get(b.texture) {
                        pass.set_bind_group(1, texture, &[]);
                        pass.draw(b.first..b.first + b.count, 0..1);
                    }
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
