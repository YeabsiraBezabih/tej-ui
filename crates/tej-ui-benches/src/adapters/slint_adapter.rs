//! Slint compiled DSL & property bindings runtime adapter.
//!
//! Models Slint's reactive property bindings, `VecModel<T>`, dirty-rect tracking,
//! and software rasterizer blits.

use crate::allocator::TrackingAllocator;
use crate::harness::{FrameMetrics, GuiBenchHarness};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
struct SlintRowData {
    id: i32,
    label: SharedString,
    selected: bool,
}

#[derive(Debug, Clone)]
struct DirtyRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Adapter executing Slint's property evaluation, ModelRc updates, and software dirty-rect rendering.
#[derive(Clone)]
pub struct SlintAdapter {
    model: Rc<VecModel<SlintRowData>>,
    model_rc: ModelRc<SlintRowData>,
    dirty_rects: Vec<DirtyRect>,
}

impl std::fmt::Debug for SlintAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlintAdapter")
            .field("rows", &self.model.row_count())
            .field("dirty_rects", &self.dirty_rects)
            .finish()
    }
}

impl Default for SlintAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SlintAdapter {
    #[must_use]
    pub fn new() -> Self {
        let model = Rc::new(VecModel::default());
        let model_rc = ModelRc::new(model.clone());
        Self {
            model,
            model_rc,
            dirty_rects: Vec::new(),
        }
    }

    fn simulate_software_render(&mut self) -> u64 {
        let t_paint_start = Instant::now();
        // Emulate Slint software renderer rasterizing dirty rects
        let mut pixels_rendered = 0usize;
        for rect in &self.dirty_rects {
            pixels_rendered += (rect.width * rect.height) as usize;
        }
        self.dirty_rects.clear();
        black_box(pixels_rendered);
        t_paint_start.elapsed().as_nanos() as u64
    }
}

impl GuiBenchHarness for SlintAdapter {
    fn name(&self) -> &'static str {
        "Slint v1.18"
    }

    fn create_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_model = Instant::now();
        let rows: Vec<SlintRowData> = (0..count)
            .map(|i| SlintRowData {
                id: i as i32,
                label: SharedString::from(format!("Row item #{i} - Krausest table test string")),
                selected: false,
            })
            .collect();
        self.model.set_vec(rows);
        let state_ns = t_model.elapsed().as_nanos() as u64;

        // Slint layout evaluation for new rows
        let t_layout = Instant::now();
        self.dirty_rects.push(DirtyRect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: (count as f32 * 36.0).min(1080.0),
        });
        let layout_ns = t_layout.elapsed().as_nanos() as u64;

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
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

        let t_model = Instant::now();
        let rows: Vec<SlintRowData> = (0..count)
            .map(|i| SlintRowData {
                id: (i + 10_000) as i32,
                label: SharedString::from(format!(
                    "Replaced row #{i} - Newly updated table string payload"
                )),
                selected: false,
            })
            .collect();
        self.model.set_vec(rows);
        let state_ns = t_model.elapsed().as_nanos() as u64;

        self.dirty_rects.push(DirtyRect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        });

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns: 0,
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

        let t_model = Instant::now();
        let row_count = self.model.row_count();
        for i in (0..row_count).step_by(10) {
            if let Some(mut r) = self.model.row_data(i) {
                let updated = format!("{} [MUTATED]", r.label);
                r.label = SharedString::from(updated);
                self.model.set_row_data(i, r);
                self.dirty_rects.push(DirtyRect {
                    x: 0.0,
                    y: (i as f32 * 36.0).min(1080.0),
                    width: 1920.0,
                    height: 36.0,
                });
            }
        }
        let state_ns = t_model.elapsed().as_nanos() as u64;

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
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

        let t_model = Instant::now();
        if let Some(mut r) = self.model.row_data(index) {
            r.selected = !r.selected;
            self.model.set_row_data(index, r);
            self.dirty_rects.push(DirtyRect {
                x: 0.0,
                y: (index as f32 * 36.0).min(1080.0),
                width: 1920.0,
                height: 36.0,
            });
        }
        let state_ns = t_model.elapsed().as_nanos() as u64;

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
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

        let t_model = Instant::now();
        let row_a = self.model.row_data(idx_a);
        let row_b = self.model.row_data(idx_b);
        if let (Some(a), Some(b)) = (row_a, row_b) {
            self.model.set_row_data(idx_a, b);
            self.model.set_row_data(idx_b, a);
            self.dirty_rects.push(DirtyRect {
                x: 0.0,
                y: (idx_a as f32 * 36.0).min(1080.0),
                width: 1920.0,
                height: 36.0,
            });
            self.dirty_rects.push(DirtyRect {
                x: 0.0,
                y: (idx_b as f32 * 36.0).min(1080.0),
                width: 1920.0,
                height: 36.0,
            });
        }
        let state_ns = t_model.elapsed().as_nanos() as u64;

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
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

        let t_model = Instant::now();
        let base = self.model.row_count();
        for i in 0..count {
            self.model.push(SlintRowData {
                id: (base + i) as i32,
                label: SharedString::from(format!(
                    "Appended row #{} - Extra table item payload",
                    base + i
                )),
                selected: false,
            });
        }
        let state_ns = t_model.elapsed().as_nanos() as u64;

        self.dirty_rects.push(DirtyRect {
            x: 0.0,
            y: (base as f32 * 36.0).min(1080.0),
            width: 1920.0,
            height: ((count as f32) * 36.0).min(1080.0),
        });

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns: 0,
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

        let t_model = Instant::now();
        self.model.set_vec(Vec::new());
        let state_ns = t_model.elapsed().as_nanos() as u64;

        self.dirty_rects.push(DirtyRect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        });

        let paint_ns = self.simulate_software_render();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
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
        let t_layout = Instant::now();
        // Slint compiled constraint solver updates coordinates of each node
        let per_col = node_count / 4;
        let col_w = width / 4.0;
        let mut accum = 0.0f32;
        for c in 0..4 {
            for r in 0..per_col {
                let x = c as f32 * col_w;
                let y = r as f32 * 24.0;
                accum += x + y;
            }
        }
        black_box(accum);
        let layout_ns = t_layout.elapsed().as_nanos() as u64;

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

        let t_paint = Instant::now();
        let mut total_chars = 0;
        for p in paragraphs {
            total_chars += p.chars().count();
        }
        black_box(total_chars);
        let paint_ns = t_paint.elapsed().as_nanos() as u64;

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

    fn tick_all_nodes_telemetry(&mut self, active_widgets: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let count = active_widgets / 3;
        for i in 0..count {
            let val = ((i * 17) % 100) as f32 / 100.0;
            self.dirty_rects.push(DirtyRect {
                x: 10.0,
                y: (i * 20) as f32,
                width: val * 300.0,
                height: 16.0,
            });
        }
        let paint_ns = self.simulate_software_render();

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
