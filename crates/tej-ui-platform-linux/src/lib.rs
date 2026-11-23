//! `tej-platform-linux`: Linux platform shell, Wayland/X11 hooks, and 2D surface compositor.
//!
//! Provides Linux window management, Wayland CSD/SSD Adwaita decorations,
//! XDG Desktop Portal theme synchronization, libinput kinetic scrolling,
//! and fast 2D surface composition.

pub mod decorations;
pub mod font_engine;
pub mod font_table;
pub mod scroll;
pub mod surface;
pub mod theme;
pub mod window;

pub use decorations::{AdwaitaHeaderBar, WindowDecorationMode};
pub use font_engine::{CachedGlyph, FontEngine};
pub use scroll::LibinputScrollPhysics;
pub use surface::LinuxSurface;
pub use theme::{LinuxDesktopTheme, XdgColorScheme};
pub use window::LinuxWindow;
