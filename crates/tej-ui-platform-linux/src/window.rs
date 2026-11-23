//! Native Linux window implementation supporting Wayland, X11, and headless environments.

use crate::surface::LinuxSurface;
use minifb::{MouseButton as MfbMouseButton, MouseMode, Scale, Window, WindowOptions};
use parking_lot::Mutex;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle, XlibDisplayHandle, XlibWindowHandle,
};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use tej_ui_hal::{
    CursorIcon, ImeCursorArea, MouseButton, PlatformWindow, TejWindowHost, UiEvent, WindowConfig,
    WindowId,
};
use tej_ui_layout::{Point, Size};

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
    scale_factor: f32,
    native_window: Mutex<Option<NativeWindowHandle>>,
    event_queue: Mutex<Vec<UiEvent>>,
    last_mouse_pos: Mutex<Option<Point>>,
    last_mouse_down: Mutex<bool>,
    cursor: Mutex<CursorIcon>,
    ime_area: Mutex<ImeCursorArea>,
    ime_allowed: Mutex<bool>,
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
            scale_factor: 1.0,
            native_window: Mutex::new(native_window),
            event_queue: Mutex::new(Vec::new()),
            last_mouse_pos: Mutex::new(initial_mouse),
            last_mouse_down: Mutex::new(false),
            cursor: Mutex::new(CursorIcon::Default),
            ime_area: Mutex::new(ImeCursorArea::default()),
            ime_allowed: Mutex::new(true),
            redraw_requested: true,
        }
    }

    /// Check whether a native display window is currently open.
    /// Creates a native Linux window from standard window configuration.
    #[must_use]
    pub fn from_config(config: WindowConfig) -> Self {
        Self::new(config)
    }

    /// Present rendered surface to the native window compositor.
    pub fn present(&self, surface: &LinuxSurface) {
        self.update_surface(surface);
    }

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

    /// Push an IME pre-edit composition update event.
    pub fn push_ime_preedit(&self, text: String, cursor: Option<(usize, usize)>) {
        if *self.ime_allowed.lock() {
            self.event_queue
                .lock()
                .push(UiEvent::ImePreedit { text, cursor });
        }
    }

    /// Push an IME commit composition event.
    pub fn push_ime_commit(&self, text: String) {
        if *self.ime_allowed.lock() {
            self.event_queue.lock().push(UiEvent::ImeCommit { text });
        }
    }

    /// Returns the currently requested cursor icon.
    #[must_use]
    pub fn cursor(&self) -> CursorIcon {
        *self.cursor.lock()
    }

    /// Returns the current IME bounding area.
    #[must_use]
    pub fn ime_area(&self) -> ImeCursorArea {
        *self.ime_area.lock()
    }
}

impl PlatformWindow for LinuxWindow {
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
                // Poll window resize events dynamically from compositor
                let (win_w, win_h) = w.0.get_size();
                #[allow(clippy::cast_precision_loss)]
                let current_size = Size::new(win_w as f32, win_h as f32);
                if self.size != current_size && win_w > 0 && win_h > 0 {
                    self.size = current_size;
                    events.push(UiEvent::Resize {
                        width: current_size.width,
                        height: current_size.height,
                    });
                }

                // Poll mouse position with Pass mode so we always receive real-time cursor coords
                if let Some((x, y)) = w.0.get_mouse_pos(MouseMode::Pass) {
                    let pt = Point::new(x, y);
                    let mut last = self.last_mouse_pos.lock();
                    if *last != Some(pt) {
                        *last = Some(pt);
                        events.push(UiEvent::MouseMove { position: pt });
                    }
                }

                // Poll mouse button down / up edge transitions
                let is_down = w.0.get_mouse_down(MfbMouseButton::Left);
                let mut last_down = self.last_mouse_down.lock();
                if is_down && !*last_down {
                    *last_down = true;
                    let pos = self.last_mouse_pos.lock().unwrap_or(Point::ZERO);
                    events.push(UiEvent::MouseDown {
                        button: MouseButton::Left,
                        position: pos,
                    });
                } else if !is_down && *last_down {
                    *last_down = false;
                    let pos = self.last_mouse_pos.lock().unwrap_or(Point::ZERO);
                    events.push(UiEvent::MouseUp {
                        button: MouseButton::Left,
                        position: pos,
                    });
                }

                // Poll scroll wheel
                if let Some((dx, dy)) = w.0.get_scroll_wheel() {
                    if dx != 0.0 || dy != 0.0 {
                        events.push(UiEvent::Scroll {
                            delta_x: dx,
                            delta_y: dy,
                            position: self.last_mouse_pos.lock().unwrap_or(Point::ZERO),
                        });
                    }
                }

                // Poll keyboard presses with edge triggering and key repeats
                let shift = w.0.is_key_down(minifb::Key::LeftShift)
                    || w.0.is_key_down(minifb::Key::RightShift);
                let ctrl = w.0.is_key_down(minifb::Key::LeftCtrl)
                    || w.0.is_key_down(minifb::Key::RightCtrl);

