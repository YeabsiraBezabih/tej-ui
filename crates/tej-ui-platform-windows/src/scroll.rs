//! Windows Precision Touchpad and mouse wheel kinetic scroll physics.

use tej_ui_layout::Point;

/// Windows mouse wheel standard detent quantum (`WHEEL_DELTA` = 120).
pub const WHEEL_DELTA: f32 = 120.0;

/// Windows kinetic scroll model with acceleration and friction damping.
#[derive(Debug, Clone)]
pub struct Win32ScrollPhysics {
    scroll_lines_per_click: f32,
    line_height_pixels: f32,
    accumulated_wheel_delta: f32,
    velocity: Point,
    friction: f32,
}

impl Win32ScrollPhysics {
    /// Creates a scroll controller with Windows system defaults (3 lines per detent, 20px line height).
    pub fn new() -> Self {
        Self {
            scroll_lines_per_click: 3.0,
            line_height_pixels: 20.0,
            accumulated_wheel_delta: 0.0,
            velocity: Point::ZERO,
            friction: 5.5,
        }
    }

    /// Configures the user's Windows mouse settings (`SPI_GETWHEELSCROLLLINES`).
    pub fn set_scroll_lines(&mut self, lines: f32) {
        self.scroll_lines_per_click = lines.max(1.0);
    }

    /// Handles a `WM_MOUSEWHEEL` (vertical) or `WM_MOUSEHWHEEL` (horizontal) message.
    pub fn handle_wheel_message(&mut self, raw_delta: f32, is_horizontal: bool) -> f32 {
        // High-precision smooth scrolling (fractional WHEEL_DELTA)
        let normalized = raw_delta / WHEEL_DELTA;
        let pixel_delta = normalized * self.scroll_lines_per_click * self.line_height_pixels;

        if is_horizontal {
            self.velocity.x += pixel_delta * 12.0;
        } else {
            self.velocity.y += pixel_delta * 12.0;
        }

        pixel_delta
    }

    /// Advances kinetic momentum per frame step, returning current frame offset.
    pub fn step(&mut self, dt_seconds: f32) -> Point {
        if self.velocity.x.abs() < 0.1 && self.velocity.y.abs() < 0.1 {
            self.velocity = Point::ZERO;
            return Point::ZERO;
        }

        let decay = (-self.friction * dt_seconds).exp();
        let displacement = Point::new(self.velocity.x * dt_seconds, self.velocity.y * dt_seconds);

        self.velocity.x *= decay;
        self.velocity.y *= decay;

        displacement
    }

    /// Checks if scrolling is stationary.
    pub fn is_at_rest(&self) -> bool {
        self.velocity.x.abs() < 0.1 && self.velocity.y.abs() < 0.1
    }

    /// Accumulated wheel delta.
    pub fn accumulated_wheel_delta(&self) -> f32 {
        self.accumulated_wheel_delta
    }

    /// Stops kinetic scroll abruptly (e.g. on pointer down).
    pub fn stop(&mut self) {
        self.velocity = Point::ZERO;
    }
}

impl Default for Win32ScrollPhysics {
    fn default() -> Self {
        Self::new()
    }
}
