//! GPUI architecture adapter (Zed GUI Engine model).
//!
//! Uses Entity Handle + Model Context, Taffy Flexbox layout engine,
//! and GPU quad/glyph command batching.

use crate::allocator::TrackingAllocator;
use crate::harness::{FrameMetrics, GuiBenchHarness};
use std::hint::black_box;
use std::time::Instant;
use taffy::prelude::*;

#[derive(Debug, Clone)]
struct RowModel {
    id: usize,
    label: String,
    selected: bool,
    dirty: bool,
}

#[derive(Debug, Clone)]
struct GpuQuadCommand {
    bounds: [f32; 4],
    color: [f32; 4],
    corner_radius: f32,
}

#[derive(Debug, Clone)]
struct GpuGlyphCommand {
    text: String,
    origin: [f32; 2],
    font_size: f32,
    color: [f32; 4],
}

/// Adapter modeling GPUI's Entity Handle + Taffy flexbox + GPU batching architecture.
#[derive(Debug)]
pub struct GpuiAdapter {
    tree: TaffyTree<()>,
    root: NodeId,
    rows: Vec<RowModel>,
    quad_batch: Vec<GpuQuadCommand>,
    glyph_batch: Vec<GpuGlyphCommand>,
}

impl Default for GpuiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuiAdapter {
    #[must_use]
    pub fn new() -> Self {
        let mut tree = TaffyTree::new();
        let root = tree
            .new_leaf(Style {
                size: Size {
                    width: length(1920.0),
                    height: length(1080.0),
                },
                flex_direction: FlexDirection::Column,
                ..Default::default()
            })
            .expect("Failed to create Taffy root");

        Self {
            tree,
            root,
            rows: Vec::new(),
            quad_batch: Vec::new(),
            glyph_batch: Vec::new(),
        }
    }

    fn rebuild_taffy_tree(&mut self) -> (u64, u64) {
        let t_recon_start = Instant::now();
        self.tree.clear();
        let root_style = Style {
            size: Size {
                width: length(1920.0),
                height: length(1080.0),
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };

        let mut child_nodes = Vec::with_capacity(self.rows.len());
        for _ in 0..self.rows.len() {
            let row_node = self
                .tree
                .new_leaf(Style {
                    size: Size {
                        width: percent(1.0),
                        height: length(36.0),
                    },
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::CENTER,
                    ..Default::default()
                })
                .unwrap();
            child_nodes.push(row_node);
        }

        self.root = self
            .tree
            .new_with_children(root_style, &child_nodes)
            .unwrap();
        let recon_ns = t_recon_start.elapsed().as_nanos() as u64;

        let t_layout_start = Instant::now();
        self.tree
            .compute_layout(
                self.root,
                Size {
                    width: AvailableSpace::Definite(1920.0),
                    height: AvailableSpace::Definite(1080.0),
                },
            )
            .unwrap();
        let layout_ns = t_layout_start.elapsed().as_nanos() as u64;

        (recon_ns, layout_ns)
    }

    fn emit_gpu_command_batches(&mut self) -> u64 {
        let t_paint_start = Instant::now();
        self.quad_batch.clear();
        self.glyph_batch.clear();

        for (idx, row) in self.rows.iter().enumerate() {
            let y = idx as f32 * 36.0;
            // Background quad
            self.quad_batch.push(GpuQuadCommand {
                bounds: [0.0, y, 1920.0, 36.0],
                color: if row.selected {
                    [0.2, 0.4, 0.8, 1.0]
                } else {
                    [0.1, 0.1, 0.1, 1.0]
                },
                corner_radius: 2.0,
            });

            // Text glyph batch
            self.glyph_batch.push(GpuGlyphCommand {
                text: format!("{}", row.id),
                origin: [16.0, y + 20.0],
                font_size: 14.0,
                color: [0.9, 0.9, 0.9, 1.0],
            });
            self.glyph_batch.push(GpuGlyphCommand {
                text: row.label.clone(),
                origin: [120.0, y + 20.0],
                font_size: 14.0,
                color: [1.0, 1.0, 1.0, 1.0],
            });

            // Action & Delete buttons
            self.quad_batch.push(GpuQuadCommand {
                bounds: [800.0, y + 6.0, 80.0, 24.0],
                color: [0.3, 0.3, 0.3, 1.0],
                corner_radius: 4.0,
            });
            self.quad_batch.push(GpuQuadCommand {
                bounds: [900.0, y + 6.0, 60.0, 24.0],
                color: [0.8, 0.2, 0.2, 1.0],
                corner_radius: 4.0,
            });
        }

        black_box(&self.quad_batch);
        black_box(&self.glyph_batch);
        t_paint_start.elapsed().as_nanos() as u64
    }

    fn measure_recon_and_paint<F: FnOnce(&mut Self)>(&mut self, op: F) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_recon_start = Instant::now();
        op(self);
        let recon_ns = t_recon_start.elapsed().as_nanos() as u64;

        let paint_ns = self.emit_gpu_command_batches();
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
}

impl GuiBenchHarness for GpuiAdapter {
    fn name(&self) -> &'static str {
        "GPUI (Zed Engine)"
    }

