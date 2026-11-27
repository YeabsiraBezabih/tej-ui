//! Placeholder loading skeleton (shadcn `Skeleton`).

use tej_ui_layout::{Dimension, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{PaintContext, View};

/// Retained state for a mounted Skeleton.
#[derive(Debug, Default)]
pub struct SkeletonState {
    pub bounds: Rect,
    pub node: u32,
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
            node: 0,
        }
    }

    fn rebuild(&self, _state: &mut Self::State) {}

    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {}

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let style = LayoutStyle {
            width: self.width,
            height: self.height,
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        // Base muted skeleton block
        ctx.fill_rounded_rect(bounds, self.corner_radius, ctx.theme.colors.muted);

        // Subtle shimmer highlight stripe
        if bounds.size.width > 8.0 && bounds.size.height > 2.0 {
            let shimmer_w = (bounds.size.width * 0.35).max(8.0);
            let shimmer_x = bounds.origin.x + (bounds.size.width - shimmer_w) * 0.35;
            let shimmer_rect = Rect::new(shimmer_x, bounds.origin.y, shimmer_w, bounds.size.height);
            ctx.fill_rounded_rect(
                shimmer_rect,
                self.corner_radius,
                ctx.theme.colors.foreground.with_alpha(20),
            );
        }
    }
}
