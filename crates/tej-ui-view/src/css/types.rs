//! Fundamental value types and units for CSS properties.

use tej_ui_hal::Color;

/// Length or percentage dimension value.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum LengthPercentage {
    /// Dimension in points/pixels.
    Points(f32),
    /// Dimension in percentage (0.0 to 1.0, e.g. 1.0 = 100%).
    Percent(f32),
    /// Character width unit (e.g. 65ch for readable column widths).
    Ch(f32),
    /// Viewport width unit (0.0 to 1.0).
    Vw(f32),
    /// Viewport height unit (0.0 to 1.0).
    Vh(f32),
    /// Dynamic viewport height unit.
    Dvh(f32),
    /// Automatic intrinsic content sizing.
    #[default]
    Auto,
    /// Shrink-to-fit content size.
    FitContent,
    /// Minimum intrinsic content size.
    MinContent,
    /// Maximum intrinsic content size.
    MaxContent,
    /// Clamped value (min, val, max).
    Clamp(f32, f32, f32),
}

/// Color value or special keyword (transparent, currentColor).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ColorValue {
    /// Explicit RGBA color.
    Color(Color),
    /// Current text color context.
    CurrentColor,
    /// Fully transparent.
    #[default]
    Transparent,
}

impl From<Color> for ColorValue {
    fn from(c: Color) -> Self {
        Self::Color(c)
    }
}

/// Animation and transition timing acceleration curves.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum TimingFunction {
    /// Linear velocity.
    Linear,
    /// Ease acceleration curve.
    #[default]
    Ease,
    /// Ease-in accelerating curve.
    EaseIn,
    /// Ease-out decelerating curve.
    EaseOut,
    /// Ease-in-out curve.
    EaseInOut,
    /// Cubic Bezier curve defined by control points (x1, y1, x2, y2).
    CubicBezier(f32, f32, f32, f32),
    /// Stepped transition for sprite animations.
    Steps(u32),
}
