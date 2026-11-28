//! egui immediate-mode framework adapter.

use crate::allocator::TrackingAllocator;
use crate::harness::{FrameMetrics, GuiBenchHarness};
use egui::{Context, RawInput, ViewportId};
use std::hint::black_box;
use std::time::Instant;

#[derive(Debug, Clone)]
struct TableRow {
    id: usize,
    label: String,
    selected: bool,
}

/// Adapter executing live egui immediate mode pipelines.
#[derive(Debug)]
pub struct EguiAdapter {
    ctx: Context,
    rows: Vec<TableRow>,
    raw_input: RawInput,
}

impl Default for EguiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EguiAdapter {
    #[must_use]
    pub fn new() -> Self {
        let raw_input = RawInput {
            viewport_id: ViewportId::ROOT,
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1920.0, 1080.0),
            )),
            ..Default::default()
        };
        Self {
            ctx: Context::default(),
            rows: Vec::new(),
            raw_input,
        }
    }

    fn run_frame(&mut self) -> (u64, u64, u64) {
        let t0 = Instant::now();
        let full_output = self.ctx.run(self.raw_input.clone(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for row in &self.rows {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", row.id));
                            if row.selected {
                                ui.colored_label(egui::Color32::YELLOW, &row.label);
                            } else {
                                ui.label(&row.label);
                            }
                            let _ = ui.button("Action");
                            let _ = ui.button("Del");
                        });
                    }
                });
            });
        });
        let state_and_layout_ns = t0.elapsed().as_nanos() as u64;

        let t1 = Instant::now();
        let primitives = self.ctx.tessellate(full_output.shapes, 2.0);
        black_box(&primitives);
        let paint_ns = t1.elapsed().as_nanos() as u64;

        // Stage A (state): estimated 35%, Stage B (layout): estimated 65% of pass
        let state_ns = state_and_layout_ns * 35 / 100;
        let layout_ns = state_and_layout_ns - state_ns;

        (state_ns, layout_ns, paint_ns)
    }
}

impl GuiBenchHarness for EguiAdapter {
    fn name(&self) -> &'static str {
        "egui v0.32"
    }

    fn create_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        self.rows = (0..count)
            .map(|i| TableRow {
                id: i,
                label: format!("Row item #{i} - Krausest table test string"),
                selected: false,
            })
            .collect();

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

        self.rows.clear();
        self.rows.extend((0..count).map(|i| TableRow {
            id: i + 10_000,
            label: format!("Replaced row #{i} - Newly updated table string payload"),
            selected: false,
        }));

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

    fn partial_update_every_10th(&mut self) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        for i in (0..self.rows.len()).step_by(10) {
            self.rows[i].label.push_str(" [MUTATED]");
        }

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

    fn select_single_row(&mut self, index: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        if let Some(row) = self.rows.get_mut(index) {
            row.selected = !row.selected;
        }

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

    fn swap_two_rows(&mut self, idx_a: usize, idx_b: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        if idx_a < self.rows.len() && idx_b < self.rows.len() {
            self.rows.swap(idx_a, idx_b);
        }

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

    fn append_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let base = self.rows.len();
        self.rows.extend((0..count).map(|i| TableRow {
            id: base + i,
            label: format!("Appended row #{} - Extra table item payload", base + i),
            selected: false,
        }));

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

    fn clear_all_rows(&mut self) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        self.rows.clear();

        let (state_ns, layout_ns, paint_ns) = self.run_frame();
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

    fn layout_deep_dashboard(&mut self, node_count: usize, resize_step: f32) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let width = 1920.0 - resize_step;
        let mut raw = self.raw_input.clone();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(width, 1080.0),
        ));

        let t_layout_start = Instant::now();
        let output = self.ctx.run(raw, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.columns(4, |cols| {
                    let per_col = node_count / 4;
                    for (c_idx, col) in cols.iter_mut().enumerate() {
                        for row_idx in 0..per_col {
                            col.label(format!("Node c{c_idx}r{row_idx}"));
                        }
                    }
                });
            });
        });
        let layout_ns = t_layout_start.elapsed().as_nanos() as u64;

        let t_paint = Instant::now();
        let primitives = self.ctx.tessellate(output.shapes, 2.0);
        black_box(&primitives);
        let paint_ns = t_paint.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns,
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn render_multilingual_document(&mut self, paragraphs: &[String]) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let output = self.ctx.run(self.raw_input.clone(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for p in paragraphs {
                    ui.label(p);
                }
            });
        });

        let t_paint = Instant::now();
        let primitives = self.ctx.tessellate(output.shapes, 2.0);
        black_box(&primitives);
        let paint_ns = t_paint.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: total_ns.saturating_sub(paint_ns),
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

        // 100% full frame dynamic redraw: progress bars, counters, and waveform points
        let count = active_widgets / 3;
        let output = self.ctx.run(self.raw_input.clone(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for i in 0..count {
                    let val = crate::count_to_f32((i * 17) % 100) / 100.0;
                    ui.add(egui::ProgressBar::new(val).text(format!("{:.1}%", val * 100.0)));
                    ui.label(format!("Sensor #{i}: {:.4}", val * 42.195));
                }
            });
        });

        let t_paint = Instant::now();
        let primitives = self.ctx.tessellate(output.shapes, 2.0);
        black_box(&primitives);
        let paint_ns = t_paint.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: total_ns.saturating_sub(paint_ns),
            text_and_paint_ns: paint_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }
}
