//! Hover helper tooltip popup (shadcn `Tooltip`).

use tej_ui_core::Signal;
use tej_ui_hal::Color;
use tej_ui_layout::{Dimension, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{PaintContext, View};

/// Retained state for a mounted Tooltip.
#[derive(Debug, Default)]
pub struct TooltipState {
    pub bounds: Rect,
    pub is_visible: bool,
    pub text: String,
    pub node: u32,
}

/// Floating tooltip communicating context on hover.
#[derive(Debug, Clone)]
pub struct Tooltip {
    text: String,
    is_visible: Signal<bool>,
}

impl Tooltip {
    /// Creates a tooltip with text content and visibility signal.
    #[must_use]
    pub fn new(text: impl Into<String>, is_visible: Signal<bool>) -> Self {
        Self {
            text: text.into(),
            is_visible,
        }
    }

    /// Sets tooltip visibility.
    pub fn set_visible(&self, visible: bool) {
        self.is_visible.set(visible);
    }

    fn measured_width(&self) -> f32 {
        (self.text.chars().count() as f32) * 7.5 + 16.0
    }
}

impl View for Tooltip {
    type State = TooltipState;

    fn build(&self) -> Self::State {
        let width = self.measured_width();
        TooltipState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 28.0)),
            is_visible: self.is_visible.get(),
            text: self.text.clone(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.is_visible = self.is_visible.get();
        state.text = self.text.clone();
        state.bounds.size = Size::new(self.measured_width(), 28.0);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let style = if state.is_visible {
            LayoutStyle {
                width: Dimension::Points(self.measured_width()),
                height: Dimension::Points(28.0),
                ..Default::default()
            }
        } else {
            LayoutStyle {
                width: Dimension::Points(0.0),
                height: Dimension::Points(0.0),
                ..Default::default()
            }
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        if !state.is_visible {
            return;
        }

        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        // Dark tooltip pill (uses dark zinc surface in both light and dark themes)
        let pill_bg = Color::from_rgb_u32(0x18181B);
        let pill_fg = Color::from_rgb_u32(0xFAFAFA);
        let radius = ctx.theme.radii.md;

        ctx.fill_rounded_rect(bounds, radius, ctx.theme.colors.border);
        let inner = Rect::new(
            bounds.origin.x + 1.0,
            bounds.origin.y + 1.0,
            (bounds.size.width - 2.0).max(0.0),
            (bounds.size.height - 2.0).max(0.0),
        );
        ctx.fill_rounded_rect(inner, (radius - 1.0).max(0.0), pill_bg);

        let font_size = ctx.theme.typography.caption.font_size;
        let text_pt = Point::new(
            bounds.origin.x + 8.0,
            bounds.origin.y + (bounds.size.height - font_size) * 0.5,
        );
        ctx.draw_text(&state.text, text_pt, font_size, pill_fg);
    }
}
