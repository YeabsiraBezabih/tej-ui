//! Accessibility node representation.

use crate::announcement::AnnouncementPoliteness;
use crate::role::A11yRole;
use smallvec::SmallVec;
use tej_ui_layout::Rect;

/// Unique accessibility node identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct A11yId(pub u32);

/// Accessibility node descriptor exposed to platform assistive technologies.
#[derive(Clone, Debug)]
pub struct A11yNode {
    pub id: A11yId,
    pub role: A11yRole,
    pub label: Option<String>,
    pub value: Option<String>,
    pub bounds: Rect,
    pub is_focused: bool,
    pub is_disabled: bool,
    pub is_checked: Option<bool>,
    /// Live region politeness for dynamic updates (aria-live).
    pub live_region: Option<AnnouncementPoliteness>,
    /// Descriptive text supplementary to label (aria-describedby).
    pub description: Option<String>,
    /// Whether this node is hidden from assistive tech (aria-hidden).
    pub is_hidden: bool,
    pub children: SmallVec<[A11yId; 4]>,
}

impl A11yNode {
    /// Create a new accessibility node.
    #[must_use]
    pub fn new(id: A11yId, role: A11yRole) -> Self {
        Self {
            id,
            role,
            label: None,
            value: None,
            bounds: Rect::ZERO,
            is_focused: false,
            is_disabled: false,
            is_checked: None,
            live_region: None,
            description: None,
            is_hidden: false,
            children: SmallVec::new(),
        }
    }

    /// Set node label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set node bounds.
    #[must_use]
    pub const fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }
}
