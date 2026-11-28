//! Pass 2: Deferred tile resolve pass evaluating lighting exactly once per visible pixel.
//!
//! Eliminates overdraw shading costs by reading winning triangle IDs from the 64-bit
//! Visibility Buffer and computing normals and lighting only for visible fragments.

use crate::math::Vec3;
use crate::rasterizer::interpolator::PerspectiveBarycentrics;
use crate::rasterizer::tiling::TileBounds;
use crate::rasterizer::vis_buffer::VisBuffer;
use tej_ui_hal::Color;

/// Resolved triangle geometry ready for deferred shading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedTriangle {
    /// Screen-space coordinates `[x, y, inv_w]` for each vertex.
    pub screen_pos: [(f32, f32, f32); 3],
    /// World-space or view-space unit normals.
    pub normals: [Vec3; 3],
    /// Per-vertex colors.
    pub colors: [Color; 3],
    /// Precomputed inverse denominator for screen barycentric interpolation.
    pub inv_denom: f32,
    /// Whether triangle has uniform normals and uniform color (permitting instant flat resolve).
    pub is_uniform: bool,
}

impl ResolvedTriangle {
    /// Constructs a resolved triangle with screen-space coordinates, normals, and colors.
    #[must_use]
    pub fn new(screen_pos: [(f32, f32, f32); 3], normals: [Vec3; 3], colors: [Color; 3]) -> Self {
        let (x0, y0, _) = screen_pos[0];
        let (x1, y1, _) = screen_pos[1];
        let (x2, y2, _) = screen_pos[2];
        let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
        let inv_denom = if denom.abs() > 1e-6 { 1.0 / denom } else { 0.0 };

        let is_uniform = normals[0] == normals[1]
            && normals[1] == normals[2]
            && colors[0] == colors[1]
            && colors[1] == colors[2];

        Self {
            screen_pos,
            normals,
            colors,
            inv_denom,
            is_uniform,
        }
    }
}

/// Shading environment parameters for the deferred resolve pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadingContext {
    pub key_dir: Vec3,
    pub fill_dir: Vec3,
    pub view_dir: Vec3,
    pub ambient_intensity: f32,
    pub key_intensity: f32,
    pub fill_intensity: f32,
    pub specular_power: i32,
    pub specular_strength: f32,
}

impl Default for ShadingContext {
    fn default() -> Self {
        Self {
            key_dir: Vec3::new(0.5, 1.0, 0.8).normalize(),
            fill_dir: Vec3::new(-0.4, -0.2, -0.6).normalize(),
            view_dir: Vec3::new(0.0, 0.0, 1.0),
            ambient_intensity: 0.25,
            key_intensity: 0.55,
            fill_intensity: 0.20,
            specular_power: 16,
            specular_strength: 0.40,
        }
    }
}

