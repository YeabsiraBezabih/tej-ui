//! Dirty-region HUD visualizer and frame performance profiler.

use std::time::{Duration, Instant};
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::Rect;

/// Performance metrics collected for a single rendered frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameMetrics {
    /// Incremental frame index.
    pub frame_number: u64,
    /// Time spent executing layout and painting.
    pub frame_duration: Duration,
    /// Number of distinct dirty bounding boxes composited.
    pub dirty_rect_count: usize,
    /// Number of draw calls culled due to dirty-region clipping.
    pub culled_draw_calls: usize,
}

/// On-screen Heads-Up Display (HUD) overlay that flashes dirty rects and renders FPS counters.
#[derive(Debug)]
pub struct DirtyHudOverlay {
    enabled: bool,
    show_paint_flashing: bool,
    frame_counter: u64,
    last_frame_time: Instant,
    recent_metrics: Vec<FrameMetrics>,
    flash_color: Color,
}

impl Default for DirtyHudOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl DirtyHudOverlay {
    /// Creates a new `DirtyHudOverlay`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled: true,
            show_paint_flashing: true,
            frame_counter: 0,
            last_frame_time: Instant::now(),
            recent_metrics: Vec::with_capacity(120),
            flash_color: Color::rgba(0, 255, 128, 64), // Translucent green flash
        }
    }

    /// Enables the dirty-rect HUD overlay.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables the dirty-rect HUD overlay.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Whether the HUD is currently enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Records metrics for a completed frame.
    pub fn record_frame(
        &mut self,
        duration: Duration,
        dirty_rect_count: usize,
        culled_draw_calls: usize,
    ) {
        self.frame_counter += 1;
        self.last_frame_time = Instant::now();

        if self.recent_metrics.len() >= 120 {
            self.recent_metrics.remove(0);
        }

        self.recent_metrics.push(FrameMetrics {
            frame_number: self.frame_counter,
            frame_duration: duration,
            dirty_rect_count,
            culled_draw_calls,
        });
    }

    /// Renders paint flashing rectangles over the dirty regions on the target surface.
    pub fn render_dirty_flashes(&self, surface: &mut dyn SurfaceCompositor, dirty_rects: &[Rect]) {
        if !self.enabled || !self.show_paint_flashing {
            return;
        }

        for &rect in dirty_rects {
            // Fill translucent dirty boundary box
            surface.fill_rect(rect, self.flash_color);
        }
    }

    /// Calculates rolling average frame time in milliseconds.
    #[must_use]
    pub fn average_frame_time_ms(&self) -> f32 {
        if self.recent_metrics.is_empty() {
            return 0.0;
        }
        let total: Duration = self.recent_metrics.iter().map(|m| m.frame_duration).sum();
        let sample_count = f32::from(u16::try_from(self.recent_metrics.len()).unwrap_or(u16::MAX));
        (total.as_secs_f32() * 1000.0) / sample_count
    }

    /// Estimates current Frames Per Second (FPS).
    #[must_use]
    pub fn estimated_fps(&self) -> f32 {
        let avg_ms = self.average_frame_time_ms();
        if avg_ms > 0.0 { 1000.0 / avg_ms } else { 60.0 }
    }
}
