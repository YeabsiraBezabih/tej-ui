//! Collapsible application sidebar (shadcn `Sidebar`).

use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// An item in the application sidebar navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub is_active: bool,
}

impl SidebarItem {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            is_active: false,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    #[must_use]
    pub const fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }
}

/// Retained state for a mounted Sidebar.
#[derive(Debug, Default)]
pub struct SidebarState {
    pub bounds: Rect,
    pub is_collapsed: bool,
    pub active_width: f32,
}

/// Collapsible desktop application sidebar layout.
#[derive(Debug, Clone)]
pub struct Sidebar {
    is_collapsed: Signal<bool>,
    expanded_width: f32,
    collapsed_width: f32,
    items: Vec<SidebarItem>,
}

impl Sidebar {
    /// Creates a new sidebar bound to a collapse signal.
    #[must_use]
    pub fn new(is_collapsed: Signal<bool>, items: Vec<SidebarItem>) -> Self {
        Self {
            is_collapsed,
            expanded_width: 240.0,
            collapsed_width: 64.0,
            items,
        }
    }

    /// Sets custom expanded and collapsed widths.
    #[must_use]
    pub const fn widths(mut self, expanded: f32, collapsed: f32) -> Self {
        self.expanded_width = expanded;
        self.collapsed_width = collapsed;
        self
    }

    /// Returns sidebar navigation items.
    #[must_use]
    pub fn items(&self) -> &[SidebarItem] {
        &self.items
    }

    /// Toggles the collapsed state.
    pub fn toggle_collapse(&self) {
        self.is_collapsed.set(!self.is_collapsed.get());
    }
}

impl View for Sidebar {
    type State = SidebarState;

    fn build(&self) -> Self::State {
        let is_collapsed = self.is_collapsed.get();
        let width = if is_collapsed {
            self.collapsed_width
        } else {
            self.expanded_width
        };

        SidebarState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 768.0)),
            is_collapsed,
            active_width: width,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let is_collapsed = self.is_collapsed.get();
        let width = if is_collapsed {
            self.collapsed_width
        } else {
            self.expanded_width
        };
        state.is_collapsed = is_collapsed;
        state.active_width = width;
        state.bounds.size.width = width;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
