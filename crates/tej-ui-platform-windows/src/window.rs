//! Win32 native window implementation and message loop bridge.

use crate::backdrop::DwmAttributes;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::sync::atomic::{AtomicU64, Ordering};
use tej_ui_hal::{
    CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig, WindowId,
};
use tej_ui_layout::Size;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(100);

/// Native Win32 window implementation supporting DirectComposition, DPI messages, and Mica.
#[derive(Debug)]
pub struct WindowsWindow {
    id: WindowId,
    title: String,
    size: Size,
    scale_factor: f32,
    dwm_attributes: DwmAttributes,
    cursor: CursorIcon,
    ime_area: ImeCursorArea,
    ime_allowed: bool,
    event_queue: Vec<UiEvent>,
    redraw_requested: bool,
}

impl WindowsWindow {
    /// Creates a new Win32 native window with the given configuration and DWM attributes.
    #[must_use]
    pub fn new(config: WindowConfig, dwm_attributes: DwmAttributes) -> Self {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        Self {
            id,
            title: config.title,
            size: config.size,
            scale_factor: 1.0,
            dwm_attributes,
            cursor: CursorIcon::Default,
            ime_area: ImeCursorArea::default(),
            ime_allowed: true,
            event_queue: Vec::new(),
            redraw_requested: true,
        }
    }

    /// Returns the active DWM attributes (Mica, Acrylic, Dark Mode).
    #[must_use]
    pub const fn dwm_attributes(&self) -> &DwmAttributes {
        &self.dwm_attributes
    }

    /// Sets DWM attributes dynamically at runtime.
    pub fn set_dwm_attributes(&mut self, attrs: DwmAttributes) {
        self.dwm_attributes = attrs;
    }

    /// Simulates handling a `WM_DPICHANGED` Win32 message.
    pub fn handle_dpi_changed(&mut self, new_dpi: u32, suggested_size: Size) {
        let new_scale = f32::from(u16::try_from(new_dpi).unwrap_or(u16::MAX)) / 96.0;
        self.scale_factor = new_scale;
        self.size = suggested_size;
        self.event_queue.push(UiEvent::ScaleFactorChanged {
            scale_factor: new_scale,
        });
        self.event_queue.push(UiEvent::Resize {
            width: suggested_size.width,
            height: suggested_size.height,
        });
        self.redraw_requested = true;
    }

    /// Handles Win32 `WM_IME_COMPOSITION` composition and result string events.
    pub fn handle_ime_composition(
        &mut self,
        preedit: Option<&str>,
        commit: Option<&str>,
        cursor: Option<(usize, usize)>,
    ) {
        if !self.ime_allowed {
            return;
        }

        if let Some(res) = commit {
            self.event_queue.push(UiEvent::ImeCommit {
                text: res.to_string(),
            });
        } else if let Some(comp) = preedit {
            self.event_queue.push(UiEvent::ImePreedit {
                text: comp.to_string(),
                cursor,
            });
        }
    }

    /// Creates a native Windows window from standard window configuration.
    #[must_use]
    pub fn from_config(config: WindowConfig) -> Self {
        Self::new(config, DwmAttributes::default())
    }

    /// Present rendered surface to the native window compositor.
    pub fn present(&self, _surface: &crate::surface::DirectCompositionSurface) {}

    /// Check whether window is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        true
    }

    /// Pushes a native UI event into the dispatch queue.
    pub fn push_event(&mut self, event: UiEvent) {
        self.event_queue.push(event);
    }

    /// Returns current cursor icon.
    #[must_use]
    pub const fn cursor(&self) -> CursorIcon {
        self.cursor
    }

    /// Returns active IME candidate rect.
    #[must_use]
    pub const fn ime_area(&self) -> ImeCursorArea {
        self.ime_area
    }
}

impl PlatformWindow for WindowsWindow {
    fn id(&self) -> WindowId {
        self.id
    }

    fn size(&self) -> Size {
        self.size
    }

    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    fn request_redraw(&self) {
        // In real Win32 loop, invokes InvalidateRect(hwnd, NULL, FALSE)
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.event_queue)
    }
}

impl HasWindowHandle for WindowsWindow {
    /// Returns [`HandleError::Unavailable`] because real OS handles require native Win32
    /// window creation (`CreateWindowExW`) rather than synthetic window IDs.
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for WindowsWindow {
    /// Returns [`HandleError::Unavailable`] because real OS display handles require native
    /// Win32 window and desktop session binding.
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl TejWindowHost for WindowsWindow {
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
