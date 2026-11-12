//! `tej-hal`: Hardware Abstraction Layer & Hybrid OS Compositor traits.
//!
//! Provides abstract interfaces for native window management, multi-monitor display metrics,
//! unified events, dirty-region tile composition, and 2D hardware surfaces.

pub mod dirty;
pub mod event;
pub mod monitor;
pub mod surface;
pub mod window;

pub use dirty::DirtyRectTracker;
pub use event::{MouseButton, UiEvent};
pub use monitor::{MonitorId, MonitorInfo, MonitorManager};
pub use surface::{Color, SurfaceCompositor};
pub use window::{PlatformWindow, WindowConfig, WindowId, WindowRegistry};
