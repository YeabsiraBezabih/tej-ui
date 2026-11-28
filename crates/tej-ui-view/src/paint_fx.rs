//! High-performance analytic rendering and shader primitives.
//! Implements Bayer 8x8 ordered dithering, signed distance field (SDF) rounded rectangles,
//! and analytic Gaussian drop shadows via the polynomial error function `erf()`.

use tej_ui_layout::Point;

/// Computes the 8x8 Bayer ordered dithering threshold for pixel coordinates `(x, y)`.
/// Returns a normalized threshold in `[0.0, 1.0]`.
#[inline]
#[must_use]
pub fn bayer8(x: u32, y: u32) -> f32 {
    let qx = x % 8;
    let qy = y % 8;
    let a = qx ^ qy;
    let m = ((a & 1) << 5)
        | ((qy & 1) << 4)
        | ((a & 2) << 2)
        | ((qy & 2) << 1)
        | ((a & 4) >> 1)
        | ((qy & 4) >> 2);
    (f32::from(u8::try_from(m).unwrap_or(63)) + 0.5) / 64.0
}

/// Evaluates whether a pixel at `(x, y)` is active for a target opacity / density `[0.0, 1.0]`.
#[inline]
#[must_use]
pub fn dither_test(x: u32, y: u32, density: f32) -> bool {
    density > bayer8(x, y)
}

/// Evaluates a high-precision polynomial approximation of the Gaussian error function `erf(x)`.
/// Maximum error is less than 1.5e-7.
#[inline]
#[must_use]
pub fn erf_approx(x: f32) -> f32 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let abs_x = x.abs();

    // Abramowitz and Stegun formula 7.1.26
    let t = 1.0 / (1.0 + 0.327_591_1 * abs_x);
    let poly = (((((1.061_405_4 * t - 1.453_152) * t) + 1.421_413_8) * t - 0.284_496_74) * t
        + 0.254_829_6)
        * t;
    sign * (1.0 - poly * (-abs_x * abs_x).exp())
}

/// Evaluates the signed distance from point `p` (relative to box center) to a rounded rectangle
/// with half-extents `half_size` and corner radii `(top_left, top_right, bottom_right, bottom_left)`.
#[must_use]
pub fn sd_rounded_rect(p: Point, half_size: Point, radii: (f32, f32, f32, f32)) -> f32 {
    let (r_tl, r_tr, r_br, r_bl) = radii;
    let r = if p.x < 0.0 {
        if p.y < 0.0 { r_tl } else { r_bl }
    } else if p.y < 0.0 {
        r_tr
    } else {
        r_br
    };
    let clamped_r = r.min(half_size.x).min(half_size.y);
    let qx = p.x.abs() - half_size.x + clamped_r;
    let qy = p.y.abs() - half_size.y + clamped_r;
    let outside_dist = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let inside_dist = qx.max(qy).min(0.0);
    outside_dist + inside_dist - clamped_r
}

/// Evaluates the analytic Gaussian drop shadow alpha coverage at signed distance `d` with blur `sigma`.
#[inline]
#[must_use]
pub fn gaussian_shadow_alpha(distance: f32, sigma: f32) -> f32 {
    let s = sigma.max(0.01);
    let norm = s * std::f32::consts::SQRT_2;
    0.5 * (1.0 - erf_approx(distance / norm))
}
