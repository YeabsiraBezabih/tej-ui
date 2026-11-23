//! GPU cluster descriptor ring and VRAM slot recycling pool.

use std::collections::HashMap;

use super::types::GpuBufferAllocation;

/// Fixed-size GPU descriptor ring and VRAM slot recycling pool.
///
/// Pre-allocates a fixed number of 4 KiB GPU buffer slots (e.g., 4,096 slots = 16 MiB VRAM)
/// and recycles evicted slots in $O(1)$ time to prevent memory fragmentation and GC pauses.
#[derive(Debug, Clone)]
pub struct GpuMeshletRing {
    max_slots: u32,
    free_slots: Vec<u32>,
    resident_slots: HashMap<u32, GpuBufferAllocation>,
}

impl GpuMeshletRing {
    /// Creates a new GPU meshlet ring with a maximum slot capacity.
    #[must_use]
    pub fn new(max_slots: u32) -> Self {
        let mut free_slots = Vec::with_capacity(max_slots as usize);
        for i in (0..max_slots).rev() {
            free_slots.push(i);
        }
        Self {
            max_slots,
            free_slots,
            resident_slots: HashMap::with_capacity(max_slots as usize),
        }
    }

    /// Allocates a resident GPU slot for a cluster.
    pub fn allocate(
        &mut self,
        cluster_id: u32,
        vertex_count: u16,
        triangle_count: u16,
        lod_level: u8,
    ) -> Option<GpuBufferAllocation> {
        if let Some(existing) = self.resident_slots.get(&cluster_id) {
            return Some(*existing);
        }

        let slot_index = self.free_slots.pop()?;
        let byte_offset = slot_index * 4096;
        let alloc = GpuBufferAllocation::new(
            slot_index,
            byte_offset,
            vertex_count,
            triangle_count,
            lod_level,
        );

        self.resident_slots.insert(cluster_id, alloc);
        Some(alloc)
    }

    /// Evicts a cluster from GPU memory, recycling its slot index into the free list.
    pub fn evict(&mut self, cluster_id: u32) -> Option<GpuBufferAllocation> {
        let alloc = self.resident_slots.remove(&cluster_id)?;
        self.free_slots.push(alloc.slot_index);
        Some(alloc)
    }

    /// Returns the maximum slot capacity.
    #[must_use]
    pub const fn max_slots(&self) -> u32 {
        self.max_slots
    }

    /// Returns the allocation descriptor for a cluster if resident in GPU memory.
    #[must_use]
    pub fn get(&self, cluster_id: u32) -> Option<&GpuBufferAllocation> {
        self.resident_slots.get(&cluster_id)
    }

    /// Returns `true` if a cluster is currently resident in GPU memory.
    #[must_use]
    pub fn is_resident(&self, cluster_id: u32) -> bool {
        self.resident_slots.contains_key(&cluster_id)
    }

    /// Returns the number of currently resident clusters.
    #[must_use]
    pub fn resident_count(&self) -> usize {
        self.resident_slots.len()
    }

    /// Returns the total resident VRAM footprint in bytes.
    #[must_use]
    pub fn resident_vram_bytes(&self) -> usize {
        self.resident_slots.len() * 4096
    }

    /// Returns an iterator over all currently resident allocations.
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &GpuBufferAllocation)> {
        self.resident_slots.iter()
    }
}
