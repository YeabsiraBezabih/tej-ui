//! Rich interactive multi-line text editor widget with undo/redo and IME support.

use crate::editor::buffer::TextBuffer;
use crate::editor::history::{EditAction, UndoStack};
use crate::editor::ime::ImeState;
use crate::editor::selection::{CursorPos, TextSelection};
use parking_lot::Mutex;
use std::sync::Arc;
use tej_ui_hal::UiEvent;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::{EventPropagation, View};

#[derive(Clone, Debug)]
struct RichTextEditorCore {
    buffer: TextBuffer,
    selection: TextSelection,
    history: UndoStack,
    ime: ImeState,
    read_only: bool,
}

impl RichTextEditorCore {
    fn insert_text(&mut self, text: &str) {
        if self.read_only || text.is_empty() {
            return;
        }

        let replacing_selection = !self.selection.is_collapsed();
        if replacing_selection {
            self.history.begin_transaction();
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

        if replacing_selection {
            self.history.commit_transaction();
        }
    }

    fn backspace(&mut self) {
        if self.read_only {
            return;
        }

        if self.selection.is_collapsed() {
            let head = self.buffer.clamp_pos(self.selection.head);
            let prev_pos = if head.col > 0 {
                Some(CursorPos::new(head.line, head.col - 1))
            } else if head.line > 0 {
                let prev_line = head.line - 1;
                Some(CursorPos::new(prev_line, self.buffer.line_len(prev_line)))
            } else {
                None
            };

            if let Some(start) = prev_pos {
                let deleted = self.buffer.delete_range(start, head);
                if !deleted.is_empty() {
                    self.history.push(EditAction::Delete {
                        pos: start,
                        text: deleted,
                    });
                    self.selection = TextSelection::cursor(start);
                }
            }
        } else {
            let s = self.selection.start();
            let e = self.selection.end();
            let deleted = self.buffer.delete_range(s, e);
            self.history.push(EditAction::Delete {
                pos: s,
                text: deleted,
            });
            self.selection = TextSelection::cursor(s);
        }
    }

    fn delete_forward(&mut self) {
        if self.read_only {
            return;
        }

        if self.selection.is_collapsed() {
            let head = self.buffer.clamp_pos(self.selection.head);
            let line_len = self.buffer.line_len(head.line);
            let next_pos = if head.col < line_len {
                Some(CursorPos::new(head.line, head.col + 1))
            } else if head.line + 1 < self.buffer.line_count() {
                Some(CursorPos::new(head.line + 1, 0))
            } else {
                None
            };

            if let Some(end) = next_pos {
                let deleted = self.buffer.delete_range(head, end);
                if !deleted.is_empty() {
                    self.history.push(EditAction::Delete {
                        pos: head,
                        text: deleted,
                    });
                    self.selection = TextSelection::cursor(head);
                }
            }
        } else {
            let s = self.selection.start();
            let e = self.selection.end();
            let deleted = self.buffer.delete_range(s, e);
            self.history.push(EditAction::Delete {
                pos: s,
                text: deleted,
            });
            self.selection = TextSelection::cursor(s);
        }
    }

    fn newline(&mut self) {
        self.insert_text("\n");
    }

    fn select_all(&mut self) {
        let last_line = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_line);
        self.selection = TextSelection::new(CursorPos::zero(), CursorPos::new(last_line, last_col));
    }

