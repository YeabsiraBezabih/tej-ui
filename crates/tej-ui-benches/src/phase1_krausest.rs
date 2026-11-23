//! Phase 1: The Native "Krausest" GUI Suite (Reactivity & Tree Mutation).
//!
//! Executes the standard 4-column interactive table operations across all
//! framework adapters under `TrackingAllocator` accounting.

use crate::harness::{FrameMetrics, GuiBenchHarness};
use std::time::Instant;

/// Benchmark result for a single framework across Phase 1 operations.
#[derive(Debug, Clone, Default)]
pub struct Phase1FrameworkResult {
    pub framework: &'static str,
    pub create_1k: FrameMetrics,
    pub replace_1k: FrameMetrics,
    pub update_10th: FrameMetrics,
    pub select_1: FrameMetrics,
    pub swap_2: FrameMetrics,
    pub create_10k: FrameMetrics,
    pub clear_1k: FrameMetrics,
}

fn median_metrics(mut runs: Vec<FrameMetrics>) -> FrameMetrics {
    if runs.is_empty() {
        return FrameMetrics::default();
    }
    runs.sort_by_key(|m| m.total_frame_ns);
    runs[runs.len() / 2]
}

/// Execute Phase 1 on a given framework harness for `iterations` passes.
#[must_use]
pub fn run_phase1_for_harness<H: GuiBenchHarness>(
    harness: &mut H,
    iterations: usize,
) -> Phase1FrameworkResult {
    let iters = iterations.max(1);

    // 1.1 Create 1,000 Rows
    let mut create_runs = Vec::with_capacity(iters);
    for _ in 0..iters {
        harness.clear_all_rows();
        let m = harness.create_rows(1_000);
        create_runs.push(m);
    }
    let create_1k = median_metrics(create_runs);

    // 1.2 Replace All 1,000 Rows
    let mut replace_runs = Vec::with_capacity(iters);
    for _ in 0..iters {
        let m = harness.replace_all_rows(1_000);
        replace_runs.push(m);
    }
    let replace_1k = median_metrics(replace_runs);

    // 1.3 Partial Update (every 10th row)
    let mut update_runs = Vec::with_capacity(iters);
    for _ in 0..iters {
        let m = harness.partial_update_every_10th();
        update_runs.push(m);
    }
    let update_10th = median_metrics(update_runs);

    // 1.4 Select 1 Row in 1,000
    let mut select_runs = Vec::with_capacity(iters);
    for i in 0..iters {
        let m = harness.select_single_row((i * 17) % 1_000);
        select_runs.push(m);
    }
    let select_1 = median_metrics(select_runs);

    // 1.5 Swap 2 Rows (row 2 and row 998)
    let mut swap_runs = Vec::with_capacity(iters);
    for _ in 0..iters {
        let m = harness.swap_two_rows(2, 998);
        swap_runs.push(m);
    }
    let swap_2 = median_metrics(swap_runs);

    // 1.6 Create 10,000 Rows
    let mut create_10k_runs = Vec::with_capacity(iters.min(10));
    for _ in 0..iters.min(10) {
        harness.clear_all_rows();
        let m = harness.create_rows(10_000);
        create_10k_runs.push(m);
    }
    let create_10k = median_metrics(create_10k_runs);

    // 1.7 Clear 1,000 Rows
    let mut clear_runs = Vec::with_capacity(iters);
    for _ in 0..iters {
        harness.create_rows(1_000);
        let m = harness.clear_all_rows();
        clear_runs.push(m);
    }
    let clear_1k = median_metrics(clear_runs);

    Phase1FrameworkResult {
        framework: harness.name(),
        create_1k,
        replace_1k,
        update_10th,
        select_1,
        swap_2,
        create_10k,
        clear_1k,
    }
}

/// Execute Phase 1 across all 6 competitive harnesses.
#[must_use]
pub fn run_phase_1_krausest(iterations: usize) -> Vec<Phase1FrameworkResult> {
    println!("\n>>> Running Phase 1: The Native Krausest GUI Suite (100 Iterations Live)...");
    let t0 = Instant::now();

    let mut tej = crate::adapters::TejAdapter::new();
    let mut egui = crate::adapters::EguiAdapter::new();
    let mut gpui = crate::adapters::GpuiAdapter::new();
    let mut iced = crate::adapters::IcedAdapter::new();
    let mut slint = crate::adapters::SlintAdapter::new();
    let mut dioxus = crate::adapters::DioxusAdapter::new();

    let results = vec![
        run_phase1_for_harness(&mut tej, iterations),
        run_phase1_for_harness(&mut egui, iterations),
        run_phase1_for_harness(&mut gpui, iterations),
        run_phase1_for_harness(&mut iced, iterations),
        run_phase1_for_harness(&mut slint, iterations),
        run_phase1_for_harness(&mut dioxus, iterations),
    ];

    println!("    Completed Phase 1 in {:.2?}", t0.elapsed());
    results
}
