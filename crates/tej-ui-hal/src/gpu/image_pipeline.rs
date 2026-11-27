//! Instanced textured-quad render pipeline for RGBA images and zero-copy 3D bridge textures.

use crate::gpu::bridge_3d::GpuTextureBridge;
use crate::gpu::context::GpuContext;
use crate::gpu::types::ScissorRect;
use bytemuck::{Pod, Zeroable};
use core::ops::Range;

/// WGSL source for the textured-quad image pipeline.
pub const IMAGE_SHADER_WGSL: &str = include_str!("shaders/ui_image.wgsl");

/// Exactly 64-byte cache-line friendly instanced textured quad primitive.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ImageInstance {
    /// Logical destination bounds `[x, y, width, height]`.
    pub bounds: [f32; 4],
    /// Normalized source UV rectangle `[u_min, v_min, u_max, v_max]`.
    pub uv_bounds: [f32; 4],
    /// Logical clipping rectangle `[x, y, width, height]`.
    pub clip_bounds: [f32; 4],
    /// Rendering parameters `[opacity, corner_radius, reserved0, reserved1]`.
    pub params: [f32; 4],
}

const _: () = assert!(core::mem::size_of::<ImageInstance>() == 64);

impl ImageInstance {
    /// Construct a textured quad instance with logical bounds, UV rect, and opacity.
    #[must_use]
    pub fn new(bounds: [f32; 4], uv_bounds: [f32; 4], opacity: f32) -> Self {
        Self {
            bounds,
            uv_bounds,
            clip_bounds: [-1e6, -1e6, 2e6, 2e6],
            params: [opacity.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
        }
    }

    /// Construct a full-texture `[0, 0, 1, 1]` quad at full opacity.
    #[must_use]
    pub fn full_rect(bounds: [f32; 4]) -> Self {
        Self::new(bounds, [0.0, 0.0, 1.0, 1.0], 1.0)
    }

    /// Apply a logical clip rectangle to this image instance.
    #[must_use]
    pub const fn with_clip(mut self, clip_bounds: [f32; 4]) -> Self {
        self.clip_bounds = clip_bounds;
        self
    }

    /// Apply a uniform rounded corner radius (in logical pixels) to this image instance.
    #[must_use]
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.params[1] = radius.max(0.0);
        self
    }

    /// Returns the configured opacity in `[0.0, 1.0]`.
    #[must_use]
    pub const fn opacity(&self) -> f32 {
        self.params[0]
    }

    /// Returns the configured logical corner radius.
    #[must_use]
    pub const fn corner_radius(&self) -> f32 {
        self.params[1]
    }
}

/// A contiguous batch of textured quads sharing a common scissor box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageBatch {
    pub instance_range: Range<u32>,
    pub scissor: ScissorRect,
}

/// Buffer pool for instanced textured quads with zero steady-state heap allocations.
#[derive(Debug)]
pub struct ImageBufferPool {
    cpu_instances: Vec<ImageInstance>,
    batches: Vec<ImageBatch>,
    gpu_buffer: wgpu::Buffer,
    gpu_capacity: usize,
}

impl ImageBufferPool {
    #[must_use]
    pub fn new(device: &wgpu::Device, initial_capacity: usize) -> Self {
        let capacity = initial_capacity.max(16).next_power_of_two();
        let buffer_size = (capacity * core::mem::size_of::<ImageInstance>()) as u64;

        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_image_storage_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            cpu_instances: Vec::with_capacity(capacity),
            batches: Vec::with_capacity(16),
            gpu_buffer,
            gpu_capacity: capacity,
        }
    }

    pub fn begin_frame(&mut self) {
        self.cpu_instances.clear();
        self.batches.clear();
    }

    pub fn push_instance(&mut self, instance: ImageInstance, scissor: ScissorRect) {
        let current_index = self.cpu_instances.len() as u32;
        self.cpu_instances.push(instance);

        if let Some(last_batch) = self.batches.last_mut() {
            if last_batch.scissor == scissor && last_batch.instance_range.end == current_index {
                last_batch.instance_range.end = current_index + 1;
                return;
            }
        }

        self.batches.push(ImageBatch {
            instance_range: current_index..current_index + 1,
            scissor,
        });
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.cpu_instances.is_empty() {
            return false;
        }

        let needed_capacity = self.cpu_instances.len();
        let mut reallocated = false;

        if needed_capacity > self.gpu_capacity {
            let new_capacity = needed_capacity.next_power_of_two();
            let new_size = (new_capacity * core::mem::size_of::<ImageInstance>()) as u64;

            self.gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("tej_image_storage_buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.gpu_capacity = new_capacity;
            reallocated = true;
        }

        let bytes = bytemuck::cast_slice(&self.cpu_instances);
        queue.write_buffer(&self.gpu_buffer, 0, bytes);

        reallocated
    }

    #[must_use]
    pub fn batches(&self) -> &[ImageBatch] {
        &self.batches
    }

    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }

    #[must_use]
    pub fn instance_count(&self) -> usize {
        self.cpu_instances.len()
    }
}

