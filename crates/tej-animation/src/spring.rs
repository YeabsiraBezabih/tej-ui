//! Spring physics configuration and numerical integrator.

/// Physical parameters governing spring oscillation and damping behavior.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SpringConfig {
    /// Mass of the spring object in kg (typically 1.0).
    pub mass: f32,
    /// Spring stiffness constant k in N/m. Higher values result in faster, stiffer oscillation.
    pub stiffness: f32,
    /// Damping coefficient c in N·s/m. Controls how quickly oscillations decay.
    pub damping: f32,
    /// Velocity threshold under which movement is considered at rest (default 0.001).
    pub rest_speed_threshold: f32,
    /// Distance threshold under which displacement is considered at rest (default 0.001).
    pub rest_displacement_threshold: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::snappy()
    }
}

impl SpringConfig {
    /// Create a new spring configuration with custom physical parameters.
    #[must_use]
    pub const fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            mass,
            stiffness,
            damping,
            rest_speed_threshold: 0.001,
            rest_displacement_threshold: 0.001,
        }
    }

    /// Fast and responsive spring with minimal overshoot. Ideal for UI buttons and modals.
    #[must_use]
    pub const fn snappy() -> Self {
        Self {
            mass: 1.0,
            stiffness: 180.0,
            damping: 24.0,
            rest_speed_threshold: 0.001,
            rest_displacement_threshold: 0.001,
        }
    }

    /// Playful spring with visible bounce and overshoot. Great for badges and celebratory toggles.
    #[must_use]
    pub const fn bouncy() -> Self {
        Self {
            mass: 1.0,
            stiffness: 120.0,
            damping: 14.0,
            rest_speed_threshold: 0.001,
            rest_displacement_threshold: 0.001,
        }
    }

    /// Soft, critically damped spring with no overshoot. Ideal for page transitions and drawer sheets.
    #[must_use]
    pub const fn gentle() -> Self {
        Self {
            mass: 1.0,
            stiffness: 100.0,
            damping: 20.0,
            rest_speed_threshold: 0.001,
            rest_displacement_threshold: 0.001,
        }
    }

    /// Damping ratio $\zeta = \frac{c}{2\sqrt{km}}$.
    /// - $\zeta < 1.0$: Underdamped (oscillates with overshoot).
    /// - $\zeta = 1.0$: Critically damped (fastest return to equilibrium without oscillating).
    /// - $\zeta > 1.0$: Overdamped (sluggish decay without oscillating).
    #[must_use]
    pub fn damping_ratio(&self) -> f32 {
        self.damping / (2.0 * (self.mass * self.stiffness).sqrt())
    }

    /// Undamped angular frequency $\omega_n = \sqrt{k/m}$ in radians per second.
    #[must_use]
    pub fn natural_frequency(&self) -> f32 {
        (self.stiffness / self.mass).sqrt()
    }
}

/// Dynamic spring simulation tracking position, target, and velocity over time.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SpringSimulation {
    /// Active spring physics configuration.
    pub config: SpringConfig,
    /// Current numerical position.
    pub current: f32,
    /// Desired equilibrium target position.
    pub target: f32,
    /// Current instantaneous velocity in units per second.
    pub velocity: f32,
}

impl SpringSimulation {
    /// Create a new spring simulation initialized at rest at `initial_value`.
    #[must_use]
    pub const fn new(initial_value: f32, config: SpringConfig) -> Self {
        Self {
            config,
            current: initial_value,
            target: initial_value,
            velocity: 0.0,
        }
    }

    /// Update target equilibrium value, allowing continuous retargeting without popping.
    pub fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    /// Check if the spring has settled at equilibrium.
    #[must_use]
    pub fn is_at_rest(&self) -> bool {
        let displacement = (self.current - self.target).abs();
        let speed = self.velocity.abs();
        displacement < self.config.rest_displacement_threshold
            && speed < self.config.rest_speed_threshold
    }

    /// Advance the simulation by delta time `dt` seconds using symplectic Euler integration.
    pub fn step(&mut self, dt: f32) -> f32 {
        if self.is_at_rest() {
            self.current = self.target;
            self.velocity = 0.0;
            return self.current;
        }

        // Substep large delta times (e.g. dropped frames) to prevent numerical divergence
        let max_dt = 1.0 / 60.0;
        let mut remaining = dt;

        while remaining > 0.0 {
            let step_dt = remaining.min(max_dt);
            let displacement = self.current - self.target;
            let spring_force = -self.config.stiffness * displacement;
            let damping_force = -self.config.damping * self.velocity;
            let acceleration = (spring_force + damping_force) / self.config.mass;

            // Semi-implicit Euler integration preserves energy characteristics
            self.velocity += acceleration * step_dt;
            self.current += self.velocity * step_dt;

            remaining -= step_dt;
        }

        if self.is_at_rest() {
            self.current = self.target;
            self.velocity = 0.0;
        }

        self.current
    }
}
