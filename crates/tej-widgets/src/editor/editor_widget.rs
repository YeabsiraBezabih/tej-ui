//! Rich interactive multi-line text editor widget with undo/redo and IME support.

use crate::editor::buffer::TextBuffer;
use crate::editor::history::{EditAction, UndoStack};
use crate::editor::ime::ImeState;
use crate::editor::selection::{CursorPos, TextSelection};
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Interactive multi-line text editor component supporting selections, undo/redo, and native IME.
#[derive(Clone, Debug)]
pub struct RichTextEditor {
    buffer: TextBuffer,
    selection: TextSelection,
    history: UndoStack,
    ime: ImeState,
    font_size: f32,
    line_height: f32,
    show_line_numbers: bool,
    read_only: bool,
}

impl Default for RichTextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl RichTextEditor {
    /// Create a new empty rich text editor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            selection: TextSelection::cursor(CursorPos::zero()),
            history: UndoStack::default(),
            ime: ImeState::new(),
            font_size: 14.0,
            line_height: 20.0,
            show_line_numbers: true,
            read_only: false,
        }
    }

    /// Construct editor pre-populated with initial text content.
    #[must_use]
    pub fn with_text(text: &str) -> Self {
        let mut editor = Self::new();
        editor.buffer = TextBuffer::from_text(text);
        editor
    }

    /// Set font size in logical points.
    #[must_use]
    pub const fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set line height in logical points.
    #[must_use]
    pub const fn line_height(mut self, height: f32) -> Self {
        self.line_height = height;
        self
    }

    /// Toggle line number display in left gutter.
    #[must_use]
    pub const fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Set read-only state.
    #[must_use]
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Active text buffer reference.
    #[must_use]
    pub const fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    /// Full text content of the buffer.
    #[must_use]
    pub fn text(&self) -> String {
        self.buffer.text()
    }

    /// Active selection range.
    #[must_use]
    pub const fn selection(&self) -> TextSelection {
        self.selection
    }

    /// Active IME composition state.
    #[must_use]
    pub const fn ime(&self) -> &ImeState {
        &self.ime
    }

    /// Insert text at active cursor or replace active selection.
    pub fn insert_text(&mut self, text: &str) {
        if self.read_only || text.is_empty() {
            return;
        }

        // Delete selection if not collapsed
        if !self.selection.is_collapsed() {
            let s = self.selection.start();
            let e = self.selection.end();
            let deleted = self.buffer.delete_range(s, e);
            self.history.push(EditAction::Delete {
                pos: s,
                text: deleted,
            });
            self.selection = TextSelection::cursor(s);
        }

        let pos = self.selection.head;
        let new_pos = self.buffer.insert_str(pos, text);
        self.history.push(EditAction::Insert {
            pos,
            text: text.to_string(),
        });
        self.selection = TextSelection::cursor(new_pos);
    }

    /// Delete character before cursor (Backspace) or delete active selection.
    pub fn backspace(&mut self) {
        if self.read_only {
            return;
        }

        if !self.selection.is_collapsed() {
            let s = self.selection.start();
            let e = self.selection.end();
            let deleted = self.buffer.delete_range(s, e);
            self.history.push(EditAction::Delete {
                pos: s,
                text: deleted,
            });
            self.selection = TextSelection::cursor(s);
        } else if let Some(new_pos) = self.buffer.delete_char_before(self.selection.head) {
            self.selection = TextSelection::cursor(new_pos);
        }
    }

    /// Delete character after cursor (Delete key).
    pub fn delete_forward(&mut self) {
        if self.read_only {
            return;
        }

        if !self.selection.is_collapsed() {
            let s = self.selection.start();
            let e = self.selection.end();
            let deleted = self.buffer.delete_range(s, e);
            self.history.push(EditAction::Delete {
                pos: s,
                text: deleted,
            });
            self.selection = TextSelection::cursor(s);
        } else if let Some(new_pos) = self.buffer.delete_char_after(self.selection.head) {
            self.selection = TextSelection::cursor(new_pos);
        }
    }

    /// Insert a newline character at current cursor position.
    pub fn newline(&mut self) {
        self.insert_text("\n");
    }

    /// Select all text across the entire buffer.
    pub fn select_all(&mut self) {
        let last_line = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_line);
        self.selection = TextSelection::new(CursorPos::zero(), CursorPos::new(last_line, last_col));
    }

    /// Undo last edit action, returning true if an action was undone.
    pub fn undo(&mut self) -> bool {
        if self.read_only {
            return false;
        }
        if let Some(pos) = self.history.undo(&mut self.buffer) {
            self.selection = TextSelection::cursor(pos);
            true
        } else {
            false
        }
    }

    /// Redo last undone action, returning true if an action was reapplied.
    pub fn redo(&mut self) -> bool {
        if self.read_only {
            return false;
        }
        if let Some(pos) = self.history.redo(&mut self.buffer) {
            self.selection = TextSelection::cursor(pos);
            true
        } else {
            false
        }
    }

    /// Move cursor left, optionally extending selection.
    pub fn move_cursor_left(&mut self, select: bool) {
        let head = self.selection.head;
        let new_head = if head.col > 0 {
            CursorPos::new(head.line, head.col - 1)
        } else if head.line > 0 {
            let prev_line = head.line - 1;
            CursorPos::new(prev_line, self.buffer.line_len(prev_line))
        } else {
            head
        };

        if select {
            self.selection.head = new_head;
        } else {
            self.selection = TextSelection::cursor(new_head);
        }
    }

    /// Move cursor right, optionally extending selection.
    pub fn move_cursor_right(&mut self, select: bool) {
        let head = self.selection.head;
        let line_len = self.buffer.line_len(head.line);
        let new_head = if head.col < line_len {
            CursorPos::new(head.line, head.col + 1)
        } else if head.line + 1 < self.buffer.line_count() {
            CursorPos::new(head.line + 1, 0)
        } else {
            head
        };

        if select {
            self.selection.head = new_head;
        } else {
            self.selection = TextSelection::cursor(new_head);
        }
    }

    /// Move cursor up by one line, optionally extending selection.
    pub fn move_cursor_up(&mut self, select: bool) {
        let head = self.selection.head;
        let new_head = if head.line > 0 {
            let target_line = head.line - 1;
            let target_col = head.col.min(self.buffer.line_len(target_line));
            CursorPos::new(target_line, target_col)
        } else {
            CursorPos::zero()
        };

        if select {
            self.selection.head = new_head;
        } else {
            self.selection = TextSelection::cursor(new_head);
        }
    }

    /// Move cursor down by one line, optionally extending selection.
    pub fn move_cursor_down(&mut self, select: bool) {
        let head = self.selection.head;
        let new_head = if head.line + 1 < self.buffer.line_count() {
            let target_line = head.line + 1;
            let target_col = head.col.min(self.buffer.line_len(target_line));
            CursorPos::new(target_line, target_col)
        } else {
            CursorPos::new(head.line, self.buffer.line_len(head.line))
        };

        if select {
            self.selection.head = new_head;
        } else {
            self.selection = TextSelection::cursor(new_head);
        }
    }

    /// Handle incoming native OS IME composition update event.
    pub fn update_ime(&mut self, preedit: String, cursor: usize) {
        self.ime.update(preedit, cursor);
    }

    /// Commit active IME composition, inserting finalized text into buffer.
    pub fn commit_ime(&mut self) {
        let text = self.ime.commit();
        if !text.is_empty() {
            self.insert_text(&text);
        }
    }

    /// Cancel active IME composition session.
    pub fn cancel_ime(&mut self) {
        self.ime.cancel();
    }
}

/// Runtime state for `RichTextEditor`.
#[derive(Debug, Default)]
pub struct RichTextEditorState {
    pub bounds: Rect,
    pub line_count: usize,
    pub is_composing: bool,
}

impl View for RichTextEditor {
    type State = RichTextEditorState;

    fn build(&self) -> Self::State {
        let height = (self.buffer.line_count() as f32 * self.line_height).max(100.0);
        RichTextEditorState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(f32::NAN, height)),
            line_count: self.buffer.line_count(),
            is_composing: self.ime.is_composing,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.line_count = self.buffer.line_count();
        state.is_composing = self.ime.is_composing;
        let height = (self.buffer.line_count() as f32 * self.line_height).max(100.0);
        state.bounds.size.height = height;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
