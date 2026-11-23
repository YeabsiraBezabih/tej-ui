//! `tej-platform-windows`: Native Win32 / DirectComposition platform shell for `tej-ui`.
//!
//! Provides Win32 window management, DWM Mica/Acrylic translucent materials,
//! non-client hit-testing (WM_NCHITTEST) with Windows 11 Snap Layouts,
//! Windows Precision Touchpad scroll physics, and DirectComposition tile compositing.

pub mod backdrop;
pub mod hit_test;
pub mod scroll;
pub mod surface;
pub mod theme;
pub mod window;

pub use backdrop::{DwmAttributes, WindowBackdrop};
pub use hit_test::{NcHitCode, Win32WindowFrame};
pub use scroll::{WHEEL_DELTA, Win32ScrollPhysics};
pub use surface::DirectCompositionSurface;
pub use theme::{Win32SystemTheme, Win32ThemeMode};
pub use window::WindowsWindow;
