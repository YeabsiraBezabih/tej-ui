use crate::event::UiEvent;
use std::fmt::Debug;
use tej_layout::Size;

/// Unique identifier for an open OS window.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowId(pub u64);

/// Configuration settings for creating a native OS window.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    pub decorated: bool,
    pub high_dpi: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("tej-ui Application"),
            size: Size::new(800.0, 600.0),
            resizable: true,
            decorated: true,
            high_dpi: true,
        }
    }
}

/// Abstract Hardware Abstraction Layer for OS native windows.
pub trait PlatformWindow: Debug + Send + Sync {
    /// Window identifier.
    fn id(&self) -> WindowId;

    /// Current client area size.
    fn size(&self) -> Size;

    /// Request a window repaint from the OS compositor.
    fn request_redraw(&self);

    /// Set window title.
    fn set_title(&mut self, title: &str);

    /// Poll pending window events.
    fn poll_events(&mut self) -> Vec<UiEvent>;
}
