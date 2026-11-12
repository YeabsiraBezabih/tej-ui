//! Slide-over drawer panel (shadcn `Sheet`).

use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Edge from which the sheet drawer slides out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// Retained state for a mounted Sheet.
#[derive(Debug, Default)]
pub struct SheetState {
    pub bounds: Rect,
    pub is_open: bool,
}

/// Slide-over drawer panel anchored to a screen edge.
#[derive(Debug, Clone)]
pub struct Sheet {
    is_open: Signal<bool>,
    side: SheetSide,
    size_points: f32,
    title: String,
}

impl Sheet {
    /// Creates a new sheet drawer.
    #[must_use]
    pub fn new(is_open: Signal<bool>, title: impl Into<String>) -> Self {
        Self {
            is_open,
            side: SheetSide::Right,
            size_points: 380.0,
            title: title.into(),
        }
    }

    /// Sets edge side from which the sheet slides.
    #[must_use]
    pub const fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    /// Sets drawer width or height in points.
    #[must_use]
    pub const fn size(mut self, points: f32) -> Self {
        self.size_points = points;
        self
    }

    /// Title of the sheet drawer.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Closes the sheet.
    pub fn close(&self) {
        self.is_open.set(false);
    }
}

impl View for Sheet {
    type State = SheetState;

    fn build(&self) -> Self::State {
        let is_open = self.is_open.get();
        let size = match self.side {
            SheetSide::Left | SheetSide::Right => Size::new(self.size_points, 768.0),
            SheetSide::Top | SheetSide::Bottom => Size::new(1024.0, self.size_points),
        };

        SheetState {
            bounds: Rect::from_origin_size(Point::ZERO, size),
            is_open,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.is_open = self.is_open.get();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
