//! Native OS window host contract unifying window handles, event loops, and IME.

use crate::event::UiEvent;
use crate::window::PlatformWindow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Cross-platform standard cursor icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    Grab,
    Grabbing,
    NotAllowed,
    ColResize,
    RowResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
}

/// Screen bounding box for IME candidate window positioning.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ImeCursorArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ImeCursorArea {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Unified Hardware Abstraction Layer contract for native OS window shells.
///
/// Combines `raw-window-handle` 0.6 surface acquisition with DPI scaling,
/// input event loops, cursor management, and native IME pre-edit/commit hooks.
pub trait TejWindowHost: HasWindowHandle + HasDisplayHandle + PlatformWindow {
    /// Window title.
    fn title(&self) -> &str;

    /// Set window cursor icon.
    fn set_cursor(&mut self, cursor: CursorIcon);

    /// Position IME candidate window at caret rect.
    fn set_ime_cursor_area(&mut self, area: ImeCursorArea);

    /// Enable or disable IME text input reception.
    fn set_ime_allowed(&mut self, allowed: bool);

    /// Whether IME composition is currently enabled.
    fn ime_allowed(&self) -> bool;

    /// Poll pending window events from OS message queue.
    fn poll_host_events(&mut self) -> Vec<UiEvent> {
        self.poll_events()
    }

    /// Access the underlying platform window reference.
    fn as_platform_window(&self) -> &dyn PlatformWindow
    where
        Self: Sized,
    {
        self
    }
}
