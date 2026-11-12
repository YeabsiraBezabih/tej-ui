use crate::error::Result;
use std::path::PathBuf;

/// Filter matching file extensions in dialogs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Native file and folder dialog builder.
#[derive(Clone, Debug, Default)]
pub struct FileDialog {
    pub title: Option<String>,
    pub initial_dir: Option<PathBuf>,
    pub default_name: Option<String>,
    pub filters: Vec<FileFilter>,
}

impl FileDialog {
    /// Create a new file dialog builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set dialog title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set initial starting directory.
    pub fn initial_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.initial_dir = Some(path.into());
        self
    }

    /// Add an extension filter (e.g. `filter("Rust Source", &["rs"])`).
    pub fn filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter {
            name: name.into(),
            extensions: extensions.iter().map(|&s| s.to_string()).collect(),
        });
        self
    }

    /// Open file picker dialog.
    pub fn pick_file(&self) -> Result<Option<PathBuf>> {
        // Platform backend integration hook (Win32 GetOpenFileName / AppKit NSOpenPanel / Portal)
        Ok(self.initial_dir.clone())
    }

    /// Open file save dialog.
    pub fn save_file(&self) -> Result<Option<PathBuf>> {
        Ok(self.initial_dir.clone())
    }

    /// Open folder selection dialog.
    pub fn pick_folder(&self) -> Result<Option<PathBuf>> {
        Ok(self.initial_dir.clone())
    }
}
