//! Hardware Abstraction Layer for OS native windows and multi-window registry.

use crate::event::UiEvent;
use std::collections::HashMap;
use std::fmt::Debug;
use tej_ui_layout::Size;

/// Unique identifier for an open OS window.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowId(pub u64);

/// Classification of native window presentation and modality semantics.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum WindowKind {
    /// Standard independent top-level application window.
    #[default]
    TopLevel,
    /// Floating utility palette or tool window (always-on-top, non-activating).
    UtilityPalette,
    /// Modal dialog that blocks interaction with its parent window until dismissed.
    ModalDialog,
}

/// Configuration settings for creating a native OS window.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    pub decorated: bool,
    pub high_dpi: bool,
    pub kind: WindowKind,
    pub parent: Option<WindowId>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("tej-ui Application"),
            size: Size::new(800.0, 600.0),
            resizable: true,
            decorated: true,
            high_dpi: true,
            kind: WindowKind::TopLevel,
            parent: None,
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

    /// Check if the window is currently open and active.
    fn is_open(&self) -> bool {
        true
    }
}

/// Multi-window registry for tracking active top-level windows, utility palettes, and modal ownership.
#[derive(Debug, Default)]
pub struct WindowRegistry {
    windows: HashMap<WindowId, Box<dyn PlatformWindow>>,
    focused_window: Option<WindowId>,
    /// Map of child window ID -> parent window ID.
    parent_map: HashMap<WindowId, WindowId>,
    /// Map of parent window ID -> list of active child modal IDs.
    modal_children: HashMap<WindowId, Vec<WindowId>>,
}

impl WindowRegistry {
    /// Creates a new empty window registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused_window: None,
            parent_map: HashMap::new(),
            modal_children: HashMap::new(),
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

    /// Registers a child modal window owned by `parent_id`.
    ///
    /// The parent window becomes blocked until the modal dialog is unregistered.
    pub fn register_modal(
        &mut self,
        modal: Box<dyn PlatformWindow>,
        parent_id: WindowId,
    ) -> WindowId {
        let id = modal.id();
        self.windows.insert(id, modal);
        self.parent_map.insert(id, parent_id);
        self.modal_children.entry(parent_id).or_default().push(id);
        // Focus moves to newly opened modal dialog
        self.focused_window = Some(id);
        id
    }

    /// Returns `true` if interaction with `window_id` is currently blocked by an active modal dialog.
    #[must_use]
    pub fn is_window_blocked(&self, window_id: WindowId) -> bool {
        self.modal_children
            .get(&window_id)
            .is_some_and(|children| !children.is_empty())
    }

    /// Returns the parent window of `child_id`, if it has one.
    #[must_use]
    pub fn parent_of(&self, child_id: WindowId) -> Option<WindowId> {
        self.parent_map.get(&child_id).copied()
    }

    /// Returns all active child modals owned by `parent_id`.
    #[must_use]
    pub fn modals_for(&self, parent_id: WindowId) -> &[WindowId] {
        self.modal_children
            .get(&parent_id)
            .map_or(&[], Vec::as_slice)
    }

    /// Unregisters and closes a window by ID, unblocking its parent if it was a modal.
    pub fn unregister(&mut self, id: WindowId) -> Option<Box<dyn PlatformWindow>> {
        let removed = self.windows.remove(&id);

        // If this was a modal, remove from parent's modal children list
        if let Some(parent_id) = self.parent_map.remove(&id) {
            if let Some(children) = self.modal_children.get_mut(&parent_id) {
                children.retain(|&c| c != id);
            }
            // If the closed modal was focused, restore focus to its parent
            if self.focused_window == Some(id) {
                self.focused_window = Some(parent_id);
            }
        } else if self.focused_window == Some(id) {
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
