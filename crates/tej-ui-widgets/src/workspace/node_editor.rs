//! Visual node graph editor with bezier connection links and typed port pins.
//! Styled with Tej's dark card headers, port status colors, and subtle grid dots.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A connection pin on a node (input or output).
#[derive(Debug, Clone, PartialEq)]
pub struct NodePin {
    pub id: String,
    pub label: String,
    pub is_input: bool,
    pub pin_color: Color,
}

/// A node placed on the visual graph canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub inputs: Vec<NodePin>,
    pub outputs: Vec<NodePin>,
}

/// A bezier link connecting two pins.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeConnection {
    pub from_node_id: String,
    pub from_pin_id: String,
    pub to_node_id: String,
    pub to_pin_id: String,
}

/// State for the node editor.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodeEditorState {
    pub nodes: Vec<GraphNode>,
    pub connections: Vec<NodeConnection>,
    pub selected_node_id: Option<String>,
    pub pan_offset_x: f32,
    pub pan_offset_y: f32,
    pub zoom: f32,
}

/// Tej styled Node Editor component.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEditor {
    pub bounds: Rect,
    pub background: Color,
    pub grid_color: Color,
    pub link_color: Color,
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::from_rgb_u32(0x101828), // Tej Gray 900
            grid_color: Color::from_rgb_u32(0x1D2939), // Tej Gray 800
            link_color: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
        }
    }
}
