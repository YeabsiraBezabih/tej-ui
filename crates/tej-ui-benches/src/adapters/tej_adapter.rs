//! `tej-ui` framework adapter.
//!
//! Uses generational arena `Copy` signals, flat SoA `LayoutArena`,
//! zero-copy memory models, and dirty-region compositing.

use crate::allocator::TrackingAllocator;
use crate::harness::{FrameMetrics, GuiBenchHarness};
use std::hint::black_box;
use std::time::Instant;
use tej_ui_core::Signal;
use tej_ui_hal::DirtyRectTracker;
use tej_ui_layout::{
    AlignItems, Dimension, FlexDirection, LayoutArena, LayoutEngine, LayoutStyle, Rect, Size,
};
use tej_ui_text::{SystemFont, TextDirection, TextShaper};

#[inline]
fn format_krausest_row_label(i: usize) -> String {
    format!("Row item #{i} - Krausest table test string")
}

#[inline]
fn format_replaced_row_label(i: usize) -> String {
    format!("Replaced row #{i} - Newly updated table string payload")
}

#[derive(Debug, Clone)]
struct TejTableRow {
    id: usize,
    label: Signal<String>,
    selected: Signal<bool>,
    node_id: u32,
}

/// Adapter executing `tej-ui` native engine pipelines.
#[derive(Debug)]
pub struct TejAdapter {
    arena: LayoutArena,
    root: u32,
    rows: Vec<TejTableRow>,
    dirty_tracker: DirtyRectTracker,
    shaper: TextShaper,
}

impl Default for TejAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl TejAdapter {
    #[must_use]
    pub fn new() -> Self {
        let mut arena = LayoutArena::with_capacity(1024);
        let root = arena.allocate_node(LayoutStyle {
            width: Dimension::Points(1920.0),
            height: Dimension::Points(1080.0),
            direction: FlexDirection::Column,
            ..Default::default()
        });
        Self {
            arena,
            root,
            rows: Vec::new(),
            dirty_tracker: DirtyRectTracker::new(),
            shaper: TextShaper::new(),
        }
    }

    fn rebuild_layout_arena(&mut self) -> (u64, u64) {
        let t_recon_start = Instant::now();
        self.arena.clear();
        self.arena.reserve(self.rows.len() * 5 + 1);
        self.root = self.arena.allocate_node(LayoutStyle {
            width: Dimension::Points(1920.0),
            height: Dimension::Points(1080.0),
            direction: FlexDirection::Column,
            ..Default::default()
        });

        let col_styles = [
            LayoutStyle {
                width: Dimension::Points(80.0),
                height: Dimension::Points(24.0),
                ..Default::default()
            },
            LayoutStyle {
                width: Dimension::Points(600.0),
                height: Dimension::Points(24.0),
                ..Default::default()
            },
            LayoutStyle {
                width: Dimension::Points(80.0),
                height: Dimension::Points(24.0),
                ..Default::default()
            },
            LayoutStyle {
                width: Dimension::Points(60.0),
                height: Dimension::Points(24.0),
                ..Default::default()
            },
        ];

        let row_style = LayoutStyle {
            width: Dimension::Percent(100.0),
            height: Dimension::Points(36.0),
            direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..Default::default()
        };

        let row_nodes =
            self.arena
                .bulk_allocate_table_rows(self.root, self.rows.len(), row_style, &col_styles);
        for (i, &node_id) in row_nodes.iter().enumerate() {
            self.rows[i].node_id = node_id;
        }
        let recon_ns = t_recon_start.elapsed().as_nanos() as u64;

        let t_layout_start = Instant::now();
        LayoutEngine::compute_layout(&mut self.arena, self.root, Size::new(1920.0, 1080.0));
        let layout_ns = t_layout_start.elapsed().as_nanos() as u64;

        (recon_ns, layout_ns)
    }

    fn emit_dirty_rect_paint(&mut self) -> u64 {
        let t_paint_start = Instant::now();
        let dirty_bounds = self.dirty_tracker.rects();
        black_box(dirty_bounds);
        self.dirty_tracker.reset();
        t_paint_start.elapsed().as_nanos() as u64
    }
}

