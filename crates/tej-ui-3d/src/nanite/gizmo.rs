//! 3D Transform Gizmo and instant ACID undo/redo interaction manager.

use mydb::spatial_viewport::SpatialViewportManager;
use mydb::{GizmoTransform, Storage};

/// Active manipulation mode for the 3D transform gizmo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// Manages interactive 3D transform gizmos and instant transactional undo/redo.
#[derive(Debug, Clone, Default)]
pub struct NaniteGizmoManager {
    selected_cluster_id: Option<u32>,
    mode: GizmoMode,
    is_dragging: bool,
    drag_start_pos: [f32; 3],
}

impl NaniteGizmoManager {
    /// Creates a new gizmo manager with no selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects a cluster for transformation.
    pub fn select(&mut self, cluster_id: u32) {
        self.selected_cluster_id = Some(cluster_id);
    }

    /// Clears the current cluster selection.
    pub fn deselect(&mut self) {
        self.selected_cluster_id = None;
        self.is_dragging = false;
    }

    /// Returns the currently selected cluster ID.
    #[must_use]
    pub const fn selected(&self) -> Option<u32> {
        self.selected_cluster_id
    }

    /// Starts a drag operation from a given world space position.
    pub fn start_drag(&mut self, pos: [f32; 3]) {
        self.is_dragging = true;
        self.drag_start_pos = pos;
    }

    /// Returns `true` if a drag operation is in progress.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Returns the initial position where dragging started.
    #[must_use]
    pub const fn drag_start_pos(&self) -> [f32; 3] {
        self.drag_start_pos
    }

    /// Sets the active gizmo transformation mode.
    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
    }

    /// Returns the active gizmo transformation mode.
    #[must_use]
    pub const fn mode(&self) -> GizmoMode {
        self.mode
    }

    /// Applies a delta translation to the selected cluster in the database.
    pub fn apply_translation<S: Storage>(
        &mut self,
        viewport: &mut SpatialViewportManager<S>,
        delta_xyz: [f32; 3],
    ) -> Result<(), mydb_core::DbError> {
        let Some(cluster_id) = self.selected_cluster_id else {
            return Ok(());
        };

        viewport.apply_transform(GizmoTransform {
            cluster_id,
            translation: delta_xyz,
        })?;
        Ok(())
    }

    /// Reverts the last transformation in $O(1)$ time via database snapshot isolation.
    pub fn undo<S: Storage>(&mut self, viewport: &mut SpatialViewportManager<S>) -> bool {
        viewport.undo().map(|opt| opt.is_some()).unwrap_or(false)
    }

    /// Re-applies the last undone transformation in $O(1)$ time.
    pub fn redo<S: Storage>(&mut self, viewport: &mut SpatialViewportManager<S>) -> bool {
        viewport.redo().map(|opt| opt.is_some()).unwrap_or(false)
    }
}
