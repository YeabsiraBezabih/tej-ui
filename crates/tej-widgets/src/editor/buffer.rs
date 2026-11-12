//! Multi-line UTF-8 text buffer with fast coordinate mapping.

use crate::editor::selection::CursorPos;

/// In-memory multi-line text buffer storing lines of text with coordinate mapping.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextBuffer {
    lines: Vec<String>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for TextBuffer {
    fn from(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(String::from).collect()
        };
        Self { lines }
    }
}

impl TextBuffer {
    /// Create a new empty text buffer containing one empty line.
    #[must_use]
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    /// Construct a text buffer pre-populated with string content.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        Self::from(text)
    }

    /// Read-only slice of lines in the buffer.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Number of lines in the buffer.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Full text content joined with newline characters.
    #[must_use]
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Character length of specified line.
    #[must_use]
    pub fn line_len(&self, line: usize) -> usize {
        self.lines.get(line).map_or(0, |l| l.chars().count())
    }

    /// Clamp arbitrary cursor coordinates to valid bounds within the buffer.
    #[must_use]
    pub fn clamp_pos(&self, pos: CursorPos) -> CursorPos {
        let line = pos.line.min(self.lines.len().saturating_sub(1));
        let col = pos.col.min(self.line_len(line));
        CursorPos::new(line, col)
    }

    /// Insert single character at cursor position, returning updated cursor position.
    pub fn insert_char(&mut self, pos: CursorPos, ch: char) -> CursorPos {
        let clamped = self.clamp_pos(pos);
        if ch == '\n' {
            let line = &self.lines[clamped.line];
            let byte_idx = line
                .char_indices()
                .nth(clamped.col)
                .map_or(line.len(), |(idx, _)| idx);

            let rest = line[byte_idx..].to_string();
            self.lines[clamped.line].truncate(byte_idx);
            self.lines.insert(clamped.line + 1, rest);
            CursorPos::new(clamped.line + 1, 0)
        } else {
            let line = &mut self.lines[clamped.line];
            let byte_idx = line
                .char_indices()
                .nth(clamped.col)
                .map_or(line.len(), |(idx, _)| idx);
            line.insert(byte_idx, ch);
            CursorPos::new(clamped.line, clamped.col + 1)
        }
    }

    /// Insert a string slice at cursor position, handling embedded newlines.
    pub fn insert_str(&mut self, mut pos: CursorPos, text: &str) -> CursorPos {
        for ch in text.chars() {
            pos = self.insert_char(pos, ch);
        }
        pos
    }

    /// Delete character immediately before cursor (Backspace behavior).
    pub fn delete_char_before(&mut self, pos: CursorPos) -> Option<CursorPos> {
        let clamped = self.clamp_pos(pos);
        if clamped.col > 0 {
            let line = &mut self.lines[clamped.line];
            let byte_idx = line.char_indices().nth(clamped.col - 1)?.0;
            line.remove(byte_idx);
            Some(CursorPos::new(clamped.line, clamped.col - 1))
        } else if clamped.line > 0 {
            // Join current line to previous line
            let current_line = self.lines.remove(clamped.line);
            let prev_line = &mut self.lines[clamped.line - 1];
            let prev_col = prev_line.chars().count();
            prev_line.push_str(&current_line);
            Some(CursorPos::new(clamped.line - 1, prev_col))
        } else {
            None
        }
    }

    /// Delete character immediately after cursor (Delete key behavior).
    pub fn delete_char_after(&mut self, pos: CursorPos) -> Option<CursorPos> {
        let clamped = self.clamp_pos(pos);
        let line_len = self.line_len(clamped.line);
        if clamped.col < line_len {
            let line = &mut self.lines[clamped.line];
            let byte_idx = line.char_indices().nth(clamped.col)?.0;
            line.remove(byte_idx);
            Some(clamped)
        } else if clamped.line + 1 < self.lines.len() {
            let next_line = self.lines.remove(clamped.line + 1);
            self.lines[clamped.line].push_str(&next_line);
            Some(clamped)
        } else {
            None
        }
    }

    /// Delete a range of text between start and end positions, returning the deleted text.
    pub fn delete_range(&mut self, start: CursorPos, end: CursorPos) -> String {
        let s = self.clamp_pos(start.min(end));
        let e = self.clamp_pos(start.max(end));

        if s == e {
            return String::new();
        }

        if s.line == e.line {
            let line = &mut self.lines[s.line];
            let start_b = line
                .char_indices()
                .nth(s.col)
                .map_or(line.len(), |(idx, _)| idx);
            let end_b = line
                .char_indices()
                .nth(e.col)
                .map_or(line.len(), |(idx, _)| idx);
            let deleted = line[start_b..end_b].to_string();
            line.replace_range(start_b..end_b, "");
            deleted
        } else {
            let mut deleted = String::new();
            // Start line tail
            let s_byte = self.lines[s.line]
                .char_indices()
                .nth(s.col)
                .map_or(self.lines[s.line].len(), |(idx, _)| idx);
            deleted.push_str(&self.lines[s.line][s_byte..]);
            deleted.push('\n');
            self.lines[s.line].truncate(s_byte);

            // Middle lines deleted completely
            for _ in (s.line + 1)..e.line {
                let removed = self.lines.remove(s.line + 1);
                deleted.push_str(&removed);
                deleted.push('\n');
            }

            // End line head merged into start line
            let e_line = self.lines.remove(s.line + 1);
            let e_byte = e_line
                .char_indices()
                .nth(e.col)
                .map_or(e_line.len(), |(idx, _)| idx);
            deleted.push_str(&e_line[..e_byte]);
            self.lines[s.line].push_str(&e_line[e_byte..]);

            deleted
        }
    }
}