    fn create_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        self.rows = (0..count)
            .map(|i| RowModel {
                id: i,
                label: format!("Row item #{i} - Krausest table test string"),
                selected: false,
                dirty: true,
            })
            .collect();

        let (recon_ns, layout_ns) = self.rebuild_taffy_tree();
        let paint_ns = self.emit_gpu_command_batches();
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

        self.rows.clear();
        self.rows.extend((0..count).map(|i| RowModel {
            id: i + 10_000,
            label: format!("Replaced row #{i} - Newly updated table string payload"),
            selected: false,
            dirty: true,
        }));

        let (recon_ns, layout_ns) = self.rebuild_taffy_tree();
        let paint_ns = self.emit_gpu_command_batches();
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
        self.measure_recon_and_paint(|this| {
            for i in (0..this.rows.len()).step_by(10) {
                this.rows[i].label.push_str(" [MUTATED]");
                this.rows[i].dirty = true;
            }
        })
    }

    fn select_single_row(&mut self, index: usize) -> FrameMetrics {
        self.measure_recon_and_paint(|this| {
            if let Some(row) = this.rows.get_mut(index) {
                row.selected = !row.selected;
                row.dirty = true;
            }
        })
    }

    fn swap_two_rows(&mut self, idx_a: usize, idx_b: usize) -> FrameMetrics {
        self.measure_recon_and_paint(|this| {
            if idx_a < this.rows.len() && idx_b < this.rows.len() {
                this.rows.swap(idx_a, idx_b);
            }
        })
    }

    fn append_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let base = self.rows.len();
        self.rows.extend((0..count).map(|i| RowModel {
            id: base + i,
            label: format!("Appended row #{} - Extra table item payload", base + i),
            selected: false,
            dirty: true,
        }));

        let (recon_ns, layout_ns) = self.rebuild_taffy_tree();
        let paint_ns = self.emit_gpu_command_batches();
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
        let (recon_ns, layout_ns) = self.rebuild_taffy_tree();
        let paint_ns = self.emit_gpu_command_batches();
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

    fn layout_deep_dashboard(&mut self, node_count: usize, resize_step: f32) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let width = 1920.0 - resize_step;
        let t_layout_start = Instant::now();
        let mut local_tree = TaffyTree::<()>::new();
        let per_col = node_count / 4;
        let mut col_nodes = Vec::with_capacity(4);

        for _ in 0..4 {
            let mut leaf_nodes = Vec::with_capacity(per_col);
            for _ in 0..per_col {
                let leaf = local_tree
                    .new_leaf(Style {
                        size: Size {
                            width: percent(1.0),
                            height: length(24.0),
                        },
                        ..Default::default()
                    })
                    .unwrap();
                leaf_nodes.push(leaf);
            }
            let col = local_tree
                .new_with_children(
                    Style {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                    &leaf_nodes,
                )
                .unwrap();
            col_nodes.push(col);
        }

        let dash_root = local_tree
            .new_with_children(
                Style {
                    size: Size {
                        width: length(width),
                        height: length(1080.0),
                    },
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                &col_nodes,
            )
            .unwrap();

        local_tree
            .compute_layout(
                dash_root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(1080.0),
                },
            )
            .unwrap();
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

        self.glyph_batch.clear();
        for (idx, p) in paragraphs.iter().enumerate() {
            self.glyph_batch.push(GpuGlyphCommand {
                text: p.clone(),
                origin: [20.0, (idx * 24) as f32],
                font_size: 14.0,
                color: [1.0, 1.0, 1.0, 1.0],
            });
        }
        black_box(&self.glyph_batch);

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: 0,
            text_and_paint_ns: total_ns,
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
        self.quad_batch.clear();
        self.glyph_batch.clear();

        for i in 0..count {
            let val = ((i * 17) % 100) as f32 / 100.0;
            self.quad_batch.push(GpuQuadCommand {
                bounds: [10.0, (i * 20) as f32, val * 300.0, 16.0],
                color: [0.1, 0.7, 0.2, 1.0],
                corner_radius: 2.0,
            });
            self.glyph_batch.push(GpuGlyphCommand {
                text: format!("GPUI Telemetry #{i}: {:.2}", val * 100.0),
                origin: [320.0, (i * 20 + 14) as f32],
                font_size: 12.0,
                color: [1.0, 1.0, 1.0, 1.0],
            });
        }
        black_box(&self.quad_batch);
        black_box(&self.glyph_batch);

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: 0,
            layout_ns: 0,
            text_and_paint_ns: total_ns,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }
}