impl GuiBenchHarness for TejAdapter {
    fn name(&self) -> &'static str {
        "tej-ui"
    }

    fn create_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        tej_ui_core::ReactiveRuntime::reserve_signals(count * 2);
        self.rows.clear();
        self.rows.reserve(count);
        for i in 0..count {
            self.rows.push(TejTableRow {
                id: i,
                label: Signal::new(format_krausest_row_label(i)),
                selected: Signal::new(false),
                node_id: 0,
            });
        }

        let (recon_ns, layout_ns) = self.rebuild_layout_arena();
        self.dirty_tracker.mark_full_dirty();
        let paint_ns = self.emit_dirty_rect_paint();

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: recon_ns,
            layout_ns,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn replace_all_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        tej_ui_core::ReactiveRuntime::reserve_signals(count * 2);
        self.rows.clear();
        self.rows.reserve(count);
        for i in 0..count {
            self.rows.push(TejTableRow {
                id: i + 10_000,
                label: Signal::new(format_replaced_row_label(i)),
                selected: Signal::new(false),
                node_id: 0,
            });
        }

        let (recon_ns, layout_ns) = self.rebuild_layout_arena();
        self.dirty_tracker.mark_full_dirty();
        let paint_ns = self.emit_dirty_rect_paint();

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: recon_ns,
            layout_ns,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn partial_update_every_10th(&mut self) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_recon_start = Instant::now();
        for i in (0..self.rows.len()).step_by(10) {
            let row = &self.rows[i];
            let mut val = row.label.get();
            val.push_str(" [MUTATED]");
            row.label.set(val);

            // Invalidate only the fine-grained dirty bounding rect
            let y = (i as f32 * 36.0).min(1080.0);
            self.dirty_tracker.add_rect(Rect::new(0.0, y, 1920.0, 36.0));
        }
        let recon_ns = t_recon_start.elapsed().as_nanos() as u64;

        // Zero layout recomputation needed because row height is static
        let paint_ns = self.emit_dirty_rect_paint();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: recon_ns,
            layout_ns: 0,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn select_single_row(&mut self, index: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_recon_start = Instant::now();
        if let Some(row) = self.rows.get(index) {
            let cur = row.selected.get();
            row.selected.set(!cur);
            let y = (index as f32 * 36.0).min(1080.0);
            self.dirty_tracker.add_rect(Rect::new(0.0, y, 1920.0, 36.0));
        }
        let recon_ns = t_recon_start.elapsed().as_nanos() as u64;

        let paint_ns = self.emit_dirty_rect_paint();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: recon_ns,
            layout_ns: 0,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn swap_two_rows(&mut self, idx_a: usize, idx_b: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_recon_start = Instant::now();
        if idx_a < self.rows.len() && idx_b < self.rows.len() {
            self.rows.swap(idx_a, idx_b);
            let y_a = (idx_a as f32 * 36.0).min(1080.0);
            let y_b = (idx_b as f32 * 36.0).min(1080.0);
            self.dirty_tracker
                .add_rect(Rect::new(0.0, y_a, 1920.0, 36.0));
            self.dirty_tracker
                .add_rect(Rect::new(0.0, y_b, 1920.0, 36.0));
        }
        let recon_ns = t_recon_start.elapsed().as_nanos() as u64;

        let paint_ns = self.emit_dirty_rect_paint();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: recon_ns,
            layout_ns: 0,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn append_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let base = self.rows.len();
        self.rows.extend((0..count).map(|i| TejTableRow {
            id: base + i,
            label: Signal::new(format!(
                "Appended row #{} - Extra table item payload",
                base + i
            )),
            selected: Signal::new(false),
            node_id: 0,
        }));

        let (recon_ns, layout_ns) = self.rebuild_layout_arena();
        self.dirty_tracker.mark_full_dirty();
        let paint_ns = self.emit_dirty_rect_paint();

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: recon_ns,
            layout_ns,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn clear_all_rows(&mut self) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        self.rows.clear();
        self.arena.clear();
        self.arena.compact();
        self.root = self.arena.allocate_node(LayoutStyle {
            width: Dimension::Points(1920.0),
            height: Dimension::Points(1080.0),
            direction: FlexDirection::Column,
            ..Default::default()
        });

        self.dirty_tracker.mark_full_dirty();
        let paint_ns = self.emit_dirty_rect_paint();

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: 0,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn layout_deep_dashboard(&mut self, node_count: usize, resize_step: f32) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let width = 1920.0 - resize_step;
        let mut local_arena = LayoutArena::with_capacity(node_count + 5);
        let dash_root = local_arena.allocate_node(LayoutStyle {
            width: Dimension::Points(width),
            height: Dimension::Points(1080.0),
            direction: FlexDirection::Row,
            ..Default::default()
        });

        let per_col = node_count / 4;
        for _ in 0..4 {
            let col = local_arena.allocate_node(LayoutStyle {
                flex_grow: 1.0,
                direction: FlexDirection::Column,
                ..Default::default()
            });
            for _ in 0..per_col {
                let leaf = local_arena.allocate_node(LayoutStyle {
                    height: Dimension::Points(24.0),
                    ..Default::default()
                });
                local_arena.add_child(col, leaf);
            }
            local_arena.add_child(dash_root, col);
        }

        let t_layout_start = Instant::now();
        LayoutEngine::compute_layout(&mut local_arena, dash_root, Size::new(width, 1080.0));
        let layout_ns = t_layout_start.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns,
            text_and_paint_ns: 0,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn render_multilingual_document(&mut self, paragraphs: &[String]) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_shape_start = Instant::now();
        let mut total_glyphs = 0usize;
        for p in paragraphs {
            let run = self
                .shaper
                .shape(p, SystemFont::Body, 14.0, TextDirection::LeftToRight);
            total_glyphs += run.glyphs.len();
        }
        black_box(total_glyphs);
        let shape_ns = t_shape_start.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: 0,
            text_and_paint_ns: shape_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn tick_all_nodes_telemetry(&mut self, active_widgets: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let count = active_widgets / 3;
        for i in 0..count {
            let val = ((i * 17) % 100) as f32 / 100.0;
            let y = (i * 20) as f32;
            self.dirty_tracker
                .add_rect(Rect::new(10.0, y, val * 300.0, 16.0));
        }

        let paint_ns = self.emit_dirty_rect_paint();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: 0,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }
}
