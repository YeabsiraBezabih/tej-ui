use crate::error::Result;
use std::fmt::Debug;

/// System tray event interactions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrayEvent {
    LeftClick,
    RightClick,
    DoubleClick,
}

/// System tray icon configuration and service handle.
#[derive(Clone, Debug, Default)]
pub struct TrayIcon {
    pub title: Option<String>,
    pub tooltip: Option<String>,
    pub badge_count: Option<u32>,
    pub is_visible: bool,
}

impl TrayIcon {
    /// Create a new tray icon with title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            tooltip: None,
            badge_count: None,
            is_visible: true,
        }
    }

    /// Set hover tooltip string.
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Set numeric notification badge counter.
    pub fn badge(mut self, count: u32) -> Self {
        self.badge_count = Some(count);
        self
    }

    /// Show or hide the tray icon.
    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.is_visible = visible;
        Ok(())
    }
}
