//! Native Linux window implementation supporting Wayland, X11, and headless environments.

use crate::surface::LinuxSurface;
use minifb::{MouseButton as MfbMouseButton, MouseMode, Scale, Window, WindowOptions};
use parking_lot::Mutex;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use tej_hal::{MouseButton, PlatformWindow, UiEvent, WindowConfig, WindowId};
use tej_layout::{Point, Size};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

struct NativeWindowHandle(Window);

// SAFETY: `NativeWindowHandle` wraps an OS window. In Linux/X11/Wayland, window event polling
// and buffer presentations are guarded by a Mutex, ensuring thread-safe synchronized access.
unsafe impl Send for NativeWindowHandle {}
// SAFETY: Access to the window is serialized via `parking_lot::Mutex`, preventing concurrent mutation.
unsafe impl Sync for NativeWindowHandle {}

/// Native Linux window implementation supporting Wayland/X11 and headless test runners.
pub struct LinuxWindow {
    id: WindowId,
    title: String,
    size: Size,
    native_window: Mutex<Option<NativeWindowHandle>>,
    event_queue: Mutex<Vec<UiEvent>>,
    last_mouse_pos: Mutex<Option<Point>>,
    redraw_requested: bool,
}

impl fmt::Debug for LinuxWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinuxWindow")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("size", &self.size)
            .field("has_native_window", &self.native_window.lock().is_some())
            .finish()
    }
}

impl LinuxWindow {
    /// Open a new Linux native window with configuration.
    pub fn new(config: WindowConfig) -> Self {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let width = (config.size.width.max(100.0) as usize).min(8192);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let height = (config.size.height.max(100.0) as usize).min(8192);

        let options = WindowOptions {
            resize: true,
            scale: Scale::X1,
            ..WindowOptions::default()
        };

        // Attempt to create native OS window via X11/Wayland; fallback to headless if unavailable
        let mut native = Window::new(&config.title, width, height, options).ok();
        let initial_mouse = native
            .as_mut()
            .and_then(|w| w.get_mouse_pos(MouseMode::Discard))
            .map(|(x, y)| Point::new(x, y));
        let native_window = native.map(NativeWindowHandle);

        Self {
            id,
            title: config.title,
            size: config.size,
            native_window: Mutex::new(native_window),
            event_queue: Mutex::new(Vec::new()),
            last_mouse_pos: Mutex::new(initial_mouse),
            redraw_requested: true,
        }
    }

    /// Check whether a native display window is currently open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.native_window
            .lock()
            .as_ref()
            .is_some_and(|w| w.0.is_open())
    }

    /// Update native window display with rendered surface buffer.
    pub fn update_surface(&self, surface: &LinuxSurface) {
        if let Some(w) = self.native_window.lock().as_mut() {
            let _ =
                w.0.update_with_buffer(surface.pixels(), surface.width(), surface.height());
        }
    }

    /// Check whether a redraw has been requested.
    #[must_use]
    pub const fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    /// Push an event into the pending queue (e.g. from OS event loop).
    pub fn push_event(&self, event: UiEvent) {
        self.event_queue.lock().push(event);
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
        // Redraw will be handled on next frame tick
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        if let Some(w) = self.native_window.lock().as_mut() {
            w.0.set_title(title);
        }
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        let mut events = std::mem::take(&mut *self.event_queue.lock());

        if let Some(w) = self.native_window.lock().as_mut() {
            if w.0.is_open() {
                if let Some((x, y)) = w.0.get_mouse_pos(MouseMode::Discard) {
                    let pt = Point::new(x, y);
                    let mut last = self.last_mouse_pos.lock();
                    if *last != Some(pt) {
                        *last = Some(pt);
                        events.push(UiEvent::MouseMove { position: pt });
                    }
                }

                if w.0.get_mouse_down(MfbMouseButton::Left) {
                    if let Some((x, y)) = w.0.get_mouse_pos(MouseMode::Discard) {
                        events.push(UiEvent::MouseDown {
                            button: MouseButton::Left,
                            position: Point::new(x, y),
                        });
                    }
                }

                if let Some((dx, dy)) = w.0.get_scroll_wheel() {
                    if dx != 0.0 || dy != 0.0 {
                        events.push(UiEvent::Scroll {
                            delta_x: dx,
                            delta_y: dy,
                            position: Point::ZERO,
                        });
                    }
                }
            }
        }

        events
    }
}
