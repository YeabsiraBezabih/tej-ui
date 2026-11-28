//! Keyboard focus management, Tab-traversal order, and active focus rings.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use tej_ui_layout::Rect;

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

/// Direction for linear sequential Tab focus traversal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FocusDirection {
    Next,
    Previous,
}

/// 2D spatial directional arrow traversal direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpatialDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Central focus manager orchestrating keyboard focus navigation across the platform window.
#[derive(Clone, Debug, Default)]
pub struct FocusManager {
    /// Currently focused widget ID, or `None` if unfocused.
    current_focus: Option<FocusId>,
    /// Ordered list of focusable elements in Tab traversal sequence.
    focus_order: Vec<FocusId>,
    /// Stack of active modal focus trap scopes.
    trap_stack: Vec<Vec<FocusId>>,
    /// Registered 2D bounding boxes for spatial arrow navigation.
    spatial_bounds: HashMap<FocusId, Rect>,
}

impl FocusManager {
    /// Create a new focus manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_focus: None,
            focus_order: Vec::new(),
            trap_stack: Vec::new(),
            spatial_bounds: HashMap::new(),
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

    /// Register spatial layout bounds for 2D directional arrow navigation.
    pub fn register_spatial_bounds(&mut self, id: FocusId, bounds: Rect) {
        self.spatial_bounds.insert(id, bounds);
    }

    /// Unregister a widget when it unmounts from the view tree.
    pub fn unregister_target(&mut self, id: FocusId) {
        self.focus_order.retain(|&x| x != id);
        self.spatial_bounds.remove(&id);
        if self.current_focus == Some(id) {
            self.current_focus = None;
        }
    }

    /// Pushes a modal focus trap scope, restricting Tab traversal strictly to `allowed` elements.
    pub fn push_trap_scope(&mut self, allowed: Vec<FocusId>) {
        if let Some(&first) = allowed.first()
            && !allowed.contains(&self.current_focus.unwrap_or(FocusId(0)))
        {
            self.current_focus = Some(first);
        }
        self.trap_stack.push(allowed);
    }

    /// Pops the active modal focus trap scope, returning to parent focus context.
    pub fn pop_trap_scope(&mut self) -> Option<Vec<FocusId>> {
        self.trap_stack.pop()
    }

    /// Returns `true` if an active modal focus trap scope is currently restricting navigation.
    #[must_use]
    pub fn is_trapped(&self) -> bool {
        !self.trap_stack.is_empty()
    }

    /// Returns the currently active focus order, taking active trap scopes into account.
    #[must_use]
    pub fn active_order(&self) -> &[FocusId] {
        if let Some(top_trap) = self.trap_stack.last() {
            top_trap.as_slice()
        } else {
            self.focus_order.as_slice()
        }
    }

    /// Move focus to next/previous element according to Tab / Shift-Tab direction.
    pub fn traverse(&mut self, direction: FocusDirection) -> Option<FocusId> {
        let order = self.active_order();
        if order.is_empty() {
            return None;
        }

        let next_idx = match self.current_focus {
            Some(curr_id) => {
                if let Some(pos) = order.iter().position(|&x| x == curr_id) {
                    match direction {
                        FocusDirection::Next => (pos + 1) % order.len(),
                        FocusDirection::Previous => {
                            if pos == 0 {
                                order.len() - 1
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
                FocusDirection::Previous => order.len() - 1,
            },
        };

        let next_id = order[next_idx];
        self.current_focus = Some(next_id);
        Some(next_id)
    }

    /// Navigates focus in 2D space using spatial arrow keys (Up, Down, Left, Right).
    pub fn navigate_spatial(&mut self, direction: SpatialDirection) -> Option<FocusId> {
        let curr_id = self.current_focus?;
        let curr_rect = self.spatial_bounds.get(&curr_id)?;

        let curr_center_x = curr_rect.origin.x + curr_rect.size.width * 0.5;
        let curr_center_y = curr_rect.origin.y + curr_rect.size.height * 0.5;

        let mut best_candidate: Option<(FocusId, f32)> = None;

        for (&candidate_id, &cand_rect) in &self.spatial_bounds {
            if candidate_id == curr_id {
                continue;
            }

            let cand_center_x = cand_rect.origin.x + cand_rect.size.width * 0.5;
            let cand_center_y = cand_rect.origin.y + cand_rect.size.height * 0.5;

            let dx = cand_center_x - curr_center_x;
            let dy = cand_center_y - curr_center_y;

            let (primary_dist, ortho_dist) = match direction {
                SpatialDirection::Right => (dx, dy.abs()),
                SpatialDirection::Left => (-dx, dy.abs()),
                SpatialDirection::Down => (dy, dx.abs()),
                SpatialDirection::Up => (-dy, dx.abs()),
            };

            if primary_dist > 0.0 {
                // Heavily penalize orthogonal offset to prioritize intuitive axial navigation
                let score = primary_dist + ortho_dist * 4.0;
                match best_candidate {
                    Some((_, best_score)) if score < best_score => {
                        best_candidate = Some((candidate_id, score));
                    }
                    None => {
                        best_candidate = Some((candidate_id, score));
                    }
                    _ => {}
                }
            }
        }

        if let Some((best_id, _)) = best_candidate {
            self.current_focus = Some(best_id);
            Some(best_id)
        } else {
            None
        }
    }
}
