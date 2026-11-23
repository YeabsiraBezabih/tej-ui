//! Linux libinput kinetic scroll inertia model.

use tej_ui_layout::Point;

/// Linux libinput kinetic scroll physics.
#[derive(Debug, Clone)]
pub struct LibinputScrollPhysics {
    velocity: Point,
    friction: f32,
}

impl LibinputScrollPhysics {
    pub fn new() -> Self {
        Self {
            velocity: Point::ZERO,
            friction: 4.8,
        }
    }

    /// Feeds a scroll event from libinput / Wayland axis event.
    pub fn handle_axis_delta(&mut self, delta: Point) {
        self.velocity.x += delta.x * 60.0;
        self.velocity.y += delta.y * 60.0;
    }

    /// Advances kinetic momentum per frame step.
    pub fn step(&mut self, dt: f32) -> Point {
        if self.velocity.x.abs() < 0.1 && self.velocity.y.abs() < 0.1 {
            self.velocity = Point::ZERO;
            return Point::ZERO;
        }

        let decay = (-self.friction * dt).exp();
        let displacement = Point::new(self.velocity.x * dt, self.velocity.y * dt);

        self.velocity.x *= decay;
        self.velocity.y *= decay;

        displacement
    }

    pub fn is_at_rest(&self) -> bool {
        self.velocity.x.abs() < 0.1 && self.velocity.y.abs() < 0.1
    }

    pub fn stop(&mut self) {
        self.velocity = Point::ZERO;
    }
}

impl Default for LibinputScrollPhysics {
    fn default() -> Self {
        Self::new()
    }
}
