//! Multi-monitor discovery, DPI scaling factors, and screen layout coordinates.

use std::fmt::Debug;
use tej_layout::{Point, Rect, Size};

/// Unique identifier for an attached physical monitor/display.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MonitorId(pub u32);

/// Metrics and configuration for an attached display.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorInfo {
    /// Unique monitor ID.
    pub id: MonitorId,
    /// Human-readable monitor name (e.g. "Dell U2720Q", "Built-in Retina Display").
    pub name: String,
    /// Bounding rectangle in virtual screen coordinates.
    pub bounds: Rect,
    /// DPI scaling factor (e.g. 1.0 for 96 DPI, 1.25, 1.5, 2.0 for Retina/4K).
    pub scale_factor: f32,
    /// Whether this display is the primary system monitor.
    pub is_primary: bool,
}

impl MonitorInfo {
    /// Creates a new `MonitorInfo`.
    #[must_use]
    pub fn new(
        id: MonitorId,
        name: impl Into<String>,
        bounds: Rect,
        scale_factor: f32,
        is_primary: bool,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            bounds,
            scale_factor: if scale_factor > 0.0 {
                scale_factor
            } else {
                1.0
            },
            is_primary,
        }
    }

    /// Physical pixel resolution of the monitor.
    #[must_use]
    pub fn physical_size(&self) -> Size {
        Size::new(
            self.bounds.width() * self.scale_factor,
            self.bounds.height() * self.scale_factor,
        )
    }

    /// Converts logical points to physical pixels on this monitor.
    #[must_use]
    pub fn logical_to_physical(&self, point: Point) -> Point {
        Point::new(point.x * self.scale_factor, point.y * self.scale_factor)
    }

    /// Converts physical pixels to logical points on this monitor.
    #[must_use]
    pub fn physical_to_logical(&self, point: Point) -> Point {
        let inv = 1.0 / self.scale_factor;
        Point::new(point.x * inv, point.y * inv)
    }
}

/// System-wide monitor manager tracking all attached displays and window-to-monitor mapping.
#[derive(Debug, Clone)]
pub struct MonitorManager {
    monitors: Vec<MonitorInfo>,
}

impl Default for MonitorManager {
    fn default() -> Self {
        Self::with_default_display()
    }
}

impl MonitorManager {
    /// Creates a monitor manager with a single standard 1080p display at 1.0x scale.
    #[must_use]
    pub fn with_default_display() -> Self {
        Self {
            monitors: vec![MonitorInfo::new(
                MonitorId(1),
                "Primary Display",
                Rect::from_origin_size(Point::new(0.0, 0.0), Size::new(1920.0, 1080.0)),
                1.0,
                true,
            )],
        }
    }

    /// Registers a monitor.
    pub fn add_monitor(&mut self, monitor: MonitorInfo) {
        self.monitors.retain(|m| m.id != monitor.id);
        self.monitors.push(monitor);
    }

    /// Returns all attached monitors.
    #[must_use]
    pub fn monitors(&self) -> &[MonitorInfo] {
        &self.monitors
    }

    /// Returns the primary display.
    #[must_use]
    pub fn primary_monitor(&self) -> Option<&MonitorInfo> {
        self.monitors
            .iter()
            .find(|m| m.is_primary)
            .or_else(|| self.monitors.first())
    }

    /// Finds the monitor that contains the given virtual screen point.
    #[must_use]
    pub fn monitor_at_point(&self, point: Point) -> Option<&MonitorInfo> {
        self.monitors.iter().find(|m| m.bounds.contains(point))
    }
}
