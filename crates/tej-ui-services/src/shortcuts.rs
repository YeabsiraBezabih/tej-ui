use crate::error::Result;
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type ShortcutHandler = Arc<dyn Fn() + Send + Sync>;

/// Keyboard modifiers for shortcuts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Public keyboard modifier flags accessed directly as booleans by callers"
)]
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

    /// Returns the primary platform modifier (`Cmd` on macOS, `Ctrl` on Windows and Linux).
    #[inline]
    #[must_use]
    pub const fn primary_command() -> Self {
        if cfg!(target_os = "macos") {
            Self::CMD
        } else {
            Self::CTRL
        }
    }
}

/// A complete single-stroke keyboard shortcut definition.
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

    /// Primary command shortcut (Cmd on macOS, Ctrl on Windows/Linux).
    pub fn primary(key: impl Into<String>) -> Self {
        Self::new(Modifiers::primary_command(), key)
    }
}

/// A keyboard chord sequence consisting of one or more successive keystrokes (e.g. `Ctrl+K, Ctrl+S`).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ShortcutChord {
    pub strokes: SmallVec<[Shortcut; 2]>,
}

impl ShortcutChord {
    /// Single keystroke shortcut chord.
    pub fn single(shortcut: Shortcut) -> Self {
        let mut strokes = SmallVec::new();
        strokes.push(shortcut);
        Self { strokes }
    }

    /// Two-stroke chord sequence (e.g. `Ctrl+K, Ctrl+S`).
    pub fn sequence(first: Shortcut, second: Shortcut) -> Self {
        let mut strokes = SmallVec::new();
        strokes.push(first);
        strokes.push(second);
        Self { strokes }
    }
}

impl From<Shortcut> for ShortcutChord {
    fn from(s: Shortcut) -> Self {
        Self::single(s)
    }
}

/// Scope priority for shortcut resolution.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ShortcutScope {
    /// Active local widget or focused sub-pane.
    Local(u32),
    /// Active window instance.
    Window(u64),
    /// Application-wide global desktop accelerator.
    #[default]
    Global,
}

/// Pending chord state waiting for the next keystroke in a sequence.
#[derive(Debug, Clone)]
struct PendingChordState {
    stroke: Shortcut,
    started_at: Instant,
}

/// Global and scoped shortcut manager supporting chord sequences and conflict resolution.
#[derive(Default)]
pub struct ShortcutManager {
    registry: RwLock<HashMap<(ShortcutScope, ShortcutChord), ShortcutHandler>>,
    pending_chord: RwLock<Option<PendingChordState>>,
    chord_timeout: Duration,
}

impl fmt::Debug for ShortcutManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShortcutManager")
            .field("registered_count", &self.registry.read().len())
            .field("has_pending_chord", &self.pending_chord.read().is_some())
            .finish_non_exhaustive()
    }
}

impl ShortcutManager {
    /// Create a new shortcut manager with default 1.5s chord timeout.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
            pending_chord: RwLock::new(None),
            chord_timeout: Duration::from_millis(1500),
        }
    }

    /// Register a global shortcut handler.
    pub fn register<F: Fn() + Send + Sync + 'static>(
        &self,
        shortcut: impl Into<ShortcutChord>,
        handler: F,
    ) -> Result<()> {
        self.register_scoped(ShortcutScope::Global, shortcut, handler)
    }

    /// Register a scoped shortcut handler.
    pub fn register_scoped<F: Fn() + Send + Sync + 'static>(
        &self,
        scope: ShortcutScope,
        shortcut: impl Into<ShortcutChord>,
        handler: F,
    ) -> Result<()> {
        let chord = shortcut.into();
        let mut reg = self.registry.write();
        reg.insert((scope, chord), Arc::new(handler));
        Ok(())
    }

    /// Dispatches an incoming keystroke across scoped registries (Local -> Window -> Global).
    ///
    /// Resolves two-stroke chord sequences (`Ctrl+K, S`) transparently with timeout tracking.
    pub fn dispatch(&self, stroke: &Shortcut) -> bool {
        self.dispatch_scoped(stroke, None, None)
    }

    /// Dispatches an incoming keystroke with explicit active window and local widget contexts.
    pub fn dispatch_scoped(
        &self,
        stroke: &Shortcut,
        active_window: Option<u64>,
        active_local: Option<u32>,
    ) -> bool {
        // 1. Check if we have a pending chord stroke that has not timed out
        let prev_opt = self.pending_chord.write().take();
        if let Some(prev) = prev_opt
            && prev.started_at.elapsed() <= self.chord_timeout
        {
            let chord = ShortcutChord::sequence(prev.stroke, stroke.clone());
            if self.try_dispatch_chord(&chord, active_window, active_local) {
                return true;
            }
        }

        // 2. Check if this stroke initiates a multi-stroke chord sequence in the registry
        let reg = self.registry.read();
        let is_prefix = reg
            .keys()
            .any(|(_, chord)| chord.strokes.len() > 1 && chord.strokes.first() == Some(stroke));
        drop(reg);

        if is_prefix {
            let mut pending = self.pending_chord.write();
            *pending = Some(PendingChordState {
                stroke: stroke.clone(),
                started_at: Instant::now(),
            });
            return true;
        }

        // 3. Dispatch single-stroke shortcut
        let single_chord = ShortcutChord::single(stroke.clone());
        self.try_dispatch_chord(&single_chord, active_window, active_local)
    }

    fn try_dispatch_chord(
        &self,
        chord: &ShortcutChord,
        active_window: Option<u64>,
        active_local: Option<u32>,
    ) -> bool {
        let reg = self.registry.read();

        // Check local scope first (highest precedence)
        if let Some(local_id) = active_local
            && let Some(handler) = reg.get(&(ShortcutScope::Local(local_id), chord.clone()))
        {
            handler();
            return true;
        }

        // Check window scope second
        if let Some(win_id) = active_window
            && let Some(handler) = reg.get(&(ShortcutScope::Window(win_id), chord.clone()))
        {
            handler();
            return true;
        }

        // Check global scope last
        if let Some(handler) = reg.get(&(ShortcutScope::Global, chord.clone())) {
            handler();
            return true;
        }

        false
    }
}
