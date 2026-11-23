//! Interactive OrbitControls for manipulating 3D cameras using mouse events.

use crate::camera::PerspectiveCamera;
use crate::math::Vec3;
use tej_ui_hal::{MouseButton, UiEvent};
use tej_ui_layout::Point;

/// Canonical camera view orientation presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPreset {
    /// 3/4 isometric perspective view.
    Isometric,
    /// Top-down view facing down from +Y.
    Top,
    /// Front view facing the model.
    Front,
    /// Side profile view.
    Side,
    /// Reset target to origin and default framing.
    Reset,
}

/// Interactive 3D camera controller supporting dynamic target panning, 360° orbiting, and presets.
#[derive(Debug, Clone)]
pub struct OrbitControls {
    /// Distance from camera to target.
    pub radius: f32,
    /// Azimuth angle in radians (horizontal rotation).
    pub theta: f32,
    /// Elevation angle in radians (vertical pitch).
    pub phi: f32,
    /// Sensitivity for mouse dragging rotation.
    pub rotate_speed: f32,
    /// Sensitivity for mouse scroll wheel zoom.
    pub zoom_speed: f32,
    /// Minimum zoom radius.
    pub min_radius: f32,
    /// Maximum zoom radius.
    pub max_radius: f32,
    is_orbiting: bool,
    is_panning: bool,
    last_mouse_pos: Point,
}

