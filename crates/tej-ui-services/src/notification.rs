//! Platform notifications, actionable responses, and OS share sheets.

use std::path::PathBuf;

/// Actionable button on a system notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationAction {
    pub id: String,
    pub title: String,
    pub is_destructive: bool,
    pub requires_authentication: bool,
}

impl NotificationAction {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            is_destructive: false,
            requires_authentication: false,
        }
    }
}

/// OS desktop or mobile push notification descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub subtitle: Option<String>,
    pub actions: Vec<NotificationAction>,
    pub play_sound: bool,
}

/// Builder for constructing OS notifications.
#[derive(Debug, Clone)]
pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    pub fn new(id: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            notification: Notification {
                id: id.into(),
                title: title.into(),
                body: body.into(),
                subtitle: None,
                actions: Vec::new(),
                play_sound: true,
            },
        }
    }

    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.notification.subtitle = Some(subtitle.into());
        self
    }

    #[must_use]
    pub fn action(mut self, action: NotificationAction) -> Self {
        self.notification.actions.push(action);
        self
    }

    /// Enables the default notification sound.
    #[must_use]
    pub fn with_sound(mut self) -> Self {
        self.notification.play_sound = true;
        self
    }

    /// Suppresses the notification sound.
    #[must_use]
    pub fn silent(mut self) -> Self {
        self.notification.play_sound = false;
        self
    }

    #[must_use]
    pub fn build(self) -> Notification {
        self.notification
    }
}

/// Content payload for the native OS Share Sheet (AirDrop / Windows Share / Android Intent).
#[derive(Debug, Clone, PartialEq)]
pub enum SharePayload {
    Text(String),
    Url(String),
    Files(Vec<PathBuf>),
}

/// Share sheet invocation coordinator.
#[derive(Debug, Clone, Default)]
pub struct ShareSheetDispatcher {
    last_shared: Option<SharePayload>,
}

impl ShareSheetDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn share(&mut self, payload: SharePayload) {
        self.last_shared = Some(payload);
    }

    pub fn last_shared(&self) -> Option<&SharePayload> {
        self.last_shared.as_ref()
    }
}
