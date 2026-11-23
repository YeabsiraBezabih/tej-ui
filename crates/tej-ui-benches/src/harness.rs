//! Benchmark harness abstractions and frame metrics collection.
//!
//! Enforces identical operational signatures across all tested UI engines.

/// Fine-grained execution metrics per frame across pipeline stages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameMetrics {
    /// Stage A: Reactive state mutation & tree reconciliation / diffing.
    pub state_reconcile_ns: u64,
    /// Stage B: Layout computation (Flexbox/Grid constraint solving).
    pub layout_ns: u64,
    /// Stage C & D: Text shaping, line breaking & paint command / vertex emission.
    pub text_and_paint_ns: u64,
    /// Total wall-clock time across all pipeline stages.
    pub total_frame_ns: u64,
    /// Total heap allocations during the operation.
    pub heap_alloc_count: usize,
    /// Total bytes allocated during the operation.
    pub heap_bytes_allocated: usize,
    /// Peak live heap memory footprint in bytes during the operation.
    pub peak_live_heap_bytes: usize,
}

/// Unified benchmark harness trait implemented by all framework adapters.
pub trait GuiBenchHarness: std::fmt::Debug {
    /// Human-readable framework name.
    fn name(&self) -> &'static str;

    // --- Phase 1: Krausest Standard Table Operations ---

    /// Cold creation, layout, and first paint of `count` rows (4 columns each).
    fn create_rows(&mut self, count: usize) -> FrameMetrics;

    /// Complete dataset replacement of all rows.
    fn replace_all_rows(&mut self, count: usize) -> FrameMetrics;

    /// Mutate the label of every 10th row.
    fn partial_update_every_10th(&mut self) -> FrameMetrics;

    /// Toggle selection/highlight on a single row.
    fn select_single_row(&mut self, index: usize) -> FrameMetrics;

    /// Swap two row indices (e.g. row 2 and row 998).
    fn swap_two_rows(&mut self, idx_a: usize, idx_b: usize) -> FrameMetrics;

    /// Append `count` rows to the existing dataset.
    fn append_rows(&mut self, count: usize) -> FrameMetrics;

    /// Clear all rows and tear down widget state.
    fn clear_all_rows(&mut self) -> FrameMetrics;

    // --- Phase 2: Deep Layout Stress ---

    /// Recompute layout of a deep nested dashboard across a resize step.
    fn layout_deep_dashboard(&mut self, node_count: usize, resize_step: f32) -> FrameMetrics;

    // --- Phase 3: Text Shaping & BiDi Stress ---

    /// Render multilingual document paragraphs with mixed scripts and BiDi runs.
    fn render_multilingual_document(&mut self, paragraphs: &[String]) -> FrameMetrics;

    // --- Phase 4: High-Frequency Dynamic Telemetry (Favors Immediate Mode) ---

    /// Mutate 100% of telemetry nodes (counters, progress, waveform) on every tick.
    fn tick_all_nodes_telemetry(&mut self, active_widgets: usize) -> FrameMetrics;
}
