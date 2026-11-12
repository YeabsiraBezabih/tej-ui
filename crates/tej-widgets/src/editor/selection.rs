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
