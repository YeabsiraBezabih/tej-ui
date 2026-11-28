use std::fmt;
use std::sync::Arc;

pub type CommandAction = Arc<dyn Fn() + Send + Sync>;

/// An individual action or item inside the Command Palette.
#[derive(Clone)]
pub struct CommandItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub shortcut: Option<String>,
    pub category: String,
    pub action: Option<CommandAction>,
}

impl fmt::Debug for CommandItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandItem")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("category", &self.category)
            .field("shortcut", &self.shortcut)
            .finish_non_exhaustive()
    }
}

impl CommandItem {
    /// Create a new command item with ID, title, and category.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            shortcut: None,
            category: category.into(),
            action: None,
        }
    }

    /// Add an action callback when this command is selected.
    #[must_use]
    pub fn on_select<F: Fn() + Send + Sync + 'static>(mut self, action: F) -> Self {
        self.action = Some(Arc::new(action));
        self
    }

    /// Set keyboard shortcut badge.
    #[must_use]
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set descriptive subtitle.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}
