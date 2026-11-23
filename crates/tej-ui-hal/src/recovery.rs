//! GPU device-loss recovery and software rasterizer fallback state machine.

/// Represents the health state of the GPU render device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceHealth {
    /// Device is functioning normally.
    Healthy,
    /// Device was lost (driver reset, sleep/wake, TDR).
    DeviceLost,
    /// Currently recreating graphics pipelines and swapchain.
    Recovering,
    /// Fell back permanently or temporarily to software CPU rasterization.
    SoftwareFallback,
}

/// Token proving swapchain recreation and pipeline invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryToken {
    pub generation: u64,
}

/// Coordinates GPU device loss events, recovery attempts, and CPU fallback.
#[derive(Debug, Clone)]
pub struct DeviceLossCoordinator {
    health: DeviceHealth,
    recovery_generation: u64,
    consecutive_failures: u32,
    max_recovery_attempts: u32,
}

impl DeviceLossCoordinator {
    /// Creates a new device loss coordinator with a maximum number of recovery retries before falling back.
    pub fn new(max_recovery_attempts: u32) -> Self {
        Self {
            health: DeviceHealth::Healthy,
            recovery_generation: 0,
            consecutive_failures: 0,
            max_recovery_attempts,
        }
    }

    /// Current health of the device.
    pub fn health(&self) -> DeviceHealth {
        self.health
    }

    /// True if currently executing under software fallback mode.
    pub fn is_software_fallback(&self) -> bool {
        self.health == DeviceHealth::SoftwareFallback
    }

    /// Notifies the coordinator that a GPU device loss was detected.
    pub fn notify_device_lost(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures > self.max_recovery_attempts {
            self.health = DeviceHealth::SoftwareFallback;
        } else {
            self.health = DeviceHealth::DeviceLost;
        }
    }

    /// Initiates a recovery attempt to re-initialize GPU swapchain and resources.
    pub fn begin_recovery(&mut self) -> Option<RecoveryToken> {
        if self.health == DeviceHealth::SoftwareFallback {
            return None;
        }

        self.health = DeviceHealth::Recovering;
        self.recovery_generation += 1;
        Some(RecoveryToken {
            generation: self.recovery_generation,
        })
    }

    /// Marks recovery as successfully completed.
    pub fn complete_recovery(&mut self, token: RecoveryToken) -> bool {
        if self.health == DeviceHealth::Recovering && self.recovery_generation == token.generation {
            self.health = DeviceHealth::Healthy;
            self.consecutive_failures = 0;
            true
        } else {
            false
        }
    }

    /// Forces immediate fallback to software CPU rendering.
    pub fn force_software_fallback(&mut self) {
        self.health = DeviceHealth::SoftwareFallback;
    }

    /// Current recovery generation counter.
    pub fn current_generation(&self) -> u64 {
        self.recovery_generation
    }
}

impl Default for DeviceLossCoordinator {
    fn default() -> Self {
        Self::new(3)
    }
}
