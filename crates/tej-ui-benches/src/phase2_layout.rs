//! Phase 2: Flexbox & Grid Layout Engine Throughput (1,000 to 50,000 Nodes).
//!
//! Evaluates nested enterprise IDE dashboard layout constraint solving across
//! 100 animated window resize steps (1920px -> 1280px).

use crate::allocator::TrackingAllocator;
use std::hint::black_box;
use std::time::Instant;
use taffy::prelude as tf;
use tej_ui_layout::{
    AlignItems, Dimension, FlexDirection, LayoutArena, LayoutEngine, LayoutStyle, Size,
};

/// Metrics recorded for a single layout engine benchmark.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutBenchMetric {
    pub engine_name: &'static str,
    pub node_count: usize,
    pub total_resize_time_us: f64,
    pub us_per_resize_step: f64,
    pub ns_per_layout_node: f64,
    pub allocations_per_resize: usize,
    pub peak_memory_kb: f64,
}

fn bench_tej_layout(node_count: usize, resize_steps: usize) -> LayoutBenchMetric {
    let mut arena = LayoutArena::with_capacity(node_count + 16);
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(1920.0),
        height: Dimension::Points(1080.0),
        direction: FlexDirection::Row,
        ..Default::default()
    });

    let per_col = node_count / 4;
    for _ in 0..4 {
        let col = arena.allocate_node(LayoutStyle {
            flex_grow: 1.0,
            direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..Default::default()
        });
        for _ in 0..per_col {
            let leaf = arena.allocate_node(LayoutStyle {
                height: Dimension::Points(24.0),
                ..Default::default()
            });
            arena.add_child(col, leaf);
        }
        arena.add_child(root, col);
    }

    let snap = TrackingAllocator::reset_and_snapshot();
    let t0 = Instant::now();

    for step in 0..resize_steps {
        let width =
            1920.0 - (crate::count_to_f32(step) * (640.0 / crate::count_to_f32(resize_steps)));
        arena.mark_dirty(root);
        LayoutEngine::compute_layout(&mut arena, root, Size::new(width, 1080.0));
        black_box(arena.computed_bounds[root as usize]);
    }

    let elapsed = t0.elapsed();
    let (alloc_count, _bytes, peak_bytes) = TrackingAllocator::delta_since(snap);
    build_layout_metric(
        "tej-layout (Flat SoA)",
        node_count,
        resize_steps,
        elapsed,
        alloc_count,
        peak_bytes,
    )
}

fn build_layout_metric(
    engine_name: &'static str,
    node_count: usize,
    resize_steps: usize,
    elapsed: std::time::Duration,
    alloc_count: usize,
    peak_bytes: usize,
) -> LayoutBenchMetric {
    let total_us = elapsed.as_secs_f64() * 1e6;
    let steps_f64 = crate::count_to_f64(resize_steps);
    let total_nodes_f64 = crate::count_to_f64(resize_steps * node_count);
    LayoutBenchMetric {
        engine_name,
        node_count,
        total_resize_time_us: total_us,
        us_per_resize_step: total_us / steps_f64,
        ns_per_layout_node: (elapsed.as_secs_f64() * 1e9) / total_nodes_f64,
        allocations_per_resize: alloc_count / resize_steps,
        peak_memory_kb: crate::count_to_f64(peak_bytes) / 1024.0,
    }
}

fn bench_taffy_layout(node_count: usize, resize_steps: usize) -> LayoutBenchMetric {
    let mut tree = tf::TaffyTree::<()>::new();
    let per_col = node_count / 4;
    let mut col_nodes = Vec::with_capacity(4);

    for _ in 0..4 {
        let mut leaf_nodes = Vec::with_capacity(per_col);
        for _ in 0..per_col {
            let leaf = tree
                .new_leaf(tf::Style {
                    size: tf::Size {
                        width: tf::percent(1.0),
                        height: tf::length(24.0),
                    },
                    ..Default::default()
                })
                .unwrap();
            leaf_nodes.push(leaf);
        }
        let col = tree
            .new_with_children(
                tf::Style {
                    flex_direction: tf::FlexDirection::Column,
                    flex_grow: 1.0,
                    ..Default::default()
                },
                &leaf_nodes,
            )
            .unwrap();
        col_nodes.push(col);
    }

    let root = tree
        .new_with_children(
            tf::Style {
                size: tf::Size {
                    width: tf::length(1920.0),
                    height: tf::length(1080.0),
                },
                flex_direction: tf::FlexDirection::Row,
                ..Default::default()
            },
            &col_nodes,
        )
        .unwrap();

    let snap = TrackingAllocator::reset_and_snapshot();
    let t0 = Instant::now();

    for step in 0..resize_steps {
        let width =
            1920.0 - (crate::count_to_f32(step) * (640.0 / crate::count_to_f32(resize_steps)));
        tree.compute_layout(
            root,
            tf::Size {
                width: tf::AvailableSpace::Definite(width),
                height: tf::AvailableSpace::Definite(1080.0),
            },
        )
        .unwrap();
        black_box(tree.layout(root).unwrap());
    }

    let elapsed = t0.elapsed();
    let (alloc_count, _bytes, peak_bytes) = TrackingAllocator::delta_since(snap);
    build_layout_metric(
        "Taffy v0.15 (GPUI/Dioxus)",
        node_count,
        resize_steps,
        elapsed,
        alloc_count,
        peak_bytes,
    )
}

fn bench_egui_layout(node_count: usize, resize_steps: usize) -> LayoutBenchMetric {
    let ctx = egui::Context::default();
    let snap = TrackingAllocator::reset_and_snapshot();
    let t0 = Instant::now();

    let per_col = node_count / 4;
    for step in 0..resize_steps {
        let width =
            1920.0 - (crate::count_to_f32(step) * (640.0 / crate::count_to_f32(resize_steps)));
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(width, 1080.0),
            )),
            ..Default::default()
        };

        let output = ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.columns(4, |cols| {
                    for (c_idx, col) in cols.iter_mut().enumerate() {
                        for row_idx in 0..per_col {
                            col.label(format!("N c{c_idx}r{row_idx}"));
                        }
                    }
                });
            });
        });
        black_box(output.shapes);
    }

    let elapsed = t0.elapsed();
    let (alloc_count, _bytes, peak_bytes) = TrackingAllocator::delta_since(snap);
    build_layout_metric(
        "egui v0.32 Single-Pass",
        node_count,
        resize_steps,
        elapsed,
        alloc_count,
        peak_bytes,
    )
}

/// Execute Phase 2 layout engine benchmarks across 1,000 and 10,000 node scales.
#[must_use]
pub fn run_phase_2_layout_stress() -> Vec<LayoutBenchMetric> {
    println!(
        "\n>>> Running Phase 2: Flexbox & Grid Layout Engine Throughput (100 Resize Steps)..."
    );
    let t0 = Instant::now();

    let resize_steps = 100;
    let metrics = vec![
        bench_tej_layout(10_000, resize_steps),
        bench_taffy_layout(10_000, resize_steps),
        bench_egui_layout(10_000, resize_steps),
    ];

    println!("    Completed Phase 2 in {:.2?}", t0.elapsed());
    metrics
}
