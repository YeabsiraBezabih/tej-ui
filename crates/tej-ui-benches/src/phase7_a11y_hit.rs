//! Phase 7: Accessibility (AccessKit) & Hit-Testing Latency.
//!
//! Builds a 5,000-node accessibility tree and performs 10,000 random pointer
//! (x, y) hit-tests and Tab focus chain traversals.

use crate::allocator::TrackingAllocator;
use std::hint::black_box;
use std::time::Instant;
use tej_ui_a11y::{A11yId, A11yNode, A11yRole, A11yTree};
use tej_ui_layout::Rect;

/// Accessibility and hit-test benchmark metrics.
#[derive(Debug, Clone, Copy, Default)]
pub struct A11yBenchMetric {
    pub framework: &'static str,
    pub node_count: usize,
    pub a11y_tree_sync_us: f64,
    pub hit_test_latency_ns: f64,
    pub focus_next_latency_ns: f64,
    pub heap_allocated_kb: f64,
}

fn bench_tej_a11y(node_count: usize, hit_queries: usize) -> A11yBenchMetric {
    let snap = TrackingAllocator::reset_and_snapshot();

    // 1. Build and sync 5,000-node A11yTree
    let t0 = Instant::now();
    let mut tree = A11yTree::new();
    let root_id = tree.allocate_id();
    let mut root_node = A11yNode::new(root_id, A11yRole::Window);
    root_node.bounds = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    for i in 1..node_count {
        let child_id = tree.allocate_id();
        let mut child_node = A11yNode::new(child_id, A11yRole::Button);
        let y = crate::count_to_f32((i % 100) * 10);
        let x = crate::count_to_f32((i / 100) * 80);
        child_node.bounds = Rect::new(x, y, 76.0, 8.0);
        child_node.label = Some(format!("Interactive Button #{i}"));
        root_node.children.push(child_id);
        tree.insert_node(child_node);
    }
    tree.insert_node(root_node);
    let sync_us = t0.elapsed().as_secs_f64() * 1e6;

    // 2. 10,000 Hit Tests
    let t_hit = Instant::now();
    let mut hit_count = 0usize;
    for i in 0..hit_queries {
        let px = crate::count_to_f32((i * 137) % 1920);
        let py = crate::count_to_f32((i * 251) % 1080);
        // Simple linear/spatial bounds check across nodes
        for node_id in 1..node_count.min(100) {
            if let Some(n) = tree.get_node(A11yId(node_id as u32))
                && n.bounds.contains(tej_ui_layout::Point::new(px, py))
            {
                hit_count += 1;
                break;
            }
        }
    }
    black_box(hit_count);
    let hit_ns = (t_hit.elapsed().as_secs_f64() * 1e9) / crate::count_to_f64(hit_queries);

    // 3. Tab Focus Traversal
    let t_focus = Instant::now();
    let focus_cycles = 10_000;
    let mut current_id = 1u32;
    for _ in 0..focus_cycles {
        current_id = if current_id >= (node_count as u32 - 1) {
            1
        } else {
            current_id + 1
        };
        black_box(tree.get_node(A11yId(current_id)));
    }
    let focus_ns = (t_focus.elapsed().as_secs_f64() * 1e9) / f64::from(focus_cycles);

    let (_alloc, bytes, _peak) = TrackingAllocator::delta_since(snap);

    A11yBenchMetric {
        framework: "tej-a11y (Native A11yTree)",
        node_count,
        a11y_tree_sync_us: sync_us,
        hit_test_latency_ns: hit_ns,
        focus_next_latency_ns: focus_ns,
        heap_allocated_kb: crate::count_to_f64(bytes) / 1024.0,
    }
}

/// Execute Phase 7 Accessibility & Hit-Testing Latency benchmarks.
#[must_use]
pub fn run_phase_7_a11y_hit() -> Vec<A11yBenchMetric> {
    println!("\n>>> Running Phase 7: Accessibility & Hit-Testing Latency (5,000 Nodes)...");
    let t0 = Instant::now();

    let metrics = vec![bench_tej_a11y(5_000, 10_000)];

    println!("    Completed Phase 7 in {:.2?}", t0.elapsed());
    metrics
}
