use std::fmt;

/// Unique generational identifier for a reactive signal slot.
///
/// Exactly 8 bytes in size (`u32` index + `u32` generation).
/// Implements `Copy` so it can be passed around by value without allocations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(C)]
pub struct SignalId {
    /// Slot index within the generational arena.
    pub index: u32,
    /// Generation counter to prevent ABA reuse hazards.
    pub generation: u32,
}

impl SignalId {
    /// Create a new `SignalId` with index and generation.
    #[inline]
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Convert to packed 64-bit integer.
    #[inline]
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    /// Restore from packed 64-bit integer.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

impl fmt::Debug for SignalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignalId({}v{})", self.index, self.generation)
    }
}

impl fmt::Display for SignalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s_{}v{}", self.index, self.generation)
    }
}

/// Generational identifier for a reactive computation node (effect, memo, or leaf listener).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(C)]
pub struct NodeId {
    pub index: u32,
    pub generation: u32,
}

impl NodeId {
    #[inline]
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    #[inline]
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({}v{})", self.index, self.generation)
    }
}

/// Monotonically increasing epoch counter for glitch-free reactive propagation.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Epoch(pub u64);

impl Epoch {
    #[inline]
    #[must_use]
    pub fn advance(&mut self) -> Self {
        self.0 += 1;
        *self
    }
}
