//! Dioxus Virtual DOM & template diffing adapter.
//!
//! Models Dioxus VDOM template instantiation, diffing mutations,
//! and Taffy layout synchronization.

use crate::allocator::TrackingAllocator;
use crate::harness::{FrameMetrics, GuiBenchHarness};
use std::hint::black_box;
use std::time::Instant;
use taffy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct VDomRow {
    id: usize,
    label: String,
    selected: bool,
}

#[derive(Debug, Clone)]
enum VDomMutation {
    CreateElement {
        tag: &'static str,
        id: usize,
    },
    SetText {
        id: usize,
        text: String,
    },
    SetAttribute {
        id: usize,
        name: &'static str,
        value: String,
    },
    RemoveElement {
        id: usize,
    },
}

/// Adapter modeling Dioxus VDOM tree diffing and Taffy layout updates.
#[derive(Debug)]
pub struct DioxusAdapter {
    rows: Vec<VDomRow>,
    prev_rows: Vec<VDomRow>,
    mutations: Vec<VDomMutation>,
    taffy: TaffyTree<()>,
    root_node: NodeId,
}

impl Default for DioxusAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DioxusAdapter {
    #[must_use]
    pub fn new() -> Self {
        let mut taffy = TaffyTree::new();
        let root_node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(1920.0),
                    height: length(1080.0),
                },
                flex_direction: FlexDirection::Column,
                ..Default::default()
            })
            .unwrap();

        Self {
            rows: Vec::new(),
            prev_rows: Vec::new(),
            mutations: Vec::new(),
            taffy,
            root_node,
        }
    }

    fn diff_and_reconcile(&mut self) -> u64 {
        let t_diff_start = Instant::now();
        self.mutations.clear();

        let old_len = self.prev_rows.len();
        let new_len = self.rows.len();

        let common = old_len.min(new_len);
        for i in 0..common {
            if self.rows[i] != self.prev_rows[i] {
                if self.rows[i].label != self.prev_rows[i].label {
                    self.mutations.push(VDomMutation::SetText {
                        id: self.rows[i].id,
                        text: self.rows[i].label.clone(),
                    });
                }
                if self.rows[i].selected != self.prev_rows[i].selected {
                    self.mutations.push(VDomMutation::SetAttribute {
                        id: self.rows[i].id,
                        name: "class",
                        value: if self.rows[i].selected {
                            "selected".to_string()
                        } else {
                            "row".to_string()
                        },
                    });
                }
            }
        }

        if new_len > old_len {
            for i in old_len..new_len {
                self.mutations.push(VDomMutation::CreateElement {
                    tag: "tr",
                    id: self.rows[i].id,
                });
                self.mutations.push(VDomMutation::SetText {
                    id: self.rows[i].id,
                    text: self.rows[i].label.clone(),
                });
            }
        } else if old_len > new_len {
            for i in new_len..old_len {
                self.mutations.push(VDomMutation::RemoveElement {
                    id: self.prev_rows[i].id,
                });
            }
        }

        self.prev_rows = self.rows.clone();
        black_box(&self.mutations);
        t_diff_start.elapsed().as_nanos() as u64
    }

    fn sync_taffy_layout(&mut self) -> u64 {
        let t_layout_start = Instant::now();
        self.taffy.clear();
        let root_style = Style {
            size: Size {
                width: length(1920.0),
                height: length(1080.0),
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };

        let mut row_nodes = Vec::with_capacity(self.rows.len());
        for _ in 0..self.rows.len() {
            let node = self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: percent(1.0),
                        height: length(36.0),
                    },
                    ..Default::default()
                })
                .unwrap();
            row_nodes.push(node);
        }

        self.root_node = self
            .taffy
            .new_with_children(root_style, &row_nodes)
            .unwrap();
        self.taffy
            .compute_layout(
                self.root_node,
                Size {
                    width: AvailableSpace::Definite(1920.0),
                    height: AvailableSpace::Definite(1080.0),
                },
            )
            .unwrap();
        t_layout_start.elapsed().as_nanos() as u64
    }

    fn measure_vdom_mutation<F: FnOnce(&mut Self)>(&mut self, op: F) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();
        op(self);
        let state_ns = self.diff_and_reconcile();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns: 0,
            text_and_paint_ns: 0,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }
}

