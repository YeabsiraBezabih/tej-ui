//! Sankey flow diagram visualizer with smooth cubic bezier flow ribbons.
//! Styled with Tej's tinted brand links, node badges, and hover tooltips.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A node in a Sankey flow graph.
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyNode {
    pub id: String,
    pub title: String,
    pub column: usize,
    pub value: f32,
    pub color: Color,
}

/// A directed flow link between two Sankey nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyLink {
    pub source_id: String,
    pub target_id: String,
    pub flow_value: f32,
    pub color: Color,
}

/// State for the Sankey flow diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SankeyState {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub hovered_link_index: Option<usize>,
}

impl SankeyState {
    #[must_use]
    pub fn new(nodes: Vec<SankeyNode>, links: Vec<SankeyLink>) -> Self {
        Self {
            nodes,
            links,
            hovered_link_index: None,
        }
    }
}

/// Tej styled Sankey Diagram component.
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyDiagram {
    pub bounds: Rect,
    pub node_width: f32,
    pub column_gap: f32,
    pub node_gap: f32,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for SankeyDiagram {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            node_width: 14.0,
            column_gap: 160.0,
            node_gap: 12.0,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0x00EA_ECF0), // Tej Gray 200
            corner_radius: 8.0,
        }
    }
}
