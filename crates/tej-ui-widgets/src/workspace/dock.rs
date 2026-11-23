//! High-performance dockable workspace manager.
//! Implements binary space partitioning (BSP) dock trees, tab stacking, and layout serialization.
//! Styled with Tej's pane borders, tab bar typography, and drag overlay scrims.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Direction for a split dock panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSplitOrientation {
    Horizontal,
    Vertical,
}

/// An individual tab inside a dock panel.
#[derive(Debug, Clone, PartialEq)]
pub struct DockTab {
    pub id: String,
    pub title: String,
    pub is_closable: bool,
}

/// A node in the workspace docking BSP tree.
#[derive(Debug, Clone, PartialEq)]
pub enum DockNode {
    Tabs {
        tabs: Vec<DockTab>,
        active_tab_index: usize,
    },
    Split {
        orientation: DockSplitOrientation,
        ratio: f32, // 0.0 to 1.0
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

/// State for the workspace docking manager.
#[derive(Debug, Clone, PartialEq)]
pub struct DockManagerState {
    pub root: DockNode,
    pub active_tab_id: Option<String>,
}

impl Default for DockManagerState {
    fn default() -> Self {
        Self {
            root: DockNode::Tabs {
                tabs: Vec::new(),
                active_tab_index: 0,
            },
            active_tab_id: None,
        }
    }
}

impl DockManagerState {
    #[must_use]
    pub fn new_split(
        orientation: DockSplitOrientation,
        ratio: f32,
        first: DockNode,
        second: DockNode,
    ) -> Self {
        Self {
            root: DockNode::Split {
                orientation,
                ratio: ratio.clamp(0.1, 0.9),
                first: Box::new(first),
                second: Box::new(second),
            },
            active_tab_id: None,
        }
    }
}

/// Tej styled Dock Manager component.
#[derive(Debug, Clone, PartialEq)]
pub struct DockManager {
    pub bounds: Rect,
    pub tab_bar_height: f32,
    pub splitter_thickness: f32,
    pub background: Color,
    pub border_color: Color,
    pub active_tab_indicator: Color,
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            tab_bar_height: 38.0,
            splitter_thickness: 2.0,
            background: Color::from_rgb_u32(0xF9FAFB), // Tej Gray 50
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            active_tab_indicator: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
        }
    }
}
