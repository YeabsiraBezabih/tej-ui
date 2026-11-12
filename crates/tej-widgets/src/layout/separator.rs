//! Visual content separator / divider (shadcn `Separator`).

use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Orientation for a separator line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Retained state for a mounted Separator.
#[derive(Debug, Default)]
pub struct SeparatorState {
    pub bounds: Rect,
}

/// Thin hairline visual divider.
#[derive(Debug, Clone, Copy, Default)]
pub struct Separator {
    pub orientation: SeparatorOrientation,
    pub thickness: f32,
}

impl Separator {
    /// Creates a horizontal separator.
    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            thickness: 1.0,
        }
    }

    /// Creates a vertical separator.
    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            orientation: SeparatorOrientation::Vertical,
            thickness: 1.0,
        }
    }
}

impl View for Separator {
    type State = SeparatorState;

    fn build(&self) -> Self::State {
        let size = match self.orientation {
            SeparatorOrientation::Horizontal => Size::new(100.0, self.thickness),
            SeparatorOrientation::Vertical => Size::new(self.thickness, 100.0),
        };
        SeparatorState {
            bounds: Rect::from_origin_size(Point::ZERO, size),
        }
    }

    fn rebuild(&self, _state: &mut Self::State) {}

    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {}
}
