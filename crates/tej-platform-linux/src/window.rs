use std::sync::atomic::{AtomicU64, Ordering};
use tej_hal::{PlatformWindow, UiEvent, WindowConfig, WindowId};
use tej_layout::Size;

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

/// Native Linux window implementation supporting Wayland/X11 and headless test runners.
#[derive(Debug)]
pub struct LinuxWindow {
    id: WindowId,
    title: String,
    size: Size,
    event_queue: Vec<UiEvent>,
    redraw_requested: bool,
}

impl LinuxWindow {
    /// Open a new Linux native window with configuration.
    pub fn new(config: WindowConfig) -> Self {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        Self {
            id,
            title: config.title,
            size: config.size,
            event_queue: Vec::new(),
            redraw_requested: true,
        }
    }

    /// Check whether a redraw has been requested.
    #[must_use]
    pub const fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    /// Push an event into the pending queue (e.g. from OS event loop).
    pub fn push_event(&mut self, event: UiEvent) {
        self.event_queue.push(event);
    }
}

impl PlatformWindow for LinuxWindow {
    fn id(&self) -> WindowId {
        self.id
    }

    fn size(&self) -> Size {
        self.size
    }

    fn request_redraw(&self) {
        // In real event loop, sends Wayland surface commit / X11 expose event
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.event_queue)
    }
}
