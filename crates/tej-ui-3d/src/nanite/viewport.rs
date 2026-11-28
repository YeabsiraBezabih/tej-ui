//! High-level Nanite rendering engine coordinating the 120 FPS frame pipeline.

use std::time::Instant;

use mydb::spatial_viewport::{ClusterMetadata, CullResult, SpatialViewportManager};
use mydb::{FrustumPlanes, RaycastScope, Storage};
use mydb_core::PageId;

use super::gizmo::NaniteGizmoManager;
use super::picking::screen_point_to_ray;
use super::streamer::{GpuStagingUploader, NaniteStreamer};
use super::types::NaniteFrameStats;

/// High-level zero-copy Nanite & Gaussian Splat rendering engine.
///
/// Connects `MyDb` spatial culling directly to GPU streaming rings to hit 120 FPS
/// with out-of-core geometry streaming.
#[derive(Debug)]
pub struct NaniteRenderEngine<S: Storage> {
    viewport: SpatialViewportManager<S>,
    streamer: NaniteStreamer,
    gizmo: NaniteGizmoManager,
    last_stats: NaniteFrameStats,
}

impl<S: Storage> NaniteRenderEngine<S> {
    /// Creates a new Nanite render engine with a spatial viewport and maximum GPU slot capacity.
    #[must_use]
    pub fn new(viewport: SpatialViewportManager<S>, max_gpu_slots: u32) -> Self {
        Self {
            viewport,
            streamer: NaniteStreamer::new(max_gpu_slots),
            gizmo: NaniteGizmoManager::new(),
            last_stats: NaniteFrameStats::default(),
        }
    }

    /// Registers a cluster with its database page ID for zero-copy streaming.
    pub fn register_cluster(
        &mut self,
        meta: ClusterMetadata,
        page_id: PageId,
    ) -> Result<(), mydb_core::DbError> {
        self.streamer
            .register_cluster_page(meta.cluster_id, page_id);
        self.viewport.register_cluster(meta)
    }

    /// Stores a 64-byte aligned meshlet page directly into `MyDb` and registers its cluster ID.
    pub fn store_meshlet(
        &mut self,
        cluster_id: u32,
        header: &mydb_core::spatial::MeshletPageHeader,
        payload: &[u8],
    ) -> Result<PageId, mydb_core::DbError> {
        let page_id = self
            .viewport
            .store_meshlet_page(cluster_id, header, payload)?;
        self.streamer.register_cluster_page(cluster_id, page_id);
        Ok(page_id)
    }

    /// Mutable reference to the underlying spatial viewport.
    pub fn viewport_mut(&mut self) -> &mut SpatialViewportManager<S> {
        &mut self.viewport
    }

    /// Mutable reference to the underlying streamer.
    pub fn streamer_mut(&mut self) -> &mut NaniteStreamer {
        &mut self.streamer
    }

    /// Differential streaming into an arbitrary staging uploader.
    pub fn stream_with_uploader<U: GpuStagingUploader>(
        &mut self,
        cull_result: &CullResult,
        uploader: &mut U,
        stats: &mut NaniteFrameStats,
    ) {
        self.streamer
            .stream_differential(&self.viewport, cull_result, uploader, stats);
    }

    /// Step 1 & 2 of the frame loop: Executes SIMD culling and differential GPU staging upload.
    pub fn render_frame_differential<U: GpuStagingUploader>(
        &mut self,
        vp_matrix: &[f32; 16],
        camera_pos: [f32; 3],
        max_screen_error: f32,
        uploader: &mut U,
    ) -> &NaniteFrameStats {
        let frustum = FrustumPlanes::from_matrix(vp_matrix);
        self.render_frame_with_frustum(&frustum, camera_pos, max_screen_error, uploader)
    }

    /// Step 1 & 2 of the frame loop using explicit camera frustum planes.
    pub fn render_frame_with_frustum<U: GpuStagingUploader>(
        &mut self,
        frustum: &FrustumPlanes,
        camera_pos: [f32; 3],
        max_screen_error: f32,
        uploader: &mut U,
    ) -> &NaniteFrameStats {
        let mut stats = NaniteFrameStats::default();

        // 1. SIMD Frustum & LOD Culling (<0.35 ms)
        let cull_start = Instant::now();
        let cull_result: CullResult =
            self.viewport
                .on_camera_move(frustum, camera_pos, max_screen_error);
        stats.culling_time_us = cull_start.elapsed().as_secs_f64() * 1_000_000.0;
        stats.clusters_visible = cull_result.visible_clusters.len() as usize;

        // 2. Zero-Copy Differential GPU Staging Upload (<0.20 ms)
        self.streamer
            .stream_differential(&self.viewport, &cull_result, uploader, &mut stats);

        self.last_stats = stats;
        &self.last_stats
    }

    /// Step 4 of the frame loop: Branch-free pointer raycast picking (<15 µs).
    pub fn handle_pointer_click(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        viewport_width: f32,
        viewport_height: f32,
        inv_vp_matrix: &[f32; 16],
        camera_pos: [f32; 3],
    ) -> Option<u32> {
        let ray = screen_point_to_ray(
            screen_x,
            screen_y,
            viewport_width,
            viewport_height,
            inv_vp_matrix,
            camera_pos,
        );

        let hit = self.viewport.pick_ray(&ray, RaycastScope::VisibleOnly);
        if let Some((cluster_id, _dist)) = hit {
            self.gizmo.select(cluster_id);
            Some(cluster_id)
        } else {
            self.gizmo.deselect();
            None
        }
    }

    /// Step 5: User drags a 3D gizmo to translate the selected cluster.
    pub fn on_gizmo_drag(&mut self, delta_xyz: [f32; 3]) -> Result<(), mydb_core::DbError> {
        self.gizmo.apply_translation(&mut self.viewport, delta_xyz)
    }

    /// Reverts the last transform operation in $O(1)$ time via database snapshots.
    pub fn on_user_undo(&mut self) -> bool {
        self.gizmo.undo(&mut self.viewport)
    }

    /// Re-applies the last undone transform operation in $O(1)$ time.
    pub fn on_user_redo(&mut self) -> bool {
        self.gizmo.redo(&mut self.viewport)
    }

    /// Returns the statistics from the last rendered frame.
    #[must_use]
    pub const fn last_stats(&self) -> &NaniteFrameStats {
        &self.last_stats
    }

    /// Returns a reference to the active gizmo manager.
    #[must_use]
    pub const fn gizmo(&self) -> &NaniteGizmoManager {
        &self.gizmo
    }

    /// Returns a reference to the underlying spatial viewport.
    #[must_use]
    pub const fn viewport(&self) -> &SpatialViewportManager<S> {
        &self.viewport
    }
}
