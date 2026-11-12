//! `tej-icons`: Lightweight compile-time vector icons for `tej-ui`.
//!
//! Provides curated Lucide / Feather UI icons and custom SVG path parsing
//! rendered directly via the OS 2D surface compositor with zero heap allocation.

pub mod icon;
pub mod lucide;
pub mod path;

pub use icon::{Icon, IconSource, IconState};
pub use lucide::LucideIcon;
pub use path::{PathCommand, VectorPath};
