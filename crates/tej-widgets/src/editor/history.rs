//! Undo/redo history stack for transaction-safe text editing.

use crate::editor::buffer::TextBuffer;
use crate::editor::selection::CursorPos;

/// Individual recorded atomic text modification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EditAction {
    /// Text inserted at position.
    Insert { pos: CursorPos, text: String },
    /// Text deleted from position.
    Delete { pos: CursorPos, text: String },
}

/// Transactional undo and redo buffer with configurable history capacity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UndoStack {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    max_history: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(200)
    }
}

impl UndoStack {
    /// Create a new undo stack with maximum history limit.
    #[must_use]
    pub const fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Record a newly executed edit action. Clears redo stack.
    pub fn push(&mut self, action: EditAction) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    /// Check if an undo action is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if a redo action is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undo last modification on the buffer, returning restored cursor position.
    pub fn undo(&mut self, buf: &mut TextBuffer) -> Option<CursorPos> {
        let action = self.undo_stack.pop()?;
        let restore_pos = match &action {
            EditAction::Insert { pos, text } => {
                // Invert insert by deleting the inserted text range
                let mut end_pos = *pos;
                for ch in text.chars() {
                    if ch == '\n' {
                        end_pos.line += 1;
                        end_pos.col = 0;
                    } else {
                        end_pos.col += 1;
                    }
                }
                buf.delete_range(*pos, end_pos);
                *pos
            }
            EditAction::Delete { pos, text } => {
                // Invert delete by inserting the text back
                buf.insert_str(*pos, text)
            }
        };

        self.redo_stack.push(action);
        Some(restore_pos)
    }

    /// Redo previously undone modification on the buffer.
    pub fn redo(&mut self, buf: &mut TextBuffer) -> Option<CursorPos> {
        let action = self.redo_stack.pop()?;
        let restore_pos = match &action {
            EditAction::Insert { pos, text } => buf.insert_str(*pos, text),
            EditAction::Delete { pos, text } => {
                let mut end_pos = *pos;
                for ch in text.chars() {
                    if ch == '\n' {
                        end_pos.line += 1;
                        end_pos.col = 0;
                    } else {
                        end_pos.col += 1;
                    }
                }
                buf.delete_range(*pos, end_pos);
                *pos
            }
        };

        self.undo_stack.push(action);
        Some(restore_pos)
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
