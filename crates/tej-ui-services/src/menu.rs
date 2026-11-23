//! Native platform menu bar, macOS top menu, dock badges, and taskbar progress.

use crate::shortcuts::Shortcut;

/// Standard semantic roles for OS menu items (macOS Apple menu, Windows File/Edit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemRole {
    About,
    Preferences,
    Quit,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Undo,
    Redo,
    ToggleFullScreen,
    Minimize,
    Custom,
}

/// An entry in a top-level or submenu hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    Action {
        id: String,
        label: String,
        role: MenuItemRole,
        shortcut: Option<Shortcut>,
        enabled: bool,
        checked: bool,
    },
    Submenu {
        label: String,
        items: Vec<MenuItem>,
    },
    Separator,
}

impl MenuItem {
    pub fn action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            role: MenuItemRole::Custom,
            shortcut: None,
            enabled: true,
            checked: false,
        }
    }

    pub fn role(role: MenuItemRole, label: impl Into<String>) -> Self {
        Self::Action {
            id: format!("{:?}", role).to_lowercase(),
            label: label.into(),
            role,
            shortcut: None,
            enabled: true,
            checked: false,
        }
    }

    pub fn with_shortcut(mut self, shortcut: Shortcut) -> Self {
        if let Self::Action {
            shortcut: ref mut s,
            ..
        } = self
        {
            *s = Some(shortcut);
        }
        self
    }

    pub fn submenu(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            items,
        }
    }
}

/// Top-level application menu bar (macOS system menu bar or Windows window menu).
#[derive(Debug, Clone, Default)]
pub struct MenuBar {
    menus: Vec<(String, Vec<MenuItem>)>,
}

impl MenuBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_menu(&mut self, title: impl Into<String>, items: Vec<MenuItem>) {
        self.menus.push((title.into(), items));
    }

    pub fn menus(&self) -> &[(String, Vec<MenuItem>)] {
        &self.menus
    }

    /// Finds an action item by ID across all submenus.
    pub fn find_item(&self, id: &str) -> Option<&MenuItem> {
        for (_, items) in &self.menus {
            if let Some(item) = Self::find_in_slice(items, id) {
                return Some(item);
            }
        }
        None
    }

    fn find_in_slice<'a>(items: &'a [MenuItem], id: &str) -> Option<&'a MenuItem> {
        for item in items {
            match item {
                MenuItem::Action { id: item_id, .. } if item_id == id => return Some(item),
                MenuItem::Submenu { items: sub, .. } => {
                    if let Some(found) = Self::find_in_slice(sub, id) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

/// OS Dock icon badge or Windows taskbar progress indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskbarState {
    NoProgress,
    Indeterminate,
    Determinate { progress_percent: u8 },
    Error,
}

/// OS application badge configuration (macOS dock count / unread notifications).
#[derive(Debug, Clone, Default)]
pub struct AppBadgeController {
    badge_label: Option<String>,
    taskbar_state: Option<TaskbarState>,
}

impl AppBadgeController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_badge_count(&mut self, count: u32) {
        if count == 0 {
            self.badge_label = None;
        } else {
            self.badge_label = Some(count.to_string());
        }
    }

    pub fn set_badge_label(&mut self, label: Option<String>) {
        self.badge_label = label;
    }

    pub fn badge_label(&self) -> Option<&str> {
        self.badge_label.as_deref()
    }

    pub fn set_taskbar_state(&mut self, state: TaskbarState) {
        self.taskbar_state = Some(state);
    }

    pub fn taskbar_state(&self) -> Option<&TaskbarState> {
        self.taskbar_state.as_ref()
    }
}
