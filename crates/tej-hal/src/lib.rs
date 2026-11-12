//! `tej-hal`: Hardware Abstraction Layer & OS Compositor traits.
//!
//! Provides the cross-platform contract for native window surfaces,
//! 2-tier hybrid compositing, dirty rect clipping, keyboard focus orchestration,
//! and hardware event loops.

pub mod dirty;
pub mod event;
pub mod focus;
pub mod monitor;
pub mod surface;
pub mod window;

pub use dirty::DirtyRectTracker;
pub use event::{MouseButton, UiEvent};
pub use focus::{FocusDirection, FocusId, FocusManager};
pub use monitor::{MonitorId, MonitorInfo, MonitorManager};
pub use surface::{Color, SurfaceCompositor};
pub use window::{PlatformWindow, WindowConfig, WindowId, WindowRegistry};
