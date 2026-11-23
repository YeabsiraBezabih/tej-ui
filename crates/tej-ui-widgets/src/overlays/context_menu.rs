use std::fmt;
use std::sync::Arc;

pub type MenuAction = Arc<dyn Fn() + Send + Sync>;

/// Context menu item entry.
#[derive(Clone)]
pub enum MenuItem {
    Action {
        id: String,
        label: String,
        shortcut: Option<String>,
        action: MenuAction,
    },
    Separator,
}

impl fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Action {
                id,
                label,
                shortcut,
                ..
            } => f
                .debug_struct("MenuItem::Action")
                .field("id", id)
                .field("label", label)
                .field("shortcut", shortcut)
                .finish(),
            Self::Separator => write!(f, "MenuItem::Separator"),
        }
    }
}

impl MenuItem {
    /// Create an action item.
    pub fn action<F: Fn() + Send + Sync + 'static>(
        id: impl Into<String>,
        label: impl Into<String>,
        action: F,
    ) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            action: Arc::new(action),
        }
    }

    /// Create a divider separator.
    #[must_use]
    pub const fn separator() -> Self {
        Self::Separator
    }
}
