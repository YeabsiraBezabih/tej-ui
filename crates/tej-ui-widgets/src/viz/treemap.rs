//! Squarified Treemap partition visualizer for hierarchical data and disk/memory budgets.
//! Styled with Tej's subtle borders, surface cards, and label truncation.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A node in a treemap layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TreemapNode {
    pub name: String,
    pub value: f32,
    pub color: Color,
    pub rect: Rect,
}

/// State for the squarified treemap.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreemapState {
    pub nodes: Vec<TreemapNode>,
    pub selected_index: Option<usize>,
}

impl TreemapState {
    #[must_use]
    pub fn new(mut nodes: Vec<TreemapNode>) -> Self {
        // Sort descending for squarified partition
        nodes.sort_by(|a, b| b.value.total_cmp(&a.value));
        Self {
            nodes,
            selected_index: None,
        }
    }
}

/// Tej styled Treemap component.
#[derive(Debug, Clone, PartialEq)]
pub struct Treemap {
    pub bounds: Rect,
    pub gap: f32,
    pub corner_radius: f32,
    pub border_color: Color,
}

impl Default for Treemap {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            gap: 2.0,
            corner_radius: 4.0,
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
        }
    }
}
