//! Callout alert banner (shadcn `Alert`).

use tej_hal::Color;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

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
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.title = self.title.clone();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
