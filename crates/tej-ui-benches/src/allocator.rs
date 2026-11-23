//! Deterministic global allocation and heap memory tracking.
//!
//! Wraps `std::alloc::System` to record exact heap allocation count, bytes
//! allocated, active live bytes, and peak heap footprint across benchmark phases.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom allocator wrapping the system allocator with atomic instrumentation.
#[derive(Debug, Default)]
pub struct TrackingAllocator;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static CURRENT_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

/// Snapshot of heap allocation state at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocSnapshot {
    /// Total cumulative allocation calls.
    pub alloc_count: usize,
    /// Total cumulative deallocation calls.
    pub dealloc_count: usize,
    /// Total cumulative bytes requested via allocation.
    pub bytes_allocated: usize,
    /// Current live heap memory in bytes.
    pub live_bytes: usize,
    /// Peak live heap memory in bytes.
    pub peak_live_bytes: usize,
}

impl TrackingAllocator {
    /// Reset the peak measurement and capture the current allocation baseline.
    #[must_use]
    pub fn reset_and_snapshot() -> AllocSnapshot {
        let live = CURRENT_LIVE_BYTES.load(Ordering::Acquire);
        PEAK_LIVE_BYTES.store(live, Ordering::Release);
        AllocSnapshot {
            alloc_count: ALLOC_COUNT.load(Ordering::Acquire),
            dealloc_count: DEALLOC_COUNT.load(Ordering::Acquire),
            bytes_allocated: BYTES_ALLOCATED.load(Ordering::Acquire),
            live_bytes: live,
            peak_live_bytes: live,
        }
    }

    /// Compute relative allocation deltas (alloc_count, bytes_allocated, peak_live_bytes)
    /// since the reference snapshot was taken.
    #[must_use]
    pub fn delta_since(start: AllocSnapshot) -> (usize, usize, usize) {
        let count = ALLOC_COUNT
            .load(Ordering::Acquire)
            .saturating_sub(start.alloc_count);
        let bytes = BYTES_ALLOCATED
            .load(Ordering::Acquire)
            .saturating_sub(start.bytes_allocated);
        let current_peak = PEAK_LIVE_BYTES.load(Ordering::Acquire);
        let peak = current_peak.saturating_sub(start.live_bytes);
        (count, bytes, peak)
    }

    /// Read the current live bytes count.
    #[must_use]
    pub fn current_live_bytes() -> usize {
        CURRENT_LIVE_BYTES.load(Ordering::Acquire)
    }
}

// SAFETY: Forwards allocations to `System` while maintaining accurate atomic counters.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Delegating to `System::alloc` with the requested layout.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let size = layout.size();
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            BYTES_ALLOCATED.fetch_add(size, Ordering::Relaxed);
            let prev = CURRENT_LIVE_BYTES.fetch_add(size, Ordering::Relaxed);
            PEAK_LIVE_BYTES.fetch_max(prev + size, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Delegating to `System::dealloc` with pointer and layout.
        unsafe { System.dealloc(ptr, layout) };
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        CURRENT_LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator;
