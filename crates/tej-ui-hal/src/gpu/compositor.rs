//! Hardware-accelerated 2D UI compositor submitting instanced draw calls via WGPU.

use crate::gpu::atlas::VramAtlas;
use crate::gpu::buffer_ring::InstanceBufferPool;
use crate::gpu::clip_stack::ClipStack;
use crate::gpu::context::GpuContext;
use crate::gpu::image_pipeline::{ImageInstance, ImagePipeline};
use crate::gpu::quad_pipeline::{create_quad_bind_group, create_quad_pipeline};
use crate::gpu::surface::RenderTarget;
use crate::gpu::text_pipeline::{GlyphInstance, TextPipeline};
use crate::gpu::types::{
    BackendConfig, GpuError, PremulColor, QuadInstance, ScissorRect, ViewportUniform,
};
use crate::gpu::vector::{TessellatedMesh, VectorPipeline};

/// Per-frame performance telemetry and draw metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameStats {
    pub instance_count: u32,
    pub batch_count: u32,
    pub scissor_changes: u32,
    pub buffer_reallocated: bool,
}

/// Hardware GPU compositor executing instanced rounded-rect and shadow passes with zero allocation.
#[derive(Debug)]
pub struct GpuCompositor {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    buffer_pool: InstanceBufferPool,
    text_pipeline: TextPipeline,
    vector_pipeline: VectorPipeline,
    image_pipeline: ImagePipeline,
    clip_stack: ClipStack,
    clear_color: PremulColor,
    viewport_width_px: u32,
    viewport_height_px: u32,
    scale_factor: f32,
    target_format: wgpu::TextureFormat,
}

