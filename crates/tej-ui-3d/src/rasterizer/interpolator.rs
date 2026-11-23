//! Perspective-correct 1/W attribute interpolation.
//!
//! Converts 2D affine screen-space barycentric weights into perspective-correct
//! surface barycentrics, eliminating texture warping and shading distortion at steep angles.

use crate::math::Vec3;
use tej_ui_hal::Color;

/// Perspective-correct barycentric interpolation weights at a screen pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveBarycentrics {
    pub k0: f32,
    pub k1: f32,
    pub k2: f32,
    pub w_pixel: f32,
}

impl PerspectiveBarycentrics {
    /// Computes perspective-correct barycentric weights from screen-space affine weights
    /// and vertex reciprocal clip W coordinates (`inv_w = 1.0 / clip_w`).
    #[inline]
    #[must_use]
    pub fn compute(
        w0: f32,
        w1: f32,
        w2: f32,
        inv_w0: f32,
        inv_w1: f32,
        inv_w2: f32,
    ) -> Option<Self> {
        let interp_inv_w = w0 * inv_w0 + w1 * inv_w1 + w2 * inv_w2;
        if interp_inv_w <= 1e-9 {
            return None;
        }

        let w_pixel = 1.0 / interp_inv_w;
        let k0 = w0 * inv_w0 * w_pixel;
        let k1 = w1 * inv_w1 * w_pixel;
        let k2 = w2 * inv_w2 * w_pixel;

        Some(Self {
            k0,
            k1,
            k2,
            w_pixel,
        })
    }

    /// Interpolates a 3D vector (e.g. normal or world position) and normalizes it.
    #[inline]
    #[must_use]
    pub fn interpolate_normal(&self, n0: Vec3, n1: Vec3, n2: Vec3) -> Vec3 {
        let norm_raw = n0 * self.k0 + n1 * self.k1 + n2 * self.k2;
        if norm_raw.length_squared() > 1e-6 {
            norm_raw.normalize()
        } else {
            n0
        }
    }

    /// Interpolates vertex color attributes with perspective correction.
    #[inline]
    #[must_use]
    pub fn interpolate_color(&self, c0: Color, c1: Color, c2: Color) -> (f32, f32, f32) {
        let r = (c0.r as f32) * self.k0 + (c1.r as f32) * self.k1 + (c2.r as f32) * self.k2;
        let g = (c0.g as f32) * self.k0 + (c1.g as f32) * self.k1 + (c2.g as f32) * self.k2;
        let b = (c0.b as f32) * self.k0 + (c1.b as f32) * self.k1 + (c2.b as f32) * self.k2;
        (r, g, b)
    }

    /// Interpolates a generic scalar value.
    #[inline]
    #[must_use]
    pub fn interpolate_f32(&self, v0: f32, v1: f32, v2: f32) -> f32 {
        v0 * self.k0 + v1 * self.k1 + v2 * self.k2
    }
}
