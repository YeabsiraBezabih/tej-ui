//! Container card component (shadcn `Card`).

use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Retained state for a mounted Card.
#[derive(Debug, Default)]
pub struct CardState {
    pub bounds: Rect,
    pub title: String,
}

/// Structural container card with header, title, description, and footer.
#[derive(Debug, Clone)]
pub struct Card {
    pub title: String,
    pub description: Option<String>,
    pub footer: Option<String>,
}

impl Card {
    /// Creates a new card with a title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            footer: None,
        }
    }

    /// Sets secondary description text.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets footer text or action label.
    #[must_use]
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }
}

impl View for Card {
    type State = CardState;

    fn build(&self) -> Self::State {
        CardState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(320.0, 180.0)),
            title: self.title.clone(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.title = self.title.clone();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
