//! Iced Elm-architecture retained adapter.
//!
//! Models Iced's Message -> Update -> View rebuild cycle,
//! hierarchical widget tree state, layout passes, and quad/text render batches.

use crate::allocator::TrackingAllocator;
use crate::harness::{FrameMetrics, GuiBenchHarness};
use iced_core::layout::{Limits, Node};
use iced_core::{Length, Point, Rectangle, Size};
use std::hint::black_box;
use std::time::Instant;

#[derive(Debug, Clone)]
struct IcedRow {
    id: usize,
    label: String,
    selected: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Create(usize),
    Replace(usize),
    PartialUpdate,
    Select(usize),
    Swap(usize, usize),
    Append(usize),
    Clear,
}

#[derive(Debug, Clone)]
struct IcedQuadPrimitive {
    bounds: Rectangle,
    color: [f32; 4],
}

#[derive(Debug, Clone)]
struct IcedTextPrimitive {
    content: String,
    position: Point,
    size: f32,
}

/// Adapter executing Iced's Elm-style update, layout tree, and render pipeline.
#[derive(Debug)]
pub struct IcedAdapter {
    rows: Vec<IcedRow>,
    quads: Vec<IcedQuadPrimitive>,
    texts: Vec<IcedTextPrimitive>,
}

impl Default for IcedAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IcedAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            quads: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Create(count) => {
                self.rows = (0..count)
                    .map(|i| IcedRow {
                        id: i,
                        label: format!("Row item #{i} - Krausest table test string"),
                        selected: false,
                    })
                    .collect();
            }
            Message::Replace(count) => {
                self.rows.clear();
                self.rows.extend((0..count).map(|i| IcedRow {
                    id: i + 10_000,
                    label: format!("Replaced row #{i} - Newly updated table string payload"),
                    selected: false,
                }));
            }
            Message::PartialUpdate => {
                for i in (0..self.rows.len()).step_by(10) {
                    self.rows[i].label.push_str(" [MUTATED]");
                }
            }
            Message::Select(idx) => {
                if let Some(r) = self.rows.get_mut(idx) {
                    r.selected = !r.selected;
                }
            }
            Message::Swap(a, b) => {
                if a < self.rows.len() && b < self.rows.len() {
                    self.rows.swap(a, b);
                }
            }
            Message::Append(count) => {
                let base = self.rows.len();
                self.rows.extend((0..count).map(|i| IcedRow {
                    id: base + i,
                    label: format!("Appended row #{} - Extra table item payload", base + i),
                    selected: false,
                }));
            }
            Message::Clear => {
                self.rows.clear();
            }
        }
    }

    fn compute_iced_layout(&self, viewport_width: f32) -> (Node, u64) {
        let t_layout_start = Instant::now();
        let _limits = Limits::new(
            Size::new(0.0, 0.0),
            Size::new(viewport_width, f32::INFINITY),
        )
        .width(Length::Fixed(viewport_width));

        let mut row_nodes = Vec::with_capacity(self.rows.len());
        let mut y = 0.0f32;
        for _ in &self.rows {
            let row_limits = Limits::new(
                Size::new(viewport_width, 36.0),
                Size::new(viewport_width, 36.0),
            );
            let row_node = Node::new(row_limits.max()).move_to(Point::new(0.0, y));
            row_nodes.push(row_node);
            y += 36.0;
        }

        let root_node =
            Node::with_children(Size::new(viewport_width, y), row_nodes).move_to(Point::ORIGIN);
        black_box(&root_node);
        let layout_ns = t_layout_start.elapsed().as_nanos() as u64;
        (root_node, layout_ns)
    }

    fn emit_paint(&mut self) -> u64 {
        let t_paint_start = Instant::now();
        self.quads.clear();
        self.texts.clear();

        for (idx, row) in self.rows.iter().enumerate() {
            let y = idx as f32 * 36.0;
            self.quads.push(IcedQuadPrimitive {
                bounds: Rectangle::new(Point::new(0.0, y), Size::new(1920.0, 36.0)),
                color: if row.selected {
                    [0.2, 0.3, 0.7, 1.0]
                } else {
                    [0.08, 0.08, 0.08, 1.0]
                },
            });

            self.texts.push(IcedTextPrimitive {
                content: format!("{}", row.id),
                position: Point::new(16.0, y + 18.0),
                size: 14.0,
            });
            self.texts.push(IcedTextPrimitive {
                content: row.label.clone(),
                position: Point::new(100.0, y + 18.0),
                size: 14.0,
            });
            self.quads.push(IcedQuadPrimitive {
                bounds: Rectangle::new(Point::new(800.0, y + 6.0), Size::new(80.0, 24.0)),
                color: [0.25, 0.25, 0.25, 1.0],
            });
            self.quads.push(IcedQuadPrimitive {
                bounds: Rectangle::new(Point::new(890.0, y + 6.0), Size::new(60.0, 24.0)),
                color: [0.7, 0.2, 0.2, 1.0],
            });
        }

        black_box(&self.quads);
        black_box(&self.texts);
        t_paint_start.elapsed().as_nanos() as u64
    }

    fn execute_cycle(&mut self, msg: Message) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let t_update = Instant::now();
        self.update(msg);
        let state_ns = t_update.elapsed().as_nanos() as u64;

        let (_root, layout_ns) = self.compute_iced_layout(1920.0);
        let paint_ns = self.emit_paint();

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
}

