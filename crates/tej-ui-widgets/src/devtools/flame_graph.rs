//! Virtualized flame graph component for profiling and call-tree visualization.
//! Styled with Tej's muted palette, high-contrast text labels, and smooth zoom rulers.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A node in a call-stack flame graph.
#[derive(Debug, Clone, PartialEq)]
pub struct FlameNode {
    pub name: String,
    pub start_time_us: u64,
    pub duration_us: u64,
    pub depth: usize,
    pub color: Color,
}

/// State for the zoomable virtualized flame graph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlameGraphState {
    pub total_duration_us: u64,
    pub max_depth: usize,
    pub zoom_start_us: u64,
    pub zoom_end_us: u64,
    pub selected_node_index: Option<usize>,
    pub hovered_node_index: Option<usize>,
    pub nodes: Vec<FlameNode>,
}

impl FlameGraphState {
    #[must_use]
    pub fn new(nodes: Vec<FlameNode>, total_duration_us: u64) -> Self {
        let max_depth = nodes.iter().map(|n| n.depth).max().unwrap_or(0);
        Self {
            total_duration_us,
            max_depth,
            zoom_start_us: 0,
            zoom_end_us: total_duration_us,
            selected_node_index: None,
            hovered_node_index: None,
            nodes,
        }
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_start_us = 0;
        self.zoom_end_us = self.total_duration_us;
    }

    pub fn zoom_to_node(&mut self, index: usize) {
        if let Some(node) = self.nodes.get(index) {
            self.selected_node_index = Some(index);
            self.zoom_start_us = node.start_time_us;
            self.zoom_end_us = node.start_time_us + node.duration_us;
        }
    }
}

/// Tej styled FlameGraph component.
#[derive(Debug, Clone, PartialEq)]
pub struct FlameGraph {
    pub bounds: Rect,
    pub row_height: f32,
    pub gap: f32,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for FlameGraph {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            row_height: 22.0,
            gap: 1.0,
            background: Color::from_rgb_u32(0x101828), // Tej Gray 900
            border_color: Color::from_rgb_u32(0x344054), // Tej Gray 700
            corner_radius: 8.0,
        }
    }
}
