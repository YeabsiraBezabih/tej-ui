//! 3D Hardware texture bridge and software rasterizer with depth buffering.

use crate::camera::PerspectiveCamera;
use crate::material::LightKind;
use crate::math::Vec3;
use crate::scene::Scene;
use tej_hal::Color;
use tej_layout::Size;

/// Native GPU texture descriptor for zero-copy hardware swapchain composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuTextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: GpuTextureFormat,
}

/// Hardware texture pixel formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuTextureFormat {
    Rgba8Unorm,
    Bgra8Unorm,
}

/// Abstract opaque handle to a hardware GPU texture (e.g. `ID3D11Texture2D`, `MTLTexture`, `wgpu::Texture`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureHandle(pub u64);

/// High-performance 3D software rasterizer with $Z$-buffer for offline testing and headless fallback.
#[derive(Debug)]
pub struct Rasterizer3D {
    pub width: usize,
    pub height: usize,
    pub color_buffer: Vec<u32>, // 0xAARRGGBB
    pub depth_buffer: Vec<f32>, // Z-buffer [0.0 near .. 1.0 far]
}

impl Rasterizer3D {
    /// Creates a rasterizer with specified viewport dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            color_buffer: vec![0xFF1E1E24; size], // Background dark gray
            depth_buffer: vec![1.0; size],
        }
    }

    /// Clears color and depth buffers.
    pub fn clear(&mut self, clear_color: Color) {
        let packed = ((clear_color.a as u32) << 24)
            | ((clear_color.r as u32) << 16)
            | ((clear_color.g as u32) << 8)
            | (clear_color.b as u32);
        self.color_buffer.fill(packed);
        self.depth_buffer.fill(1.0);
    }

    /// Renders a complete 3D scene from the perspective camera.
    pub fn render_scene(&mut self, scene: &Scene, camera: &PerspectiveCamera) {
        let vp = camera.view_projection_matrix();

        let w_f32 = self.width as f32;
        let h_f32 = self.height as f32;

        for obj in &scene.objects {
            let model = obj.transform.matrix();
            let mvp = vp.mul_mat(&model);

            // Simple directional light calculation
            let mut sun_dir = Vec3::new(0.5, 1.0, 0.8).normalize();
            for light in &scene.lights {
                if let LightKind::Directional { direction } = light.kind {
                    sun_dir = direction;
                }
            }

            let indices = &obj.mesh.indices;
            let vertices = &obj.mesh.vertices;

            for tri in indices.chunks(3) {
                if tri.len() < 3 {
                    continue;
                }

                let v0 = vertices[tri[0] as usize];
                let v1 = vertices[tri[1] as usize];
                let v2 = vertices[tri[2] as usize];

                // Transform to clip space
                let p0 = mvp.transform_point(v0.position);
                let p1 = mvp.transform_point(v1.position);
                let p2 = mvp.transform_point(v2.position);

                // Simple backface culling in screen space
                let screen_x0 = (p0.x + 1.0) * 0.5 * w_f32;
                let screen_y0 = (1.0 - p0.y) * 0.5 * h_f32;

                let screen_x1 = (p1.x + 1.0) * 0.5 * w_f32;
                let screen_y1 = (1.0 - p1.y) * 0.5 * h_f32;

                let screen_x2 = (p2.x + 1.0) * 0.5 * w_f32;
                let screen_y2 = (1.0 - p2.y) * 0.5 * h_f32;

                let edge = (screen_x1 - screen_x0) * (screen_y2 - screen_y0)
                    - (screen_y1 - screen_y0) * (screen_x2 - screen_x0);
                if edge <= 0.0 {
                    continue; // Culled backface
                }

                // Compute flat shading with directional light
                let norm = v0.normal.normalize();
                let ndotl = norm.dot(sun_dir).max(0.2); // ambient base 0.2
                let base_c = obj.material.color;
                let lit_r = ((base_c.r as f32) * ndotl).min(255.0) as u8;
                let lit_g = ((base_c.g as f32) * ndotl).min(255.0) as u8;
                let lit_b = ((base_c.b as f32) * ndotl).min(255.0) as u8;
                let packed_color =
                    (0xFF << 24) | ((lit_r as u32) << 16) | ((lit_g as u32) << 8) | (lit_b as u32);

                // Rasterize bounding box
                let min_x = (screen_x0.min(screen_x1).min(screen_x2).max(0.0) as usize)
                    .min(self.width.saturating_sub(1));
                let max_x = (screen_x0.max(screen_x1).max(screen_x2).max(0.0) as usize)
                    .min(self.width.saturating_sub(1));
                let min_y = (screen_y0.min(screen_y1).min(screen_y2).max(0.0) as usize)
                    .min(self.height.saturating_sub(1));
                let max_y = (screen_y0.max(screen_y1).max(screen_y2).max(0.0) as usize)
                    .min(self.height.saturating_sub(1));

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let idx = y * self.width + x;
                        let avg_depth = (p0.z + p1.z + p2.z) / 3.0;
                        if avg_depth < self.depth_buffer[idx] {
                            self.depth_buffer[idx] = avg_depth;
                            self.color_buffer[idx] = packed_color;
                        }
                    }
                }
            }
        }
    }

    /// Size of the rasterized surface.
    #[must_use]
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }
}