/// GPU render pipeline for compositing RGBA `wgpu::TextureView` textures onto screen quads.
#[derive(Debug)]
pub struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout_0: wgpu::BindGroupLayout,
    bind_group_layout_1: wgpu::BindGroupLayout,
    bind_group_0: wgpu::BindGroup,
    default_sampler: wgpu::Sampler,
    buffer_pool: ImageBufferPool,
    target_format: wgpu::TextureFormat,
}

impl ImagePipeline {
    /// Construct a new textured-quad image pipeline sharing the compositor's viewport uniform buffer.
    #[must_use]
    pub fn new(
        ctx: &GpuContext,
        target_format: wgpu::TextureFormat,
        uniform_buffer: &wgpu::Buffer,
        initial_capacity: usize,
    ) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tej_ui_image_shader"),
                source: wgpu::ShaderSource::Wgsl(IMAGE_SHADER_WGSL.into()),
            });

        let buffer_pool = ImageBufferPool::new(ctx.device(), initial_capacity);

        let bind_group_layout_0 =
            ctx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tej_image_bind_group_layout_0"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group_layout_1 =
            ctx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tej_image_bind_group_layout_1"),
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
                });

        let bind_group_0 = Self::create_bind_group_0(
            ctx.device(),
            &bind_group_layout_0,
            uniform_buffer,
            buffer_pool.buffer(),
        );

        let default_sampler = ctx.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tej_image_default_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout =
            ctx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("tej_image_pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout_0, &bind_group_layout_1],
                    push_constant_ranges: &[],
                });

        let pipeline = ctx
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("tej_ui_image_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            pipeline,
            bind_group_layout_0,
            bind_group_layout_1,
            bind_group_0,
            default_sampler,
            buffer_pool,
            target_format,
        }
    }

    fn create_bind_group_0(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        storage: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tej_image_bind_group_0"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: storage.as_entire_binding(),
                },
            ],
        })
    }

    /// Create bind group 1 for an arbitrary RGBA `wgpu::TextureView` and optional custom sampler.
    #[must_use]
    pub fn create_texture_bind_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampler: Option<&wgpu::Sampler>,
    ) -> wgpu::BindGroup {
        let active_sampler = sampler.unwrap_or(&self.default_sampler);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tej_image_texture_bind_group"),
            layout: &self.bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(active_sampler),
                },
            ],
        })
    }

    /// Create bind group 1 directly from a zero-copy `GpuTextureBridge` (e.g., 3D viewport output).
    #[must_use]
    pub fn create_bridge_bind_group(
        &self,
        device: &wgpu::Device,
        bridge: &GpuTextureBridge,
    ) -> wgpu::BindGroup {
        self.create_texture_bind_group(device, bridge.view(), Some(bridge.sampler()))
    }

    /// Reset CPU image instance pool at the beginning of a frame.
    pub fn begin_frame(&mut self) {
        self.buffer_pool.begin_frame();
    }

    /// Enqueue a textured quad instance under the given physical scissor rect.
    pub fn push_image(&mut self, instance: ImageInstance, scissor: ScissorRect) {
        self.buffer_pool.push_instance(instance, scissor);
    }

    /// Upload queued image instances to the GPU storage buffer, rebinding if grown.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniform_buffer: &wgpu::Buffer,
    ) {
        let reallocated = self.buffer_pool.upload(device, queue);
        if reallocated {
            self.bind_group_0 = Self::create_bind_group_0(
                device,
                &self.bind_group_layout_0,
                uniform_buffer,
                self.buffer_pool.buffer(),
            );
        }
    }

    /// Record draw calls for all queued image instances bound to `texture_bind_group`.
    pub fn render<'rpass>(
        &'rpass self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        texture_bind_group: &'rpass wgpu::BindGroup,
        vp_width: u32,
        vp_height: u32,
    ) {
        if self.buffer_pool.batches().is_empty() {
            return;
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group_0, &[]);
        rpass.set_bind_group(1, texture_bind_group, &[]);

        let mut active_scissor: Option<ScissorRect> = None;

        for batch in self.buffer_pool.batches() {
            let clamped = batch.scissor.clamp_to_viewport(vp_width, vp_height);
            if clamped.is_empty() {
                continue;
            }

            if active_scissor != Some(clamped) {
                rpass.set_scissor_rect(clamped.x, clamped.y, clamped.width, clamped.height);
                active_scissor = Some(clamped);
            }

            rpass.draw(0..4, batch.instance_range.clone());
        }
    }

    #[must_use]
    pub fn image_count(&self) -> usize {
        self.buffer_pool.instance_count()
    }

    #[must_use]
    pub fn batches(&self) -> &[ImageBatch] {
        self.buffer_pool.batches()
    }

    #[must_use]
    pub fn default_sampler(&self) -> &wgpu::Sampler {
        &self.default_sampler
    }

    #[must_use]
    pub const fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }
}
