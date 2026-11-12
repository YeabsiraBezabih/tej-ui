//! `tej-platform-macos`: Native AppKit / CAMetalLayer platform shell for `tej-ui`.
//!
//! Provides AppKit window management, FullSizeContentView modern styling,
//! Retina backing scale factors, and CAMetalLayer dirty tile compositing.

pub mod style;
pub mod surface;
pub mod window;

pub use style::{MacosWindowAttributes, MacosWindowStyle};
pub use surface::MetalSurfaceCompositor;
pub use window::MacosWindow;
