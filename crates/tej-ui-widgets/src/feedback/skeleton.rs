//! Placeholder loading skeleton (shadcn `Skeleton`).

use tej_ui_layout::{Dimension, Point, Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Skeleton.
#[derive(Debug, Default)]
pub struct SkeletonState {
    pub bounds: Rect,
}

/// Shimmering placeholder box used to mimic content dimensions while loading.
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub width: Dimension,
    pub height: Dimension,
    pub corner_radius: f32,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self {
            width: Dimension::Points(120.0),
            height: Dimension::Points(20.0),
            corner_radius: 4.0,
        }
    }
}

impl Skeleton {
    /// Creates a rectangular skeleton with given dimensions.
    #[must_use]
    pub fn new(width: Dimension, height: Dimension) -> Self {
        Self {
            width,
            height,
            corner_radius: 4.0,
        }
    }

    /// Creates a circular skeleton avatar placeholder.
    #[must_use]
    pub fn circle(diameter: f32) -> Self {
        Self {
            width: Dimension::Points(diameter),
            height: Dimension::Points(diameter),
            corner_radius: diameter * 0.5,
        }
    }
}

impl View for Skeleton {
    type State = SkeletonState;

    fn build(&self) -> Self::State {
        let w = match self.width {
            Dimension::Points(p) => p,
            _ => 120.0,
        };
        let h = match self.height {
            Dimension::Points(p) => p,
            _ => 20.0,
        };
        SkeletonState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(w, h)),
        }
    }

    fn rebuild(&self, _state: &mut Self::State) {}

    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {}
}
