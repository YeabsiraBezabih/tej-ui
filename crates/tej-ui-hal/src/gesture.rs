//! Multi-touch gesture arena, recognition disambiguation, and kinetic velocity physics.

use std::collections::HashMap;
use std::fmt::Debug;
use tej_ui_layout::Point;

/// Identifier for an individual physical touch contact pointer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TouchId(pub u64);

/// Phase of an active touch contact point.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TouchPhase {
    /// Initial contact placed down.
    Started,
    /// Contact moved to a new coordinate.
    Moved,
    /// Contact lifted from display.
    Ended,
    /// Contact cancelled (e.g. system gesture interrupt or palm rejection).
    Cancelled,
}

/// Instantaneous state of an individual touch contact.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TouchPoint {
    pub id: TouchId,
    pub position: Point,
    pub start_position: Point,
    pub timestamp_millis: u64,
    pub phase: TouchPhase,
}

/// Operational state of a gesture recognizer.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GestureState {
    #[default]
    Possible,
    Began,
    Changed,
    Ended,
    Failed,
    Cancelled,
}

/// Swipe directional classification.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Continuous pan/drag gesture state tracking translation and current velocity.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct PanGesture {
    pub state: GestureState,
    pub translation: Point,
    pub velocity: Point,
    pub active_touch: Option<TouchId>,
    last_position: Point,
    last_timestamp: u64,
}

impl PanGesture {
    pub fn update(&mut self, touch: TouchPoint) {
        match touch.phase {
            TouchPhase::Started => {
                self.state = GestureState::Began;
                self.active_touch = Some(touch.id);
                self.translation = Point::ZERO;
                self.last_position = touch.position;
                self.last_timestamp = touch.timestamp_millis;
            }
            TouchPhase::Moved if self.active_touch == Some(touch.id) => {
                let dx = touch.position.x - touch.start_position.x;
                let dy = touch.position.y - touch.start_position.y;
                self.translation = Point::new(dx, dy);

                let dt_ms =
                    u16::try_from(touch.timestamp_millis.saturating_sub(self.last_timestamp))
                        .unwrap_or(u16::MAX);
                let dt = f32::from(dt_ms) / 1000.0;
                if dt > 0.001 {
                    let vx = (touch.position.x - self.last_position.x) / dt;
                    let vy = (touch.position.y - self.last_position.y) / dt;
                    self.velocity = Point::new(vx, vy);
                }

                self.last_position = touch.position;
                self.last_timestamp = touch.timestamp_millis;
                self.state = GestureState::Changed;
            }
            TouchPhase::Ended if self.active_touch == Some(touch.id) => {
                self.state = GestureState::Ended;
                self.active_touch = None;
            }
            TouchPhase::Cancelled if self.active_touch == Some(touch.id) => {
                self.state = GestureState::Cancelled;
                self.active_touch = None;
            }
            _ => {}
        }
    }
}

/// 2-finger pinch-to-zoom gesture tracking scale multiplier and centroid.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct PinchGesture {
    pub state: GestureState,
    pub scale: f32,
    pub centroid: Point,
    initial_distance: f32,
}

impl PinchGesture {
    pub fn update(&mut self, p1: TouchPoint, p2: TouchPoint) {
        let dx = p2.position.x - p1.position.x;
        let dy = p2.position.y - p1.position.y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let center = Point::new(
            f32::midpoint(p1.position.x, p2.position.x),
            f32::midpoint(p1.position.y, p2.position.y),
        );

        if self.state == GestureState::Possible || self.state == GestureState::Ended {
            self.initial_distance = dist;
            self.scale = 1.0;
            self.centroid = center;
            self.state = GestureState::Began;
        } else if p1.phase == TouchPhase::Ended || p2.phase == TouchPhase::Ended {
            self.state = GestureState::Ended;
        } else {
            self.scale = dist / self.initial_distance.max(1.0);
            self.centroid = center;
            self.state = GestureState::Changed;
        }
    }
}

/// Kinetic scroll deceleration simulator with exponential friction decay.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct KineticDeceleration {
    pub current_velocity: Point,
    pub friction_coefficient: f32,
}

impl Default for KineticDeceleration {
    fn default() -> Self {
        Self {
            current_velocity: Point::ZERO,
            friction_coefficient: 4.5,
        }
    }
}

impl KineticDeceleration {
    /// Create a new kinetic simulation starting at `initial_velocity` (pixels/second).
    #[must_use]
    pub const fn new(initial_velocity: Point, friction_coefficient: f32) -> Self {
        Self {
            current_velocity: initial_velocity,
            friction_coefficient,
        }
    }

    /// Advance simulation by `delta_seconds`, returning displacement delta $(\Delta x, \Delta y)$.
    pub fn step(&mut self, delta_seconds: f32) -> Point {
        let decay = (-self.friction_coefficient * delta_seconds).exp();
        let dx = self.current_velocity.x * (1.0 - decay) / self.friction_coefficient;
        let dy = self.current_velocity.y * (1.0 - decay) / self.friction_coefficient;

        self.current_velocity.x *= decay;
        self.current_velocity.y *= decay;

        if self.current_velocity.x.abs() < 1.0 {
            self.current_velocity.x = 0.0;
        }
        if self.current_velocity.y.abs() < 1.0 {
            self.current_velocity.y = 0.0;
        }

        Point::new(dx, dy)
    }

    /// Check if kinetic motion has completely come to rest.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.current_velocity.x == 0.0 && self.current_velocity.y == 0.0
    }
}

/// Gesture arena managing simultaneous active touches and routing to recognizers.
#[derive(Debug, Default)]
pub struct GestureArena {
    active_touches: HashMap<TouchId, TouchPoint>,
    pan: PanGesture,
    pinch: PinchGesture,
}

impl GestureArena {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Process an incoming touch event.
    pub fn handle_touch(&mut self, touch: TouchPoint) {
        match touch.phase {
            TouchPhase::Started | TouchPhase::Moved => {
                self.active_touches.insert(touch.id, touch);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.active_touches.remove(&touch.id);
            }
        }

        // Single-touch pan
        self.pan.update(touch);

        // Multi-touch pinch
        if self.active_touches.len() == 2 {
            let mut points = self.active_touches.values();
            if let (Some(&p1), Some(&p2)) = (points.next(), points.next()) {
                self.pinch.update(p1, p2);
            }
        }
    }

    /// Active pan gesture state.
    #[must_use]
    pub const fn pan(&self) -> &PanGesture {
        &self.pan
    }

    /// Active pinch gesture state.
    #[must_use]
    pub const fn pinch(&self) -> &PinchGesture {
        &self.pinch
    }

    /// Active touch pointer count.
    #[must_use]
    pub fn touch_count(&self) -> usize {
        self.active_touches.len()
    }
}
