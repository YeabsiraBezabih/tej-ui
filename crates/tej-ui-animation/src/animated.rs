//! Reactive animated signals driven by spring physics.

use crate::interpolator::Lerp;
use crate::spring::{SpringConfig, SpringSimulation};
use tej_ui_core::Signal;

/// Reactive numeric signal seamlessly updated by spring physics integration.
#[derive(Clone, Copy, Debug)]
pub struct AnimatedSpring {
    /// Reactive signal containing the current smoothed numerical value.
    signal: Signal<f32>,
    /// Underlying physical simulation model.
    sim: SpringSimulation,
}

impl AnimatedSpring {
    /// Create a new animated spring signal initialized at rest at `initial_value`.
    #[must_use]
    pub fn new(initial_value: f32, config: SpringConfig) -> Self {
        Self {
            signal: Signal::new(initial_value),
            sim: SpringSimulation::new(initial_value, config),
        }
    }

    /// Reactive signal holding the current interpolated value.
    #[must_use]
    pub const fn signal(&self) -> Signal<f32> {
        self.signal
    }

    /// Read current instantaneous value.
    #[must_use]
    pub fn get(&self) -> f32 {
        self.signal.get()
    }

    /// Target equilibrium value.
    #[must_use]
    pub fn target(&self) -> f32 {
        self.sim.target
    }

    /// Retarget spring equilibrium position.
    pub fn set_target(&mut self, new_target: f32) {
        self.sim.set_target(new_target);
    }

    /// Check if the spring is at rest at its target.
    #[must_use]
    pub fn is_at_rest(&self) -> bool {
        self.sim.is_at_rest()
    }

    /// Advance spring simulation by delta time `dt` seconds and update the reactive signal.
    pub fn step(&mut self, dt: f32) -> f32 {
        let val = self.sim.step(dt);
        self.signal.set(val);
        val
    }
}

/// Generic reactive signal animating any `Lerp` type (Points, Rectangles, Colors) via spring progress.
#[derive(Clone, Debug)]
pub struct AnimatedValue<T: Lerp + Clone + Send + Sync + 'static> {
    signal: Signal<T>,
    start_value: T,
    target_value: T,
    progress: SpringSimulation,
}

impl<T: Lerp + Clone + Send + Sync + 'static> AnimatedValue<T> {
    /// Create a new animated value with specified spring configuration.
    #[must_use]
    pub fn new(initial: T, config: SpringConfig) -> Self {
        Self {
            signal: Signal::new(initial.clone()),
            start_value: initial.clone(),
            target_value: initial,
            progress: SpringSimulation::new(1.0, config),
        }
    }

    /// Reactive signal holding current interpolated value.
    #[must_use]
    pub const fn signal(&self) -> Signal<T> {
        self.signal
    }

    /// Current interpolated value.
    pub fn get(&self) -> T {
        self.signal.get()
    }

    /// Target value towards which the signal is animating.
    #[must_use]
    pub fn target(&self) -> &T {
        &self.target_value
    }

    /// Retarget to new value, smoothly beginning a new spring transition from the current state.
    pub fn set_target(&mut self, new_target: T) {
        self.start_value = self.get();
        self.target_value = new_target;
        self.progress.current = 0.0;
        self.progress.target = 1.0;
        self.progress.velocity = 0.0;
    }

    /// Check if the value animation has completed and is at rest.
    #[must_use]
    pub fn is_at_rest(&self) -> bool {
        self.progress.is_at_rest()
    }

    /// Advance animation by `dt` seconds and update signal.
    pub fn step(&mut self, dt: f32) -> T {
        let t = self.progress.step(dt).clamp(0.0, 1.0);
        let interpolated = self.start_value.lerp(&self.target_value, t);
        self.signal.set(interpolated.clone());
        interpolated
    }
}
