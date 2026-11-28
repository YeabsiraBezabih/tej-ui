//! Single-pass directional shadow mapping with Percentage-Closer Filtering (PCF).
//!
//! Generates orthographic shadow depth maps from the key light direction and evaluates
//! soft-edged shadowing factors for visible surface fragments.

use crate::math::{Mat4, Vec3};

/// Directional light shadow map depth buffer with PCF filtering.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowMap {
    pub width: usize,
    pub height: usize,
    pub depth_buffer: Vec<f32>,
    pub light_view_proj: Mat4,
}

impl ShadowMap {
    /// Creates a shadow map for a directional light covering a spherical scene volume.
    #[must_use]
    pub fn new(
        width: usize,
        height: usize,
        light_dir: Vec3,
        scene_center: Vec3,
        scene_radius: f32,
    ) -> Self {
        let l_norm = light_dir.normalize();
        let eye = scene_center + l_norm * (scene_radius * 1.5);
        let up = if l_norm.y.abs() > 0.95 {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        let view = Mat4::look_at(eye, scene_center, up);
        let r = scene_radius * 1.2;
        let proj = Mat4::orthographic(-r, r, -r, r, 0.0, scene_radius * 3.0);
        let light_view_proj = proj.mul_mat(&view);

        Self {
            width,
            height,
            depth_buffer: vec![1.0; width * height],
            light_view_proj,
        }
    }

    /// Resets all depth buffer entries to 1.0 (far distance).
    pub fn clear(&mut self) {
        self.depth_buffer.fill(1.0);
    }

    /// Records a fragment's light-space depth into the shadow buffer.
    #[inline]
    pub fn record_depth(&mut self, x: usize, y: usize, depth: f32) -> bool {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if depth < self.depth_buffer[idx] {
                self.depth_buffer[idx] = depth;
                return true;
            }
        }
        false
    }

    /// Evaluates 4-tap Percentage-Closer Filtering (PCF) shadow factor for a world position.
    ///
    /// Returns a scalar in `[0.0, 1.0]`:
    /// - `1.0`: Fully illuminated (no shadow).
    /// - `0.0`: Completely occluded in key light shadow.
    #[must_use]
    pub fn sample_shadow_pcf(&self, world_pos: Vec3, bias: f32) -> f32 {
        let clip = self.light_view_proj.transform_point4(world_pos);
        let w = clip[3];
        if w.abs() < 1e-5 {
            return 1.0;
        }

        let inv_w = 1.0 / w;
        let ndc_x = clip[0] * inv_w;
        let ndc_y = clip[1] * inv_w;
        let ndc_z = clip[2] * inv_w;

        // Map NDC [-1, 1]^2 to texture coordinates [0, 1]^2
        let u = f32::midpoint(ndc_x, 1.0);
        let v = (1.0 - ndc_y) * 0.5;
        let current_depth = ndc_z;

        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) || current_depth > 1.0 {
            return 1.0; // Outside shadow map bounds is unshadowed
        }

        let w_f32 = crate::math::count_to_f32(self.width);
        let h_f32 = crate::math::count_to_f32(self.height);
        let px = u * w_f32;
        let py = v * h_f32;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let base_x = (px.floor() as usize).min(self.width.saturating_sub(2));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let base_y = (py.floor() as usize).min(self.height.saturating_sub(2));

        let mut lit_count = 0.0_f32;
        let offsets = [(0, 0), (1, 0), (0, 1), (1, 1)];

        for (ox, oy) in offsets {
            let sx = base_x + ox;
            let sy = base_y + oy;
            let shadow_depth = self.depth_buffer[sy * self.width + sx];

            if current_depth - bias <= shadow_depth {
                lit_count += 1.0;
            }
        }

        lit_count * 0.25
    }
}
