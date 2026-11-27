//! Zero-copy memory-mapped database row lens for virtualized UI grids.
//!
//! Exposes page-slice strings and byte slices (`&str`, `&[u8]`) directly from
//! the underlying database read transaction without copying strings into the UI heap.

/// A lightweight, zero-copy lens into a database record residing in a memory-mapped page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbRowLens<'a> {
    raw_key: &'a [u8],
    raw_value: &'a [u8],
}

impl<'a> DbRowLens<'a> {
    /// Creates a zero-copy row lens over raw memory-mapped slices.
    #[inline]
    #[must_use]
    pub const fn new(raw_key: &'a [u8], raw_value: &'a [u8]) -> Self {
        Self { raw_key, raw_value }
    }

    /// Returns the record key as a raw byte slice.
    #[inline]
    #[must_use]
    pub const fn key_bytes(&self) -> &'a [u8] {
        self.raw_key
    }

    /// Interprets key as UTF-8 string without heap allocation.
    #[inline]
    pub fn key_str(&self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.raw_key)
    }

    /// Returns the record payload as a raw byte slice.
    #[inline]
    #[must_use]
    pub const fn value_bytes(&self) -> &'a [u8] {
        self.raw_value
    }

    /// Interprets payload as UTF-8 string without heap allocation.
    #[inline]
    pub fn value_str(&self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.raw_value)
    }

    /// Slices a sub-string from payload by start and length in zero-copy fashion.
    #[inline]
    pub fn slice_str(&self, start: usize, len: usize) -> Result<&'a str, std::str::Utf8Error> {
        let clamped_start = start.min(self.raw_value.len());
        let end = clamped_start
            .checked_add(len)
            .map_or(self.raw_value.len(), |e| e.min(self.raw_value.len()));
        let slice = &self.raw_value[clamped_start..end];
        std::str::from_utf8(slice)
    }

    /// Parses a fixed-width binary column at byte offset `offset` without copying.
    #[inline]
    #[must_use]
    pub fn read_u64_le(&self, offset: usize) -> Option<u64> {
        let end = offset.checked_add(8)?;
        if end <= self.raw_value.len() {
            let bytes = self.raw_value[offset..end].try_into().ok()?;
            Some(u64::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Parses a 32-bit float column at byte offset `offset` without copying.
    #[inline]
    #[must_use]
    pub fn read_f32_le(&self, offset: usize) -> Option<f32> {
        let end = offset.checked_add(4)?;
        if end <= self.raw_value.len() {
            let bytes = self.raw_value[offset..end].try_into().ok()?;
            Some(f32::from_le_bytes(bytes))
        } else {
            None
        }
    }
}

/// Zero-copy virtual grid window coordinator.
///
/// Computes visible index bounds and projects memory-mapped rows onto visible cells.
#[derive(Debug, Clone)]
pub struct DbVirtualWindow {
    total_rows: usize,
    row_height: f32,
    viewport_height: f32,
    start_idx: usize,
    end_idx: usize,
}

impl DbVirtualWindow {
    #[must_use]
    pub fn new(total_rows: usize, row_height: f32, viewport_height: f32) -> Self {
        let mut s = Self {
            total_rows,
            row_height: row_height.max(1.0),
            viewport_height: viewport_height.max(1.0),
            start_idx: 0,
            end_idx: 0,
        };
        s.update_scroll(0.0, row_height, viewport_height);
        s
    }

    /// Updates the viewport scroll position and updates the visible window.
    pub fn update_scroll(&mut self, scroll_y: f32, row_height: f32, viewport_height: f32) {
        self.row_height = row_height.max(1.0);
        self.viewport_height = viewport_height.max(1.0);
        let (start, end) = self.visible_range(scroll_y);
        self.start_idx = start;
        self.end_idx = end;
    }

    /// Starting visible row index (inclusive).
    #[must_use]
    pub const fn start_index(&self) -> usize {
        self.start_idx
    }

    /// Ending visible row index (exclusive).
    #[must_use]
    pub const fn end_index(&self) -> usize {
        self.end_idx
    }

    /// Tests whether a given row index falls within the active visible viewport window.
    #[must_use]
    pub const fn is_visible(&self, idx: usize) -> bool {
        idx >= self.start_idx && idx < self.end_idx
    }

    /// Computes the visible range `(start_idx, end_idx)` for a scroll offset.
    #[inline]
    #[must_use]
    pub fn visible_range(&self, scroll_y: f32) -> (usize, usize) {
        let y = scroll_y.max(0.0);
        let start = (y / self.row_height).floor() as usize;
        let visible_count = ((self.viewport_height / self.row_height).ceil() as usize) + 2;
        let end = start.saturating_add(visible_count).min(self.total_rows);
        (start.min(self.total_rows), end)
    }

    /// Computes total virtual content height.
    #[inline]
    #[must_use]
    pub fn total_content_height(&self) -> f32 {
        self.total_rows as f32 * self.row_height
    }
}
