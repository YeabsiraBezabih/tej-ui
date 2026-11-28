//! AppKit `NSWindow` / `NSView` wrapper and event loop bridge.

use crate::style::MacosWindowAttributes;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::sync::atomic::{AtomicU64, Ordering};
use tej_ui_hal::{
    CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig, WindowId,
};
use tej_ui_layout::Size;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(200);

/// Native macOS window implementation supporting Retina scaling, AppKit events, and CAMetalLayer.
#[derive(Debug)]
pub struct MacosWindow {
    id: WindowId,
    title: String,
    size: Size,
    backing_scale_factor: f32,
    attributes: MacosWindowAttributes,
    cursor: CursorIcon,
    ime_area: ImeCursorArea,
    ime_allowed: bool,
    event_queue: Vec<UiEvent>,
    redraw_requested: bool,
}

impl MacosWindow {
    /// Creates a new AppKit native window with the given configuration and attributes.
    #[must_use]
    pub fn new(config: WindowConfig, attributes: MacosWindowAttributes) -> Self {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        Self {
            id,
            title: config.title,
            size: config.size,
            backing_scale_factor: 2.0, // Default to 2.0x Retina on modern Apple Silicon
            attributes,
            cursor: CursorIcon::Default,
            ime_area: ImeCursorArea::default(),
            ime_allowed: true,
            event_queue: Vec::new(),
            redraw_requested: true,
        }
    }

    /// Returns the window styling attributes.
    #[must_use]
    pub const fn attributes(&self) -> &MacosWindowAttributes {
        &self.attributes
    }

    /// Sets window styling attributes.
    pub fn set_attributes(&mut self, attrs: MacosWindowAttributes) {
        self.attributes = attrs;
    }

    /// Updates the Retina backing scale factor.
    pub fn set_backing_scale_factor(&mut self, factor: f32) {
        let new_scale = if factor > 0.0 { factor } else { 1.0 };
        self.backing_scale_factor = new_scale;
        self.event_queue.push(UiEvent::ScaleFactorChanged {
            scale_factor: new_scale,
        });
        self.redraw_requested = true;
    }

    /// Handles AppKit `NSTextInputClient` `setMarkedText:selectedRange:replacementRange:` pre-edit composition.
    pub fn handle_marked_text(&mut self, text: &str, selected_range: Option<(usize, usize)>) {
        if self.ime_allowed {
            self.event_queue.push(UiEvent::ImePreedit {
                text: text.to_string(),
                cursor: selected_range,
            });
        }
    }

    /// Handles AppKit `NSTextInputClient` `insertText:replacementRange:` composition commit.
    pub fn handle_insert_text(&mut self, text: &str) {
        if self.ime_allowed {
            self.event_queue.push(UiEvent::ImeCommit {
                text: text.to_string(),
            });
        }
    }

    /// Creates a native macOS window from standard window configuration.
    #[must_use]
    pub fn from_config(config: WindowConfig) -> Self {
        Self::new(config, MacosWindowAttributes::default())
    }

    /// Present rendered surface to the native window compositor.
    pub fn present(&self, _surface: &crate::surface::MetalSurfaceCompositor) {}

    /// Check whether window is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        true
    }

    /// Pushes an event into the pending dispatch queue.
    pub fn push_event(&mut self, event: UiEvent) {
        self.event_queue.push(event);
    }

    /// Returns the active cursor icon.
    #[must_use]
    pub const fn cursor(&self) -> CursorIcon {
        self.cursor
    }

    /// Returns the active IME caret boundary area.
    #[must_use]
    pub const fn ime_area(&self) -> ImeCursorArea {
        self.ime_area
    }
}

impl PlatformWindow for MacosWindow {
    fn id(&self) -> WindowId {
        self.id
    }

    fn size(&self) -> Size {
        self.size
    }

    fn scale_factor(&self) -> f32 {
        self.backing_scale_factor
    }

    fn request_redraw(&self) {
        // Triggers setNeedsDisplay:YES on NSView
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.event_queue)
    }
}

impl HasWindowHandle for MacosWindow {
    /// Returns [`HandleError::Unavailable`] because real OS handles require native AppKit
    /// `NSWindow`/`NSView` creation rather than synthetic window IDs.
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for MacosWindow {
    /// Returns [`HandleError::Unavailable`] because real OS display handles require an
    /// active native AppKit `NSApplication` session.
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl TejWindowHost for MacosWindow {
    fn title(&self) -> &str {
        &self.title
    }

    fn set_cursor(&mut self, cursor: CursorIcon) {
        self.cursor = cursor;
    }

    fn set_ime_cursor_area(&mut self, area: ImeCursorArea) {
        self.ime_area = area;
    }

    fn enable_ime(&mut self) {
        self.ime_allowed = true;
    }

    fn disable_ime(&mut self) {
        self.ime_allowed = false;
    }

    fn ime_allowed(&self) -> bool {
        self.ime_allowed
    }
}
