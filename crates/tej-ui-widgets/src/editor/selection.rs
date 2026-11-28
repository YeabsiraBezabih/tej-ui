//! Cursor position and multi-line selection ranges.

/// 2D cursor position in line and column character coordinates (0-indexed).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct CursorPos {
    /// 0-indexed line number.
    pub line: usize,
    /// 0-indexed UTF-8 character column offset.
    pub col: usize,
}

impl CursorPos {
    /// Create a new cursor position at specified line and column.
    #[must_use]
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    /// Origin coordinate (line 0, col 0).
    #[must_use]
    pub const fn zero() -> Self {
        Self { line: 0, col: 0 }
    }
}

/// Active text selection range with an anchor (drag start) and a head (current cursor).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct TextSelection {
    /// Initial point where selection began.
    pub anchor: CursorPos,
    /// Moving point representing the active cursor position.
    pub head: CursorPos,
}

impl TextSelection {
    /// Create a collapsed cursor selection at `pos`.
    #[must_use]
    pub const fn cursor(pos: CursorPos) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    /// Create an explicit selection range from `anchor` to `head`.
    #[must_use]
    pub const fn new(anchor: CursorPos, head: CursorPos) -> Self {
        Self { anchor, head }
    }

    /// Check if the selection is collapsed (no characters selected, pure cursor).
    #[must_use]
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// Normalized start position (min of anchor and head).
    #[must_use]
    pub fn start(&self) -> CursorPos {
        if self.anchor <= self.head {
            self.anchor
        } else {
            self.head
        }
    }

    /// Normalized end position (max of anchor and head).
    #[must_use]
    pub fn end(&self) -> CursorPos {
        if self.anchor <= self.head {
            self.head
        } else {
            self.anchor
        }
    }

    /// Collapse selection to current head cursor.
    pub fn collapse_to_head(&mut self) {
        self.anchor = self.head;
    }
}

/// Multi-cursor selection container maintaining multiple active selections.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MultiCursorSelection {
    selections: Vec<TextSelection>,
    primary_index: usize,
}

impl Default for MultiCursorSelection {
    fn default() -> Self {
        Self::new(CursorPos::zero())
    }
}

impl MultiCursorSelection {
    /// Create a multi-cursor selection with a single cursor at `pos`.
    #[must_use]
    pub fn new(pos: CursorPos) -> Self {
        Self {
            selections: vec![TextSelection::cursor(pos)],
            primary_index: 0,
        }
    }

    /// Number of active selections / cursors.
    #[must_use]
    pub fn count(&self) -> usize {
        self.selections.len()
    }

    /// Read-only slice of all active selections.
    #[must_use]
    pub fn selections(&self) -> &[TextSelection] {
        &self.selections
    }

    /// Get the primary active selection.
    #[must_use]
    pub fn primary(&self) -> &TextSelection {
        &self.selections[self
            .primary_index
            .min(self.selections.len().saturating_sub(1))]
    }

    /// Get mutable reference to primary selection.
    pub fn primary_mut(&mut self) -> &mut TextSelection {
        let idx = self
            .primary_index
            .min(self.selections.len().saturating_sub(1));
        &mut self.selections[idx]
    }

    /// Add an additional cursor at `pos`.
    pub fn add_cursor(&mut self, pos: CursorPos) {
        self.add_selection(TextSelection::cursor(pos));
    }

    /// Add an explicit selection range and merge if overlapping.
    pub fn add_selection(&mut self, sel: TextSelection) {
        self.selections.push(sel);
        self.merge_and_sort();
    }

    /// Clear all secondary selections, keeping only the primary selection.
    pub fn clear_secondary(&mut self) {
        let prim = *self.primary();
        self.selections.clear();
        self.selections.push(prim);
        self.primary_index = 0;
    }

    /// Sort selections top-to-bottom and merge overlapping or identical ranges.
    pub fn merge_and_sort(&mut self) {
        if self.selections.len() <= 1 {
            return;
        }

        self.selections.sort_by_key(TextSelection::start);

        let mut merged: Vec<TextSelection> = Vec::with_capacity(self.selections.len());
        for sel in &self.selections {
            if let Some(last) = merged.last_mut() {
                // If adjacent or overlapping on the same line, or contiguous
                if last.end() >= sel.start() {
                    let new_start = last.start();
                    let new_end = last.end().max(sel.end());
                    *last = TextSelection::new(new_start, new_end);
                    continue;
                }
            }
            merged.push(*sel);
        }

        self.selections = merged;
        if self.primary_index >= self.selections.len() {
            self.primary_index = self.selections.len().saturating_sub(1);
        }
    }
}
