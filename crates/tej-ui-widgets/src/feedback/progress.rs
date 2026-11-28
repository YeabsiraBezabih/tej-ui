//! Task completion progress indicator (shadcn `Progress`).

use tej_ui_core::Signal;
use tej_ui_layout::{Dimension, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{PaintContext, View};

/// Retained state for a mounted Progress bar.
#[derive(Debug, Default)]
pub struct ProgressState {
    pub bounds: Rect,
    pub percentage: f32,
    pub node: u32,
}

/// Horizontal progress bar reflecting task completion [0.0 .. 100.0]%.
#[derive(Debug, Clone)]
pub struct Progress {
    percentage: Signal<f32>,
    height: f32,
    width: Dimension,
}

impl Progress {
    /// Creates a progress bar bound to a reactive completion percentage signal.
    #[must_use]
    pub const fn new(percentage: Signal<f32>) -> Self {
        Self {
            percentage,
            height: 8.0,
            width: Dimension::Percent(1.0),
        }
    }

    /// Sets bar thickness.
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets explicit progress bar width constraint.
    #[must_use]
    pub const fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }
}

impl View for Progress {
    type State = ProgressState;

    fn build(&self) -> Self::State {
        let pct = self.percentage.get().clamp(0.0, 100.0);
        let default_w = match self.width {
            Dimension::Points(w) => w,
            _ => 240.0,
        };
        ProgressState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(default_w, self.height)),
            percentage: pct,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.percentage = self.percentage.get().clamp(0.0, 100.0);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let style = LayoutStyle {
            width: self.width,
            height: Dimension::Points(self.height),
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

        let radius = bounds.size.height * 0.5;
        // Track background
        ctx.fill_rounded_rect(bounds, radius, ctx.theme.colors.secondary);

        // Filled indicator clipped to rounded track bounds
        let ratio = state.percentage.clamp(0.0, 100.0) / 100.0;
        let fill_width = bounds.size.width * ratio;
        if fill_width > 0.0 {
            ctx.push_clip(bounds, [radius; 4]);
            let fill_rect = Rect::new(
                bounds.origin.x,
                bounds.origin.y,
                fill_width,
                bounds.size.height,
            );
            ctx.fill_rounded_rect(fill_rect, radius, ctx.theme.colors.primary);
            ctx.pop_clip();
        }
    }
}
