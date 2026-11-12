use crate::role::A11yRole;
use smallvec::SmallVec;
use tej_layout::Rect;

/// Unique accessibility node identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
    pub children: SmallVec<[A11yId; 4]>,
}

impl A11yNode {
    /// Create a new accessibility node.
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
            children: SmallVec::new(),
        }
    }
}
