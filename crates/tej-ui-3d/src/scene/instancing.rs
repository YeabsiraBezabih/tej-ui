//! High-Throughput InstancedMesh for rendering 100,000+ repeated objects from 1 geometry buffer.
//!
//! Stores instance transforms in cache-friendly contiguous tables compatible with `MyDb` HTAP scans.

use crate::math::{Mat4, Vec3};
use crate::scene::soa_graph::SceneGraphSoA;
use tej_ui_hal::Color;

/// 64-Byte cache-aligned instance record for a single instanced mesh entity.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceData {
    pub translation: Vec3,
    pub rotation: [f32; 4], // Quaternion [x, y, z, w]
    pub scale: Vec3,
    pub color_tint: Color,
    pub user_tag: u32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: Vec3::ONE,
            color_tint: Color::WHITE,
            user_tag: 0,
        }
    }
}

impl InstanceData {
    /// Creates a new instance record.
    #[must_use]
    pub const fn new(
        translation: Vec3,
        rotation: [f32; 4],
        scale: Vec3,
        color_tint: Color,
        user_tag: u32,
    ) -> Self {
        Self {
            translation,
            rotation,
            scale,
            color_tint,
            user_tag,
        }
    }

    /// Computes the 4x4 world transformation matrix for this instance.
    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        SceneGraphSoA::compute_local_matrix(self.translation, self.rotation, self.scale)
    }
}

/// Collection of instance transforms referencing a single shared geometry buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct InstancedMesh {
    pub mesh_id: u32,
    pub instances: Vec<InstanceData>,
}

impl InstancedMesh {
    /// Creates a new instanced mesh group with pre-allocated capacity.
    #[must_use]
    pub fn new(mesh_id: u32, capacity: usize) -> Self {
        Self {
            mesh_id,
            instances: Vec::with_capacity(capacity),
        }
    }

    /// Total number of active instances.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Returns `true` if there are no instances.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Appends a new instance to the group.
    pub fn add_instance(&mut self, instance: InstanceData) {
        self.instances.push(instance);
    }

    /// Computes world matrices for all instances in a single contiguous loop.
    #[must_use]
    pub fn compute_world_matrices(&self) -> Vec<Mat4> {
        self.instances.iter().map(InstanceData::matrix).collect()
    }
}
