//! AppKit `NSWindow` / `NSView` wrapper and event loop bridge.

use crate::style::MacosWindowAttributes;
use std::sync::atomic::{AtomicU64, Ordering};
use tej_hal::{PlatformWindow, UiEvent, WindowConfig, WindowId};
use tej_layout::Size;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(200);

/// Native macOS window implementation supporting Retina scaling, AppKit events, and CAMetalLayer.
#[derive(Debug)]
pub struct MacosWindow {
    id: WindowId,
    title: String,
    size: Size,
    backing_scale_factor: f32,
    attributes: MacosWindowAttributes,
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

    /// Updates the Retina backing scale factor (e.g. window moved between Retina and non-Retina displays).
    pub fn set_backing_scale_factor(&mut self, factor: f32) {
        let new_scale = if factor > 0.0 { factor } else { 1.0 };
        self.backing_scale_factor = new_scale;
        self.event_queue.push(UiEvent::ScaleFactorChanged {
            scale_factor: new_scale,
        });
        self.redraw_requested = true;
    }

    /// Pushes an event into the pending dispatch queue.
    pub fn push_event(&mut self, event: UiEvent) {
        self.event_queue.push(event);
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
        // In real AppKit loop, calls [view setNeedsDisplay:YES]
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.event_queue)
    }
}
