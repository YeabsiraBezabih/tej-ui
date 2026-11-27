use crate::geometry::Rect;
use crate::style::LayoutStyle;
use smallvec::SmallVec;

/// Flat Structure-of-Arrays (SoA) storage for layout nodes.
///
/// Keeps style, computed bounds, and hierarchy in separate contiguous vectors
/// to maximize CPU cache locality and eliminate pointer-heavy heap node trees.
#[derive(Debug, Default)]
pub struct LayoutArena {
    pub styles: Vec<LayoutStyle>,
    pub computed_bounds: Vec<Rect>,
    pub children: Vec<SmallVec<[u32; 4]>>,
    pub parents: Vec<Option<u32>>,
    pub dirty: Vec<bool>,
}

impl LayoutArena {
    /// Create a new, empty layout arena.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-allocate storage for `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            styles: Vec::with_capacity(capacity),
            computed_bounds: Vec::with_capacity(capacity),
            children: Vec::with_capacity(capacity),
            parents: Vec::with_capacity(capacity),
            dirty: Vec::with_capacity(capacity),
        }
    }

    /// Allocate a new layout node returning its index.
    pub fn allocate_node(&mut self, style: LayoutStyle) -> u32 {
        let index = self.styles.len() as u32;
        self.styles.push(style);
        self.computed_bounds.push(Rect::ZERO);
        self.children.push(SmallVec::new());
        self.parents.push(None);
        self.dirty.push(true);
        index
    }

    /// Bulk-allocates child nodes under `parent` in a single contiguous pass,
    /// eliminating repeated per-node reallocations and reducing cache misses.
    pub fn allocate_children_batch(&mut self, parent: u32, styles: &[LayoutStyle]) -> (u32, u32) {
        let count = styles.len();
        if count == 0 {
            return (0, 0);
        }
        let start_idx = self.styles.len() as u32;
        self.styles.reserve(count);
        self.computed_bounds.reserve(count);
        self.children.reserve(count);
        self.parents.reserve(count);
        self.dirty.reserve(count);

        for (offset, &style) in styles.iter().enumerate() {
            let _idx = start_idx + offset as u32;
            self.styles.push(style);
            self.computed_bounds.push(Rect::ZERO);
            self.children.push(SmallVec::new());
            self.parents.push(Some(parent));
            self.dirty.push(true);
        }

        if let Some(parent_children) = self.children.get_mut(parent as usize) {
            parent_children.reserve(count);
            for offset in 0..count {
                parent_children.push(start_idx + offset as u32);
            }
        }
        self.mark_dirty(parent);
        (start_idx, start_idx + count as u32)
    }

    /// Shrink underlying storage vectors to fit active items, releasing surplus capacity.
    pub fn compact(&mut self) {
        self.styles.shrink_to_fit();
        self.computed_bounds.shrink_to_fit();
        self.children.shrink_to_fit();
        self.parents.shrink_to_fit();
        self.dirty.shrink_to_fit();
    }

    /// Pre-reserves capacity in all underlying SoA storage vectors.
    pub fn reserve(&mut self, additional: usize) {
        self.styles.reserve(additional);
        self.computed_bounds.reserve(additional);
        self.children.reserve(additional);
        self.parents.reserve(additional);
        self.dirty.reserve(additional);
    }

    /// Bulk-allocates `num_rows` table rows each with uniform `col_styles` in a single contiguous pass.
    ///
    /// Pre-allocates all row and cell nodes at once, completely avoiding per-node vector resizes.
    pub fn bulk_allocate_table_rows(
        &mut self,
        parent: u32,
        num_rows: usize,
        row_style: LayoutStyle,
        col_styles: &[LayoutStyle],
    ) -> Vec<u32> {
        let num_cols = col_styles.len();
        let nodes_per_row = 1 + num_cols;
        let total_new_nodes = num_rows * nodes_per_row;
        self.reserve(total_new_nodes);

        let mut row_nodes = Vec::with_capacity(num_rows);
        let start_idx = self.styles.len() as u32;

        for r in 0..num_rows {
            let row_idx = start_idx + (r * nodes_per_row) as u32;
            row_nodes.push(row_idx);

            // Row container node
            self.styles.push(row_style);
            self.computed_bounds.push(Rect::ZERO);
            let mut row_children = SmallVec::new();
            for c in 0..num_cols {
                row_children.push(row_idx + 1 + c as u32);
            }
            self.children.push(row_children);
            self.parents.push(Some(parent));
            self.dirty.push(true);

            // Column child nodes
            for &c_style in col_styles {
                self.styles.push(c_style);
                self.computed_bounds.push(Rect::ZERO);
                self.children.push(SmallVec::new());
                self.parents.push(Some(row_idx));
                self.dirty.push(true);
            }
        }

        if let Some(parent_children) = self.children.get_mut(parent as usize) {
            parent_children.reserve(num_rows);
            parent_children.extend(row_nodes.iter().copied());
        }
        self.mark_dirty(parent);
        row_nodes
    }

    /// Add a child node to a parent container.
    ///
    /// Rejects self-parenting and ancestor cycles, and detaches `child` from any
    /// previous parent before attaching it to `parent`.
    pub fn add_child(&mut self, parent: u32, child: u32) {
        let len = self.styles.len();
        if (parent as usize) >= len || (child as usize) >= len {
            return;
        }
        if parent == child {
            return;
        }

        // Cycle check: ensure `child` is not an ancestor of `parent`
        let mut cursor = Some(parent);
        let mut steps = 0usize;
        while let Some(curr) = cursor {
            if curr == child {
                return;
            }
            steps += 1;
            if steps > len {
                return;
            }
            cursor = self.parents.get(curr as usize).copied().flatten();
        }

        // Detach from previous parent if already attached elsewhere
        if let Some(old_parent) = self.parents[child as usize] {
            if old_parent == parent {
                self.mark_dirty(parent);
                return;
            }
            if let Some(old_children) = self.children.get_mut(old_parent as usize) {
                old_children.retain(|c| *c != child);
            }
            self.mark_dirty(old_parent);
        }

        if let Some(children_vec) = self.children.get_mut(parent as usize) {
            if !children_vec.contains(&child) {
                children_vec.push(child);
            }
        }
        if let Some(parent_slot) = self.parents.get_mut(child as usize) {
            *parent_slot = Some(parent);
        }
        self.mark_dirty(parent);
    }

    /// Mark a node and its ancestors dirty for re-layout.
    pub fn mark_dirty(&mut self, mut node: u32) {
        let max_steps = self.styles.len();
        let mut steps = 0usize;
        loop {
            if let Some(d) = self.dirty.get_mut(node as usize) {
                *d = true;
            }
            steps += 1;
            if steps > max_steps {
                break;
            }
            if let Some(Some(parent)) = self.parents.get(node as usize) {
                node = *parent;
            } else {
                break;
            }
        }
    }

    /// Number of active nodes in the arena.
    #[must_use]
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Resets the arena, retaining underlying vector allocations for zero-allocation reuse.
    pub fn clear(&mut self) {
        self.styles.clear();
        self.computed_bounds.clear();
        self.children.clear();
        self.parents.clear();
        self.dirty.clear();
    }

    /// Check if arena is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    /// Finds the deepest layout node whose computed bounds contain `point`.
    #[must_use]
    pub fn hit_test(&self, root: u32, point: crate::geometry::Point) -> Option<u32> {
        if (root as usize) >= self.computed_bounds.len() {
            return None;
        }
        if !self.computed_bounds[root as usize].contains(point) {
            return None;
        }

        // Search children in reverse order (top-most sibling first)
        if let Some(children) = self.children.get(root as usize) {
            for &child in children.iter().rev() {
                if let Some(hit) = self.hit_test(child, point) {
                    return Some(hit);
                }
            }
        }

        Some(root)
    }

    /// Computes the path from the root node down to `target` (inclusive).
    ///
    /// Returns a stack-allocated [`SmallVec`] for tree depths up to 16,
    /// avoiding heap allocations during event dispatching.
    #[must_use]
    pub fn path_from_root(&self, target: u32) -> SmallVec<[u32; 16]> {
        let mut path = SmallVec::new();
        let max_steps = self.styles.len();
        let mut curr = Some(target);
        let mut steps = 0usize;
        while let Some(node) = curr {
            if steps > max_steps {
                break;
            }
            path.push(node);
            steps += 1;
            curr = self.parents.get(node as usize).copied().flatten();
        }
        path.reverse();
        path
    }
}
