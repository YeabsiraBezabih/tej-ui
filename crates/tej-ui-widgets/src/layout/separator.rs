//! Visual content separator / divider (shadcn `Separator`).

use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

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
    pub node: u32,
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
            node: 0,
        }
    }

    fn rebuild(&self, _state: &mut Self::State) {}

    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {}

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let (width, height) = match self.orientation {
            SeparatorOrientation::Horizontal => (
                tej_ui_layout::Dimension::Auto,
                tej_ui_layout::Dimension::Points(self.thickness),
            ),
            SeparatorOrientation::Vertical => (
                tej_ui_layout::Dimension::Points(self.thickness),
                tej_ui_layout::Dimension::Auto,
            ),
        };
        let style = tej_ui_layout::LayoutStyle {
            width,
            height,
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);
        ctx.fill_rect(bounds, ctx.theme.colors.border);
    }
}