                for key in w.0.get_keys_pressed(minifb::KeyRepeat::Yes) {
                    let code = match key {
                        minifb::Key::Key1 => 49,
                        minifb::Key::Key2 => 50,
                        minifb::Key::Key3 => 51,
                        minifb::Key::Key4 => 52,
                        minifb::Key::Key5 => 53,
                        minifb::Key::Key6 => 54,
                        minifb::Key::Escape => 27,
                        minifb::Key::Tab => 9,
                        minifb::Key::Enter => 13,
                        minifb::Key::Space => 32,
                        minifb::Key::Backspace => 8,
                        minifb::Key::Delete => 46,
                        minifb::Key::Up => 38,
                        minifb::Key::Down => 40,
                        minifb::Key::Left => 37,
                        minifb::Key::Right => 39,
                        minifb::Key::W => 87,
                        minifb::Key::A => 65,
                        minifb::Key::S => 83,
                        minifb::Key::D => 68,
                        minifb::Key::Q => 81,
                        minifb::Key::E => 69,
                        minifb::Key::C => 67,
                        _ => key as u32,
                    };
                    events.push(UiEvent::KeyDown { key_code: code });

                    if !ctrl {
                        if let Some(ch) = map_key_to_char(key, shift) {
                            events.push(UiEvent::TextInput {
                                text: ch.to_string(),
                            });
                        }
                    }
                }
            }
        }

        events
    }
}

impl HasWindowHandle for LinuxWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = RawWindowHandle::Xlib(XlibWindowHandle::new(self.id.0));
        // SAFETY: The window ID identifies this OS window surface throughout its lifecycle.
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

impl HasDisplayHandle for LinuxWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = RawDisplayHandle::Xlib(XlibDisplayHandle::new(None, 0));
        // SAFETY: The display handle is valid for the entire duration of the process.
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}

impl TejWindowHost for LinuxWindow {
    fn title(&self) -> &str {
        &self.title
    }

    fn set_cursor(&mut self, cursor: CursorIcon) {
        *self.cursor.lock() = cursor;
    }

    fn set_ime_cursor_area(&mut self, area: ImeCursorArea) {
        *self.ime_area.lock() = area;
    }

    fn set_ime_allowed(&mut self, allowed: bool) {
        *self.ime_allowed.lock() = allowed;
    }

    fn ime_allowed(&self) -> bool {
        *self.ime_allowed.lock()
    }
}

fn map_key_to_char(key: minifb::Key, shift: bool) -> Option<char> {
    use minifb::Key;
    match key {
        Key::A => Some(if shift { 'A' } else { 'a' }),
        Key::B => Some(if shift { 'B' } else { 'b' }),
        Key::C => Some(if shift { 'C' } else { 'c' }),
        Key::D => Some(if shift { 'D' } else { 'd' }),
        Key::E => Some(if shift { 'E' } else { 'e' }),
        Key::F => Some(if shift { 'F' } else { 'f' }),
        Key::G => Some(if shift { 'G' } else { 'g' }),
        Key::H => Some(if shift { 'H' } else { 'h' }),
        Key::I => Some(if shift { 'I' } else { 'i' }),
        Key::J => Some(if shift { 'J' } else { 'j' }),
        Key::K => Some(if shift { 'K' } else { 'k' }),
        Key::L => Some(if shift { 'L' } else { 'l' }),
        Key::M => Some(if shift { 'M' } else { 'm' }),
        Key::N => Some(if shift { 'N' } else { 'n' }),
        Key::O => Some(if shift { 'O' } else { 'o' }),
        Key::P => Some(if shift { 'P' } else { 'p' }),
        Key::Q => Some(if shift { 'Q' } else { 'q' }),
        Key::R => Some(if shift { 'R' } else { 'r' }),
        Key::S => Some(if shift { 'S' } else { 's' }),
        Key::T => Some(if shift { 'T' } else { 't' }),
        Key::U => Some(if shift { 'U' } else { 'u' }),
        Key::V => Some(if shift { 'V' } else { 'v' }),
        Key::W => Some(if shift { 'W' } else { 'w' }),
        Key::X => Some(if shift { 'X' } else { 'x' }),
        Key::Y => Some(if shift { 'Y' } else { 'y' }),
        Key::Z => Some(if shift { 'Z' } else { 'z' }),
        Key::Key0 => Some(if shift { ')' } else { '0' }),
        Key::Key1 => Some(if shift { '!' } else { '1' }),
        Key::Key2 => Some(if shift { '@' } else { '2' }),
        Key::Key3 => Some(if shift { '#' } else { '3' }),
        Key::Key4 => Some(if shift { '$' } else { '4' }),
        Key::Key5 => Some(if shift { '%' } else { '5' }),
        Key::Key6 => Some(if shift { '^' } else { '6' }),
        Key::Key7 => Some(if shift { '&' } else { '7' }),
        Key::Key8 => Some(if shift { '*' } else { '8' }),
        Key::Key9 => Some(if shift { '(' } else { '9' }),
        Key::Space => Some(' '),
        Key::Minus => Some(if shift { '_' } else { '-' }),
        Key::Equal => Some(if shift { '+' } else { '=' }),
        Key::LeftBracket => Some(if shift { '{' } else { '[' }),
        Key::RightBracket => Some(if shift { '}' } else { ']' }),
        Key::Backslash => Some(if shift { '|' } else { '\\' }),
        Key::Semicolon => Some(if shift { ':' } else { ';' }),
        Key::Apostrophe => Some(if shift { '"' } else { '\'' }),
        Key::Comma => Some(if shift { '<' } else { ',' }),
        Key::Period => Some(if shift { '>' } else { '.' }),
        Key::Slash => Some(if shift { '?' } else { '/' }),
        Key::Backquote => Some(if shift { '~' } else { '`' }),
        _ => None,
    }
}
