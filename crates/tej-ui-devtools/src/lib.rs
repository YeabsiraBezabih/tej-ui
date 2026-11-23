//! `tej-devtools`: Reactive DevTools, state-preserving hot reload, and dirty-region visualizer.
//!
//! Provides inspection tools for the live signal dependency DAG, on-screen dirty-region
//! HUD overlays with paint flashing, and hot-reload coordinators for rapid UI iteration.

pub mod dag_visualizer;
pub mod dirty_hud;
pub mod dylib_reload;
pub mod hot_reload;
pub mod inspector;

pub use dag_visualizer::{DagNodeInfo, DagSnapshot, ReactiveDagInspector};
pub use dirty_hud::{DirtyHudOverlay, FrameMetrics};
pub use dylib_reload::{DylibReloader, ReloadStatus};
pub use hot_reload::HotReloadHost;
pub use inspector::{FrameFlamegraphProfiler, FrameTimingSample, InspectedNode, WidgetInspector};
