//! Status badge indicator (shadcn `Badge`).

use tej_ui_hal::Color;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_text::FontRegistry;
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
            BadgeVariant::Success => Color::from_rgb_u32(0x00E9_F7F2),
            BadgeVariant::Warning => Color::from_rgb_u32(0x00FE_F3C7),
        }
    }

    fn measured_width(&self) -> f32 {
        let measured = FontRegistry::global().measure_text(
            &self.text,
            11.0,
            tej_ui_text::FontFamilyKind::SansSerif,
            400,
        );
        measured.width.ceil() + 16.0
    }
}

impl View for Badge {
    type State = BadgeState;

    fn build(&self) -> Self::State {
        let width = self.measured_width();
        BadgeState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 22.0)),
            text: self.text.clone(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.text.clone_from(&self.text);
        state.bounds.size = Size::new(self.measured_width(), 22.0);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let width = self.measured_width();
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
            BadgeVariant::Success => ctx.theme.colors.success,
            BadgeVariant::Warning => ctx.theme.colors.warning,
        };
        let fg = match self.variant {
            BadgeVariant::Default => ctx.theme.colors.primary_foreground,
            BadgeVariant::Secondary => ctx.theme.colors.secondary_foreground,
            BadgeVariant::Destructive => ctx.theme.colors.destructive_foreground,
            BadgeVariant::Outline => ctx.theme.colors.foreground,
            BadgeVariant::Success => ctx.theme.colors.success_foreground,
            BadgeVariant::Warning => ctx.theme.colors.warning_foreground,
        };
        let radius = ctx.theme.radii.full;
        if self.variant == BadgeVariant::Outline {
            ctx.fill_rounded_rect(bounds, radius, ctx.theme.colors.border);
            let inner = Rect::new(
                bounds.origin.x + 1.0,
                bounds.origin.y + 1.0,
                (bounds.size.width - 2.0).max(0.0),
                (bounds.size.height - 2.0).max(0.0),
            );
            ctx.fill_rounded_rect(inner, (radius - 1.0).max(0.0), ctx.theme.colors.background);
        } else {
            ctx.fill_rounded_rect(bounds, radius, bg);
        }
        let font_size = ctx.theme.typography.caption.font_size;
        let text_size = FontRegistry::global().measure_text(
            &state.text,
            font_size,
            tej_ui_text::FontFamilyKind::SansSerif,
            400,
        );
        let text_x = bounds.origin.x + ((bounds.size.width - text_size.width) * 0.5).max(4.0);
        let text_y = bounds.origin.y + ((bounds.size.height - text_size.height) * 0.5).max(0.0);
        let clip_radius = radius.min(bounds.size.height * 0.5);
        ctx.push_clip(bounds, [clip_radius; 4]);
        ctx.draw_text(&state.text, Point::new(text_x, text_y), font_size, fg);
        ctx.pop_clip();
    }
}