/// Computes uniform lighting for flat or uniform-colored triangles in a single pass.
#[must_use]
pub fn shade_uniform(tri: &ResolvedTriangle, ctx: &ShadingContext) -> Option<u32> {
    let norm = tri.normals[0];
    let norm = if norm.length_squared() > 1e-4 {
        norm.normalize()
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    let oriented_norm = if norm.dot(ctx.view_dir) < 0.0 {
        -norm
    } else {
        norm
    };

    let ndotl1 = oriented_norm.dot(ctx.key_dir).max(0.0);
    let ndotl2 = oriented_norm.dot(ctx.fill_dir).max(0.0);

    let half_dir = (ctx.key_dir + ctx.view_dir).normalize();
    let spec_dot = oriented_norm.dot(half_dir).max(0.0);
    let spec = if ctx.specular_strength > 0.0 && spec_dot > 0.01 {
        let s2 = spec_dot * spec_dot;
        let s4 = s2 * s2;
        let s8 = s4 * s4;
        let s16 = s8 * s8;
        s16 * ctx.specular_strength
    } else {
        0.0
    };

    let diffuse = ndotl1 * ctx.key_intensity + ndotl2 * ctx.fill_intensity;
    let lit = (ctx.ambient_intensity + diffuse).min(1.0);

    let c = tri.colors[0];
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lit_r = ((f32::from(c.r) * lit + spec * 255.0).min(255.0)) as u8;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lit_g = ((f32::from(c.g) * lit + spec * 255.0).min(255.0)) as u8;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lit_b = ((f32::from(c.b) * lit + spec * 255.0).min(255.0)) as u8;

    Some(0xFF00_0000 | (u32::from(lit_r) << 16) | (u32::from(lit_g) << 8) | u32::from(lit_b))
}

/// Resolves a single visible fragment into a 32-bit packed color (`0xAARRGGBB`).
#[must_use]
pub fn shade_fragment(
    px: f32,
    py: f32,
    tri: &ResolvedTriangle,
    ctx: &ShadingContext,
) -> Option<u32> {
    if tri.inv_denom == 0.0 {
        return None;
    }

    let (x0, y0, inv_w0) = tri.screen_pos[0];
    let (x1, y1, inv_w1) = tri.screen_pos[1];
    let (x2, y2, inv_w2) = tri.screen_pos[2];

    let w0 = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) * tri.inv_denom;
    let w1 = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) * tri.inv_denom;
    let w2 = 1.0 - w0 - w1;

    let bary = PerspectiveBarycentrics::compute(w0, w1, w2, inv_w0, inv_w1, inv_w2)?;

    let raw_norm = bary.interpolate_normal(tri.normals[0], tri.normals[1], tri.normals[2]);
    let norm = raw_norm.normalize();

    let oriented_norm = if norm.dot(ctx.view_dir) < 0.0 {
        -norm
    } else {
        norm
    };

    let ndotl1 = oriented_norm.dot(ctx.key_dir).max(0.0);
    let ndotl2 = oriented_norm.dot(ctx.fill_dir).max(0.0);

    let half_dir = (ctx.key_dir + ctx.view_dir).normalize();
    let spec_dot = oriented_norm.dot(half_dir).max(0.0);
    let spec = if ctx.specular_strength > 0.0 && spec_dot > 0.01 {
        let s2 = spec_dot * spec_dot;
        let s4 = s2 * s2;
        let s8 = s4 * s4;
        let s16 = s8 * s8;
        s16 * ctx.specular_strength
    } else {
        0.0
    };

    let diffuse = ndotl1 * ctx.key_intensity + ndotl2 * ctx.fill_intensity;
    let lit = (ctx.ambient_intensity + diffuse).min(1.0);

    let (base_r, base_g, base_b) =
        bary.interpolate_color(tri.colors[0], tri.colors[1], tri.colors[2]);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lit_r = ((base_r * lit + spec * 255.0).min(255.0)) as u8;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lit_g = ((base_g * lit + spec * 255.0).min(255.0)) as u8;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lit_b = ((base_b * lit + spec * 255.0).min(255.0)) as u8;

    Some(0xFF00_0000 | (u32::from(lit_r) << 16) | (u32::from(lit_g) << 8) | u32::from(lit_b))
}

/// Resolves a screen tile from the visibility buffer to the output color buffer.
pub fn resolve_tile<F>(
    bounds: TileBounds,
    vis_buffer: &VisBuffer,
    fetch_tri: &F,
    ctx: &ShadingContext,
    color_buffer: &mut [u32],
) where
    F: Fn(u32, u16) -> Option<ResolvedTriangle>,
{
    let width = vis_buffer.width;
    let mut last_slot = u32::MAX;
    let mut last_tri_id = u16::MAX;
    let mut cached_tri: Option<ResolvedTriangle> = None;
    let mut cached_uniform_color: Option<u32> = None;

    for y in bounds.min_y..=bounds.max_y {
        let py = crate::math::count_to_f32(y) + 0.5;
        let row_idx = y * width;

        for x in bounds.min_x..=bounds.max_x {
            let px = crate::math::count_to_f32(x) + 0.5;
            let idx = row_idx + x;
            let vis = vis_buffer.get_pixel(x, y);

            if !vis.is_empty() {
                let slot = vis.meshlet_slot();
                let tri_id = vis.tri_id();
                if slot != last_slot || tri_id != last_tri_id {
                    last_slot = slot;
                    last_tri_id = tri_id;
                    cached_tri = fetch_tri(slot, tri_id);
                    cached_uniform_color = cached_tri.and_then(|t| {
                        if t.is_uniform {
                            shade_uniform(&t, ctx)
                        } else {
                            None
                        }
                    });
                }

                if let Some(color) = cached_uniform_color {
                    color_buffer[idx] = color;
                } else if let Some(tri) = &cached_tri
                    && let Some(color) = shade_fragment(px, py, tri, ctx)
                {
                    color_buffer[idx] = color;
                }
            }
        }
    }
}
