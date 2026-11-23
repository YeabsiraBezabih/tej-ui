//! Stylus and active digitizer pen pressure, tilt, barrel buttons, and palm rejection.

use std::fmt::Debug;
use tej_ui_layout::Point;

/// Physical tool tip variant of the active stylus.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StylusToolType {
    #[default]
    Pen,
    Eraser,
    Pencil,
    Brush,
    Airbrush,
}

/// Instantaneous stylus pen input sample with high-precision pressure and tilt angles.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StylusPoint {
    /// 2D contact coordinate on the active surface.
    pub position: Point,
    /// Normalized transducer pressure level in range `0.0..=1.0`.
    pub pressure: f32,
    /// Tilt angle relative to vertical surface normal in radians (`0.0..=std::f32::consts::FRAC_PI_2`).
    pub altitude: f32,
    /// Clockwise direction of pen body tilt in radians (`0.0..=std::f32::consts::TAU`).
    pub azimuth: f32,
    /// Cartesian tilt X in degrees (`-90.0..=90.0`).
    pub tilt_x: f32,
    /// Cartesian tilt Y in degrees (`-90.0..=90.0`).
    pub tilt_y: f32,
    /// Contact ellipse radii (major_radius, minor_radius).
    pub contact_size: (f32, f32),
    /// Transducer tip tool type.
    pub tool_type: StylusToolType,
    /// Whether the primary barrel/side button is held down.
    pub barrel_pressed: bool,
    /// Whether the stylus is inverted (physically flipped to eraser end).
    pub is_inverted: bool,
}

impl Default for StylusPoint {
    fn default() -> Self {
        Self {
            position: Point::ZERO,
            pressure: 0.5,
            altitude: std::f32::consts::FRAC_PI_2, // Perpendicular (90 deg)
            azimuth: 0.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            contact_size: (2.0, 2.0),
            tool_type: StylusToolType::Pen,
            barrel_pressed: false,
            is_inverted: false,
        }
    }
}

impl StylusPoint {
    /// Create a stylus sample with position and pressure.
    #[must_use]
    pub fn new(position: Point, pressure: f32) -> Self {
        Self {
            position,
            pressure: pressure.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Set tilt angles in degrees.
    #[must_use]
    pub fn with_tilt(mut self, tilt_x: f32, tilt_y: f32) -> Self {
        self.tilt_x = tilt_x.clamp(-90.0, 90.0);
        self.tilt_y = tilt_y.clamp(-90.0, 90.0);
        self
    }
}

/// Palm rejection filter discarding accidental touch contacts while stylus is in proximity.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PalmRejectionFilter {
    /// Maximum touch contact major radius considered a finger (contacts above this are flagged as palm).
    pub max_finger_radius: f32,
    /// Rejection radius around active stylus tip.
    pub stylus_exclusion_radius: f32,
}

impl Default for PalmRejectionFilter {
    fn default() -> Self {
        Self {
            max_finger_radius: 20.0,
            stylus_exclusion_radius: 80.0,
        }
    }
}

impl PalmRejectionFilter {
    /// Create a palm rejection filter with custom thresholds.
    #[must_use]
    pub const fn new(max_finger_radius: f32, stylus_exclusion_radius: f32) -> Self {
        Self {
            max_finger_radius,
            stylus_exclusion_radius,
        }
    }

    /// Check if a touch input contact should be rejected as an accidental palm rest.
    #[must_use]
    pub fn is_palm(
        &self,
        touch_pos: Point,
        touch_radius: f32,
        active_stylus_pos: Option<Point>,
    ) -> bool {
        // 1. Direct contact area threshold check
        if touch_radius > self.max_finger_radius {
            return true;
        }

        // 2. Proximity to active stylus tip
        if let Some(stylus_pos) = active_stylus_pos {
            let dx = touch_pos.x - stylus_pos.x;
            let dy = touch_pos.y - stylus_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < self.stylus_exclusion_radius * self.stylus_exclusion_radius {
                return true;
            }
        }

        false
    }
}
