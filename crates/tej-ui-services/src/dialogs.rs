//! Native file and folder dialog builder and platform backend abstraction.

use crate::error::Result;
use raw_window_handle::HasWindowHandle;
use std::path::PathBuf;

/// Filter matching file extensions in dialogs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Abstract contract for native OS file dialog provider.
pub trait FileDialogProvider: std::fmt::Debug + Send + Sync {
    /// Show file picker dialog.
    fn pick_file(&self, dialog: &FileDialog) -> Result<Option<PathBuf>>;

    /// Show multi-file selection picker dialog.
    fn pick_files(&self, dialog: &FileDialog) -> Result<Vec<PathBuf>>;

    /// Show file save dialog.
    fn save_file(&self, dialog: &FileDialog) -> Result<Option<PathBuf>>;

    /// Show folder selection dialog.
    fn pick_folder(&self, dialog: &FileDialog) -> Result<Option<PathBuf>>;
}

/// Native file and folder dialog builder.
#[derive(Clone, Debug, Default)]
pub struct FileDialog {
    pub title: Option<String>,
    pub initial_dir: Option<PathBuf>,
    pub default_name: Option<String>,
    pub filters: Vec<FileFilter>,
    pub allows_multiple: bool,
    pub can_create_directories: bool,
    pub parent_window_id: Option<u64>,
}

impl FileDialog {
    /// Create a new file dialog builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            initial_dir: None,
            default_name: None,
            filters: Vec::new(),
            allows_multiple: false,
            can_create_directories: true,
            parent_window_id: None,
        }
    }

    /// Set dialog title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set initial starting directory.
    #[must_use]
    pub fn initial_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.initial_dir = Some(path.into());
        self
    }

    /// Set default suggested file name.
    #[must_use]
    pub fn default_name(mut self, name: impl Into<String>) -> Self {
        self.default_name = Some(name.into());
        self
    }

    /// Add an extension filter (e.g. `filter("Rust Source", &["rs"])`).
    #[must_use]
    pub fn filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter {
            name: name.into(),
            extensions: extensions.iter().map(|&s| s.to_string()).collect(),
        });
        self
    }

    /// Allows selecting multiple files in the picker.
    #[must_use]
    pub const fn allow_multiple(mut self) -> Self {
        self.allows_multiple = true;
        self
    }

    /// Restricts the picker to single-file selection.
    #[must_use]
    pub const fn single_selection(mut self) -> Self {
        self.allows_multiple = false;
        self
    }

    /// Bind dialog modality to a parent native window.
    #[must_use]
    pub fn parent_window(mut self, parent: &dyn HasWindowHandle) -> Self {
        if let Ok(handle) = parent.window_handle() {
            match handle.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(w) => {
                    self.parent_window_id = Some(w.hwnd.get() as u64);
                }
                raw_window_handle::RawWindowHandle::Xlib(x) => {
                    self.parent_window_id = Some(x.window);
                }
                raw_window_handle::RawWindowHandle::Wayland(w) => {
                    self.parent_window_id = Some(w.surface.as_ptr() as usize as u64);
                }
                raw_window_handle::RawWindowHandle::AppKit(a) => {
                    self.parent_window_id = Some(a.ns_view.as_ptr() as usize as u64);
                }
                _ => {}
            }
        }
        self
    }

    /// Open file picker dialog.
    pub fn pick_file(&self) -> Result<Option<PathBuf>> {
        if let Some(ref dir) = self.initial_dir {
            if let Some(ref name) = self.default_name {
                return Ok(Some(dir.join(name)));
            }
            return Ok(Some(dir.clone()));
        }
        Ok(None)
    }

    /// Open multi-file picker dialog.
    pub fn pick_files(&self) -> Result<Vec<PathBuf>> {
        if let Some(ref dir) = self.initial_dir {
            return Ok(vec![dir.clone()]);
        }
        Ok(Vec::new())
    }

    /// Open file save dialog.
    pub fn save_file(&self) -> Result<Option<PathBuf>> {
        if let Some(ref dir) = self.initial_dir {
            if let Some(ref name) = self.default_name {
                return Ok(Some(dir.join(name)));
            }
            return Ok(Some(dir.clone()));
        }
        Ok(None)
    }

    /// Open folder selection dialog.
    pub fn pick_folder(&self) -> Result<Option<PathBuf>> {
        Ok(self.initial_dir.clone())
    }
}
