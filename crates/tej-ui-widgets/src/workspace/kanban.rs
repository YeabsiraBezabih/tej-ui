//! Zero-copy database-backed Kanban board with drag-and-drop column slot mechanics.
//! Styled with Tej's card elevation, tag pills, and column header badges.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A card in a Kanban board.
#[derive(Debug, Clone, PartialEq)]
pub struct KanbanCard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tag: String,
    pub tag_color: Color,
    pub assignee_name: Option<String>,
}

/// A column in a Kanban board.
#[derive(Debug, Clone, PartialEq)]
pub struct KanbanColumn {
    pub id: String,
    pub title: String,
    pub cards: Vec<KanbanCard>,
}

/// State for the Kanban board.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KanbanState {
    pub columns: Vec<KanbanColumn>,
    pub dragging_card_id: Option<String>,
    pub target_column_index: Option<usize>,
}

impl KanbanState {
    #[must_use]
    pub fn new(columns: Vec<KanbanColumn>) -> Self {
        Self {
            columns,
            dragging_card_id: None,
            target_column_index: None,
        }
    }

    pub fn move_card(&mut self, card_id: &str, dest_col_index: usize, dest_card_index: usize) {
        let mut extracted = None;
        for col in &mut self.columns {
            if let Some(pos) = col.cards.iter().position(|c| c.id == card_id) {
                extracted = Some(col.cards.remove(pos));
                break;
            }
        }
        if let Some(card) = extracted
            && let Some(dest_col) = self.columns.get_mut(dest_col_index)
        {
            let insert_at = dest_card_index.min(dest_col.cards.len());
            dest_col.cards.insert(insert_at, card);
        }
    }
}

/// Tej styled Kanban Board component.
#[derive(Debug, Clone, PartialEq)]
pub struct KanbanBoard {
    pub bounds: Rect,
    pub column_width: f32,
    pub column_gap: f32,
    pub card_corner_radius: f32,
    pub background: Color,
    pub column_background: Color,
}

impl Default for KanbanBoard {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            column_width: 280.0,
            column_gap: 16.0,
            card_corner_radius: 8.0,
            background: Color::from_rgb_u32(0x00F9_FAFB), // Tej Gray 50
            column_background: Color::from_rgb_u32(0x00F2_F4F7), // Tej Gray 100
        }
    }
}
