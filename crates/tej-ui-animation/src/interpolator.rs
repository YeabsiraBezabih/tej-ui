//! Linear interpolation traits for animating numeric types, points, rectangles, and colors.

use tej_ui_hal::Color;
use tej_ui_layout::{Point, Rect, Size};

/// Trait for types supporting linear interpolation between values over progress factor `t` in `[0.0, 1.0]`.
pub trait Lerp {
    /// Compute linearly interpolated value at factor `t`.
    fn lerp(&self, target: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(&self, target: &Self, t: f32) -> Self {
        self + (target - self) * t
    }
}

impl Lerp for Point {
    #[inline]
    fn lerp(&self, target: &Self, t: f32) -> Self {
        Self::new(self.x.lerp(&target.x, t), self.y.lerp(&target.y, t))
    }
}

impl Lerp for Size {
    #[inline]
    fn lerp(&self, target: &Self, t: f32) -> Self {
        Self::new(
            self.width.lerp(&target.width, t),
            self.height.lerp(&target.height, t),
        )
    }
}

impl Lerp for Rect {
    #[inline]
    fn lerp(&self, target: &Self, t: f32) -> Self {
        Self::new(
            self.origin.x.lerp(&target.origin.x, t),
            self.origin.y.lerp(&target.origin.y, t),
            self.size.width.lerp(&target.size.width, t),
            self.size.height.lerp(&target.size.height, t),
        )
    }
}

impl Lerp for Color {
    #[inline]
    fn lerp(&self, target: &Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self {
            r: (f32::from(self.r) + (f32::from(target.r) - f32::from(self.r)) * t_clamped) as u8,
            g: (f32::from(self.g) + (f32::from(target.g) - f32::from(self.g)) * t_clamped) as u8,
            b: (f32::from(self.b) + (f32::from(target.b) - f32::from(self.b)) * t_clamped) as u8,
            a: (f32::from(self.a) + (f32::from(target.a) - f32::from(self.a)) * t_clamped) as u8,
        }
    }
}