impl GuiBenchHarness for IcedAdapter {
    fn name(&self) -> &'static str {
        "Iced v0.14"
    }

    fn create_rows(&mut self, count: usize) -> FrameMetrics {
        self.execute_cycle(Message::Create(count))
    }

    fn replace_all_rows(&mut self, count: usize) -> FrameMetrics {
        self.execute_cycle(Message::Replace(count))
    }

    fn partial_update_every_10th(&mut self) -> FrameMetrics {
        self.execute_cycle(Message::PartialUpdate)
    }

    fn select_single_row(&mut self, index: usize) -> FrameMetrics {
        self.execute_cycle(Message::Select(index))
    }

    fn swap_two_rows(&mut self, idx_a: usize, idx_b: usize) -> FrameMetrics {
        self.execute_cycle(Message::Swap(idx_a, idx_b))
    }

    fn append_rows(&mut self, count: usize) -> FrameMetrics {
        self.execute_cycle(Message::Append(count))
    }

    fn clear_all_rows(&mut self) -> FrameMetrics {
        self.execute_cycle(Message::Clear)
    }

    fn layout_deep_dashboard(&mut self, node_count: usize, resize_step: f32) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();
        let width = 1920.0 - resize_step;

        let t_layout_start = Instant::now();
        let per_col = node_count / 4;
        let mut col_nodes = Vec::with_capacity(4);

        for c in 0..4 {
            let mut leaves = Vec::with_capacity(per_col);
            let mut y = 0.0f32;
            for _ in 0..per_col {
                let leaf = Node::new(Size::new(width / 4.0, 24.0)).move_to(Point::new(0.0, y));
                leaves.push(leaf);
                y += 24.0;
            }
            let col = Node::with_children(Size::new(width / 4.0, y), leaves)
                .move_to(Point::new(c as f32 * (width / 4.0), 0.0));
            col_nodes.push(col);
        }

        let root = Node::with_children(Size::new(width, 1080.0), col_nodes).move_to(Point::ORIGIN);
        black_box(&root);
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

        self.texts.clear();
        for (i, p) in paragraphs.iter().enumerate() {
            self.texts.push(IcedTextPrimitive {
                content: p.clone(),
                position: Point::new(20.0, (i * 24) as f32),
                size: 14.0,
            });
        }
        black_box(&self.texts);

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
        self.quads.clear();
        self.texts.clear();

        for i in 0..count {
            let val = ((i * 17) % 100) as f32 / 100.0;
            self.quads.push(IcedQuadPrimitive {
                bounds: Rectangle::new(
                    Point::new(10.0, (i * 20) as f32),
                    Size::new(val * 300.0, 16.0),
                ),
                color: [0.1, 0.6, 0.8, 1.0],
            });
            self.texts.push(IcedTextPrimitive {
                content: format!("Iced Telemetry #{i}: {:.2}", val * 100.0),
                position: Point::new(320.0, (i * 20 + 14) as f32),
                size: 12.0,
            });
        }
        black_box(&self.quads);
        black_box(&self.texts);

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
