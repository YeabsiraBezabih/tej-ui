//! 3D Hardware texture bridge and software rasterizer with depth buffering.

use crate::camera::PerspectiveCamera;
use crate::material::LightKind;
use crate::math::Vec3;
use crate::rasterizer::{
    ClipVertex, ResolvedTriangle, ShadingContext, TileGrid, VisBuffer, clip_triangle_near_plane,
    resolve_tile,
};
use crate::scene::Scene;
use tej_ui_hal::Color;
use tej_ui_layout::Size;

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

/// High-performance 3D software rasterizer with 64-bit Visibility Buffer and deferred tile resolve.
#[derive(Debug)]
pub struct Rasterizer3D {
    pub width: usize,
    pub height: usize,
    pub color_buffer: Vec<u32>, // 0xAARRGGBB
    pub depth_buffer: Vec<f32>, // Reversed-Z [1.0 near .. 0.0 far]
    pub vis_buffer: VisBuffer,  // 64-bit Visibility Buffer
    scene_triangles: Vec<ResolvedTriangle>,
    dirty_min_x: usize,
    dirty_min_y: usize,
    dirty_max_x: usize,
    dirty_max_y: usize,
    has_dirty_rect: bool,
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
            depth_buffer: vec![0.0; size],        // Reversed-Z: 0.0 is farthest (cleared)
            vis_buffer: VisBuffer::new(width, height),
            scene_triangles: Vec::with_capacity(1024),
            dirty_min_x: 0,
            dirty_min_y: 0,
            dirty_max_x: 0,
            dirty_max_y: 0,
            has_dirty_rect: false,
        }
    }

    /// Resizes rasterizer buffers if viewport dimensions changed.
    pub fn resize(&mut self, width: usize, height: usize) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            let size = width * height;
            self.color_buffer.resize(size, 0xFF1E1E24);
            self.depth_buffer.resize(size, 0.0);
            self.vis_buffer.resize(width, height);
            self.has_dirty_rect = false;
        }
    }

    /// Clears color and depth buffers (Reversed-Z: depth cleared to 0.0).
    pub fn clear(&mut self, clear_color: Color) {
        let packed = ((clear_color.a as u32) << 24)
            | ((clear_color.r as u32) << 16)
            | ((clear_color.g as u32) << 8)
            | (clear_color.b as u32);

        if self.has_dirty_rect {
            let min_x = self.dirty_min_x;
            let max_x = self.dirty_max_x.min(self.width.saturating_sub(1));
            let min_y = self.dirty_min_y;
            let max_y = self.dirty_max_y.min(self.height.saturating_sub(1));

            for y in min_y..=max_y {
                let row_idx = y * self.width;
                for x in min_x..=max_x {
                    let idx = row_idx + x;
                    self.color_buffer[idx] = packed;
                    self.depth_buffer[idx] = 0.0;
                    self.vis_buffer.clear_pixel(x, y);
                }
            }
            self.has_dirty_rect = false;
        } else {
            self.color_buffer.fill(packed);
            self.depth_buffer.fill(0.0);
            self.vis_buffer.clear();
        }
        self.scene_triangles.clear();
    }

    /// Renders a complete 3D scene using a two-pass 64-bit Visibility Buffer architecture.
    ///
    /// - **Pass 1 (Geometry ID Rasterization)**: Rasterizes triangles into the 64-bit Visibility Buffer
    ///   with zero shading, lighting, or normal math.
    /// - **Pass 2 (Deferred Tile Resolve)**: Iterates over $16 \times 16$ screen tiles and computes
    ///   lighting and normal interpolation **exactly once per visible pixel** (zero overdraw cost).
    pub fn render_scene(&mut self, scene: &Scene, camera: &PerspectiveCamera) {
        self.scene_triangles.clear();
        let vp = camera.view_projection_matrix();

        let w_f32 = self.width as f32;
        let h_f32 = self.height as f32;

        let mut sun_dir = Vec3::new(0.5, 1.0, 0.8).normalize();
        for light in &scene.lights {
            if let LightKind::Directional { direction } = light.kind {
                sun_dir = direction;
            }
        }
        let key_dir = sun_dir;
        let fill_dir = Vec3::new(-0.4, -0.2, -0.6).normalize();

        let mut min_dirty_x = self.width;
        let mut min_dirty_y = self.height;
        let mut max_dirty_x = 0;
        let mut max_dirty_y = 0;

        let mut object_indices: Vec<usize> = (0..scene.objects.len()).collect();
        if scene.objects.len() > 1 {
            object_indices.sort_by(|&a, &b| {
                let dist_a =
                    (scene.objects[a].transform.position - camera.position).length_squared();
                let dist_b =
                    (scene.objects[b].transform.position - camera.position).length_squared();
                dist_a.total_cmp(&dist_b)
            });
        }

        // -----------------------------------------------------------------
        // PASS 1: Geometry ID Rasterization into 64-Bit Visibility Buffer
        // -----------------------------------------------------------------
        for &obj_idx in &object_indices {
            let obj = &scene.objects[obj_idx];
            let model = obj.transform.matrix();
            let mvp = vp.mul_mat(&model);

            // Coarse Early-Z rejection for occluded objects in front-to-back scenes
            let center_clip = mvp.transform_point4(Vec3::ZERO);
            if center_clip[3] > 0.01 {
                let inv_w = 1.0 / center_clip[3];
                let center_depth = center_clip[2] * inv_w;
                let cx = ((center_clip[0] * inv_w + 1.0) * 0.5 * w_f32) as usize;
                let cy = ((1.0 - center_clip[1] * inv_w) * 0.5 * h_f32) as usize;
                if cx < self.width && cy < self.height {
                    let existing_depth = self.depth_buffer[cy * self.width + cx];
                    if existing_depth > 0.0 && center_depth < existing_depth {
                        continue;
                    }
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

                let face_n = (v1.position - v0.position).cross(v2.position - v0.position);
                let face_n = if face_n.length_squared() > 1e-6 {
                    face_n.normalize()
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };

                let n0 = if v0.normal.length_squared() > 1e-4 {
                    v0.normal
                } else {
                    face_n
                };
                let n1 = if v1.normal.length_squared() > 1e-4 {
                    v1.normal
                } else {
                    face_n
                };
                let n2 = if v2.normal.length_squared() > 1e-4 {
                    v2.normal
                } else {
                    face_n
                };

                let c0 = ClipVertex {
                    clip_pos: mvp.transform_point4(v0.position),
                    normal: n0,
                    color: v0.color,
                };
                let c1 = ClipVertex {
                    clip_pos: mvp.transform_point4(v1.position),
                    normal: n1,
                    color: v1.color,
                };
                let c2 = ClipVertex {
                    clip_pos: mvp.transform_point4(v2.position),
                    normal: n2,
                    color: v2.color,
                };

                // Sutherland-Hodgman near-plane clipping
                let clipped = clip_triangle_near_plane(c0, c1, c2, camera.near.max(0.01));
                for sub_tri in &clipped.triangles[..clipped.count] {
                    let sv0 = sub_tri[0];
                    let sv1 = sub_tri[1];
                    let sv2 = sub_tri[2];

                    let inv_w0 = 1.0 / sv0.clip_pos[3];
                    let inv_w1 = 1.0 / sv1.clip_pos[3];
                    let inv_w2 = 1.0 / sv2.clip_pos[3];

                    let screen_x0 = (sv0.clip_pos[0] * inv_w0 + 1.0) * 0.5 * w_f32;
                    let screen_y0 = (1.0 - sv0.clip_pos[1] * inv_w0) * 0.5 * h_f32;
                    let sz0 = sv0.clip_pos[2] * inv_w0;

                    let screen_x1 = (sv1.clip_pos[0] * inv_w1 + 1.0) * 0.5 * w_f32;
                    let screen_y1 = (1.0 - sv1.clip_pos[1] * inv_w1) * 0.5 * h_f32;
                    let sz1 = sv1.clip_pos[2] * inv_w1;

                    let screen_x2 = (sv2.clip_pos[0] * inv_w2 + 1.0) * 0.5 * w_f32;
                    let screen_y2 = (1.0 - sv2.clip_pos[1] * inv_w2) * 0.5 * h_f32;
                    let sz2 = sv2.clip_pos[2] * inv_w2;

                    let denom = (screen_y1 - screen_y2) * (screen_x0 - screen_x2)
                        + (screen_x2 - screen_x1) * (screen_y0 - screen_y2);
                    if denom.abs() < 1e-4 {
                        continue;
                    }
                    let inv_denom = 1.0 / denom;

                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let min_x = (screen_x0.min(screen_x1).min(screen_x2).floor() as i32)
                        .max(0)
                        .min((self.width as i32) - 1) as usize;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let max_x = (screen_x0.max(screen_x1).max(screen_x2).ceil() as i32)
                        .max(0)
                        .min((self.width as i32) - 1) as usize;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let min_y = (screen_y0.min(screen_y1).min(screen_y2).floor() as i32)
                        .max(0)
                        .min((self.height as i32) - 1) as usize;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let max_y = (screen_y0.max(screen_y1).max(screen_y2).ceil() as i32)
                        .max(0)
                        .min((self.height as i32) - 1) as usize;

                    min_dirty_x = min_dirty_x.min(min_x);
                    min_dirty_y = min_dirty_y.min(min_y);
                    max_dirty_x = max_dirty_x.max(max_x);
                    max_dirty_y = max_dirty_y.max(max_y);

                    let tri_idx = self.scene_triangles.len();
                    let norm0 = model.transform_vector(sv0.normal);
                    let norm1 = model.transform_vector(sv1.normal);
                    let norm2 = model.transform_vector(sv2.normal);

                    self.scene_triangles.push(ResolvedTriangle::new(
                        [
                            (screen_x0, screen_y0, inv_w0),
                            (screen_x1, screen_y1, inv_w1),
                            (screen_x2, screen_y2, inv_w2),
                        ],
                        [norm0, norm1, norm2],
                        [sv0.color, sv1.color, sv2.color],
                    ));

                    #[allow(clippy::cast_possible_truncation)]
                    let slot = (tri_idx >> 12) as u32;
                    #[allow(clippy::cast_possible_truncation)]
                    let local_tri = (tri_idx & 0x0FFF) as u16;

                    let step_x0 = (screen_y1 - screen_y2) * inv_denom;
                    let step_y0 = (screen_x2 - screen_x1) * inv_denom;
                    let step_x1 = (screen_y2 - screen_y0) * inv_denom;
                    let step_y1 = (screen_x0 - screen_x2) * inv_denom;

                    let base_px = min_x as f32 + 0.5;
                    let base_py = min_y as f32 + 0.5;

                    let mut row_w0 = ((screen_y1 - screen_y2) * (base_px - screen_x2)
                        + (screen_x2 - screen_x1) * (base_py - screen_y2))
                        * inv_denom;
                    let mut row_w1 = ((screen_y2 - screen_y0) * (base_px - screen_x2)
                        + (screen_x0 - screen_x2) * (base_py - screen_y2))
                        * inv_denom;

                    // Rasterize ID only: zero lighting or normal math with incremental step additions
                    for y in min_y..=max_y {
                        let row_idx = y * self.width;
                        let mut w0 = row_w0;
                        let mut w1 = row_w1;

                        for x in min_x..=max_x {
                            let w2 = 1.0 - w0 - w1;

                            if w0 >= -0.001 && w1 >= -0.001 && w2 >= -0.001 {
                                let idx = row_idx + x;
                                let depth = w0 * sz0 + w1 * sz1 + w2 * sz2;

                                if depth > self.depth_buffer[idx] {
                                    self.depth_buffer[idx] = depth;
                                    self.vis_buffer
                                        .write_pixel_unchecked(idx, depth, slot, local_tri);
                                }
                            }

                            w0 += step_x0;
                            w1 += step_x1;
                        }

                        row_w0 += step_y0;
                        row_w1 += step_y1;
                    }
                }
            }
        }

        // -----------------------------------------------------------------
        // PASS 2: Deferred Tile Resolve (Shade visible pixels once)
        // -----------------------------------------------------------------
        let view_dir = (camera.position - camera.target).normalize();
        let ctx = ShadingContext {
            key_dir,
            fill_dir,
            view_dir,
            ..ShadingContext::default()
        };

        let grid = TileGrid::new(self.width, self.height);
        if min_dirty_x <= max_dirty_x && min_dirty_y <= max_dirty_y {
            let start_tx = min_dirty_x / 16;
            let end_tx = (max_dirty_x / 16).min(grid.tiles_x.saturating_sub(1));
            let start_ty = min_dirty_y / 16;
            let end_ty = (max_dirty_y / 16).min(grid.tiles_y.saturating_sub(1));

            for ty in start_ty..=end_ty {
                for tx in start_tx..=end_tx {
                    if let Some(bounds) = grid.tile_bounds(tx, ty) {
                        resolve_tile(
                            bounds,
                            &self.vis_buffer,
                            &|slot, tri_id| {
                                let idx = ((slot as usize) << 12) | (tri_id as usize);
                                self.scene_triangles.get(idx).copied()
                            },
                            &ctx,
                            &mut self.color_buffer,
                        );
                    }
                }
            }
            self.dirty_min_x = min_dirty_x;
            self.dirty_min_y = min_dirty_y;
            self.dirty_max_x = max_dirty_x;
            self.dirty_max_y = max_dirty_y;
            self.has_dirty_rect = true;
        }
    }

    /// Size of the rasterized surface.
    #[must_use]
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }
}
