//! Win32 native window implementation and message loop bridge.

use crate::backdrop::DwmAttributes;
use std::sync::atomic::{AtomicU64, Ordering};
use tej_hal::{PlatformWindow, UiEvent, WindowConfig, WindowId};
use tej_layout::Size;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(100);

/// Native Win32 window implementation supporting DirectComposition, DPI messages, and Mica.
#[derive(Debug)]
pub struct WindowsWindow {
    id: WindowId,
    title: String,
    size: Size,
    scale_factor: f32,
    dwm_attributes: DwmAttributes,
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
        let new_scale = (new_dpi as f32) / 96.0;
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

    /// Pushes a native UI event into the dispatch queue.
    pub fn push_event(&mut self, event: UiEvent) {
        self.event_queue.push(event);
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
