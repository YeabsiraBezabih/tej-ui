//! Instanced text quad pipeline rendering dynamic glyphs and color emoji from VRAM atlas.

use crate::gpu::atlas::VramAtlas;
use crate::gpu::context::GpuContext;
use crate::gpu::types::{PremulColor, ScissorRect};
use bytemuck::{Pod, Zeroable};
use core::ops::Range;

/// Exactly 80-byte cache-line friendly instanced glyph primitive.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct GlyphInstance {
    /// Logical target position and size `[x, y, width, height]`.
    pub bounds: [f32; 4],
    /// Normalized UV texture coordinates `[u_min, v_min, u_max, v_max]`.
    pub uv_bounds: [f32; 4],
    /// Premultiplied linear RGBA tint color.
    pub color: [f32; 4],
    /// Logical clipping rectangle `[x, y, width, height]`.
    pub clip_bounds: [f32; 4],
    /// Flags `[is_color_emoji: 0 or 1, reserved, reserved, reserved]`.
    pub prim_meta: [u32; 4],
}

const _: () = assert!(core::mem::size_of::<GlyphInstance>() == 80);

impl GlyphInstance {
    #[must_use]
    pub fn new(
        bounds: [f32; 4],
        uv_bounds: [f32; 4],
        color: PremulColor,
        clip_bounds: [f32; 4],
        is_color_emoji: bool,
    ) -> Self {
        Self {
            bounds,
            uv_bounds,
            color: color.channels,
            clip_bounds,
            prim_meta: [u32::from(is_color_emoji), 0, 0, 0],
        }
    }
}

/// A contiguous batch of glyphs sharing a common scissor box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBatch {
    pub instance_range: Range<u32>,
    pub scissor: ScissorRect,
}

/// Buffer pool for instanced glyph quads with zero steady-state heap allocations.
#[derive(Debug)]
pub struct TextBufferPool {
    cpu_glyphs: Vec<GlyphInstance>,
    batches: Vec<TextBatch>,
    gpu_buffer: wgpu::Buffer,
    gpu_capacity: usize,
}

impl TextBufferPool {
    #[must_use]
    pub fn new(device: &wgpu::Device, initial_capacity: usize) -> Self {
        let capacity = initial_capacity.max(64).next_power_of_two();
        let buffer_size = (capacity * core::mem::size_of::<GlyphInstance>()) as u64;

        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_text_storage_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            cpu_glyphs: Vec::with_capacity(capacity),
            batches: Vec::with_capacity(64),
            gpu_buffer,
            gpu_capacity: capacity,
        }
    }

    pub fn begin_frame(&mut self) {
        self.cpu_glyphs.clear();
        self.batches.clear();
    }

    pub fn push_glyph(&mut self, glyph: GlyphInstance, scissor: ScissorRect) {
        let current_index = self.cpu_glyphs.len() as u32;
        self.cpu_glyphs.push(glyph);

        if let Some(last_batch) = self.batches.last_mut() {
            if last_batch.scissor == scissor && last_batch.instance_range.end == current_index {
                last_batch.instance_range.end = current_index + 1;
                return;
            }
        }

        self.batches.push(TextBatch {
            instance_range: current_index..current_index + 1,
            scissor,
        });
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.cpu_glyphs.is_empty() {
            return false;
        }

        let needed_capacity = self.cpu_glyphs.len();
        let mut reallocated = false;

        if needed_capacity > self.gpu_capacity {
            let new_capacity = needed_capacity.next_power_of_two();
            let new_size = (new_capacity * core::mem::size_of::<GlyphInstance>()) as u64;

            self.gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("tej_text_storage_buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.gpu_capacity = new_capacity;
            reallocated = true;
        }

        let bytes = bytemuck::cast_slice(&self.cpu_glyphs);
        queue.write_buffer(&self.gpu_buffer, 0, bytes);

        reallocated
    }

    #[must_use]
    pub fn batches(&self) -> &[TextBatch] {
        &self.batches
    }

    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }

    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.cpu_glyphs.len()
    }
}

/// GPU render pipeline for drawing text and emoji runs directly from VRAM atlas.
#[derive(Debug)]
pub struct TextPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout_0: wgpu::BindGroupLayout,
    bind_group_layout_1: wgpu::BindGroupLayout,
    bind_group_0: wgpu::BindGroup,
    buffer_pool: TextBufferPool,
}

impl TextPipeline {
    /// Constructs a new text quad render pipeline.
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
                label: Some("tej_ui_text_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui_text.wgsl").into()),
            });

        let buffer_pool = TextBufferPool::new(ctx.device(), initial_capacity);

        let bind_group_layout_0 =
            ctx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tej_text_bind_group_layout_0"),
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
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                    label: Some("tej_text_bind_group_layout_1"),
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

        let pipeline_layout =
            ctx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("tej_text_pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout_0, &bind_group_layout_1],
                    push_constant_ranges: &[],
                });

        let pipeline = ctx
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("tej_ui_text_pipeline"),
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
            buffer_pool,
        }
    }

    fn create_bind_group_0(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        storage: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tej_text_bind_group_0"),
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

    /// Create texture bind group for an active VRAM atlas.
    pub fn create_atlas_bind_group(
        &self,
        device: &wgpu::Device,
        atlas: &VramAtlas,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tej_atlas_bind_group"),
            layout: &self.bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(atlas.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(atlas.sampler()),
                },
            ],
        })
    }

    pub fn begin_frame(&mut self) {
        self.buffer_pool.begin_frame();
    }

    pub fn push_glyph(&mut self, glyph: GlyphInstance, scissor: ScissorRect) {
        self.buffer_pool.push_glyph(glyph, scissor);
    }

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

    pub fn render<'rpass>(
        &'rpass self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        atlas_bind_group: &'rpass wgpu::BindGroup,
        vp_width: u32,
        vp_height: u32,
    ) {
        if self.buffer_pool.batches().is_empty() {
            return;
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group_0, &[]);
        rpass.set_bind_group(1, atlas_bind_group, &[]);

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
    pub fn glyph_count(&self) -> usize {
        self.buffer_pool.glyph_count()
    }

    #[must_use]
    pub fn batches(&self) -> &[TextBatch] {
        self.buffer_pool.batches()
    }
}
