//! `tej-platform-linux`: Linux platform shell, Wayland/X11 hooks, and 2D surface compositor.

pub mod font_table;
pub mod surface;
pub mod window;

pub use surface::LinuxSurface;
pub use window::LinuxWindow;
