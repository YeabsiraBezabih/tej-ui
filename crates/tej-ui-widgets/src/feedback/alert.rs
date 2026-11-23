//! Callout alert banner (shadcn `Alert`).

use tej_ui_hal::Color;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Severity classification for an alert callout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Destructive,
}

/// Retained state for a mounted Alert.
#[derive(Debug, Default)]
pub struct AlertState {
    pub bounds: Rect,
    pub title: String,
    pub node: u32,
}

/// Callout banner communicating system messages or feedback.
#[derive(Debug, Clone)]
pub struct Alert {
    pub title: String,
    pub description: Option<String>,
    pub severity: AlertSeverity,
}

impl Alert {
    /// Creates a new alert with title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            severity: AlertSeverity::Info,
        }
    }

    /// Sets secondary description text.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets severity level.
    #[must_use]
    pub const fn severity(mut self, severity: AlertSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Primary accent color based on severity.
    #[must_use]
    pub const fn accent_color(&self) -> Color {
        match self.severity {
            AlertSeverity::Info => Color::rgb(59, 130, 246),
            AlertSeverity::Success => Color::rgb(34, 197, 94),
            AlertSeverity::Warning => Color::rgb(245, 158, 11),
            AlertSeverity::Destructive => Color::rgb(239, 68, 68),
        }
    }
}

impl View for Alert {
    type State = AlertState;

    fn build(&self) -> Self::State {
        let h = if self.description.is_some() {
            68.0
        } else {
            44.0
        };
        AlertState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(420.0, h)),
            title: self.title.clone(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.title = self.title.clone();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let h = if self.description.is_some() {
            68.0
        } else {
            44.0
        };
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(420.0),
            height: tej_ui_layout::Dimension::Points(h),
            padding: tej_ui_layout::Insets::all(12.0),
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

        let accent = match self.severity {
            AlertSeverity::Destructive => ctx.theme.colors.destructive,
            _ => self.accent_color(),
        };

        // Outer border
        ctx.fill_rounded_rect(bounds, ctx.theme.radii.md, accent.with_alpha(100));

        // Inner background
        let inner = Rect::new(
            bounds.origin.x + 1.0,
            bounds.origin.y + 1.0,
            bounds.size.width - 2.0,
            bounds.size.height - 2.0,
        );
        ctx.fill_rounded_rect(inner, ctx.theme.radii.md - 1.0, ctx.theme.colors.card);

        // Left accent indicator bar
        let bar = Rect::new(bounds.origin.x, bounds.origin.y, 4.0, bounds.size.height);
        ctx.fill_rect(bar, accent);

        // Title
        let title_pt = Point::new(bounds.origin.x + 14.0, bounds.origin.y + 12.0);
        ctx.draw_text(
            &state.title,
            title_pt,
            ctx.theme.typography.h3.font_size,
            ctx.theme.colors.card_foreground,
        );

        // Description
        if let Some(desc) = &self.description {
            let desc_pt = Point::new(bounds.origin.x + 14.0, bounds.origin.y + 36.0);
            ctx.draw_text(
                desc,
                desc_pt,
                ctx.theme.typography.body_small.font_size,
                ctx.theme.colors.muted_foreground,
            );
        }
    }
}
