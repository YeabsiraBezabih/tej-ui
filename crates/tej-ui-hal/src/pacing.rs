//! Display refresh rate pacing, VSync scheduling, ProMotion (120Hz+) & VRR.

use std::time::Duration;

/// Display refresh mode and capabilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefreshRateMode {
    /// Fixed standard refresh rate (e.g. 60Hz).
    Fixed(f32),
    /// High-refresh display with ProMotion / dynamic stepping (e.g. 24Hz - 120Hz).
    ProMotion { min_hz: f32, max_hz: f32 },
    /// Variable Refresh Rate (G-Sync / FreeSync / Adaptive Sync).
    VariableRate { min_hz: f32, max_hz: f32 },
}

impl RefreshRateMode {
    pub const STANDARD_60: Self = Self::Fixed(60.0);
    pub const PROMOTION_120: Self = Self::ProMotion {
        min_hz: 24.0,
        max_hz: 120.0,
    };
    pub const GAMING_144: Self = Self::Fixed(144.0);
}

/// Frame presentation scheduler and pacing controller.
#[derive(Debug, Clone)]
pub struct FramePacer {
    mode: RefreshRateMode,
    target_hz: f32,
    frame_budget: Duration,
    total_frames_presented: u64,
    dropped_frames: u64,
}

impl FramePacer {
    /// Constructs a new frame pacer with the given refresh mode.
    pub fn new(mode: RefreshRateMode) -> Self {
        let initial_hz = match mode {
            RefreshRateMode::Fixed(hz) => hz,
            RefreshRateMode::ProMotion { max_hz, .. } => max_hz,
            RefreshRateMode::VariableRate { max_hz, .. } => max_hz,
        };
        let budget = Duration::from_secs_f32(1.0 / initial_hz);
        Self {
            mode,
            target_hz: initial_hz,
            frame_budget: budget,
            total_frames_presented: 0,
            dropped_frames: 0,
        }
    }

    /// Sets the dynamic target refresh rate within the allowed mode bounds.
    pub fn set_target_hz(&mut self, target_hz: f32) {
        let clamped = match self.mode {
            RefreshRateMode::Fixed(hz) => hz,
            RefreshRateMode::ProMotion { min_hz, max_hz }
            | RefreshRateMode::VariableRate { min_hz, max_hz } => target_hz.clamp(min_hz, max_hz),
        };
        self.target_hz = clamped;
        self.frame_budget = Duration::from_secs_f32(1.0 / clamped);
    }

    /// Returns the target refresh rate in Hz.
    pub fn target_hz(&self) -> f32 {
        self.target_hz
    }

    /// Returns the allocated duration per frame (e.g. 8.33ms for 120Hz, 16.66ms for 60Hz).
    pub fn frame_budget(&self) -> Duration {
        self.frame_budget
    }

    /// Checks if a frame presentation exceeded the budget deadline and tracks metrics.
    pub fn record_frame_execution(&mut self, elapsed: Duration) -> bool {
        self.total_frames_presented += 1;
        if elapsed > self.frame_budget {
            self.dropped_frames += 1;
            false
        } else {
            true
        }
    }

    /// Total number of presented frames.
    pub fn total_frames(&self) -> u64 {
        self.total_frames_presented
    }

    /// Number of dropped / deadline-exceeded frames.
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }

    /// Computes the presentation timestamp forecast for the next vertical blank.
    pub fn next_vsync_deadline(&self, last_vblank: Duration) -> Duration {
        last_vblank + self.frame_budget
    }
}

impl Default for FramePacer {
    fn default() -> Self {
        Self::new(RefreshRateMode::STANDARD_60)
    }
}
