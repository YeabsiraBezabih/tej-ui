//! `tej-hal`: Hardware Abstraction Layer & OS Compositor traits.
//!
//! Provides the cross-platform contract for native window surfaces,
//! 2-tier hybrid compositing, dirty rect clipping, keyboard focus orchestration,
//! and hardware event loops.

pub mod color;
pub mod dirty;
pub mod dirty_region;
pub mod dpi;
pub mod event;
pub mod focus;
pub mod gesture;
pub mod lifecycle;
pub mod monitor;
pub mod native_view;
pub mod pacing;
pub mod recovery;
pub mod stylus;
pub mod surface;
pub mod titlebar;
pub mod window;
pub mod window_host;

#[cfg(feature = "gpu")]
pub mod gpu;

pub use color::{Color, ColorLinear, ColorSpace};
pub use dirty::DirtyRectTracker;
pub use dirty_region::DirtyRegion;
pub use dpi::{BASELINE_DPI, DpiChangeEvent, DpiScale};
pub use event::{MouseButton, UiEvent};
pub use focus::{FocusDirection, FocusId, FocusManager, SpatialDirection};
pub use gesture::{
    GestureArena, GestureState, KineticDeceleration, PanGesture, PinchGesture, TouchId, TouchPhase,
    TouchPoint,
};
pub use lifecycle::{
    AppLifecycleState, CloseVetoToken, LifecycleEvent, LifecycleManager, MemoryPressureLevel,
};
pub use monitor::{MonitorId, MonitorInfo, MonitorManager};
pub use native_view::{NativeViewHandle, NativeViewHost};
pub use pacing::{FramePacer, RefreshRateMode};
pub use recovery::{DeviceHealth, DeviceLossCoordinator, RecoveryToken};
pub use stylus::{PalmRejectionFilter, StylusPoint, StylusToolType};
pub use titlebar::{CustomTitlebar, TitlebarHitRegion};

pub use surface::SurfaceCompositor;
pub use window::{PlatformWindow, WindowConfig, WindowId, WindowKind, WindowRegistry};
pub use window_host::{CursorIcon, ImeCursorArea, TejWindowHost};
