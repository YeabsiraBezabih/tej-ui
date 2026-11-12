//! `tej-platform-windows`: Native Win32 / DirectComposition platform shell for `tej-ui`.
//!
//! Provides Win32 window management, DWM Mica/Acrylic translucent materials,
//! per-monitor DPI handling, and DirectComposition tile compositing.

pub mod backdrop;
pub mod surface;
pub mod window;

pub use backdrop::{DwmAttributes, WindowBackdrop};
pub use surface::DirectCompositionSurface;
pub use window::WindowsWindow;
