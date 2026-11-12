use smallvec::SmallVec;
use tej_layout::Rect;

/// Dirty rectangle accumulator and clip manager for tile-based invalidation.
///
/// Prevents full-screen 4K rasterization passes when only fine-grained widgets change.
#[derive(Debug, Default, Clone)]
pub struct DirtyRectTracker {
    dirty_rects: SmallVec<[Rect; 8]>,
    is_fully_dirty: bool,
}

impl DirtyRectTracker {
    /// Create an empty tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dirty region rectangle to be repainted.
    pub fn add_rect(&mut self, rect: Rect) {
        if self.is_fully_dirty || rect.size.width <= 0.0 || rect.size.height <= 0.0 {
            return;
        }

        // Try merging with existing overlapping dirty rects to keep count low
        for existing in &mut self.dirty_rects {
            if existing.intersection(&rect).is_some() {
                *existing = existing.union(&rect);
                return;
            }
        }

        if self.dirty_rects.len() >= 16 {
            // Consolidate into full window invalidation if fragmentation becomes excessive
            self.mark_full_dirty();
        } else {
            self.dirty_rects.push(rect);
        }
    }

    /// Invalidate entire window area.
    pub fn mark_full_dirty(&mut self) {
        self.is_fully_dirty = true;
        self.dirty_rects.clear();
    }

    /// Check if repainting is needed.
    #[must_use]
    pub fn has_dirty(&self) -> bool {
        self.is_fully_dirty || !self.dirty_rects.is_empty()
    }

    /// Get slice of dirty bounding boxes.
    #[must_use]
    pub fn rects(&self) -> &[Rect] {
        &self.dirty_rects
    }

    /// Check if entire surface is marked dirty.
    #[must_use]
    pub const fn is_full_dirty(&self) -> bool {
        self.is_fully_dirty
    }

    /// Reset dirty tracking after frame presentation.
    pub fn reset(&mut self) {
        self.is_fully_dirty = false;
        self.dirty_rects.clear();
    }
}
