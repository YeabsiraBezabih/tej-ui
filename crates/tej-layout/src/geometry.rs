/// 2D point representation with single-precision floating point components.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 2D size dimensions (width and height).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    #[inline]
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// 2D bounding rectangle with position origin and dimensions.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Self = Self {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[inline]
    #[must_use]
    pub fn min_x(&self) -> f32 {
        self.origin.x
    }

    #[inline]
    #[must_use]
    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    #[inline]
    #[must_use]
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    #[inline]
    #[must_use]
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }

    /// Check if point `p` falls within rectangle bounds.
    #[inline]
    #[must_use]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min_x() && p.x <= self.max_x() && p.y >= self.min_y() && p.y <= self.max_y()
    }

    /// Compute intersection with another rectangle.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let x1 = self.min_x().max(other.min_x());
        let y1 = self.min_y().max(other.min_y());
        let x2 = self.max_x().min(other.max_x());
        let y2 = self.max_y().min(other.max_y());

        if x2 >= x1 && y2 >= y1 {
            Some(Self::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// Compute union bounding box encompassing both rectangles.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let x1 = self.min_x().min(other.min_x());
        let y1 = self.min_y().min(other.min_y());
        let x2 = self.max_x().max(other.max_x());
        let y2 = self.max_y().max(other.max_y());
        Self::new(x1, y1, x2 - x1, y2 - y1)
    }
}

/// Padding or margin insets around four edges.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    #[inline]
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    #[inline]
    #[must_use]
    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    #[inline]
    #[must_use]
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}
