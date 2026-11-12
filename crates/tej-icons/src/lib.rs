//! `tej-icons`: Lightweight compile-time vector icons for `tej-ui`.
//!
//! Provides a categorized catalog of 120+ standard Lucide / Feather UI icons
//! and custom SVG path parsing rendered directly via the OS 2D surface compositor
//! with zero heap allocation on steady-state paint paths.

pub mod catalog;
pub mod icon;
pub mod path;

pub use catalog::{
    ActionIcon, CommunicationIcon, DeviceIcon, EditorIcon, LucideIcon, MediaIcon, NavigationIcon,
    SystemIcon,
};
pub use icon::{Icon, IconSource, IconState};
pub use path::{PathCommand, VectorPath};