    fn undo(&mut self) -> bool {
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

    fn redo(&mut self) -> bool {
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

    fn move_cursor_left(&mut self, mode: CursorMoveMode) {
        let head = self.selection.head;
        let new_head = if head.col > 0 {
            CursorPos::new(head.line, head.col - 1)
        } else if head.line > 0 {
            let prev_line = head.line - 1;
            CursorPos::new(prev_line, self.buffer.line_len(prev_line))
        } else {
            head
        };

        match mode {
            CursorMoveMode::Extend => self.selection.head = new_head,
            CursorMoveMode::Collapse => self.selection = TextSelection::cursor(new_head),
        }
    }

    fn move_cursor_right(&mut self, mode: CursorMoveMode) {
        let head = self.selection.head;
        let line_len = self.buffer.line_len(head.line);
        let new_head = if head.col < line_len {
            CursorPos::new(head.line, head.col + 1)
        } else if head.line + 1 < self.buffer.line_count() {
            CursorPos::new(head.line + 1, 0)
        } else {
            head
        };

        match mode {
            CursorMoveMode::Extend => self.selection.head = new_head,
            CursorMoveMode::Collapse => self.selection = TextSelection::cursor(new_head),
        }
    }

    fn move_cursor_up(&mut self, mode: CursorMoveMode) {
        let head = self.selection.head;
        let new_head = if head.line > 0 {
            let target_line = head.line - 1;
            let target_col = head.col.min(self.buffer.line_len(target_line));
            CursorPos::new(target_line, target_col)
        } else {
            CursorPos::zero()
        };

        match mode {
            CursorMoveMode::Extend => self.selection.head = new_head,
            CursorMoveMode::Collapse => self.selection = TextSelection::cursor(new_head),
        }
    }

    fn move_cursor_down(&mut self, mode: CursorMoveMode) {
        let head = self.selection.head;
        let new_head = if head.line + 1 < self.buffer.line_count() {
            let target_line = head.line + 1;
            let target_col = head.col.min(self.buffer.line_len(target_line));
            CursorPos::new(target_line, target_col)
        } else {
            CursorPos::new(head.line, self.buffer.line_len(head.line))
        };

        match mode {
            CursorMoveMode::Extend => self.selection.head = new_head,
            CursorMoveMode::Collapse => self.selection = TextSelection::cursor(new_head),
        }
    }
}

/// Interactive multi-line text editor component supporting selections, undo/redo, and native IME.
#[derive(Clone, Debug)]
pub struct RichTextEditor {
    shared: Arc<Mutex<RichTextEditorCore>>,
    buffer: TextBuffer,
    ime: ImeState,
    font_size: f32,
    line_height: f32,
    show_line_numbers: bool,
    read_only: bool,
}

/// Whether cursor movement collapses or extends the active text selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMoveMode {
    /// Collapse the selection to the new cursor position.
    Collapse,
    /// Extend the selection from the anchor to the new cursor position.
    Extend,
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
        let buffer = TextBuffer::new();
        let ime = ImeState::new();
        let core = RichTextEditorCore {
            buffer: buffer.clone(),
            selection: TextSelection::cursor(CursorPos::zero()),
            history: UndoStack::default(),
            ime: ime.clone(),
            read_only: false,
        };
        Self {
            shared: Arc::new(Mutex::new(core)),
            buffer,
            ime,
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
        let buf = TextBuffer::from_text(text);
        editor.buffer = buf.clone();
        editor.shared.lock().buffer = buf;
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

    /// Show line numbers in the left gutter.
    #[must_use]
    pub const fn with_line_numbers(mut self) -> Self {
        self.show_line_numbers = true;
        self
    }

    /// Hide line numbers in the left gutter.
    #[must_use]
    pub const fn hide_line_numbers(mut self) -> Self {
        self.show_line_numbers = false;
        self
    }

    /// Mark editor as read-only.
    #[must_use]
    pub fn make_read_only(mut self) -> Self {
        self.read_only = true;
        self.shared.lock().read_only = true;
        self
    }

    /// Mark editor as editable.
    #[must_use]
    pub fn editable(mut self) -> Self {
        self.read_only = false;
        self.shared.lock().read_only = false;
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
        self.shared.lock().buffer.text()
    }

    /// Active selection range.
    #[must_use]
    pub fn selection(&self) -> TextSelection {
        self.shared.lock().selection
    }

    /// Active IME composition state.
    #[must_use]
    pub const fn ime(&self) -> &ImeState {
        &self.ime
    }

    fn sync_mirrors(&mut self, core: &RichTextEditorCore) {
        self.buffer = core.buffer.clone();
        self.ime = core.ime.clone();
    }

    /// Insert text at active cursor or atomically replace active selection.
    pub fn insert_text(&mut self, text: &str) {
        let mut core = self.shared.lock();
        core.insert_text(text);
        let cloned = core.clone();
        drop(core);
        self.sync_mirrors(&cloned);
    }

    /// Delete character before cursor (Backspace) or delete active selection.
    pub fn backspace(&mut self) {
        let mut core = self.shared.lock();
        core.backspace();
        let cloned = core.clone();
        drop(core);
        self.sync_mirrors(&cloned);
    }

    /// Delete character after cursor (Delete key) or delete active selection.
    pub fn delete_forward(&mut self) {
        let mut core = self.shared.lock();
        core.delete_forward();
        let cloned = core.clone();
        drop(core);
        self.sync_mirrors(&cloned);
    }

    /// Insert a newline character at current cursor position.
    pub fn newline(&mut self) {
        self.insert_text("\n");
    }

    /// Select all text across the entire buffer.
    pub fn select_all(&mut self) {
        self.shared.lock().select_all();
    }

    /// Undo last edit action, returning true if an action was undone.
    pub fn undo(&mut self) -> bool {
        let mut core = self.shared.lock();
        let ok = core.undo();
        let cloned = core.clone();
        drop(core);
        self.sync_mirrors(&cloned);
        ok
    }

    /// Redo last undone action, returning true if an action was reapplied.
    pub fn redo(&mut self) -> bool {
        let mut core = self.shared.lock();
        let ok = core.redo();
        let cloned = core.clone();
        drop(core);
        self.sync_mirrors(&cloned);
        ok
    }

    /// Move cursor left, collapsing or extending selection according to `mode`.
    pub fn move_cursor_left(&mut self, mode: CursorMoveMode) {
        self.shared.lock().move_cursor_left(mode);
    }

    /// Move cursor right, collapsing or extending selection according to `mode`.
    pub fn move_cursor_right(&mut self, mode: CursorMoveMode) {
        self.shared.lock().move_cursor_right(mode);
    }

    /// Move cursor up by one line, collapsing or extending selection according to `mode`.
    pub fn move_cursor_up(&mut self, mode: CursorMoveMode) {
        self.shared.lock().move_cursor_up(mode);
    }

    /// Move cursor down by one line, collapsing or extending selection according to `mode`.
    pub fn move_cursor_down(&mut self, mode: CursorMoveMode) {
        self.shared.lock().move_cursor_down(mode);
    }

    /// Handle incoming native OS IME composition update event.
    pub fn update_ime(&mut self, preedit: String, cursor: usize) {
        self.ime.update(preedit.clone(), cursor);
        self.shared.lock().ime.update(preedit, cursor);
    }

    /// Commit active IME composition, inserting finalized text into buffer.
    pub fn commit_ime(&mut self) {
        let text = self.ime.commit();
        let mut core = self.shared.lock();
        let _ = core.ime.commit();
        if !text.is_empty() {
            core.insert_text(&text);
        }
        let cloned = core.clone();
        drop(core);
        self.sync_mirrors(&cloned);
    }

    /// Cancel active IME composition session.
    pub fn cancel_ime(&mut self) {
        self.ime.cancel();
        self.shared.lock().ime.cancel();
    }
}

/// Runtime state for `RichTextEditor`.
#[derive(Debug, Default)]
pub struct RichTextEditorState {
    pub bounds: Rect,
    pub line_count: usize,
    pub is_composing: bool,
    pub node: u32,
}

impl View for RichTextEditor {
    type State = RichTextEditorState;

    fn build(&self) -> Self::State {
        let core = self.shared.lock();
        let line_count = core.buffer.line_count();
        let height = (crate::count_to_f32(line_count) * self.line_height).max(100.0);
        RichTextEditorState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(f32::NAN, height)),
            line_count,
            is_composing: core.ime.is_composing,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let core = self.shared.lock();
        state.line_count = core.buffer.line_count();
        state.is_composing = core.ime.is_composing;
        let height = (crate::count_to_f32(state.line_count) * self.line_height).max(100.0);
        state.bounds.size.height = height;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let line_count = self.shared.lock().buffer.line_count();
        let height = (crate::count_to_f32(line_count) * self.line_height).max(100.0);
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Percent(1.0),
            height: tej_ui_layout::Dimension::Points(height),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        let core = self.shared.lock();

        // Editor canvas surface
        ctx.fill_rect(bounds, ctx.theme.colors.background);

        if ctx.is_focused(state.node) {
            let border_l = Rect::new(bounds.origin.x, bounds.origin.y, 2.0, bounds.size.height);
            ctx.fill_rect(border_l, ctx.theme.colors.primary);
        }

        let gutter_w = if self.show_line_numbers { 40.0 } else { 8.0 };
        let text_origin_x = bounds.origin.x + gutter_w + 8.0;
        let char_w = self.font_size * 0.6;

        ctx.push_clip(bounds, [0.0; 4]);

        // Render selection highlight beneath text when selection is non-collapsed
        if !core.selection.is_collapsed() {
            let sel_start = core.buffer.clamp_pos(core.selection.start());
            let sel_end = core.buffer.clamp_pos(core.selection.end());
            let highlight_color = ctx.theme.colors.primary.with_alpha(96);

            for line_idx in sel_start.line..=sel_end.line {
                let start_col = if line_idx == sel_start.line {
                    sel_start.col
                } else {
                    0
                };
                let end_col = if line_idx == sel_end.line {
                    sel_end.col
                } else {
                    core.buffer.line_len(line_idx).max(1)
                };
                let sel_x = text_origin_x + crate::count_to_f32(start_col) * char_w;
                let sel_y =
                    bounds.origin.y + 8.0 + crate::count_to_f32(line_idx) * self.line_height;
                let sel_w =
                    ((crate::count_to_f32(end_col.saturating_sub(start_col))) * char_w).max(4.0);
                ctx.fill_rect(
                    Rect::new(sel_x, sel_y, sel_w, self.line_height),
                    highlight_color,
                );
            }
        }

        for (i, line_content) in core.buffer.lines().iter().enumerate() {
            let line_y = bounds.origin.y + 8.0 + crate::count_to_f32(i) * self.line_height;
            if self.show_line_numbers {
                let num_str = format!("{:>3}", i + 1);
                let num_pt = Point::new(bounds.origin.x + 8.0, line_y);
                ctx.draw_text(
                    &num_str,
                    num_pt,
                    ctx.theme.typography.code.font_size,
                    ctx.theme.colors.muted_foreground,
                );
            }

            let text_pt = Point::new(text_origin_x, line_y);
            ctx.draw_text(
                line_content,
                text_pt,
                ctx.theme.typography.body.font_size,
                ctx.theme.colors.foreground,
            );
        }

        // Render active cursor caret only when focused
        if ctx.is_focused(state.node) {
            let caret = core.buffer.clamp_pos(core.selection.head);
            let caret_x = text_origin_x + crate::count_to_f32(caret.col) * char_w;
            let caret_y =
                bounds.origin.y + 8.0 + crate::count_to_f32(caret.line) * self.line_height;
            ctx.fill_rect(
                Rect::new(caret_x, caret_y, 2.0, self.line_height),
                ctx.theme.colors.primary,
            );
        }

        ctx.pop_clip();
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        // Focusable hit target for mouse click and Tab order
        dispatcher.register_click(state.node, Arc::new(|| {}));

        // Character / text input handler
        if !self.read_only {
            let shared_text = Arc::clone(&self.shared);
            dispatcher.register_text_input(
                state.node,
                Arc::new(move |text| {
                    shared_text.lock().insert_text(text);
                }),
            );

            // Enter (13) and Space (32) handled via bubble phase since EventDispatcher routes 13|32 to click_handlers
            let shared_bubble = Arc::clone(&self.shared);
            dispatcher.register_bubble(
                state.node,
                Arc::new(move |event, _phase, _node| match event {
                    UiEvent::KeyDown { key_code: 13 } => {
                        shared_bubble.lock().newline();
                        EventPropagation::Repaint
                    }
                    UiEvent::KeyDown { key_code: 32 } => {
                        shared_bubble.lock().insert_text(" ");
                        EventPropagation::Repaint
                    }
                    _ => EventPropagation::Continue,
                }),
            );
        }

        // KeyDown handler: Backspace (8), Delete (46|127), Enter (13), Arrow keys (37..=40)
        let shared_key = Arc::clone(&self.shared);
        let read_only = self.read_only;
        dispatcher.register_key_down(
            state.node,
            Arc::new(move |key_code| {
                let mut core = shared_key.lock();
                match key_code {
                    8 if !read_only => core.backspace(),
                    46 | 127 if !read_only => core.delete_forward(),
                    13 if !read_only => core.newline(),
                    37 => core.move_cursor_left(CursorMoveMode::Collapse),
                    38 => core.move_cursor_up(CursorMoveMode::Collapse),
                    39 => core.move_cursor_right(CursorMoveMode::Collapse),
                    40 => core.move_cursor_down(CursorMoveMode::Collapse),
                    _ => {}
                }
            }),
        );
    }
}
