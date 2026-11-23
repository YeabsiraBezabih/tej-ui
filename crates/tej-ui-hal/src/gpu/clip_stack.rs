//! Hardware scissor and secondary analytic SDF clip stack.

use crate::gpu::types::{GpuError, ScissorRect};

/// An active clipping state entry combining physical pixel scissor and logical SDF bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipEntry {
    /// Physical pixel scissor box sent to the hardware rasterizer.
    pub scissor: ScissorRect,
    /// Logical bounds `[x, y, width, height]` in view units.
    pub clip_bounds: [f32; 4],
    /// Corner radii `[top_left, top_right, bottom_right, bottom_left]` for analytic fragment clipping.
    pub clip_radii: [f32; 4],
}

const EMPTY_CLIP_ENTRY: ClipEntry = ClipEntry {
    scissor: ScissorRect::new(0, 0, 0, 0),
    clip_bounds: [0.0, 0.0, 0.0, 0.0],
    clip_radii: [0.0; 4],
};

/// Pre-allocated stack maintaining nested clipping boundaries with intersection math.
#[derive(Debug)]
pub struct ClipStack {
    entries: Vec<ClipEntry>,
}

impl Default for ClipStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipStack {
    /// Constructs a clip stack with pre-allocated depth capacity.
    #[must_use]
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(32);
        entries.push(ClipEntry {
            scissor: ScissorRect::new(0, 0, 0, 0),
            clip_bounds: [-1e6, -1e6, 2e6, 2e6],
            clip_radii: [0.0; 4],
        });
        Self { entries }
    }

    /// Reset stack to viewport dimensions for a new frame without deallocating vector capacity.
    pub fn reset(&mut self, viewport_width_px: u32, viewport_height_px: u32, scale_factor: f32) {
        self.entries.clear();
        let logical_w = if scale_factor > 0.0 {
            (viewport_width_px as f32) / scale_factor
        } else {
            viewport_width_px as f32
        };
        let logical_h = if scale_factor > 0.0 {
            (viewport_height_px as f32) / scale_factor
        } else {
            viewport_height_px as f32
        };

        self.entries.push(ClipEntry {
            scissor: ScissorRect::new(0, 0, viewport_width_px, viewport_height_px),
            clip_bounds: [0.0, 0.0, logical_w, logical_h],
            clip_radii: [0.0; 4],
        });
    }

    /// Push a nested clip rectangle with corner radii, intersecting with current top.
    pub fn push(&mut self, logical_bounds: [f32; 4], corner_radii: [f32; 4], scale_factor: f32) {
        let parent = self.current();

        // 1. Intersect logical bounds
        let px0 = parent.clip_bounds[0];
        let py0 = parent.clip_bounds[1];
        let px1 = px0 + parent.clip_bounds[2];
        let py1 = py0 + parent.clip_bounds[3];

        let cx0 = logical_bounds[0];
        let cy0 = logical_bounds[1];
        let cx1 = cx0 + logical_bounds[2];
        let cy1 = cy0 + logical_bounds[3];

        let ix0 = px0.max(cx0);
        let iy0 = py0.max(cy0);
        let ix1 = px1.min(cx1);
        let iy1 = py1.min(cy1);

        let intersected_logical = if ix1 > ix0 && iy1 > iy0 {
            [ix0, iy0, ix1 - ix0, iy1 - iy0]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        // 2. Compute physical scissor box
        let sf = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let sx0 = (cx0 * sf).floor().max(0.0) as u32;
        let sy0 = (cy0 * sf).floor().max(0.0) as u32;
        let sx1 = (cx1 * sf).ceil().max(0.0) as u32;
        let sy1 = (cy1 * sf).ceil().max(0.0) as u32;

        let child_scissor = if sx1 > sx0 && sy1 > sy0 {
            ScissorRect::new(sx0, sy0, sx1 - sx0, sy1 - sy0)
        } else {
            ScissorRect::new(0, 0, 0, 0)
        };

        let intersected_scissor = parent.scissor.intersect(child_scissor);

        self.entries.push(ClipEntry {
            scissor: intersected_scissor,
            clip_bounds: intersected_logical,
            clip_radii: corner_radii,
        });
    }

    /// Pop the top clip boundary. Returns `ClipStackUnderflow` if trying to pop root.
    pub fn pop(&mut self) -> Result<(), GpuError> {
        if self.entries.len() <= 1 {
            return Err(GpuError::ClipStackUnderflow);
        }
        self.entries.pop();
        Ok(())
    }

    /// Returns the currently active clipping entry.
    #[must_use]
    pub fn current(&self) -> &ClipEntry {
        self.entries.last().unwrap_or(&EMPTY_CLIP_ENTRY)
    }

    /// Returns `true` if current clip region has zero area (culled from rendering).
    #[must_use]
    pub fn is_culled(&self) -> bool {
        let cur = self.current();
        cur.scissor.is_empty() || cur.clip_bounds[2] <= 0.0 || cur.clip_bounds[3] <= 0.0
    }

    /// Current stack depth.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.entries.len()
    }
}
