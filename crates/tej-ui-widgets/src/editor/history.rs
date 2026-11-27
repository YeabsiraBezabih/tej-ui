//! Undo/redo history stack for transaction-safe text editing with typing coalescing and multi-cursor batches.

use crate::editor::buffer::TextBuffer;
use crate::editor::selection::CursorPos;

/// Individual recorded atomic text modification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EditAction {
    /// Text inserted at position.
    Insert { pos: CursorPos, text: String },
    /// Text deleted from position.
    Delete { pos: CursorPos, text: String },
    /// Group of multiple simultaneous edits (e.g. multi-cursor edits or atomic transactions).
    Batch(Vec<EditAction>),
}

/// Transactional undo and redo buffer with configurable history capacity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UndoStack {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    transaction_depth: usize,
    pending_transaction: Vec<EditAction>,
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
            transaction_depth: 0,
            pending_transaction: Vec::new(),
            max_history,
        }
    }

    /// Record a newly executed edit action. Clears redo stack.
    pub fn push(&mut self, action: EditAction) {
        if self.transaction_depth > 0 {
            self.pending_transaction.push(action);
            return;
        }

        self.redo_stack.clear();
        if self.max_history == 0 {
            return;
        }

        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(action);
    }

    /// Push an edit action with typing coalescing.
    /// Consecutive single-character inserts at adjacent cursor columns are merged into one.
    pub fn push_coalesced(&mut self, action: EditAction) {
        if self.transaction_depth > 0 {
            self.pending_transaction.push(action);
            return;
        }

        if let EditAction::Insert { pos, text } = &action {
            if let Some(EditAction::Insert {
                pos: prev_pos,
                text: prev_text,
            }) = self.undo_stack.last_mut()
            {
                // Only coalesce single-line continuous typing bursts without newlines
                if !prev_text.contains('\n') && !text.contains('\n') {
                    let expected_col = prev_pos.col + prev_text.chars().count();
                    if prev_pos.line == pos.line && expected_col == pos.col {
                        prev_text.push_str(text);
                        self.redo_stack.clear();
                        return;
                    }
                }
            }
        }

        self.push(action);
    }

    /// Begin an atomic multi-edit transaction (e.g. for multi-cursor edits).
    pub fn begin_transaction(&mut self) {
        self.transaction_depth += 1;
    }

    /// Commit the current atomic transaction, coalescing recorded actions into a single `Batch`.
    pub fn commit_transaction(&mut self) {
        if self.transaction_depth == 0 {
            return;
        }
        self.transaction_depth -= 1;
        if self.transaction_depth == 0 && !self.pending_transaction.is_empty() {
            let actions = std::mem::take(&mut self.pending_transaction);
            self.redo_stack.clear();
            if self.max_history == 0 {
                return;
            }
            if self.undo_stack.len() >= self.max_history {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(EditAction::Batch(actions));
        }
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
        let restore_pos = invert_action(&action, buf);
        self.redo_stack.push(action);
        restore_pos
    }

    /// Redo previously undone modification on the buffer.
    pub fn redo(&mut self, buf: &mut TextBuffer) -> Option<CursorPos> {
        let action = self.redo_stack.pop()?;
        let restore_pos = forward_action(&action, buf);
        self.undo_stack.push(action);
        restore_pos
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.pending_transaction.clear();
        self.transaction_depth = 0;
    }
}

fn invert_action(action: &EditAction, buf: &mut TextBuffer) -> Option<CursorPos> {
    match action {
        EditAction::Insert { pos, text } => {
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
            Some(*pos)
        }
        EditAction::Delete { pos, text } => {
            let restore_pos = buf.insert_str(*pos, text);
            Some(restore_pos)
        }
        EditAction::Batch(actions) => {
            let mut last_pos = None;
            for act in actions.iter().rev() {
                let p = invert_action(act, buf);
                if last_pos.is_none() {
                    last_pos = p;
                }
            }
            last_pos
        }
    }
}

fn forward_action(action: &EditAction, buf: &mut TextBuffer) -> Option<CursorPos> {
    match action {
        EditAction::Insert { pos, text } => Some(buf.insert_str(*pos, text)),
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
            Some(*pos)
        }
        EditAction::Batch(actions) => {
            let mut last_pos = None;
            for act in actions.iter() {
                last_pos = forward_action(act, buf);
            }
            last_pos
        }
    }
}
