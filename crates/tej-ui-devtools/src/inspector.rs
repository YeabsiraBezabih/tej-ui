//! Runtime widget hierarchy inspector and per-frame flamegraph timing profiler.

use std::fmt::Debug;
use tej_ui_layout::{LayoutArena, Rect};

/// Descriptive metadata for an individual inspected layout node.
#[derive(Clone, PartialEq, Debug)]
pub struct InspectedNode {
    pub id: u32,
    pub bounds: Rect,
    pub depth: usize,
    pub children: Vec<u32>,
    pub is_dirty: bool,
}

/// Runtime widget hierarchy inspector traversing flat SoA layout storage.
#[derive(Debug, Default)]
pub struct WidgetInspector;

impl WidgetInspector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extract hierarchical node tree starting from `root`.
    #[must_use]
    pub fn inspect_hierarchy(&self, arena: &LayoutArena, root: u32) -> Vec<InspectedNode> {
        let mut result = Vec::new();
        if (root as usize) < arena.len() {
            Self::traverse_recursive(arena, root, 0, &mut result);
        }
        result
    }

    fn traverse_recursive(
        arena: &LayoutArena,
        node: u32,
        depth: usize,
        out: &mut Vec<InspectedNode>,
    ) {
        let bounds = arena
            .computed_bounds
            .get(node as usize)
            .copied()
            .unwrap_or(Rect::ZERO);
        let children = arena
            .children
            .get(node as usize)
            .map(|c| c.to_vec())
            .unwrap_or_default();
        let is_dirty = arena.dirty.get(node as usize).copied().unwrap_or(false);

        out.push(InspectedNode {
            id: node,
            bounds,
            depth,
            children: children.clone(),
            is_dirty,
        });

        for child in children {
            Self::traverse_recursive(arena, child, depth + 1, out);
        }
    }

    /// Format widget hierarchy as an indented ASCII tree string.
    #[must_use]
    pub fn format_tree(&self, arena: &LayoutArena, root: u32) -> String {
        use std::fmt::Write;
        let nodes = self.inspect_hierarchy(arena, root);
        let mut out = String::new();
        for n in nodes {
            let indent = "  ".repeat(n.depth);
            let _ = writeln!(
                out,
                "{indent}[Node #{}] ({:.1}x{:.1} at {:.1},{:.1}) dirty={}",
                n.id,
                n.bounds.size.width,
                n.bounds.size.height,
                n.bounds.origin.x,
                n.bounds.origin.y,
                n.is_dirty
            );
        }
        out
    }
}

/// Recorded elapsed time sample across distinct frame execution phases.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct FrameTimingSample {
    pub reactivity_us: u64,
    pub layout_us: u64,
    pub events_us: u64,
    pub paint_us: u64,
    pub present_us: u64,
}

impl FrameTimingSample {
    /// Total duration of the frame in microseconds.
    #[must_use]
    pub const fn total_duration_us(&self) -> u64 {
        self.reactivity_us + self.layout_us + self.events_us + self.paint_us + self.present_us
    }

    /// Check if frame breached 60Hz 16.6ms threshold (16,666 us).
    #[must_use]
    pub const fn is_jank(&self) -> bool {
        self.total_duration_us() > 16_666
    }
}

/// Frame-time flamegraph profiler aggregating execution bottlenecks across render passes.
#[derive(Debug, Default)]
pub struct FrameFlamegraphProfiler {
    samples: Vec<FrameTimingSample>,
}

impl FrameFlamegraphProfiler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a frame timing sample.
    pub fn record_sample(&mut self, sample: FrameTimingSample) {
        self.samples.push(sample);
    }

    /// Count of recorded frames.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.samples.len()
    }

    /// Average frame duration across all samples in microseconds.
    #[must_use]
    pub fn average_duration_us(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let total: u64 = self
            .samples
            .iter()
            .map(FrameTimingSample::total_duration_us)
            .sum();
        let total_us = f64::from(u32::try_from(total).unwrap_or(u32::MAX));
        let count = f64::from(u16::try_from(self.samples.len()).unwrap_or(u16::MAX));
        total_us / count
    }

    /// Count of dropped / jank frames (>16.6ms).
    #[must_use]
    pub fn jank_count(&self) -> usize {
        self.samples.iter().filter(|s| s.is_jank()).count()
    }
}