impl GuiBenchHarness for DioxusAdapter {
    fn name(&self) -> &'static str {
        "Dioxus v0.7 (VDOM)"
    }

    fn create_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        self.rows = (0..count)
            .map(|i| VDomRow {
                id: i,
                label: format!("Row item #{i} - Krausest table test string"),
                selected: false,
            })
            .collect();

        let state_ns = self.diff_and_reconcile();
        let layout_ns = self.sync_taffy_layout();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns,
            text_and_paint_ns: 0,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn replace_all_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        self.rows = (0..count)
            .map(|i| VDomRow {
                id: i + 10_000,
                label: format!("Replaced row #{i} - Newly updated table string payload"),
                selected: false,
            })
            .collect();

        let state_ns = self.diff_and_reconcile();
        let layout_ns = self.sync_taffy_layout();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns,
            text_and_paint_ns: 0,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }

    fn partial_update_every_10th(&mut self) -> FrameMetrics {
        self.measure_vdom_mutation(|this| {
            for i in (0..this.rows.len()).step_by(10) {
                this.rows[i].label.push_str(" [MUTATED]");
            }
        })
    }

    fn select_single_row(&mut self, index: usize) -> FrameMetrics {
        self.measure_vdom_mutation(|this| {
            if let Some(r) = this.rows.get_mut(index) {
                r.selected = !r.selected;
            }
        })
    }

    fn swap_two_rows(&mut self, idx_a: usize, idx_b: usize) -> FrameMetrics {
        self.measure_vdom_mutation(|this| {
            if idx_a < this.rows.len() && idx_b < this.rows.len() {
                this.rows.swap(idx_a, idx_b);
            }
        })
    }

    fn append_rows(&mut self, count: usize) -> FrameMetrics {
        let snap = TrackingAllocator::reset_and_snapshot();
        let t_start = Instant::now();

        let base = self.rows.len();
        self.rows.extend((0..count).map(|i| VDomRow {
            id: base + i,
            label: format!("Appended row #{} - Extra table item payload", base + i),
            selected: false,
        }));

        let state_ns = self.diff_and_reconcile();
        let layout_ns = self.sync_taffy_layout();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns,
            text_and_paint_ns: 0,
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
        let state_ns = self.diff_and_reconcile();
        let layout_ns = self.sync_taffy_layout();
        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: state_ns,
            layout_ns,
            text_and_paint_ns: 0,
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

        let t_diff = Instant::now();
        let mut text_nodes = Vec::with_capacity(paragraphs.len());
        for (i, p) in paragraphs.iter().enumerate() {
            text_nodes.push(VDomMutation::SetText {
                id: i,
                text: p.clone(),
            });
        }
        black_box(&text_nodes);
        let diff_ns = t_diff.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: diff_ns,
            layout_ns: 0,
            text_and_paint_ns: 0,
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
        let t_diff = Instant::now();
        let mut telemetry_mutations = Vec::with_capacity(count);
        for i in 0..count {
            let val = crate::count_to_f32((i * 17) % 100) / 100.0;
            telemetry_mutations.push(VDomMutation::SetAttribute {
                id: i,
                name: "value",
                value: format!("{val:.2}"),
            });
        }
        black_box(&telemetry_mutations);
        let diff_ns = t_diff.elapsed().as_nanos() as u64;

        let total_ns = t_start.elapsed().as_nanos() as u64;
        let (alloc_count, bytes_alloc, peak_bytes) = TrackingAllocator::delta_since(snap);

        FrameMetrics {
            state_reconcile_ns: diff_ns,
            layout_ns: 0,
            text_and_paint_ns: 0,
            total_frame_ns: total_ns,
            heap_alloc_count: alloc_count,
            heap_bytes_allocated: bytes_alloc,
            peak_live_heap_bytes: peak_bytes,
        }
    }
}
