//! `tej-platform-macos`: Native AppKit / CAMetalLayer platform shell for `tej-ui`.
//!
//! Provides AppKit window management, FullSizeContentView modern styling,
//! authentic macOS Traffic Light buttons geometry, elastic rubber-banding scroll physics,
//! Apple system appearance / accent color synchronization, and CAMetalLayer compositing.

pub mod appearance;
pub mod scroll;
pub mod style;
pub mod surface;
pub mod traffic_lights;
pub mod window;

pub use appearance::{AppleAccentColor, MacAppearance, MacosAppearanceManager};
pub use scroll::MacScrollPhysics;
pub use style::{MacosWindowAttributes, MacosWindowStyle};
pub use surface::MetalSurfaceCompositor;
pub use traffic_lights::{HoveredTrafficLight, TrafficLightsController};
pub use window::MacosWindow;
