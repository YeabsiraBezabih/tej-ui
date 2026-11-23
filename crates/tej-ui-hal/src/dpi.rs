//! Per-monitor DPI scaling, fractional scaling factors, and coordinate spaces.

use tej_ui_layout::Point;

/// Logical DPI baseline standard (96 DPI on Windows/Linux, 72/144 on macOS/iOS).
pub const BASELINE_DPI: f32 = 96.0;

/// Represents the display scaling factor for a monitor or window surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DpiScale {
    factor: f32,
}

impl DpiScale {
    /// Creates a new DPI scaling factor. Factor is clamped to minimum 0.5 and maximum 8.0.
    #[inline]
    pub fn new(factor: f32) -> Self {
        let clamped = factor.clamp(0.5, 8.0);
        Self { factor: clamped }
    }

    /// Creates a DPI scale from physical dots-per-inch based on 96 DPI baseline.
    #[inline]
    pub fn from_dpi(dpi: f32) -> Self {
        Self::new(dpi / BASELINE_DPI)
    }

    /// Baseline 100% scale (factor 1.0).
    #[inline]
    pub const fn default_100() -> Self {
        Self { factor: 1.0 }
    }

    /// 150% fractional scale (common on 1080p/1440p laptops).
    #[inline]
    pub const fn scale_150() -> Self {
        Self { factor: 1.5 }
    }

    /// 200% HiDPI / Retina scale (e.g. 4K displays or Apple Retina).
    #[inline]
    pub const fn scale_200() -> Self {
        Self { factor: 2.0 }
    }

    /// The raw scale factor multiplier.
    #[inline]
    pub fn factor(&self) -> f32 {
        self.factor
    }

    /// Converts logical points to physical device pixels.
    #[inline]
    pub fn logical_to_physical(&self, logical: f32) -> f32 {
        logical * self.factor
    }

    /// Converts logical 2D point to physical pixel coordinates.
    #[inline]
    pub fn logical_to_physical_point(&self, pt: Point) -> Point {
        Point::new(pt.x * self.factor, pt.y * self.factor)
    }

    /// Converts physical device pixels to logical points.
    #[inline]
    pub fn physical_to_logical(&self, physical: f32) -> f32 {
        physical / self.factor
    }

    /// Converts physical 2D point to logical point coordinates.
    #[inline]
    pub fn physical_to_logical_point(&self, pt: Point) -> Point {
        Point::new(pt.x / self.factor, pt.y / self.factor)
    }

    /// Snaps a logical coordinate to the nearest physical pixel boundary to eliminate blurry edges.
    #[inline]
    pub fn snap_to_physical_pixel(&self, logical: f32) -> f32 {
        (logical * self.factor).round() / self.factor
    }
}

impl Default for DpiScale {
    fn default() -> Self {
        Self::default_100()
    }
}

/// Event dispatched when a window transitions across monitors with different DPIs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DpiChangeEvent {
    pub old_scale: DpiScale,
    pub new_scale: DpiScale,
}

impl DpiChangeEvent {
    pub fn new(old_scale: DpiScale, new_scale: DpiScale) -> Self {
        Self {
            old_scale,
            new_scale,
        }
    }

    /// Ratio change between new scale and old scale.
    pub fn ratio(&self) -> f32 {
        self.new_scale.factor() / self.old_scale.factor()
    }
}
