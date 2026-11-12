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

    /// Add a child node to a parent container.
    pub fn add_child(&mut self, parent: u32, child: u32) {
        if let Some(children_vec) = self.children.get_mut(parent as usize) {
            children_vec.push(child);
        }
        if let Some(parent_slot) = self.parents.get_mut(child as usize) {
            *parent_slot = Some(parent);
        }
        self.mark_dirty(parent);
    }

    /// Mark a node and its ancestors dirty for re-layout.
    pub fn mark_dirty(&mut self, mut node: u32) {
        loop {
            if let Some(d) = self.dirty.get_mut(node as usize) {
                *d = true;
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

    /// Check if arena is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}
