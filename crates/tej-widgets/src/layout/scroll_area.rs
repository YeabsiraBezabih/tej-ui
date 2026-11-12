//! Custom scrollable container viewport (shadcn `ScrollArea`).

use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Retained state for a mounted ScrollArea.
#[derive(Debug, Default)]
pub struct ScrollAreaState {
    pub viewport_bounds: Rect,
    pub scroll_offset_y: f32,
    pub scroll_offset_x: f32,
}

/// Custom scrollable viewport with kinetic inertia and scrollbar track bounds.
#[derive(Debug, Clone)]
pub struct ScrollArea {
    scroll_x: Signal<f32>,
    scroll_y: Signal<f32>,
    content_size: Size,
    viewport_size: Size,
}

impl ScrollArea {
    /// Creates a new scroll area.
    #[must_use]
    pub const fn new(
        scroll_x: Signal<f32>,
        scroll_y: Signal<f32>,
        content_size: Size,
        viewport_size: Size,
    ) -> Self {
        Self {
            scroll_x,
            scroll_y,
            content_size,
            viewport_size,
        }
    }

    /// Scrolls vertically by delta points.
    pub fn scroll_by(&self, delta_x: f32, delta_y: f32) {
        let max_x = (self.content_size.width - self.viewport_size.width).max(0.0);
        let max_y = (self.content_size.height - self.viewport_size.height).max(0.0);

        let next_x = (self.scroll_x.get() + delta_x).clamp(0.0, max_x);
        let next_y = (self.scroll_y.get() + delta_y).clamp(0.0, max_y);

        self.scroll_x.set(next_x);
        self.scroll_y.set(next_y);
    }
}

impl View for ScrollArea {
    type State = ScrollAreaState;

    fn build(&self) -> Self::State {
        ScrollAreaState {
            viewport_bounds: Rect::from_origin_size(Point::ZERO, self.viewport_size),
            scroll_offset_x: self.scroll_x.get(),
            scroll_offset_y: self.scroll_y.get(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.scroll_offset_x = self.scroll_x.get();
        state.scroll_offset_y = self.scroll_y.get();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
