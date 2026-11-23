//! Desktop and mobile power/lifecycle state machine with unsaved-work close interception and memory pressure handling.

use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// Operational lifecycle state of the application.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AppLifecycleState {
    /// Foreground, focused, active, and actively receiving user inputs.
    #[default]
    Active,
    /// Foreground, but obscured or unfocused (e.g. system notification or modal interruption).
    Inactive,
    /// Minimized or backgrounded; rendering throttled or paused.
    Hidden,
    /// Frozen / suspended by operating system power management or app lifecycle coordinator.
    Suspended,
}

/// OS-reported memory pressure severity level.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum MemoryPressureLevel {
    /// Normal memory usage; no action required.
    #[default]
    Normal,
    /// Moderate pressure: release cached glyphs, offscreen textures, and speculative buffers.
    Moderate,
    /// Critical pressure: aggressive purge of all non-essential memory to avoid OS OOM termination.
    Critical,
}

/// Token passed to close event handlers to allow intercepting and vetoing window or app close.
#[derive(Debug, Default)]
pub struct CloseVetoToken {
    vetoed: RwLock<bool>,
    reasons: RwLock<Vec<String>>,
}

impl CloseVetoToken {
    /// Create a new un-vetoed close token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Veto the close action, recording the reason (e.g. unsaved document modifications).
    pub fn veto(&self, reason: impl Into<String>) {
        if let Ok(mut v) = self.vetoed.write() {
            *v = true;
        }
        if let Ok(mut r) = self.reasons.write() {
            r.push(reason.into());
        }
    }

    /// Check if any handler has vetoed the close request.
    #[must_use]
    pub fn is_vetoed(&self) -> bool {
        self.vetoed.read().map(|v| *v).unwrap_or(false)
    }

    /// Returns all registered veto reasons.
    #[must_use]
    pub fn reasons(&self) -> Vec<String> {
        self.reasons.read().map(|r| r.clone()).unwrap_or_default()
    }
}

/// External or OS-dispatched lifecycle event.
#[derive(Debug)]
pub enum LifecycleEvent {
    /// Transition into active foreground.
    ForegroundActive,
    /// Transition into inactive foreground.
    ForegroundInactive,
    /// Transition into background / hidden state.
    BackgroundHidden,
    /// OS power-save suspension or mobile app freeze.
    Suspend,
    /// Waking up from suspension.
    Resume,
    /// System low-memory notification.
    LowMemory(MemoryPressureLevel),
    /// Window or application close requested with veto opportunity.
    CloseRequested(Arc<CloseVetoToken>),
}

type TransitionCallback = Box<dyn Fn(AppLifecycleState, AppLifecycleState) + Send + Sync>;
type MemoryCallback = Box<dyn Fn(MemoryPressureLevel) + Send + Sync>;
type CloseCallback = Box<dyn Fn(&CloseVetoToken) + Send + Sync>;

/// Central lifecycle state machine coordinator.
pub struct LifecycleManager {
    current_state: AppLifecycleState,
    transition_listeners: Vec<TransitionCallback>,
    memory_listeners: Vec<MemoryCallback>,
    close_listeners: Vec<CloseCallback>,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LifecycleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LifecycleManager")
            .field("current_state", &self.current_state)
            .field(
                "transition_listeners_count",
                &self.transition_listeners.len(),
            )
            .field("memory_listeners_count", &self.memory_listeners.len())
            .field("close_listeners_count", &self.close_listeners.len())
            .finish()
    }
}

impl LifecycleManager {
    /// Create a new lifecycle manager starting in `Active` state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_state: AppLifecycleState::Active,
            transition_listeners: Vec::new(),
            memory_listeners: Vec::new(),
            close_listeners: Vec::new(),
        }
    }

    /// Current operational lifecycle state.
    #[must_use]
    pub const fn state(&self) -> AppLifecycleState {
        self.current_state
    }

    /// Register a callback invoked whenever the operational lifecycle state transitions.
    pub fn on_transition<F>(&mut self, callback: F)
    where
        F: Fn(AppLifecycleState, AppLifecycleState) + Send + Sync + 'static,
    {
        self.transition_listeners.push(Box::new(callback));
    }

    /// Register a callback for OS low memory notifications.
    pub fn on_low_memory<F>(&mut self, callback: F)
    where
        F: Fn(MemoryPressureLevel) + Send + Sync + 'static,
    {
        self.memory_listeners.push(Box::new(callback));
    }

    /// Register a callback for intercepting window or application close requests.
    pub fn on_close_requested<F>(&mut self, callback: F)
    where
        F: Fn(&CloseVetoToken) + Send + Sync + 'static,
    {
        self.close_listeners.push(Box::new(callback));
    }

    /// Dispatch an incoming lifecycle event and execute registered hooks.
    pub fn dispatch(&mut self, event: LifecycleEvent) {
        match event {
            LifecycleEvent::ForegroundActive => self.transition_to(AppLifecycleState::Active),
            LifecycleEvent::ForegroundInactive => self.transition_to(AppLifecycleState::Inactive),
            LifecycleEvent::BackgroundHidden => self.transition_to(AppLifecycleState::Hidden),
            LifecycleEvent::Suspend => self.transition_to(AppLifecycleState::Suspended),
            LifecycleEvent::Resume => self.transition_to(AppLifecycleState::Active),
            LifecycleEvent::LowMemory(level) => {
                for listener in &self.memory_listeners {
                    listener(level);
                }
            }
            LifecycleEvent::CloseRequested(token) => {
                for listener in &self.close_listeners {
                    listener(&token);
                }
            }
        }
    }

    /// Request application or window close, consulting all registered veto listeners.
    ///
    /// Returns `true` if close is allowed to proceed, or `false` if vetoed by unsaved work.
    pub fn request_close(&self) -> Result<(), Vec<String>> {
        let token = Arc::new(CloseVetoToken::new());
        for listener in &self.close_listeners {
            listener(&token);
        }

        if token.is_vetoed() {
            Err(token.reasons())
        } else {
            Ok(())
        }
    }

    fn transition_to(&mut self, next_state: AppLifecycleState) {
        if self.current_state != next_state {
            let prev = self.current_state;
            self.current_state = next_state;
            for listener in &self.transition_listeners {
                listener(prev, next_state);
            }
        }
    }
}
