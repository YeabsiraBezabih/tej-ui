//! Programmatic screen-reader announcements and live region queues.
//!
//! Provides ARIA-equivalent polite and assertive announcement channels for assistive technologies
//! (VoiceOver, Narrator, Orca, TalkBack) without disrupting user navigation.

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Screen-reader notification urgency level according to ARIA live-region standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnnouncementPoliteness {
    /// Screen reader finishes current speech before announcing (aria-live="polite").
    #[default]
    Polite,
    /// Screen reader interrupts current speech immediately (aria-live="assertive").
    Assertive,
}

/// A queued screen-reader announcement item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    /// Text to be vocalized by the screen reader.
    pub message: String,
    /// Politeness level.
    pub politeness: AnnouncementPoliteness,
    /// Timestamp when announcement was posted.
    pub created_at: Instant,
}

impl Announcement {
    /// Creates a polite announcement.
    pub fn polite(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            politeness: AnnouncementPoliteness::Polite,
            created_at: Instant::now(),
        }
    }

    /// Creates an assertive (interrupting) announcement.
    pub fn assertive(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            politeness: AnnouncementPoliteness::Assertive,
            created_at: Instant::now(),
        }
    }
}

/// Thread-safe queue collecting live region announcements for platform assistive tech dispatch.
#[derive(Debug, Default, Clone)]
pub struct AnnouncementQueue {
    queue: Arc<Mutex<Vec<Announcement>>>,
}

impl AnnouncementQueue {
    /// Creates a new announcement queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Post a polite announcement.
    pub fn announce_polite(&self, message: impl Into<String>) {
        self.queue.lock().push(Announcement::polite(message));
    }

    /// Post an assertive announcement.
    pub fn announce_assertive(&self, message: impl Into<String>) {
        self.queue.lock().push(Announcement::assertive(message));
    }

    /// Post an announcement with explicit politeness level.
    pub fn announce(&self, message: impl Into<String>, politeness: AnnouncementPoliteness) {
        let ann = Announcement {
            message: message.into(),
            politeness,
            created_at: Instant::now(),
        };
        self.queue.lock().push(ann);
    }

    /// Takes all pending announcements from the queue, clearing it for the next tick.
    pub fn drain_pending(&self) -> Vec<Announcement> {
        let mut q = self.queue.lock();
        std::mem::take(&mut *q)
    }

    /// Number of announcements currently pending delivery.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.queue.lock().len()
    }

    /// Check if queue has pending announcements.
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.queue.lock().is_empty()
    }
}
