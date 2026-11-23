//! Flat Structure-of-Arrays (SoA) Transform Hierarchy for cache-friendly scene updates.
//!
//! Eliminates pointer chasing, heap allocations, and recursive tree traversals.
//! Updates hundreds of thousands of node matrices in a single contiguous SIMD-friendly loop.

use crate::math::{Mat4, Vec3};

/// Constant sentinel denoting a root node without a parent.
pub const NO_PARENT: u32 = u32::MAX;

/// Flat Structure-of-Arrays scene graph hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneGraphSoA {
    /// Parent index for each node (`NO_PARENT` for root nodes).
    pub parent_ids: Vec<u32>,
    /// Local translation vectors.
    pub local_translations: Vec<Vec3>,
    /// Local rotation quaternions `[x, y, z, w]`.
    pub local_rotations: Vec<[f32; 4]>,
    /// Local scale factors.
    pub local_scales: Vec<Vec3>,
    /// Cached world transformation matrices.
    pub world_matrices: Vec<Mat4>,
    /// Dirty flags tracking modified local transforms.
    pub dirty_bits: Vec<bool>,
    /// Optional mesh/model handle bound to this node.
    pub mesh_bindings: Vec<Option<u32>>,
    /// Node debug names.
    pub names: Vec<String>,
}

impl Default for SceneGraphSoA {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneGraphSoA {
    /// Creates an empty SoA scene graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parent_ids: Vec::new(),
            local_translations: Vec::new(),
            local_rotations: Vec::new(),
            local_scales: Vec::new(),
            world_matrices: Vec::new(),
            dirty_bits: Vec::new(),
            mesh_bindings: Vec::new(),
            names: Vec::new(),
        }
    }

    /// Number of nodes in the scene graph.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.parent_ids.len()
    }

    /// Returns `true` if the scene graph contains no nodes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parent_ids.is_empty()
    }

    /// Adds a new node to the hierarchy and returns its `u32` index.
    pub fn add_node(
        &mut self,
        name: &str,
        parent: Option<u32>,
        translation: Vec3,
        rotation: [f32; 4],
        scale: Vec3,
    ) -> u32 {
        #[allow(clippy::cast_possible_truncation)]
        let id = self.parent_ids.len() as u32;
        let parent_id = parent.unwrap_or(NO_PARENT);

        self.parent_ids.push(parent_id);
        self.local_translations.push(translation);
        self.local_rotations.push(rotation);
        self.local_scales.push(scale);
        self.world_matrices.push(Mat4::IDENTITY);
        self.dirty_bits.push(true);
        self.mesh_bindings.push(None);
        self.names.push(name.to_string());

        id
    }

    /// Binds a mesh or meshlet cluster ID to a node.
    pub fn bind_mesh(&mut self, node_id: u32, mesh_id: u32) {
        if let Some(binding) = self.mesh_bindings.get_mut(node_id as usize) {
            *binding = Some(mesh_id);
        }
    }

    /// Sets the local translation, rotation, and scale of a node and marks it dirty.
    pub fn set_local_transform(
        &mut self,
        node_id: u32,
        translation: Vec3,
        rotation: [f32; 4],
        scale: Vec3,
    ) {
        let idx = node_id as usize;
        if idx < self.parent_ids.len() {
            self.local_translations[idx] = translation;
            self.local_rotations[idx] = rotation;
            self.local_scales[idx] = scale;
            self.dirty_bits[idx] = true;
        }
    }

    /// Marks a node and all of its descendants as dirty.
    pub fn mark_dirty(&mut self, node_id: u32) {
        let idx = node_id as usize;
        if idx < self.dirty_bits.len() {
            self.dirty_bits[idx] = true;
        }
    }

    /// Evaluates and updates world matrices in a single contiguous top-down pass.
    pub fn update_world_transforms(&mut self) {
        let count = self.parent_ids.len();

        for i in 0..count {
            let parent = self.parent_ids[i];
            let parent_dirty = if parent != NO_PARENT {
                self.dirty_bits[parent as usize]
            } else {
                false
            };

            let is_dirty = self.dirty_bits[i] || parent_dirty;
            if is_dirty {
                self.dirty_bits[i] = true;

                let t = self.local_translations[i];
                let r = self.local_rotations[i];
                let s = self.local_scales[i];
                let local_mat = Self::compute_local_matrix(t, r, s);

                if parent == NO_PARENT {
                    self.world_matrices[i] = local_mat;
                } else {
                    let parent_world = self.world_matrices[parent as usize];
                    self.world_matrices[i] = parent_world.mul_mat(&local_mat);
                }
            }
        }

        // Clear all dirty bits after world matrix evaluation
        self.dirty_bits.fill(false);
    }

    /// Computes a local transformation matrix $T \cdot R \cdot S$ from components.
    #[must_use]
    pub fn compute_local_matrix(translation: Vec3, rotation: [f32; 4], scale: Vec3) -> Mat4 {
        let rot_mat = Self::quat_to_mat4(rotation);

        Mat4 {
            cols: [
                [
                    rot_mat.cols[0][0] * scale.x,
                    rot_mat.cols[0][1] * scale.x,
                    rot_mat.cols[0][2] * scale.x,
                    0.0,
                ],
                [
                    rot_mat.cols[1][0] * scale.y,
                    rot_mat.cols[1][1] * scale.y,
                    rot_mat.cols[1][2] * scale.y,
                    0.0,
                ],
                [
                    rot_mat.cols[2][0] * scale.z,
                    rot_mat.cols[2][1] * scale.z,
                    rot_mat.cols[2][2] * scale.z,
                    0.0,
                ],
                [translation.x, translation.y, translation.z, 1.0],
            ],
        }
    }

    /// Converts a normalized unit quaternion `[x, y, z, w]` into a 4x4 rotation matrix.
    #[must_use]
    pub fn quat_to_mat4(q: [f32; 4]) -> Mat4 {
        let [x, y, z, w] = q;

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;

        Mat4 {
            cols: [
                [1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0],
                [2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0],
                [2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Retrieves the evaluated world matrix for a node.
    #[must_use]
    pub fn world_matrix(&self, node_id: u32) -> Option<&Mat4> {
        self.world_matrices.get(node_id as usize)
    }
}
