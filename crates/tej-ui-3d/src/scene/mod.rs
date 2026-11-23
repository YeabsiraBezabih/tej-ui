//! 3D Scene graph hierarchy, objects, and local transforms.

pub mod instancing;
pub mod soa_graph;

use crate::material::{Light, StandardMaterial};
use crate::math::{Mat4, Vec3};
use crate::mesh::Mesh;
pub use instancing::{InstanceData, InstancedMesh};
pub use soa_graph::{NO_PARENT, SceneGraphSoA};

/// 3D affine transform (position, rotation euler, scaling).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles in radians (X, Y, Z)
    pub scale: Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

impl Transform3D {
    /// Computes local 4x4 model matrix $M = T \times R_y \times R_x \times S$.
    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        let t = Mat4::translation(self.position);
        let ry = Mat4::rotation_y(self.rotation.y);
        let rx = Mat4::rotation_x(self.rotation.x);
        let s = Mat4::scale(self.scale);

        t.mul_mat(&ry).mul_mat(&rx).mul_mat(&s)
    }
}

/// An entity in the 3D scene containing geometry, transform, and material.
#[derive(Debug, Clone, PartialEq)]
pub struct Object3D {
    pub id: u64,
    pub name: String,
    pub mesh: Mesh,
    pub transform: Transform3D,
    pub material: StandardMaterial,
}

impl Object3D {
    /// Creates a new 3D object.
    #[must_use]
    pub fn new(id: u64, name: impl Into<String>, mesh: Mesh, material: StandardMaterial) -> Self {
        Self {
            id,
            name: name.into(),
            mesh,
            transform: Transform3D::default(),
            material,
        }
    }
}

/// 3D Scene containing objects and illumination sources.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub objects: Vec<Object3D>,
    pub lights: Vec<Light>,
}

impl Scene {
    /// Creates an empty 3D scene.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a 3D object to the scene.
    pub fn add_object(&mut self, object: Object3D) {
        self.objects.push(object);
    }

    /// Adds a light source to the scene.
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }
}
