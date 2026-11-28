//! Common types and descriptors for the Nanite and Gaussian Splat rendering pipeline.

use std::fmt;

/// Representation of a GPU buffer allocation for a resident meshlet cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuBufferAllocation {
    /// Slot index within the GPU vertex/index descriptor ring.
    pub slot_index: u32,
    /// Byte offset within the GPU resident storage buffer.
    pub byte_offset: u32,
    /// Number of vertices stored in this allocation.
    pub vertex_count: u16,
    /// Number of triangles stored in this allocation.
    pub triangle_count: u16,
    /// Level-of-Detail (LOD) level.
    pub lod_level: u8,
}

impl GpuBufferAllocation {
    /// Creates a new GPU buffer allocation descriptor.
    #[must_use]
    pub const fn new(
        slot_index: u32,
        byte_offset: u32,
        vertex_count: u16,
        triangle_count: u16,
        lod_level: u8,
    ) -> Self {
        Self {
            slot_index,
            byte_offset,
            vertex_count,
            triangle_count,
            lod_level,
        }
    }

    /// Releases or marks this allocation as freed.
    pub fn free(&self) {
        // Slot recycling logic handled by GpuMeshletRing
    }
}

/// Statistics collected for each rendered frame in the 120 FPS frame loop.
#[derive(Debug, Clone, Copy, Default)]
pub struct NaniteFrameStats {
    /// Number of clusters tested against camera frustum.
    pub clusters_tested: usize,
    /// Number of clusters determined visible in this frame.
    pub clusters_visible: usize,
    /// Differential number of clusters newly uploaded to GPU staging.
    pub clusters_uploaded: usize,
    /// Differential number of clusters evicted from GPU VRAM.
    pub clusters_evicted: usize,
    /// Time spent in camera frustum and LOD culling in microseconds.
    pub culling_time_us: f64,
    /// Time spent in differential GPU staging upload in microseconds.
    pub staging_upload_time_us: f64,
    /// Total VRAM consumed by active resident clusters in bytes.
    pub resident_vram_bytes: usize,
}

impl fmt::Display for NaniteFrameStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Visible: {} | +Uploaded: {} | -Evicted: {} | Culling: {:.2}µs | Staging: {:.2}µs | VRAM: {:.2} MB",
            self.clusters_visible,
            self.clusters_uploaded,
            self.clusters_evicted,
            self.culling_time_us,
            self.staging_upload_time_us,
            f64::from(u32::try_from(self.resident_vram_bytes / 1024).unwrap_or(u32::MAX)) / 1024.0,
        )
    }
}
