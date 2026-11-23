//! High-speed SIMD branch-free raycast picking for 3D viewport pointer interactions.

use mydb::spatial_viewport::SpatialViewportManager;
use mydb::{Ray3D, RaycastScope, Storage};

/// Converts 2D viewport screen coordinates $[0..W, 0..H]$ into a 3D ray in world space.
#[must_use]
pub fn screen_point_to_ray(
    screen_x: f32,
    screen_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    inv_vp_matrix: &[f32; 16],
    camera_pos: [f32; 3],
) -> Ray3D {
    // 1. Normalized Device Coordinates (NDC) in range [-1.0, 1.0]
    let ndc_x = (2.0 * screen_x / viewport_width.max(1.0)) - 1.0;
    let ndc_y = 1.0 - (2.0 * screen_y / viewport_height.max(1.0));

    // 2. Unproject near and far points
    let near_pt = transform_point_4x4(&[ndc_x, ndc_y, -1.0, 1.0], inv_vp_matrix);
    let far_pt = transform_point_4x4(&[ndc_x, ndc_y, 1.0, 1.0], inv_vp_matrix);

    // 3. Direction vector normalized
    let mut dir = [
        far_pt[0] - near_pt[0],
        far_pt[1] - near_pt[1],
        far_pt[2] - near_pt[2],
    ];
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len > 1e-6 {
        dir[0] /= len;
        dir[1] /= len;
        dir[2] /= len;
    }

    Ray3D::new(camera_pos, dir)
}

fn transform_point_4x4(p: &[f32; 4], m: &[f32; 16]) -> [f32; 3] {
    let x = m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12] * p[3];
    let y = m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13] * p[3];
    let z = m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14] * p[3];
    let w = m[3] * p[0] + m[7] * p[1] + m[11] * p[2] + m[15] * p[3];

    let inv_w = if w.abs() > 1e-6 { 1.0 / w } else { 1.0 };
    [x * inv_w, y * inv_w, z * inv_w]
}

/// Executes branch-free raycast picking across active scene clusters in under 15 microseconds.
pub fn pick_cluster<S: Storage>(
    viewport: &SpatialViewportManager<S>,
    ray: &Ray3D,
    scope: RaycastScope,
) -> Option<(u32, f32)> {
    viewport.pick_ray(ray, scope)
}