impl Default for OrbitControls {
    fn default() -> Self {
        Self {
            radius: 3.5,
            theta: 0.0,
            phi: std::f32::consts::FRAC_PI_2,
            rotate_speed: 0.008,
            zoom_speed: 0.08,
            min_radius: 0.2,
            max_radius: 120.0,
            is_orbiting: false,
            is_panning: false,
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

    /// Syncs orbit controls state from an existing camera's position and target.
    pub fn sync_from_camera(&mut self, camera: &PerspectiveCamera) {
        let diff = camera.position - camera.target;
        let dist = diff.length();
        if dist > 1e-4 {
            self.radius = dist.clamp(self.min_radius, self.max_radius);
            let cos_phi = (diff.y / dist).clamp(-1.0, 1.0);
            self.phi = cos_phi.acos().clamp(0.05, std::f32::consts::PI - 0.05);
            self.theta = diff.x.atan2(diff.z);
        }
    }

    /// Dynamically shifts the camera target to a specific 3D point and updates position.
    pub fn focus_point(&mut self, target: Vec3, radius: f32, camera: &mut PerspectiveCamera) {
        camera.target = target;
        self.radius = radius.clamp(self.min_radius, self.max_radius);
        self.update_camera(camera);
    }

    /// Sets a canonical camera view preset.
    pub fn set_preset(&mut self, preset: CameraPreset, camera: &mut PerspectiveCamera) {
        match preset {
            CameraPreset::Isometric => {
                self.theta = std::f32::consts::FRAC_PI_4;
                self.phi = std::f32::consts::FRAC_PI_3;
            }
            CameraPreset::Top => {
                self.theta = 0.0;
                self.phi = 0.05;
            }
            CameraPreset::Front => {
                self.theta = 0.0;
                self.phi = std::f32::consts::FRAC_PI_2;
            }
            CameraPreset::Side => {
                self.theta = std::f32::consts::FRAC_PI_2;
                self.phi = std::f32::consts::FRAC_PI_2;
            }
            CameraPreset::Reset => {
                camera.target = Vec3::ZERO;
                self.radius = 3.5;
                self.theta = 0.0;
                self.phi = std::f32::consts::FRAC_PI_2;
            }
        }
        self.update_camera(camera);
    }

    /// Dynamically translates the camera target and eye position in camera view plane.
    pub fn pan(&mut self, dx: f32, dy: f32, camera: &mut PerspectiveCamera) {
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward).normalize();
        let pan_speed = (self.radius * 0.0018).max(0.004);
        let delta = (right * (-dx * pan_speed)) + (up * (dy * pan_speed));
        camera.target = camera.target + delta;
        camera.position = camera.position + delta;
    }

    /// Navigates camera forward/backward along viewing direction like an in-game entity.
    pub fn move_forward(&mut self, step: f32, camera: &mut PerspectiveCamera) {
        let forward = (camera.target - camera.position).normalize();
        let delta = forward * step;
        camera.position = camera.position + delta;
        camera.target = camera.target + delta;
    }

    /// Strafes camera left/right perpendicular to view direction like an in-game entity.
    pub fn strafe(&mut self, step: f32, camera: &mut PerspectiveCamera) {
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();
        let delta = right * step;
        camera.position = camera.position + delta;
        camera.target = camera.target + delta;
    }

    /// Ascends or descends camera vertically along world Y axis.
    pub fn ascend(&mut self, step: f32, camera: &mut PerspectiveCamera) {
        let delta = Vec3::new(0.0, step, 0.0);
        camera.position = camera.position + delta;
        camera.target = camera.target + delta;
    }

    /// Handles a native UI event and updates camera orientation.
    pub fn handle_event(&mut self, event: &UiEvent, camera: &mut PerspectiveCamera) {
        match event {
            UiEvent::MouseDown { button, position } => {
                if *button == MouseButton::Left {
                    self.is_orbiting = true;
                    self.last_mouse_pos = *position;
                } else if *button == MouseButton::Right || *button == MouseButton::Middle {
                    self.is_panning = true;
                    self.last_mouse_pos = *position;
                }
            }
            UiEvent::MouseUp { button, .. } => {
                if *button == MouseButton::Left {
                    self.is_orbiting = false;
                } else if *button == MouseButton::Right || *button == MouseButton::Middle {
                    self.is_panning = false;
                }
            }
            UiEvent::MouseMove { position } => {
                let dx = position.x - self.last_mouse_pos.x;
                let dy = position.y - self.last_mouse_pos.y;
                self.last_mouse_pos = *position;

                if self.is_orbiting {
                    self.theta -= dx * self.rotate_speed;
                    // Subtract dy so dragging mouse up tilts camera up (natural intuitive pitch)
                    self.phi = (self.phi - dy * self.rotate_speed)
                        .clamp(0.05, std::f32::consts::PI - 0.05);
                    self.update_camera(camera);
                } else if self.is_panning {
                    self.pan(dx, dy, camera);
                }
            }
            UiEvent::Scroll { delta_y, .. } => {
                // Smooth exponential zoom: scales proportionally with distance to prevent getting stuck
                let factor = (-delta_y.clamp(-5.0, 5.0) * self.zoom_speed).exp();
                self.radius = (self.radius * factor).clamp(self.min_radius, self.max_radius);
                self.update_camera(camera);
            }
            UiEvent::KeyDown { key_code } => {
                let speed = (self.radius * 0.06).clamp(0.08, 1.2);
                match *key_code {
                    87 | 38 => self.move_forward(speed, camera), // W or Up Arrow
                    83 | 40 => self.move_forward(-speed, camera), // S or Down Arrow
                    65 | 37 => self.strafe(-speed, camera),      // A or Left Arrow
                    68 | 39 => self.strafe(speed, camera),       // D or Right Arrow
                    32 | 69 => self.ascend(speed, camera),       // Space or E (Ascend)
                    81 | 67 => self.ascend(-speed, camera),      // Q or C (Descend)
                    _ => {}
                }
            }
            UiEvent::TextInput { text } => {
                let speed = (self.radius * 0.06).clamp(0.08, 1.2);
                match text.as_str() {
                    "w" | "W" => self.move_forward(speed, camera),
                    "s" | "S" => self.move_forward(-speed, camera),
                    "a" | "A" => self.strafe(-speed, camera),
                    "d" | "D" => self.strafe(speed, camera),
                    "e" | "E" | " " => self.ascend(speed, camera),
                    "q" | "Q" | "c" | "C" => self.ascend(-speed, camera),
                    _ => {}
                }
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
