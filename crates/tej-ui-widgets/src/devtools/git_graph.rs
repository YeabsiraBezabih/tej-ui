//! Git commit DAG graph with multi-lane branch routing and merge curve rendering.
//! Styled with Tej's tag pills, commit SHA typography, and branch color tokens.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// An individual commit in a Git DAG graph.
#[derive(Debug, Clone, PartialEq)]
pub struct GitCommit {
    pub hash_short: String,
    pub message: String,
    pub author: String,
    pub timestamp_str: String,
    pub lane: usize,
    pub parent_indices: Vec<usize>,
    pub branch_tags: Vec<String>,
}

/// State for the Git DAG graph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GitGraphState {
    pub commits: Vec<GitCommit>,
    pub selected_commit_index: Option<usize>,
}

impl GitGraphState {
    #[must_use]
    pub fn new(commits: Vec<GitCommit>) -> Self {
        Self {
            commits,
            selected_commit_index: None,
        }
    }
}

/// Tej styled Git Graph component.
#[derive(Debug, Clone, PartialEq)]
pub struct GitGraph {
    pub bounds: Rect,
    pub row_height: f32,
    pub lane_width: f32,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for GitGraph {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            row_height: 36.0,
            lane_width: 16.0,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            corner_radius: 8.0,
        }
    }
}
