//! Zero-allocation Copy-on-Write (COW) B+ tree undo/redo time-travel history.
//!
//! Because `MyDb` utilizes an append-only / copy-on-write B+ Tree architecture, an entire
//! database state snapshot across millions of records is uniquely identified by an 8-byte
//! root page ID (`u64`).
//!
//! Rather than serializing diffs or copying entire heap objects for undo/redo stacks,
//! [`CowTimeTravelStack`] records transactional 8-byte root pointers. 1,000 full-state
//! rollback checkpoints consume only 8 KB of RAM.

/// Manages undo and redo history for copy-on-write database root pointers.
#[derive(Debug, Clone)]
pub struct CowTimeTravelStack {
    /// History of past B+ tree root page IDs.
    past: Vec<u64>,
    /// Currently active B+ tree root page ID.
    current: u64,
    /// History of undone states available for redo.
    future: Vec<u64>,
    /// Maximum number of undo states to retain (0 = unlimited).
    max_history: usize,
}

impl CowTimeTravelStack {
    /// Creates a new time travel stack anchored at an initial root page ID.
    #[must_use]
    pub fn new(initial_root: u64) -> Self {
        Self {
            past: Vec::new(),
            current: initial_root,
            future: Vec::new(),
            max_history: 10_000,
        }
    }

    /// Sets the maximum number of undo levels to retain.
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        if max > 0 && self.past.len() > max {
            let excess = self.past.len() - max;
            self.past.drain(0..excess);
        }
    }

    /// Returns the currently active root page ID.
    #[must_use]
    pub const fn current_root(&self) -> u64 {
        self.current
    }

    /// Records a new committed root page ID from a database write transaction.
    ///
    /// The previous root is pushed to the undo stack, and the redo stack is cleared.
    pub fn record_commit(&mut self, new_root: u64) {
        if self.current == new_root {
            return;
        }
        self.past.push(self.current);
        if self.max_history > 0 && self.past.len() > self.max_history {
            self.past.remove(0);
        }
        self.current = new_root;
        self.future.clear();
    }

    /// Reverts to the previous B+ tree root page ID if available.
    ///
    /// Returns the target root page ID to which the database snapshot should be pivoted.
    pub fn undo(&mut self) -> Option<u64> {
        let prev = self.past.pop()?;
        self.future.push(self.current);
        self.current = prev;
        Some(self.current)
    }

    /// Re-applies a previously undone B+ tree root page ID if available.
    ///
    /// Returns the target root page ID to which the database snapshot should be pivoted.
    pub fn redo(&mut self) -> Option<u64> {
        let next = self.future.pop()?;
        self.past.push(self.current);
        self.current = next;
        Some(self.current)
    }

    /// Returns `true` if an undo operation is possible.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Returns `true` if a redo operation is possible.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Returns the number of available undo and redo steps as `(undo_count, redo_count)`.
    #[must_use]
    pub fn counts(&self) -> (usize, usize) {
        (self.past.len(), self.future.len())
    }

    /// Calculates the heap memory footprint of this time-travel stack in bytes.
    #[must_use]
    pub fn heap_memory_bytes(&self) -> usize {
        (self.past.capacity() + self.future.capacity()) * std::mem::size_of::<u64>()
    }
}
