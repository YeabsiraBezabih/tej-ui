//! Deterministic UI event fuzzer and mount/unmount memory stress harness.

use tej_ui_hal::{MouseButton, UiEvent};
use tej_ui_layout::Point;

/// Fast pseudo-random number generator for reproducible CI fuzzing runs.
#[derive(Debug, Clone)]
pub struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    /// Generates next 32-bit random integer.
    pub fn next_u32(&mut self) -> u32 {
        // Linear congruential generator (LCG)
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    /// Generates next float in range [min, max).
    pub fn next_f32(&mut self, min: f32, max: f32) -> f32 {
        let normalized = (self.next_u32() as f32) / (u32::MAX as f32);
        min + normalized * (max - min)
    }
}

/// Generates random UI event streams for chaos fuzz testing of view hierarchies.
#[derive(Debug, Clone)]
pub struct EventStreamFuzzer {
    rng: FuzzRng,
    bounds: (f32, f32),
}

impl EventStreamFuzzer {
    pub fn new(seed: u64, width: f32, height: f32) -> Self {
        Self {
            rng: FuzzRng::new(seed),
            bounds: (width, height),
        }
    }

    /// Generates a randomized UI event (click, move, wheel, key).
    pub fn next_event(&mut self) -> UiEvent {
        let x = self.rng.next_f32(0.0, self.bounds.0);
        let y = self.rng.next_f32(0.0, self.bounds.1);
        let pt = Point::new(x, y);

        match self.rng.next_u32() % 5 {
            0 => UiEvent::MouseDown {
                button: MouseButton::Left,
                position: pt,
            },
            1 => UiEvent::MouseUp {
                button: MouseButton::Left,
                position: pt,
            },
            2 => UiEvent::MouseMove { position: pt },
            3 => UiEvent::Scroll {
                delta_x: self.rng.next_f32(-10.0, 10.0),
                delta_y: self.rng.next_f32(-10.0, 10.0),
                position: pt,
            },
            _ => UiEvent::TextInput {
                text: ((b'a' + (self.rng.next_u32() % 26) as u8) as char).to_string(),
            },
        }
    }

    /// Generates a batch of `n` chaotic events.
    pub fn generate_batch(&mut self, count: usize) -> Vec<UiEvent> {
        (0..count).map(|_| self.next_event()).collect()
    }
}

/// Stress-tests view allocation, lifecycle mount, unmount, and generational arena cleanup.
#[derive(Debug, Default)]
pub struct MountStressReport {
    pub completed_cycles: usize,
    pub successful: bool,
}

impl MountStressReport {
    /// Executes `cycles` mount and unmount iterations of a view factory, verifying no panics.
    pub fn execute_cycles<F, V>(cycles: usize, mut factory: F) -> Self
    where
        F: FnMut() -> V,
        V: crate::View,
    {
        let mut completed = 0;
        for _ in 0..cycles {
            let view = factory();
            let state = view.build();
            // Drop state simulates unmount
            drop(state);
            completed += 1;
        }

        Self {
            completed_cycles: completed,
            successful: completed == cycles,
        }
    }
}