impl GpuCompositor {
    /// Construct a new GPU compositor for a given target texture format and configuration.
    #[must_use]
    pub fn new(
        ctx: &GpuContext,
        target_format: wgpu::TextureFormat,
        config: &BackendConfig,
    ) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tej_ui_quad_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui_quad.wgsl").into()),
            });

        let uniform_buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_viewport_uniform"),
            size: core::mem::size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buffer_pool = InstanceBufferPool::new(ctx.device(), config.initial_instance_capacity);

        let (bind_group_layout, pipeline) =
            create_quad_pipeline(ctx.device(), target_format, &shader);

        let bind_group = create_quad_bind_group(
            ctx.device(),
            &bind_group_layout,
            &uniform_buffer,
            buffer_pool.buffer(),
        );

        let text_pipeline = TextPipeline::new(
            ctx,
            target_format,
            &uniform_buffer,
            config.initial_instance_capacity,
        );

        let vector_pipeline = VectorPipeline::new(ctx, target_format, &uniform_buffer);

        let image_pipeline = ImagePipeline::new(ctx, target_format, &uniform_buffer, 64);

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            uniform_buffer,
            buffer_pool,
            text_pipeline,
            vector_pipeline,
            image_pipeline,
            clip_stack: ClipStack::new(),
            clear_color: PremulColor::TRANSPARENT,
            viewport_width_px: 1,
            viewport_height_px: 1,
            scale_factor: 1.0,
            target_format,
        }
    }

    /// Begin a new rendering frame, resetting pools and viewport uniforms.
    pub fn begin_frame(
        &mut self,
        width_px: u32,
        height_px: u32,
        scale_factor: f32,
        clear_color: PremulColor,
    ) {
        self.viewport_width_px = width_px.max(1);
        self.viewport_height_px = height_px.max(1);
        self.scale_factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        self.clear_color = clear_color;

        self.buffer_pool.begin_frame();
        self.text_pipeline.begin_frame();
        self.image_pipeline.begin_frame();
        self.clip_stack.reset(
            self.viewport_width_px,
            self.viewport_height_px,
            self.scale_factor,
        );
    }

    /// Push an analytic clipping rectangle with corner radii.
    pub fn push_clip(&mut self, logical_bounds: [f32; 4], corner_radii: [f32; 4]) {
        self.clip_stack
            .push(logical_bounds, corner_radii, self.scale_factor);
    }

    /// Pop the top clipping rectangle.
    pub fn pop_clip(&mut self) -> Result<(), GpuError> {
        self.clip_stack.pop()
    }

    /// Enqueue a raw quad instance, injecting active clipping bounds.
    pub fn draw_quad(&mut self, mut instance: QuadInstance) {
        if self.clip_stack.is_culled() {
            return;
        }

        let cur_clip = self.clip_stack.current();
        instance.clip_bounds = cur_clip.clip_bounds;
        instance.clip_radii = cur_clip.clip_radii;

        self.buffer_pool.push_instance(instance, cur_clip.scissor);
    }

    /// Convenience helper for drawing a solid rectangle.
    pub fn draw_rect(&mut self, bounds: [f32; 4], color: PremulColor) {
        self.draw_quad(QuadInstance::solid_rect(bounds, color));
    }

    /// Convenience helper for drawing a rounded rectangle.
    pub fn draw_rounded_rect(&mut self, bounds: [f32; 4], radii: [f32; 4], color: PremulColor) {
        self.draw_quad(QuadInstance::rounded_rect(bounds, radii, color));
    }

    /// Convenience helper for drawing a bordered rectangle.
    pub fn draw_bordered_rect(
        &mut self,
        bounds: [f32; 4],
        radii: [f32; 4],
        fill: PremulColor,
        border: PremulColor,
        widths: [f32; 4],
    ) {
        self.draw_quad(QuadInstance::bordered_rect(
            bounds, radii, fill, border, widths,
        ));
    }

    /// Convenience helper for drawing a soft drop shadow.
    pub fn draw_drop_shadow(
        &mut self,
        bounds: [f32; 4],
        radii: [f32; 4],
        color: PremulColor,
        blur: f32,
        spread: f32,
    ) {
        self.draw_quad(QuadInstance::drop_shadow(
            bounds, radii, color, blur, spread,
        ));
    }

    /// Enqueue a glyph quad instance with active clipping applied.
    pub fn draw_glyph(
        &mut self,
        bounds: [f32; 4],
        uv_bounds: [f32; 4],
        color: PremulColor,
        kind: super::text_pipeline::GlyphKind,
    ) {
        if self.clip_stack.is_culled() {
            return;
        }

        let cur_clip = self.clip_stack.current();
        let inst = GlyphInstance::new(bounds, uv_bounds, color, cur_clip.clip_bounds, kind);
        self.text_pipeline.push_glyph(inst, cur_clip.scissor);
    }

    /// Enqueue a textured image quad with logical bounds, UV rect, and opacity.
    pub fn draw_textured_quad(&mut self, bounds: [f32; 4], uv_bounds: [f32; 4], opacity: f32) {
        self.draw_image_instance(ImageInstance::new(bounds, uv_bounds, opacity));
    }

    /// Enqueue a raw `ImageInstance`, injecting active clipping bounds and scissor.
    pub fn draw_image_instance(&mut self, mut instance: ImageInstance) {
        if self.clip_stack.is_culled() {
            return;
        }

        let cur_clip = self.clip_stack.current();
        instance.clip_bounds = cur_clip.clip_bounds;
        self.image_pipeline.push_image(instance, cur_clip.scissor);
    }

    /// Render queued textured quads bound to `texture_view` onto `target_view` using `LoadOp::Load`.
    pub fn render_image_pass(
        &mut self,
        ctx: &GpuContext,
        target_view: &wgpu::TextureView,
        texture_view: &wgpu::TextureView,
        sampler: Option<&wgpu::Sampler>,
    ) {
        if self.image_pipeline.batches().is_empty() {
            return;
        }

        let uniform = ViewportUniform {
            viewport_size: [
                f32::from(u16::try_from(self.viewport_width_px).unwrap_or(u16::MAX)),
                f32::from(u16::try_from(self.viewport_height_px).unwrap_or(u16::MAX)),
            ],
            scale_factor: self.scale_factor,
            pad: 0.0,
        };
        ctx.queue()
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        self.image_pipeline
            .upload(ctx.device(), ctx.queue(), &self.uniform_buffer);

        let tex_bind_group =
            self.image_pipeline
                .create_texture_bind_group(ctx.device(), texture_view, sampler);

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tej_image_pass_encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tej_image_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.image_pipeline.render(
                &mut rpass,
                &tex_bind_group,
                self.viewport_width_px,
                self.viewport_height_px,
            );
        }

        ctx.queue().submit(Some(encoder.finish()));
    }

    /// Submits enqueued primitives to the target surface without an atlas.
    pub fn submit(
        &mut self,
        ctx: &GpuContext,
        target: &mut RenderTarget<'_>,
    ) -> Result<FrameStats, GpuError> {
        self.submit_frame(ctx, target, None, None)
    }

    /// Submits enqueued primitives and text glyphs to the target surface.
    pub fn submit_with_atlas(
        &mut self,
        ctx: &GpuContext,
        target: &mut RenderTarget<'_>,
        atlas: Option<&mut VramAtlas>,
    ) -> Result<FrameStats, GpuError> {
        self.submit_frame(ctx, target, atlas, None)
    }

    /// Submits enqueued quads, text glyphs, and optional vector mesh geometry.
    pub fn submit_frame(
        &mut self,
        ctx: &GpuContext,
        target: &mut RenderTarget<'_>,
        mut atlas: Option<&mut VramAtlas>,
        vector_mesh: Option<&TessellatedMesh>,
    ) -> Result<FrameStats, GpuError> {
        let vector_triangles = vector_mesh.map_or(0, |m| m.indices.len() / 3);
        let instance_count = (self.buffer_pool.instance_count()
            + self.text_pipeline.glyph_count()
            + vector_triangles) as u32;

        let buffer_reallocated = self.upload_frame_state(ctx, vector_mesh);
        let atlas_bind_group = if let Some(ref mut a) = atlas {
            a.flush(ctx.queue());
            Some(self.text_pipeline.create_atlas_bind_group(ctx.device(), a))
        } else {
            None
        };

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tej_compositor_encoder"),
            });

        let mut window_frame = None;
        let target_view = match target {
            RenderTarget::Window(win) => {
                let frame = win.acquire_frame(ctx)?;
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                window_frame = Some(frame);
                view
            }
            RenderTarget::Headless(headless) => headless.view().clone(),
        };

        let batch_count =
            (self.buffer_pool.batches().len() + self.text_pipeline.batches().len()) as u32;
        let scissor_changes = self.encode_render_pass(
            &mut encoder,
            &target_view,
            atlas_bind_group.as_ref(),
            vector_mesh,
        );

        ctx.queue().submit(Some(encoder.finish()));

        if let Some(frame) = window_frame {
            frame.present();
        }

        Ok(FrameStats {
            instance_count,
            batch_count,
            scissor_changes,
            buffer_reallocated,
        })
    }

    fn upload_frame_state(
        &mut self,
        ctx: &GpuContext,
        vector_mesh: Option<&TessellatedMesh>,
    ) -> bool {
        let uniform = ViewportUniform {
            viewport_size: [
                f32::from(u16::try_from(self.viewport_width_px).unwrap_or(u16::MAX)),
                f32::from(u16::try_from(self.viewport_height_px).unwrap_or(u16::MAX)),
            ],
            scale_factor: self.scale_factor,
            pad: 0.0,
        };
        ctx.queue()
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let buffer_reallocated = self.buffer_pool.upload(ctx.device(), ctx.queue());
        if buffer_reallocated {
            self.bind_group = create_quad_bind_group(
                ctx.device(),
                &self.bind_group_layout,
                &self.uniform_buffer,
                self.buffer_pool.buffer(),
            );
        }

        self.text_pipeline
            .upload(ctx.device(), ctx.queue(), &self.uniform_buffer);

        if let Some(mesh) = vector_mesh {
            self.vector_pipeline.upload(ctx.device(), ctx.queue(), mesh);
        }

        buffer_reallocated
    }

    fn encode_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        atlas_bind_group: Option<&wgpu::BindGroup>,
        vector_mesh: Option<&TessellatedMesh>,
    ) -> u32 {
        let clear_wgpu = wgpu::Color {
            r: f64::from(self.clear_color.channels[0]),
            g: f64::from(self.clear_color.channels[1]),
            b: f64::from(self.clear_color.channels[2]),
            a: f64::from(self.clear_color.channels[3]),
        };

        let mut scissor_changes = 0u32;
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tej_compositor_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_wgpu),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if !self.buffer_pool.batches().is_empty() {
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            let mut active_scissor: Option<ScissorRect> = None;
            for batch in self.buffer_pool.batches() {
                let clamped = batch
                    .scissor
                    .clamp_to_viewport(self.viewport_width_px, self.viewport_height_px);
                if clamped.is_empty() {
                    continue;
                }

                if active_scissor != Some(clamped) {
                    rpass.set_scissor_rect(clamped.x, clamped.y, clamped.width, clamped.height);
                    active_scissor = Some(clamped);
                    scissor_changes += 1;
                }

                rpass.draw(0..4, batch.instance_range.clone());
            }
        }

        if let Some(mesh) = vector_mesh
            && !mesh.is_empty()
        {
            // Reset scissor to full viewport so the last quad batch's scissor does not clip vectors.
            rpass.set_scissor_rect(0, 0, self.viewport_width_px, self.viewport_height_px);
            self.vector_pipeline.render(
                &mut rpass,
                None,
                self.viewport_width_px,
                self.viewport_height_px,
            );
        }

        if let Some(bg) = atlas_bind_group {
            self.text_pipeline.render(
                &mut rpass,
                bg,
                self.viewport_width_px,
                self.viewport_height_px,
            );
        }

        scissor_changes
    }

    #[must_use]
    pub fn pool_cpu_capacity(&self) -> usize {
        self.buffer_pool.cpu_capacity()
    }

    #[must_use]
    pub fn pool_gpu_capacity(&self) -> usize {
        self.buffer_pool.gpu_capacity()
    }

    #[must_use]
    pub const fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    #[must_use]
    pub const fn vector_pipeline(&self) -> &VectorPipeline {
        &self.vector_pipeline
    }

    pub fn vector_pipeline_mut(&mut self) -> &mut VectorPipeline {
        &mut self.vector_pipeline
    }

    #[must_use]
    pub const fn image_pipeline(&self) -> &ImagePipeline {
        &self.image_pipeline
    }

    pub fn image_pipeline_mut(&mut self) -> &mut ImagePipeline {
        &mut self.image_pipeline
    }
}
