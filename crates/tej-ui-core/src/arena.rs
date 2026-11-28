use crate::error::{CoreError, Result};
use crate::id::SignalId;
use std::any::Any;
use std::fmt;

/// An individual slot inside the generational signal arena.
struct Slot {
    generation: u32,
    value: Option<Box<dyn Any + Send + Sync>>,
    next_free: Option<u32>,
}

/// Generational arena for allocating and managing reactive signal storage.
///
/// Recycles vacant slots using a free list while incrementing generation counters
/// to detect stale access patterns.
pub struct SignalArena {
    slots: Vec<Slot>,
    free_head: Option<u32>,
    active_count: usize,
}

impl Default for SignalArena {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalArena {
    /// Create a new, empty `SignalArena`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: None,
            active_count: 0,
        }
    }

    /// Pre-allocate capacity for `capacity` slots.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: None,
            active_count: 0,
        }
    }

    /// Pre-reserves capacity for additional slots.
    pub fn reserve(&mut self, additional: usize) {
        self.slots.reserve(additional);
    }

    /// Allocate a new slot containing `value`, returning its generational `SignalId`.
    pub fn insert<T: Any + Send + Sync + 'static>(&mut self, value: T) -> SignalId {
        let boxed = Box::new(value);
        if let Some(free_idx) = self.free_head {
            let slot = &mut self.slots[free_idx as usize];
            self.free_head = slot.next_free;
            slot.value = Some(boxed);
            slot.next_free = None;
            self.active_count += 1;
            SignalId::new(free_idx, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            let generation = 1;
            self.slots.push(Slot {
                generation,
                value: Some(boxed),
                next_free: None,
            });
            self.active_count += 1;
            SignalId::new(index, generation)
        }
    }

    /// Access a reference to the typed value stored at `id`.
    pub fn get<T: 'static>(&self, id: SignalId) -> Result<&T> {
        let slot = self
            .slots
            .get(id.index as usize)
            .ok_or(CoreError::SlotNotFound(id.index))?;

        if slot.generation != id.generation {
            return Err(CoreError::StaleSignal {
                id: id.index,
                expected: id.generation,
                actual: slot.generation,
            });
        }

        let boxed = slot
            .value
            .as_ref()
            .ok_or(CoreError::SlotNotFound(id.index))?;

        boxed
            .downcast_ref::<T>()
            .ok_or(CoreError::TypeMismatch(id.index))
    }

    /// Access a mutable reference to the typed value stored at `id`.
    pub fn get_mut<T: 'static>(&mut self, id: SignalId) -> Result<&mut T> {
        let slot = self
            .slots
            .get_mut(id.index as usize)
            .ok_or(CoreError::SlotNotFound(id.index))?;

        if slot.generation != id.generation {
            return Err(CoreError::StaleSignal {
                id: id.index,
                expected: id.generation,
                actual: slot.generation,
            });
        }

        let boxed = slot
            .value
            .as_mut()
            .ok_or(CoreError::SlotNotFound(id.index))?;

        boxed
            .downcast_mut::<T>()
            .ok_or(CoreError::TypeMismatch(id.index))
    }

    /// Overwrite the value stored at `id`.
    pub fn set<T: Any + Send + Sync + 'static>(&mut self, id: SignalId, value: T) -> Result<()> {
        let slot = self
            .slots
            .get_mut(id.index as usize)
            .ok_or(CoreError::SlotNotFound(id.index))?;

        if slot.generation != id.generation {
            return Err(CoreError::StaleSignal {
                id: id.index,
                expected: id.generation,
                actual: slot.generation,
            });
        }

        // Reuse existing boxed allocation if slot already contains type T (zero-allocation steady state)
        if let Some(ref mut existing) = slot.value
            && let Some(target) = existing.downcast_mut::<T>()
        {
            *target = value;
            return Ok(());
        }

        slot.value = Some(Box::new(value));
        Ok(())
    }

    /// Remove and invalidate the slot at `id`, returning its previous value.
    pub fn remove<T: 'static>(&mut self, id: SignalId) -> Result<T> {
        let slot = self
            .slots
            .get_mut(id.index as usize)
            .ok_or(CoreError::SlotNotFound(id.index))?;

        if slot.generation != id.generation {
            return Err(CoreError::StaleSignal {
                id: id.index,
                expected: id.generation,
                actual: slot.generation,
            });
        }

        let val = slot.value.take().ok_or(CoreError::SlotNotFound(id.index))?;

        slot.generation = slot.generation.wrapping_add(1);
        slot.next_free = self.free_head;
        self.free_head = Some(id.index);
        self.active_count = self.active_count.saturating_sub(1);

        match val.downcast::<T>() {
            Ok(concrete) => Ok(*concrete),
            Err(_) => Err(CoreError::TypeMismatch(id.index)),
        }
    }

    /// Remove and invalidate the slot at `id` without requiring the concrete type `T`.
    ///
    /// Returns `true` if the slot was active and removed, or `false` if already stale/absent.
    pub fn remove_untyped(&mut self, id: SignalId) -> bool {
        let Some(slot) = self.slots.get_mut(id.index as usize) else {
            return false;
        };

        if slot.generation != id.generation || slot.value.is_none() {
            return false;
        }

        slot.value = None;
        slot.generation = slot.generation.wrapping_add(1);
        slot.next_free = self.free_head;
        self.free_head = Some(id.index);
        self.active_count = self.active_count.saturating_sub(1);
        true
    }

    /// Check if a `SignalId` is currently valid in the arena.
    #[must_use]
    pub fn is_valid(&self, id: SignalId) -> bool {
        self.slots
            .get(id.index as usize)
            .is_some_and(|s| s.generation == id.generation && s.value.is_some())
    }

    /// Number of actively allocated slots.
    #[must_use]
    pub const fn active_count(&self) -> usize {
        self.active_count
    }

    /// Total slot capacity in memory.
    #[must_use]
    pub fn total_capacity(&self) -> usize {
        self.slots.len()
    }
}

impl fmt::Debug for SignalArena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignalArena")
            .field("active_count", &self.active_count)
            .field("total_slots", &self.slots.len())
            .field("free_head", &self.free_head)
            .finish()
    }
}
