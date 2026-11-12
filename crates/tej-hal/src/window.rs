//! Hardware Abstraction Layer for OS native windows and multi-window registry.

use crate::event::UiEvent;
use std::collections::HashMap;
use std::fmt::Debug;
use tej_layout::Size;

/// Unique identifier for an open OS window.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowId(pub u64);

/// Configuration settings for creating a native OS window.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    pub decorated: bool,
    pub high_dpi: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("tej-ui Application"),
            size: Size::new(800.0, 600.0),
            resizable: true,
            decorated: true,
            high_dpi: true,
        }
    }
}

/// Abstract Hardware Abstraction Layer for OS native windows.
pub trait PlatformWindow: Debug + Send + Sync {
    /// Window identifier.
    fn id(&self) -> WindowId;

    /// Current client area size.
    fn size(&self) -> Size;

    /// Current monitor DPI scaling factor (e.g. 1.0, 1.25, 2.0).
    fn scale_factor(&self) -> f32 {
        1.0
    }

    /// Request a window repaint from the OS compositor.
    fn request_redraw(&self);

    /// Set window title.
    fn set_title(&mut self, title: &str);

    /// Poll pending window events.
    fn poll_events(&mut self) -> Vec<UiEvent>;
}

/// Multi-window registry for tracking active top-level windows and modals.
#[derive(Debug, Default)]
pub struct WindowRegistry {
    windows: HashMap<WindowId, Box<dyn PlatformWindow>>,
    focused_window: Option<WindowId>,
}

impl WindowRegistry {
    /// Creates a new empty window registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused_window: None,
        }
    }

    /// Registers an open window.
    pub fn register(&mut self, window: Box<dyn PlatformWindow>) -> WindowId {
        let id = window.id();
        self.windows.insert(id, window);
        if self.focused_window.is_none() {
            self.focused_window = Some(id);
        }
        id
    }

    /// Unregisters and closes a window by ID.
    pub fn unregister(&mut self, id: WindowId) -> Option<Box<dyn PlatformWindow>> {
        let removed = self.windows.remove(&id);
        if self.focused_window == Some(id) {
            self.focused_window = self.windows.keys().next().copied();
        }
        removed
    }

    /// Returns the number of currently open windows.
    #[must_use]
    pub fn count(&self) -> usize {
        self.windows.len()
    }

    /// Retrieves a reference to an open window.
    #[must_use]
    pub fn get(&self, id: WindowId) -> Option<&dyn PlatformWindow> {
        self.windows.get(&id).map(AsRef::as_ref)
    }

    /// Retrieves a mutable reference to an open window.
    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut Box<dyn PlatformWindow>> {
        self.windows.get_mut(&id)
    }

    /// Returns the currently focused window ID.
    #[must_use]
    pub const fn focused_window_id(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Sets the focused window.
    pub fn set_focused_window(&mut self, id: WindowId) {
        if self.windows.contains_key(&id) {
            self.focused_window = Some(id);
        }
    }
}
