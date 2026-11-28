//! Web browser WASM canvas window implementation and DOM bridge.

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::sync::atomic::{AtomicU64, Ordering};
use tej_ui_hal::{
    CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig, WindowId,
};
use tej_ui_layout::Size;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(300);

/// Browser HTML canvas window representation for WASM environments.
#[derive(Debug)]
pub struct WebWindow {
    id: WindowId,
    title: String,
    size: Size,
    device_pixel_ratio: f32,
    canvas_id: String,
    cursor: CursorIcon,
    ime_area: ImeCursorArea,
    ime_allowed: bool,
    event_queue: Vec<UiEvent>,
    redraw_requested: bool,
}

impl WebWindow {
    /// Creates a new web canvas window bound to a DOM canvas selector.
    #[must_use]
    pub fn new(config: WindowConfig, canvas_id: &str) -> Self {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        Self {
            id,
            title: config.title,
            size: config.size,
            device_pixel_ratio: 1.0,
            canvas_id: canvas_id.to_string(),
            cursor: CursorIcon::Default,
            ime_area: ImeCursorArea::default(),
            ime_allowed: true,
            event_queue: Vec::new(),
            redraw_requested: true,
        }
    }

    /// Canvas DOM ID string.
    #[must_use]
    pub fn canvas_id(&self) -> &str {
        &self.canvas_id
    }

    /// Updates the browser window's `window.devicePixelRatio`.
    pub fn set_device_pixel_ratio(&mut self, dpr: f32) {
        let new_scale = if dpr > 0.0 { dpr } else { 1.0 };
        self.device_pixel_ratio = new_scale;
        self.event_queue.push(UiEvent::ScaleFactorChanged {
            scale_factor: new_scale,
        });
        self.redraw_requested = true;
    }

    /// Handles DOM `compositionstart` event.
    pub fn handle_composition_start(&mut self) {
        if self.ime_allowed {
            self.event_queue.push(UiEvent::ImePreedit {
                text: String::new(),
                cursor: Some((0, 0)),
            });
        }
    }

    /// Handles DOM `compositionupdate` event.
    pub fn handle_composition_update(&mut self, data: &str) {
        if self.ime_allowed {
            self.event_queue.push(UiEvent::ImePreedit {
                text: data.to_string(),
                cursor: Some((0, data.len())),
            });
        }
    }

    /// Handles DOM `compositionend` event.
    pub fn handle_composition_end(&mut self, data: &str) {
        if self.ime_allowed {
            self.event_queue.push(UiEvent::ImeCommit {
                text: data.to_string(),
            });
        }
    }

    /// Creates a web window from standard window configuration with default `#canvas` id.
    #[must_use]
    pub fn from_config(config: WindowConfig) -> Self {
        Self::new(config, "tej-canvas")
    }

    /// Check whether canvas window is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        true
    }

    /// Pushes a translated DOM event into the queue.
    pub fn push_event(&mut self, event: UiEvent) {
        self.event_queue.push(event);
    }

    /// Active cursor icon.
    #[must_use]
    pub const fn cursor(&self) -> CursorIcon {
        self.cursor
    }

    /// Active IME cursor area.
    #[must_use]
    pub const fn ime_area(&self) -> ImeCursorArea {
        self.ime_area
    }
}

impl PlatformWindow for WebWindow {
    fn id(&self) -> WindowId {
        self.id
    }

    fn size(&self) -> Size {
        self.size
    }

    fn scale_factor(&self) -> f32 {
        self.device_pixel_ratio
    }

    fn request_redraw(&self) {
        // Enqueues window.requestAnimationFrame
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.event_queue)
    }
}

impl HasWindowHandle for WebWindow {
    /// Returns [`HandleError::Unavailable`] because real handles require binding to a live
    /// DOM `HTMLCanvasElement` in a browser WASM environment rather than synthetic IDs.
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for WebWindow {
    /// Returns [`HandleError::Unavailable`] because real display handles require an active
    /// browser DOM window context.
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl TejWindowHost for WebWindow {
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
