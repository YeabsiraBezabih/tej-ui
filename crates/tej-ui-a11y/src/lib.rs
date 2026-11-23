//! `tej-a11y`: Semantic accessibility tree bridge for platform assistive technology.
//!
//! Exposes widget nodes, bounding rects, and semantic roles to screen readers.

pub mod accesskit_bridge;
pub mod adapter;
pub mod announcement;
pub mod bridge;
pub mod node;
pub mod preferences;
pub mod role;
pub mod tree;

pub use accesskit_bridge::{
    AccessKitBridge, convert_a11y_node, from_accesskit_node_id, to_accesskit_node_id,
};
pub use adapter::{HeadlessA11yAdapter, PlatformA11yAdapter, WebAriaAdapter, WebAriaShadowNode};
pub use announcement::{Announcement, AnnouncementPoliteness, AnnouncementQueue};
pub use bridge::{A11yBridge, A11yBridgeEvent};
pub use node::{A11yId, A11yNode};
pub use preferences::A11yPreferences;
pub use role::{A11yAction, A11yRole};
pub use tree::A11yTree;
