//! Interactive OrbitControls for manipulating 3D cameras using mouse events.

use crate::camera::PerspectiveCamera;
use crate::math::Vec3;
use tej_hal::{MouseButton, UiEvent};
use tej_layout::Point;

/// Interactive 3D camera controller matching Three.js `OrbitControls`.
#[derive(Debug, Clone)]
pub struct OrbitControls {
    /// Distance from camera to target.
    pub radius: f32,
    /// Azimuth angle in radians (horizontal rotation).
    pub theta: f32,
    /// Elevation angle in radians (vertical pitch).
    pub phi: f32,
    /// Sensitivity for mouse dragging.
    pub rotate_speed: f32,
    /// Sensitivity for mouse scroll wheel zoom.
    pub zoom_speed: f32,
    /// Minimum zoom radius.
    pub min_radius: f32,
    /// Maximum zoom radius.
    pub max_radius: f32,
    is_dragging: bool,
    last_mouse_pos: Point,
}

impl Default for OrbitControls {
    fn default() -> Self {
        Self {
            radius: 5.0,
            theta: 0.0,
            phi: std::f32::consts::FRAC_PI_4, // 45 degrees
            rotate_speed: 0.01,
            zoom_speed: 0.5,
            min_radius: 1.0,
            max_radius: 50.0,
            is_dragging: false,
            last_mouse_pos: Point::ZERO,
        }
    }
}

impl OrbitControls {
    /// Creates a new `OrbitControls` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles a native UI event and updates camera orientation.
    pub fn handle_event(&mut self, event: &UiEvent, camera: &mut PerspectiveCamera) {
        match event {
            UiEvent::MouseDown { button, position } if *button == MouseButton::Left => {
                self.is_dragging = true;
                self.last_mouse_pos = *position;
            }
            UiEvent::MouseUp { button, .. } if *button == MouseButton::Left => {
                self.is_dragging = false;
            }
            UiEvent::MouseMove { position } if self.is_dragging => {
                let dx = position.x - self.last_mouse_pos.x;
                let dy = position.y - self.last_mouse_pos.y;
                self.last_mouse_pos = *position;

                self.theta -= dx * self.rotate_speed;
                self.phi =
                    (self.phi + dy * self.rotate_speed).clamp(0.05, std::f32::consts::PI - 0.05);

                self.update_camera(camera);
            }
            UiEvent::Scroll { delta_y, .. } => {
                self.radius = (self.radius - delta_y * self.zoom_speed)
                    .clamp(self.min_radius, self.max_radius);
                self.update_camera(camera);
            }
            _ => {}
        }
    }

    /// Updates camera eye position based on spherical coordinates (radius, theta, phi).
    pub fn update_camera(&self, camera: &mut PerspectiveCamera) {
        let x = self.radius * self.phi.sin() * self.theta.sin();
        let y = self.radius * self.phi.cos();
        let z = self.radius * self.phi.sin() * self.theta.cos();

        camera.position = camera.target + Vec3::new(x, y, z);
    }
}
