//! Frame ticker synchronized with platform VSync clock.

use std::time::Instant;

/// Central frame ticker coordinating delta times across active animations.
#[derive(Debug)]
pub struct VsyncTicker {
    /// Timestamp of previous ticked frame.
    last_tick: Option<Instant>,
    /// Minimum target frame delta time (e.g. 1/60s or 1/120s).
    target_fps: f32,
    /// Total elapsed time in seconds since ticker start.
    elapsed_seconds: f32,
}

impl Default for VsyncTicker {
    fn default() -> Self {
        Self::new(60.0)
    }
}

impl VsyncTicker {
    /// Create a new ticker with expected display refresh rate.
    #[must_use]
    pub fn new(target_fps: f32) -> Self {
        Self {
            last_tick: None,
            target_fps: target_fps.max(30.0),
            elapsed_seconds: 0.0,
        }
    }

    /// Target display refresh rate in frames per second.
    #[must_use]
    pub const fn target_fps(&self) -> f32 {
        self.target_fps
    }

    /// Total seconds elapsed since ticker creation.
    #[must_use]
    pub const fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    /// Advance ticker clock and return elapsed delta time in seconds, clamped to 100ms.
    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let dt = match self.last_tick {
            Some(prev) => {
                let duration = now.duration_since(prev).as_secs_f32();
                // Clamp delta time to avoid physics explosion on long background stalls
                duration.clamp(0.001, 0.1)
            }
            None => 1.0 / self.target_fps,
        };

        self.last_tick = Some(now);
        self.elapsed_seconds += dt;
        dt
    }

    /// Manually reset ticker clock without advancing delta time.
    pub fn reset(&mut self) {
        self.last_tick = None;
        self.elapsed_seconds = 0.0;
    }
}
