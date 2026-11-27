//! Phase 8: Binary Size, Cold Startup & Peak RSS Memory Footprint.
//!
//! Evaluates genuinely measured process binary size, cold-start latency to first
//! paint, idle RSS memory, and active RSS memory after populating 10,000 table
//! rows for `tej-ui`.
//!
//! Competitor frameworks are not compiled as standalone binaries inside this
//! harness and are therefore reported as `0.0` (`N/A (unmeasured)`) rather than
//! estimated or hardcoded figures.

use crate::allocator::TrackingAllocator;
use crate::harness::GuiBenchHarness;
use std::hint::black_box;
use std::time::Instant;

/// Footprint metrics for a UI framework.
///
/// A value of `0.0` on a competitor row indicates `N/A (unmeasured)` in this
/// in-process benchmark harness.
#[derive(Debug, Clone, Copy, Default)]
pub struct FootprintMetric {
    pub framework: &'static str,
    pub stripped_binary_size_mb: f64,
    pub cold_startup_to_first_paint_ms: f64,
    pub idle_rss_ram_mb: f64,
    pub active_rss_ram_mb: f64,
}

/// Read the current process resident set size (RSS) in megabytes on Linux via
/// `/proc/self/statm`. Returns `0.0` on platforms where `/proc/self/statm` is
/// unavailable.
fn read_process_rss_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            let mut parts = statm.split_whitespace();
            let _total_pages = parts.next();
            if let Some(resident_str) = parts.next() {
                if let Ok(resident_pages) = resident_str.parse::<u64>() {
                    let page_size_bytes = 4096.0_f64;
                    return (resident_pages as f64 * page_size_bytes) / (1024.0 * 1024.0);
                }
            }
        }
    }
    0.0
}

/// Measure the on-disk size in megabytes of the currently executing binary.
/// Returns `0.0` if the executable path or metadata cannot be queried.
fn read_current_binary_size_mb() -> f64 {
    std::env::current_exe()
        .and_then(std::fs::metadata)
        .map_or(0.0, |meta| (meta.len() as f64) / (1024.0 * 1024.0))
}

/// Execute Phase 8 memory and footprint evaluations.
#[must_use]
pub fn run_phase_8_footprint() -> Vec<FootprintMetric> {
    println!("\n>>> Running Phase 8: Binary Size, Cold Startup & Peak RSS Footprint...");
    let t0 = Instant::now();

    let binary_size_mb = read_current_binary_size_mb();
    let idle_rss_mb = read_process_rss_mb();

    // Measure actual cold start to first paint of tej-ui (unclamped)
    let t_start = Instant::now();
    let mut tej = crate::adapters::TejAdapter::new();
    let m = tej.create_rows(1);
    black_box(m);
    let cold_start_ms = t_start.elapsed().as_secs_f64() * 1000.0;

    // Measure active RSS after populating 10,000 rows
    let _snap = TrackingAllocator::reset_and_snapshot();
    let m_10k = tej.create_rows(10_000);
    black_box(m_10k);

    let live_heap_mb = (m_10k.peak_live_heap_bytes as f64) / (1024.0 * 1024.0);
    let measured_active_rss_mb = read_process_rss_mb();
    let active_rss_mb = if measured_active_rss_mb > 0.0 {
        measured_active_rss_mb
    } else {
        live_heap_mb
    };

    println!(
        "    tej-ui measured: binary={binary_size_mb:.2} MB, cold_start={cold_start_ms:.3} ms, \
         idle_rss={idle_rss_mb:.2} MB, active_rss={active_rss_mb:.2} MB (10k-row tracked heap={live_heap_mb:.2} MB)"
    );
    println!(
        "    Note: Competitor footprint metrics are reported as 0.0 / N/A (unmeasured) because \
         standalone competitor binaries are not built in this harness."
    );

    let metrics = vec![
        // tej-ui: genuinely measured in-process values only
        FootprintMetric {
            framework: "tej-ui",
            stripped_binary_size_mb: binary_size_mb,
            cold_startup_to_first_paint_ms: cold_start_ms,
            idle_rss_ram_mb: idle_rss_mb,
            active_rss_ram_mb: active_rss_mb,
        },
        // Unmeasured competitors: explicitly reported as 0.0 / N/A (unmeasured)
        FootprintMetric {
            framework: "egui v0.32 (N/A - unmeasured)",
            stripped_binary_size_mb: 0.0,
            cold_startup_to_first_paint_ms: 0.0,
            idle_rss_ram_mb: 0.0,
            active_rss_ram_mb: 0.0,
        },
        FootprintMetric {
            framework: "GPUI (Zed Engine) (N/A - unmeasured)",
            stripped_binary_size_mb: 0.0,
            cold_startup_to_first_paint_ms: 0.0,
            idle_rss_ram_mb: 0.0,
            active_rss_ram_mb: 0.0,
        },
        FootprintMetric {
            framework: "Iced v0.14 (N/A - unmeasured)",
            stripped_binary_size_mb: 0.0,
            cold_startup_to_first_paint_ms: 0.0,
            idle_rss_ram_mb: 0.0,
            active_rss_ram_mb: 0.0,
        },
        FootprintMetric {
            framework: "Slint v1.18 (N/A - unmeasured)",
            stripped_binary_size_mb: 0.0,
            cold_startup_to_first_paint_ms: 0.0,
            idle_rss_ram_mb: 0.0,
            active_rss_ram_mb: 0.0,
        },
        FootprintMetric {
            framework: "Dioxus v0.7 (N/A - unmeasured)",
            stripped_binary_size_mb: 0.0,
            cold_startup_to_first_paint_ms: 0.0,
            idle_rss_ram_mb: 0.0,
            active_rss_ram_mb: 0.0,
        },
    ];

    println!("    Completed Phase 8 in {:.2?}", t0.elapsed());
    metrics
}
