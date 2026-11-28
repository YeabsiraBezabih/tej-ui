//! GPU render pipeline for tessellated 2D vector paths and immediate-mode canvas geometry.

use crate::gpu::context::GpuContext;
use crate::gpu::types::ScissorRect;
use crate::gpu::vector::tessellator::{TessellatedMesh, VectorVertex};

/// GPU render pipeline and vertex/index buffer cache for 2D vector geometry.
#[derive(Debug)]
pub struct VectorPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    current_index_count: u32,
}

impl VectorPipeline {
    const INITIAL_VERTEX_CAPACITY: usize = 2048;
    const INITIAL_INDEX_CAPACITY: usize = 6144;

    /// Constructs a new vector render pipeline bound to the shared viewport uniform.
    #[must_use]
    pub fn new(
        ctx: &GpuContext,
        target_format: wgpu::TextureFormat,
        uniform_buffer: &wgpu::Buffer,
    ) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tej_ui_vector_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui_vector.wgsl").into()),
            });

        let bind_group_layout =
            ctx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tej_vector_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tej_vector_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline =
            Self::create_render_pipeline(ctx.device(), &shader, target_format, &bind_group_layout);

        let vertex_buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_vector_vertex_buffer"),
            size: (Self::INITIAL_VERTEX_CAPACITY * core::mem::size_of::<VectorVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_vector_index_buffer"),
            size: (Self::INITIAL_INDEX_CAPACITY * core::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            vertex_buffer,
            index_buffer,
            vertex_capacity: Self::INITIAL_VERTEX_CAPACITY,
            index_capacity: Self::INITIAL_INDEX_CAPACITY,
            current_index_count: 0,
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        target_format: wgpu::TextureFormat,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tej_vector_pipeline_layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<VectorVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
            ],
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tej_ui_vector_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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
        })
    }

    /// Uploads mesh vertex and index data to VRAM, reallocating GPU buffers if needed.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &TessellatedMesh) {
        if mesh.is_empty() {
            self.current_index_count = 0;
            return;
        }

        if mesh.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = mesh.vertices.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("tej_vector_vertex_buffer"),
                size: (self.vertex_capacity * core::mem::size_of::<VectorVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if mesh.indices.len() > self.index_capacity {
            self.index_capacity = mesh.indices.len().next_power_of_two();
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("tej_vector_index_buffer"),
                size: (self.index_capacity * core::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&mesh.indices));
        self.current_index_count = mesh.indices.len() as u32;
    }

    /// Re-binds uniform buffer if it was reallocated elsewhere.
    pub fn update_uniform_binding(&mut self, device: &wgpu::Device, uniform_buffer: &wgpu::Buffer) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tej_vector_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
    }

    /// Renders uploaded vector geometry into the given render pass.
    pub fn render<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        scissor: Option<ScissorRect>,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        if self.current_index_count == 0 {
            return;
        }

        if let Some(sc) = scissor {
            let clamped = sc.clamp_to_viewport(viewport_width, viewport_height);
            if clamped.is_empty() {
                return;
            }
            rpass.set_scissor_rect(clamped.x, clamped.y, clamped.width, clamped.height);
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.current_index_count, 0, 0..1);
    }

    #[must_use]
    pub const fn index_count(&self) -> u32 {
        self.current_index_count
    }
}
