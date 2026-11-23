//! Zero-allocation storage buffer pool and draw batch coalescing.

use crate::gpu::types::{QuadInstance, ScissorRect};
use core::ops::Range;

/// A contiguous range of quad instances sharing an identical hardware scissor box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawBatch {
    pub instance_range: Range<u32>,
    pub scissor: ScissorRect,
}

/// Dynamic storage buffer pool providing zero steady-state heap allocations.
#[derive(Debug)]
pub struct InstanceBufferPool {
    cpu_instances: Vec<QuadInstance>,
    batches: Vec<DrawBatch>,
    gpu_buffer: wgpu::Buffer,
    gpu_capacity: usize,
}

impl InstanceBufferPool {
    /// Constructs a new instance pool with pre-allocated CPU and GPU storage.
    #[must_use]
    pub fn new(device: &wgpu::Device, initial_capacity: usize) -> Self {
        let capacity = initial_capacity.max(64).next_power_of_two();
        let buffer_size = (capacity * core::mem::size_of::<QuadInstance>()) as u64;

        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_instance_storage_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            cpu_instances: Vec::with_capacity(capacity),
            batches: Vec::with_capacity(64),
            gpu_buffer,
            gpu_capacity: capacity,
        }
    }

    /// Prepares pool for a new frame by clearing collections without dropping capacity.
    pub fn begin_frame(&mut self) {
        self.cpu_instances.clear();
        self.batches.clear();
    }

    /// Appends a quad instance, coalescing adjacent quads into the active batch if scissors match.
    pub fn push_instance(&mut self, instance: QuadInstance, scissor: ScissorRect) {
        let current_index = self.cpu_instances.len() as u32;
        self.cpu_instances.push(instance);

        if let Some(last_batch) = self.batches.last_mut() {
            if last_batch.scissor == scissor && last_batch.instance_range.end == current_index {
                last_batch.instance_range.end = current_index + 1;
                return;
            }
        }

        self.batches.push(DrawBatch {
            instance_range: current_index..current_index + 1,
            scissor,
        });
    }

    /// Uploads CPU instances to the GPU storage buffer.
    ///
    /// Reallocates GPU buffer to next power of two ONLY if capacity is exceeded.
    /// Returns `true` if GPU buffer was reallocated (bind group must be refreshed),
    /// or `false` in steady state.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.cpu_instances.is_empty() {
            return false;
        }

        let needed_capacity = self.cpu_instances.len();
        let mut reallocated = false;

        if needed_capacity > self.gpu_capacity {
            let new_capacity = needed_capacity.next_power_of_two();
            let new_size = (new_capacity * core::mem::size_of::<QuadInstance>()) as u64;

            self.gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("tej_instance_storage_buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.gpu_capacity = new_capacity;
            reallocated = true;
        }

        let bytes = bytemuck::cast_slice(&self.cpu_instances);
        queue.write_buffer(&self.gpu_buffer, 0, bytes);

        reallocated
    }

    #[must_use]
    pub fn batches(&self) -> &[DrawBatch] {
        &self.batches
    }

    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }

    #[must_use]
    pub fn instance_count(&self) -> usize {
        self.cpu_instances.len()
    }

    #[must_use]
    pub fn cpu_capacity(&self) -> usize {
        self.cpu_instances.capacity()
    }

    #[must_use]
    pub fn gpu_capacity(&self) -> usize {
        self.gpu_capacity
    }
}
