//! Homogeneous near-plane polygon clipping using Sutherland-Hodgman.
//!
//! Prevents geometry from abruptly popping out of existence when moving near or through
//! surfaces during 6-DOF entity walkthroughs.

use crate::math::Vec3;
use tej_ui_hal::Color;

/// A vertex with homogeneous clip-space coordinates and surface attributes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipVertex {
    /// Homogeneous clip coordinates `[x, y, z, w]`.
    pub clip_pos: [f32; 4],
    /// World or model surface normal.
    pub normal: Vec3,
    /// Vertex color.
    pub color: Color,
}

impl ClipVertex {
    /// Linearly interpolates between two clip vertices at parameter `t` in `[0.0, 1.0]`.
    #[must_use]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let inv_t = 1.0 - t;
        let clip_pos = [
            self.clip_pos[0] * inv_t + other.clip_pos[0] * t,
            self.clip_pos[1] * inv_t + other.clip_pos[1] * t,
            self.clip_pos[2] * inv_t + other.clip_pos[2] * t,
            self.clip_pos[3] * inv_t + other.clip_pos[3] * t,
        ];
        let norm_raw = self.normal * inv_t + other.normal * t;
        let normal = if norm_raw.length_squared() > 1e-6 {
            norm_raw.normalize()
        } else {
            self.normal
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let red = (f32::from(self.color.r) * inv_t + f32::from(other.color.r) * t).clamp(0.0, 255.0)
            as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let g = (f32::from(self.color.g) * inv_t + f32::from(other.color.g) * t).clamp(0.0, 255.0)
            as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let b = (f32::from(self.color.b) * inv_t + f32::from(other.color.b) * t).clamp(0.0, 255.0)
            as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let a = (f32::from(self.color.a) * inv_t + f32::from(other.color.a) * t).clamp(0.0, 255.0)
            as u8;

        Self {
            clip_pos,
            normal,
            color: Color::rgba(red, g, b, a),
        }
    }
}

/// Static buffer holding up to 2 sub-triangles resulting from a near-plane clip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippedTriangles {
    pub triangles: [[ClipVertex; 3]; 2],
    pub count: usize,
}

impl ClippedTriangles {
    /// Creates an empty collection of clipped triangles.
    pub const EMPTY: Self = Self {
        triangles: [
            [
                ClipVertex {
                    clip_pos: [0.0; 4],
                    normal: Vec3::ZERO,
                    color: Color::BLACK,
                },
                ClipVertex {
                    clip_pos: [0.0; 4],
                    normal: Vec3::ZERO,
                    color: Color::BLACK,
                },
                ClipVertex {
                    clip_pos: [0.0; 4],
                    normal: Vec3::ZERO,
                    color: Color::BLACK,
                },
            ],
            [
                ClipVertex {
                    clip_pos: [0.0; 4],
                    normal: Vec3::ZERO,
                    color: Color::BLACK,
                },
                ClipVertex {
                    clip_pos: [0.0; 4],
                    normal: Vec3::ZERO,
                    color: Color::BLACK,
                },
                ClipVertex {
                    clip_pos: [0.0; 4],
                    normal: Vec3::ZERO,
                    color: Color::BLACK,
                },
            ],
        ],
        count: 0,
    };
}

/// Clips a triangle against the homogeneous near plane (`w >= near_w`).
///
/// Implements the Sutherland-Hodgman re-entrant polygon clipping algorithm,
/// maintaining exact counter-clockwise / clockwise vertex winding order.
#[must_use]
pub fn clip_triangle_near_plane(
    v0: ClipVertex,
    v1: ClipVertex,
    v2: ClipVertex,
    near_w: f32,
) -> ClippedTriangles {
    let d0 = v0.clip_pos[3] - near_w;
    let d1 = v1.clip_pos[3] - near_w;
    let d2 = v2.clip_pos[3] - near_w;

    // Fast-path: all inside (common case)
    if d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0 {
        let mut result = ClippedTriangles::EMPTY;
        result.triangles[0] = [v0, v1, v2];
        result.count = 1;
        return result;
    }

    // Fast-path: all outside
    if d0 < 0.0 && d1 < 0.0 && d2 < 0.0 {
        return ClippedTriangles::EMPTY;
    }

    let input = [v0, v1, v2];
    let mut output: [ClipVertex; 4] = [v0; 4];
    let mut out_len = 0usize;

    for i in 0..3 {
        let current = input[i];
        let next = input[(i + 1) % 3];

        let d_curr = current.clip_pos[3] - near_w;
        let d_next = next.clip_pos[3] - near_w;

        let curr_inside = d_curr >= 0.0;
        let next_inside = d_next >= 0.0;

        if curr_inside {
            if next_inside {
                // Both inside -> emit next
                if out_len < 4 {
                    output[out_len] = next;
                    out_len += 1;
                }
            } else {
                // Inside to outside -> emit intersection
                let denom = d_curr - d_next;
                let t = if denom.abs() > 1e-6 {
                    (d_curr / denom).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                if out_len < 4 {
                    output[out_len] = current.lerp(&next, t);
                    out_len += 1;
                }
            }
        } else if next_inside {
            // Outside to inside -> emit intersection then next
            let denom = d_curr - d_next;
            let t = if denom.abs() > 1e-6 {
                (d_curr / denom).clamp(0.0, 1.0)
            } else {
                0.5
            };
            if out_len < 4 {
                output[out_len] = current.lerp(&next, t);
                out_len += 1;
            }
            if out_len < 4 {
                output[out_len] = next;
                out_len += 1;
            }
        }
    }

    match out_len.cmp(&3) {
        std::cmp::Ordering::Less => ClippedTriangles::EMPTY,
        std::cmp::Ordering::Equal => {
            let mut result = ClippedTriangles::EMPTY;
            result.triangles[0] = [output[0], output[1], output[2]];
            result.count = 1;
            result
        }
        std::cmp::Ordering::Greater => {
            // Triangulate 4-gon [P0, P1, P2, P3] into 2 triangles
            let mut result = ClippedTriangles::EMPTY;
            result.triangles[0] = [output[0], output[1], output[2]];
            result.triangles[1] = [output[0], output[2], output[3]];
            result.count = 2;
            result
        }
    }
}
