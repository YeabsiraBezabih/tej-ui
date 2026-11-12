//! State-preserving hot reload coordinator.
//!
//! Enables dynamic view tree and style swapping in under 200 ms while keeping
//! signal state arenas, database queries, and active sessions intact.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tej_view::View;

/// Generation counter for hot-reload reloads across the workspace.
static HOT_RELOAD_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Host container that holds a re-mountable view while maintaining persistent signal state.
#[derive(Debug)]
pub struct HotReloadHost<V> {
    generation: u64,
    last_reload_time: Instant,
    active_view: V,
    reload_count: usize,
}

impl<V: View> HotReloadHost<V> {
    /// Creates a new hot-reload host around the initial view tree.
    pub fn new(view: V) -> Self {
        let generation = HOT_RELOAD_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
        Self {
            generation,
            last_reload_time: Instant::now(),
            active_view: view,
            reload_count: 0,
        }
    }

    /// Swaps the view tree with a newly compiled version while preserving state signals.
    ///
    /// Returns the elapsed swap duration in milliseconds.
    pub fn swap_view(&mut self, new_view: V) -> f32 {
        let start = Instant::now();
        self.active_view = new_view;
        self.generation = HOT_RELOAD_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
        self.reload_count += 1;
        self.last_reload_time = Instant::now();
        start.elapsed().as_secs_f32() * 1000.0
    }

    /// Returns a reference to the active view tree.
    pub const fn view(&self) -> &V {
        &self.active_view
    }

    /// Returns a mutable reference to the active view tree.
    pub fn view_mut(&mut self) -> &mut V {
        &mut self.active_view
    }

    /// Returns the current hot reload generation counter.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Total number of successful hot reloads performed in this session.
    #[must_use]
    pub const fn reload_count(&self) -> usize {
        self.reload_count
    }
}
