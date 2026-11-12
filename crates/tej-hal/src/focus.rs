//! Keyboard focus management, Tab-traversal order, and active focus rings.

use std::sync::atomic::{AtomicU32, Ordering};

/// Unique focus target identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FocusId(pub u32);

static NEXT_FOCUS_ID: AtomicU32 = AtomicU32::new(1);

impl FocusId {
    /// Generate a new globally unique focus target ID.
    #[must_use]
    pub fn next_unique() -> Self {
        Self(NEXT_FOCUS_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Direction for focus traversal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FocusDirection {
    Next,
    Previous,
}

/// Central focus manager orchestrating keyboard focus navigation across the platform window.
#[derive(Clone, Debug, Default)]
pub struct FocusManager {
    /// Currently focused widget ID, or `None` if unfocused.
    current_focus: Option<FocusId>,
    /// Ordered list of focusable elements in Tab traversal sequence.
    focus_order: Vec<FocusId>,
}

impl FocusManager {
    /// Create a new focus manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_focus: None,
            focus_order: Vec::new(),
        }
    }

    /// Currently focused target ID, if any.
    #[must_use]
    pub const fn current_focus(&self) -> Option<FocusId> {
        self.current_focus
    }

    /// Check if target `id` currently holds focus.
    #[must_use]
    pub fn is_focused(&self, id: FocusId) -> bool {
        self.current_focus == Some(id)
    }

    /// Explicitly request focus for target `id`.
    pub fn request_focus(&mut self, id: FocusId) {
        self.current_focus = Some(id);
    }

    /// Clear all active focus.
    pub fn clear_focus(&mut self) {
        self.current_focus = None;
    }

    /// Register a widget in the Tab traversal sequence.
    pub fn register_target(&mut self, id: FocusId) {
        if !self.focus_order.contains(&id) {
            self.focus_order.push(id);
        }
    }

    /// Unregister a widget when it unmounts from the view tree.
    pub fn unregister_target(&mut self, id: FocusId) {
        self.focus_order.retain(|&x| x != id);
        if self.current_focus == Some(id) {
            self.current_focus = None;
        }
    }

    /// Move focus to next/previous element according to Tab / Shift-Tab direction.
    pub fn traverse(&mut self, direction: FocusDirection) -> Option<FocusId> {
        if self.focus_order.is_empty() {
            return None;
        }

        let next_idx = match self.current_focus {
            Some(curr_id) => {
                if let Some(pos) = self.focus_order.iter().position(|&x| x == curr_id) {
                    match direction {
                        FocusDirection::Next => (pos + 1) % self.focus_order.len(),
                        FocusDirection::Previous => {
                            if pos == 0 {
                                self.focus_order.len() - 1
                            } else {
                                pos - 1
                            }
                        }
                    }
                } else {
                    0
                }
            }
            None => match direction {
                FocusDirection::Next => 0,
                FocusDirection::Previous => self.focus_order.len() - 1,
            },
        };

        let next_id = self.focus_order[next_idx];
        self.current_focus = Some(next_id);
        Some(next_id)
    }
}
