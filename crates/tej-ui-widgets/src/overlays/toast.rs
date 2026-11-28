use std::fmt::Debug;

/// Severity level of a toast alert.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ToastSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// A toast notification item.
#[derive(Clone, Debug)]
pub struct ToastItem {
    pub id: u64,
    pub title: String,
    pub message: Option<String>,
    pub severity: ToastSeverity,
    pub duration_ms: u32,
}

impl ToastItem {
    /// Create a new toast notification.
    pub fn new(id: u64, title: impl Into<String>, severity: ToastSeverity) -> Self {
        Self {
            id,
            title: title.into(),
            message: None,
            severity,
            duration_ms: 4000,
        }
    }

    /// Add descriptive message.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}
