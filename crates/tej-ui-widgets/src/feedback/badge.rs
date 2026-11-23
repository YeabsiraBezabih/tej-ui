//! Status badge indicator (shadcn `Badge`).

use tej_ui_hal::Color;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Visual presentation style variant for a badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
}

/// Retained state for a mounted Badge.
#[derive(Debug, Default)]
pub struct BadgeState {
    pub bounds: Rect,
    pub text: String,
    pub node: u32,
}

/// Compact status pill or counter indicator.
#[derive(Debug, Clone)]
pub struct Badge {
    pub text: String,
    pub variant: BadgeVariant,
}

impl Badge {
    /// Creates a default badge.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Default,
        }
    }

    /// Sets the badge variant.
    #[must_use]
    pub const fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Primary background color associated with the variant.
    #[must_use]
    pub const fn background_color(&self) -> Color {
        match self.variant {
            BadgeVariant::Default => Color::rgb(15, 23, 42),
            BadgeVariant::Secondary => Color::rgb(241, 245, 249),
            BadgeVariant::Destructive => Color::rgb(239, 68, 68),
            BadgeVariant::Outline => Color::rgba(0, 0, 0, 0),
            BadgeVariant::Success => Color::from_rgb_u32(0xE9F7F2),
            BadgeVariant::Warning => Color::from_rgb_u32(0xFEF3C7),
        }
    }
}

impl View for Badge {
    type State = BadgeState;

    fn build(&self) -> Self::State {
        let width = (self.text.len() as f32) * 7.5 + 16.0;
        BadgeState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 22.0)),
            text: self.text.clone(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.text = self.text.clone();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let width = (self.text.len() as f32) * 7.5 + 16.0;
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(width),
            height: tej_ui_layout::Dimension::Points(22.0),
            padding: tej_ui_layout::Insets::symmetric(2.0, 8.0),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        state.bounds = Rect::from_origin_size(Point::ZERO, Size::new(width, 22.0));
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);
        let bg = match self.variant {
            BadgeVariant::Default => ctx.theme.colors.primary,
            BadgeVariant::Secondary => ctx.theme.colors.secondary,
            BadgeVariant::Destructive => ctx.theme.colors.destructive,
            BadgeVariant::Outline => Color::TRANSPARENT,
            BadgeVariant::Success => Color::from_rgb_u32(0xE9F7F2),
            BadgeVariant::Warning => Color::from_rgb_u32(0xFEF3C7),
        };
        let fg = match self.variant {
            BadgeVariant::Default => ctx.theme.colors.primary_foreground,
            BadgeVariant::Secondary => ctx.theme.colors.secondary_foreground,
            BadgeVariant::Destructive => ctx.theme.colors.destructive_foreground,
            BadgeVariant::Outline => ctx.theme.colors.foreground,
            BadgeVariant::Success => Color::from_rgb_u32(0x259676),
            BadgeVariant::Warning => Color::from_rgb_u32(0xD97706),
        };
        let radius = ctx.theme.radii.full;
        if self.variant == BadgeVariant::Outline {
            ctx.fill_rounded_rect(bounds, radius, ctx.theme.colors.border);
        } else {
            ctx.fill_rounded_rect(bounds, radius, bg);
        }
        let font_size = ctx.theme.typography.caption.font_size;
        let text_origin = Point::new(bounds.origin.x + 8.0, bounds.origin.y + 4.0);
        ctx.draw_text(&state.text, text_origin, font_size, fg);
    }
}
