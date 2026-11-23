//! macOS elastic rubber-banding kinetic scroll physics.

use tej_ui_layout::Point;

/// macOS kinetic scroll physics with overscroll rubber-banding.
#[derive(Debug, Clone)]
pub struct MacScrollPhysics {
    scroll_offset: Point,
    velocity: Point,
    content_size: (f32, f32),
    viewport_size: (f32, f32),
    is_user_dragging: bool,
    spring_stiffness: f32,
    spring_damping: f32,
}

impl MacScrollPhysics {
    pub fn new(content_height: f32, viewport_height: f32) -> Self {
        Self {
            scroll_offset: Point::ZERO,
            velocity: Point::ZERO,
            content_size: (0.0, content_height),
            viewport_size: (0.0, viewport_height),
            is_user_dragging: false,
            spring_stiffness: 250.0,
            spring_damping: 30.0,
        }
    }

    /// User began trackpad gesture or drag.
    pub fn begin_gesture(&mut self) {
        self.is_user_dragging = true;
        self.velocity = Point::ZERO;
    }

    /// Applies raw delta from Magic Trackpad or Magic Mouse.
    pub fn handle_trackpad_delta(&mut self, delta_y: f32) {
        let max_scroll = (self.content_size.1 - self.viewport_size.1).max(0.0);

        // When crossing or already in overscroll region, dampen by 0.55
        let target_y = self.scroll_offset.y + delta_y;
        if target_y < 0.0 {
            if self.scroll_offset.y < 0.0 {
                self.scroll_offset.y += delta_y * 0.55;
            } else {
                let overscroll = target_y;
                self.scroll_offset.y = overscroll * 0.55;
            }
        } else if target_y > max_scroll {
            if self.scroll_offset.y > max_scroll {
                self.scroll_offset.y += delta_y * 0.55;
            } else {
                let overscroll = target_y - max_scroll;
                self.scroll_offset.y = max_scroll + overscroll * 0.55;
            }
        } else {
            self.scroll_offset.y = target_y;
        }

        self.velocity.y = delta_y * 60.0;
    }

    /// User lifted fingers from trackpad.
    pub fn end_gesture(&mut self) {
        self.is_user_dragging = false;
    }

    /// Advances physics simulation per frame step.
    pub fn step(&mut self, dt: f32) -> Point {
        let max_scroll = (self.content_size.1 - self.viewport_size.1).max(0.0);

        if !self.is_user_dragging {
            // Check if in overscroll region -> apply restorative spring force
            if self.scroll_offset.y < 0.0 {
                // Pulled past top: spring pulls back toward 0.0
                let displacement = self.scroll_offset.y;
                let spring_force = -self.spring_stiffness * displacement;
                let damping_force = -self.spring_damping * self.velocity.y;
                let accel = spring_force + damping_force;

                self.velocity.y += accel * dt;
                self.scroll_offset.y += self.velocity.y * dt;

                if self.scroll_offset.y.abs() < 0.5 && self.velocity.y.abs() < 1.0 {
                    self.scroll_offset.y = 0.0;
                    self.velocity.y = 0.0;
                }
            } else if self.scroll_offset.y > max_scroll {
                // Pulled past bottom: spring pulls back toward max_scroll
                let displacement = self.scroll_offset.y - max_scroll;
                let spring_force = -self.spring_stiffness * displacement;
                let damping_force = -self.spring_damping * self.velocity.y;
                let accel = spring_force + damping_force;

                self.velocity.y += accel * dt;
                self.scroll_offset.y += self.velocity.y * dt;

                if (self.scroll_offset.y - max_scroll).abs() < 0.5 && self.velocity.y.abs() < 1.0 {
                    self.scroll_offset.y = max_scroll;
                    self.velocity.y = 0.0;
                }
            } else {
                // Within bounds: standard Apple exponential kinetic deceleration
                let decay = (-5.0 * dt).exp();
                self.scroll_offset.y += self.velocity.y * dt;
                self.velocity.y *= decay;

                if self.velocity.y.abs() < 1.0 {
                    self.velocity.y = 0.0;
                }
            }
        }

        self.scroll_offset
    }

    /// Current scroll offset.
    pub fn offset(&self) -> Point {
        self.scroll_offset
    }

    /// True if scroll is currently at rest.
    pub fn is_at_rest(&self) -> bool {
        !self.is_user_dragging
            && self.velocity.y.abs() < 0.5
            && (self.scroll_offset.y >= 0.0
                && self.scroll_offset.y <= (self.content_size.1 - self.viewport_size.1).max(0.0))
    }
}
