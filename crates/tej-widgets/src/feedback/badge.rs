//! Status badge indicator (shadcn `Badge`).

use tej_hal::Color;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Visual presentation style variant for a badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

/// Retained state for a mounted Badge.
#[derive(Debug, Default)]
pub struct BadgeState {
    pub bounds: Rect,
    pub text: String,
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
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.text = self.text.clone();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
