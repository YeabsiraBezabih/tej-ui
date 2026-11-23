//! Multi-rect dirty region tracking and partial layer invalidation.

use tej_ui_layout::Rect;

/// Manages a collection of dirty rectangles, merging overlapping rects to minimize draw calls.
#[derive(Debug, Clone, Default)]
pub struct DirtyRegion {
    rects: Vec<Rect>,
    bounding_box: Option<Rect>,
    max_rects: usize,
}

impl DirtyRegion {
    /// Creates a dirty region tracker with a max rect threshold before collapsing to a single bounding box.
    pub fn new(max_rects: usize) -> Self {
        Self {
            rects: Vec::new(),
            bounding_box: None,
            max_rects: max_rects.max(1),
        }
    }

    /// Marks a rectangular area as dirty.
    pub fn mark_dirty(&mut self, rect: Rect) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }

        // Update total bounding box
        self.bounding_box = Some(match self.bounding_box {
            Some(bb) => bb.union(&rect),
            None => rect,
        });

        // If we exceeded max separate rects, we collapse to bounding box to prevent $O(N^2)$ blit overhead
        if self.rects.len() >= self.max_rects {
            self.rects.clear();
            if let Some(bb) = self.bounding_box {
                self.rects.push(bb);
            }
            return;
        }

        // Try merging with an existing intersecting rectangle
        for existing in &mut self.rects {
            if existing.intersects(&rect) {
                *existing = existing.union(&rect);
                return;
            }
        }

        self.rects.push(rect);
    }

    /// True if there are no dirty regions (zero-draw idle frame).
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    /// Returns the minimal enclosing bounding box of all dirty regions.
    pub fn bounding_box(&self) -> Option<Rect> {
        self.bounding_box
    }

    /// Returns the slice of active dirty rectangles to clip and repaint.
    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    /// Clears all dirty state after presentation.
    pub fn clear(&mut self) {
        self.rects.clear();
        self.bounding_box = None;
    }
}
