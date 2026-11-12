//! `tej-a11y`: Semantic accessibility tree bridge for platform assistive technology.
//!
//! Exposes widget nodes, bounding rects, and semantic roles to screen readers.

pub mod node;
pub mod role;
pub mod tree;

pub use node::{A11yId, A11yNode};
pub use role::{A11yAction, A11yRole};
pub use tree::A11yTree;
