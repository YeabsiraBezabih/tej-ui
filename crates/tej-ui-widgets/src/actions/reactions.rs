//! Emoji reactions bar, floating Speed Dial, and mobile swipe-to-action handlers.
//! Styled with Tej's pill surfaces, elevation drop shadows, and micro-hover states.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A single emoji reaction count.
#[derive(Debug, Clone, PartialEq)]
pub struct EmojiReaction {
    pub emoji: String,
    pub count: usize,
    pub has_reacted: bool,
}

/// State for the emoji reaction bar.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReactionBarState {
    pub reactions: Vec<EmojiReaction>,
}

impl ReactionBarState {
    pub fn toggle_reaction(&mut self, emoji: &str) {
        if let Some(r) = self.reactions.iter_mut().find(|r| r.emoji == emoji) {
            if r.has_reacted {
                r.has_reacted = false;
                r.count = r.count.saturating_sub(1);
            } else {
                r.has_reacted = true;
                r.count += 1;
            }
        } else {
            self.reactions.push(EmojiReaction {
                emoji: emoji.to_string(),
                count: 1,
                has_reacted: true,
            });
        }
    }
}

/// Floating Speed Dial action button.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeedDialAction {
    pub id: String,
    pub label: String,
    pub icon_name: String,
}

/// State for floating Speed Dial.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpeedDialState {
    pub is_open: bool,
    pub actions: Vec<SpeedDialAction>,
}

/// Tej styled Reaction Bar component.
#[derive(Debug, Clone, PartialEq)]
pub struct ReactionBar {
    pub bounds: Rect,
    pub background: Color,
    pub active_pill_color: Color,
    pub corner_radius: f32,
}

impl Default for ReactionBar {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::WHITE,
            active_pill_color: Color::from_rgb_u32(0xF9F5FF), // Tej Brand 50
            corner_radius: 9999.0,                            // Full pill
        }
    }
}
