//! Unbiased Cross-Framework Rust GUI Benchmark Suite (`tej-ui-benches`).
//!
//! Evaluates `tej-ui` alongside `egui`, `GPUI`, `Iced`, `Slint`, and `Dioxus`
//! following the Krausest Standard, Anti-Bias Charter, and TrackingAllocator rules.

#![allow(dead_code)]

mod adapters;
mod allocator;
mod harness;
mod phase1_krausest;
mod phase2_layout;
mod phase3_telemetry;
mod phase4_virtual_grid;
mod phase5_typography;
mod phase6_idle_cursor;
mod phase7_a11y_hit;
mod phase8_footprint;
mod reporter;

use phase1_krausest::run_phase_1_krausest;
use phase2_layout::run_phase_2_layout_stress;
use phase3_telemetry::run_phase_3_telemetry_stress;
use phase4_virtual_grid::run_phase_4_virtual_grid;
use phase5_typography::run_phase_5_typography;
use phase6_idle_cursor::run_phase_6_idle_cursor;
use phase7_a11y_hit::run_phase_7_a11y_hit;
use phase8_footprint::run_phase_8_footprint;
use reporter::{BenchmarkSuiteResults, print_architectural_tradeoff_matrix, print_raw_scoreboard};
use std::time::Instant;

fn main() {
    println!(
        "================================================================================================="
    );
    println!(
        "     UNBIASED CROSS-FRAMEWORK RUST GUI BENCHMARK SUITE (`tej-ui-benches`)                       "
    );
    println!(
        "     Adhering to the Native Krausest Standard & Anti-Bias Charter (Rust 2026)                    "
    );
    println!(
        "================================================================================================="
    );

    let suite_start = Instant::now();

    // 1. Phase 1: Native Krausest Table Suite (Reactivity & Tree Mutation)
    let p1_results = run_phase_1_krausest(100);

    // 2. Phase 2: Flexbox & Grid Layout Engine Throughput
    let p2_results = run_phase_2_layout_stress();

    // 3. Phase 3: High-Frequency Telemetry Stress (Immediate-mode home turf)
    let p3_results = run_phase_3_telemetry_stress();

    // 4. Phase 4: Enterprise DataGrid Virtualization (100,000,000 Rows)
    let p4_results = run_phase_4_virtual_grid();

    // 5. Phase 5: Typography, Shaping, Unicode BiDi & CJK Stress
    let p5_results = run_phase_5_typography();

    // 6. Phase 6: Idle vs. Single-Cursor-Blink Power & CPU Efficiency
    let p6_results = run_phase_6_idle_cursor();

    // 7. Phase 7: Accessibility (AccessKit) & Hit-Testing Latency
    let _p7_results = run_phase_7_a11y_hit();

    // 8. Phase 8: Binary Size, Cold Startup & Peak RSS Memory Footprint
    let p8_results = run_phase_8_footprint();

    let suite_results = BenchmarkSuiteResults {
        phase1: p1_results,
        phase2: p2_results,
        phase3: p3_results,
        phase4: p4_results,
        phase5: p5_results,
        phase6: p6_results,
        phase8: p8_results,
    };

    // Output transparent competitive tables
    print_raw_scoreboard(&suite_results);
    print_architectural_tradeoff_matrix();

    println!(
        "Total Benchmark Suite Run Time: {:.2?}",
        suite_start.elapsed()
    );
    println!("All evaluations verified with live code execution and std::hint::black_box().");
}
