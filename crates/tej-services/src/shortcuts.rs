use crate::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub type ShortcutHandler = Arc<dyn Fn() + Send + Sync>;

/// Keyboard modifiers for shortcuts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on macOS, Win on Windows
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
        meta: false,
    };

    pub const CMD: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
        meta: true,
    };

    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    pub const CTRL_SHIFT: Self = Self {
        ctrl: true,
        alt: false,
        shift: true,
        meta: false,
    };
}

/// A complete keyboard shortcut definition.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: String,
}

impl Shortcut {
    /// Create a new shortcut.
    pub fn new(modifiers: Modifiers, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into().to_uppercase(),
        }
    }
}

/// Global shortcut manager registering application and desktop-wide accelerators.
#[derive(Default)]
pub struct ShortcutManager {
    registry: RwLock<HashMap<Shortcut, ShortcutHandler>>,
}

impl fmt::Debug for ShortcutManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShortcutManager")
            .field("registered_count", &self.registry.read().len())
            .finish()
    }
}

impl ShortcutManager {
    /// Create a new shortcut manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a shortcut handler.
    pub fn register<F: Fn() + Send + Sync + 'static>(
        &self,
        shortcut: Shortcut,
        handler: F,
    ) -> Result<()> {
        let mut reg = self.registry.write();
        reg.insert(shortcut, Arc::new(handler));
        Ok(())
    }

    /// Dispatch an incoming key combination.
    pub fn dispatch(&self, shortcut: &Shortcut) -> bool {
        let reg = self.registry.read();
        if let Some(handler) = reg.get(shortcut) {
            handler();
            true
        } else {
            false
        }
    }
}
