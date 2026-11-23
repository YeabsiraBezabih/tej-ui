//! Zero-copy differential GPU staging streamer for Nanite and Gaussian Splats.

use std::collections::HashMap;
use std::time::Instant;

use mydb::Storage;
use mydb::spatial_viewport::{CullResult, SpatialViewportManager};
use mydb_core::PageId;
use mydb_core::spatial::MeshletPageHeader;

use super::meshlet_cache::GpuMeshletRing;
use super::types::NaniteFrameStats;

/// Trait abstracting GPU staging and resident buffer transfers.
///
/// Implementable by `wgpu::Queue`, native Vulkan command buffers, Metal blit encoders,
/// or in-memory simulation rings.
pub trait GpuStagingUploader {
    /// Transfers a zero-copy 4 KiB meshlet payload directly into GPU resident memory.
    fn upload_meshlet(
        &mut self,
        slot_index: u32,
        payload: &[u8],
        vertex_count: u16,
        triangle_count: u16,
    );

    /// Releases or marks a resident slot as evicted.
    fn evict_meshlet(&mut self, slot_index: u32);
}

/// In-memory simulated GPU uploader for headless environments and verification tests.
#[derive(Debug, Default)]
pub struct MockGpuUploader {
    pub uploaded_slots: HashMap<u32, Vec<u8>>,
}

impl GpuStagingUploader for MockGpuUploader {
    fn upload_meshlet(
        &mut self,
        slot_index: u32,
        payload: &[u8],
        _vertex_count: u16,
        _triangle_count: u16,
    ) {
        self.uploaded_slots.insert(slot_index, payload.to_vec());
    }

    fn evict_meshlet(&mut self, slot_index: u32) {
        self.uploaded_slots.remove(&slot_index);
    }
}

/// Coordinates zero-copy differential streaming from `MyDb` pages into GPU VRAM.
#[derive(Debug)]
pub struct NaniteStreamer {
    ring: GpuMeshletRing,
    cluster_page_map: HashMap<u32, PageId>,
}

impl NaniteStreamer {
    /// Creates a new Nanite streamer with a maximum GPU resident slot count.
    #[must_use]
    pub fn new(max_resident_slots: u32) -> Self {
        Self {
            ring: GpuMeshletRing::new(max_resident_slots),
            cluster_page_map: HashMap::new(),
        }
    }

    /// Registers the mapping between a cluster ID and its 4 KiB database `PageId`.
    pub fn register_cluster_page(&mut self, cluster_id: u32, page_id: PageId) {
        self.cluster_page_map.insert(cluster_id, page_id);
    }

    /// Executes the differential GPU streaming step for the current frame.
    ///
    /// - Evicts clusters in `cull_result.to_evict`.
    /// - Uploads clusters in `cull_result.to_upload` zero-copy via `with_meshlet_bytes`.
    pub fn stream_differential<S: Storage, U: GpuStagingUploader>(
        &mut self,
        viewport: &SpatialViewportManager<S>,
        cull_result: &CullResult,
        uploader: &mut U,
        stats: &mut NaniteFrameStats,
    ) {
        let staging_start = Instant::now();

        // 1. Process evictions
        for &cluster_id in &cull_result.to_evict {
            if let Some(alloc) = self.ring.evict(cluster_id) {
                uploader.evict_meshlet(alloc.slot_index);
            }
        }
        stats.clusters_evicted = cull_result.to_evict.len();

        // 2. Process zero-copy staging uploads
        let mut uploaded_count = 0;
        for &cluster_id in &cull_result.to_upload {
            let Some(&page_id) = self.cluster_page_map.get(&cluster_id) else {
                continue;
            };

            let _ = viewport.with_meshlet_bytes(page_id, |page_bytes| {
                let Ok(header) = MeshletPageHeader::from_page_bytes(page_bytes) else {
                    return;
                };
                let payload = MeshletPageHeader::payload_slice(page_bytes);

                if let Some(alloc) = self.ring.allocate(
                    cluster_id,
                    header.vertex_count,
                    header.triangle_count,
                    header.lod_level,
                ) {
                    uploader.upload_meshlet(
                        alloc.slot_index,
                        payload,
                        header.vertex_count,
                        header.triangle_count,
                    );
                    uploaded_count += 1;
                }
            });
        }

        stats.clusters_uploaded = uploaded_count;
        stats.resident_vram_bytes = self.ring.resident_vram_bytes();
        stats.staging_upload_time_us = staging_start.elapsed().as_micros() as f64;
    }

    /// Accessor for the underlying GPU descriptor ring.
    #[must_use]
    pub const fn ring(&self) -> &GpuMeshletRing {
        &self.ring
    }
}
