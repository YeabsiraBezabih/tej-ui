//! 3D Perspective camera and screen-to-world raycasting for UI interaction.

use crate::math::{Mat4, Vec3};
use tej_ui_layout::Point;

/// 3D Perspective camera matching Three.js `PerspectiveCamera`.
#[derive(Debug, Clone, PartialEq)]
pub struct PerspectiveCamera {
    /// Field of view in radians (vertical).
    pub fov_y: f32,
    /// Aspect ratio (width / height).
    pub aspect: f32,
    /// Near clipping plane.
    pub near: f32,
    /// Far clipping plane.
    pub far: f32,
    /// Camera world position.
    pub position: Vec3,
    /// Look-at target.
    pub target: Vec3,
    /// Camera up vector.
    pub up: Vec3,
    /// Whether to use Reversed-Z projection (near -> 1.0, far -> 0.0) for infinite precision.
    pub reversed_z: bool,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            fov_y: 45.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            position: Vec3::new(0.0, 2.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::UP,
            reversed_z: true,
        }
    }
}

impl PerspectiveCamera {
    /// Creates a new perspective camera.
    #[must_use]
    pub fn new(fov_degrees: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov_y: fov_degrees.to_radians(),
            aspect,
            near,
            far,
            ..Default::default()
        }
    }

    /// Computes view matrix.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    /// Computes perspective projection matrix.
    #[must_use]
    pub fn projection_matrix(&self) -> Mat4 {
        if self.reversed_z {
            Mat4::perspective_reversed_z(self.fov_y, self.aspect, self.near, self.far)
        } else {
            Mat4::perspective(self.fov_y, self.aspect, self.near, self.far)
        }
    }

    /// Computes combined View-Projection matrix ($M_{VP} = M_P \times M_V$).
    #[must_use]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix().mul_mat(&self.view_matrix())
    }

    /// Generates a world-space Ray from a 2D screen coordinate.
    #[must_use]
    pub fn screen_to_ray(
        &self,
        screen_pt: Point,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Ray {
        // Normalize screen point to Normalized Device Coordinates (NDC) [-1, 1]
        let ndc_x = (2.0 * screen_pt.x / viewport_width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pt.y / viewport_height);

        let tan_half_fov = (self.fov_y * 0.5).tan();
        let ray_dir_cam = Vec3::new(
            ndc_x * tan_half_fov * self.aspect,
            ndc_y * tan_half_fov,
            -1.0,
        )
        .normalize();

        // Transform direction by camera orientation
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let true_up = right.cross(forward);

        let world_dir =
            (right * ray_dir_cam.x + true_up * ray_dir_cam.y + forward * (-ray_dir_cam.z))
                .normalize();

        Ray {
            origin: self.position,
            direction: world_dir,
        }
    }
}

/// 3D Ray for collision detection and UI hit-testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Tests intersection with a bounding sphere at center with radius.
    ///
    /// Returns the hit distance $t \ge 0$ along the ray, if any.
    #[must_use]
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) * 0.5;
            if t >= 0.0 { Some(t) } else { None }
        }
    }
}
