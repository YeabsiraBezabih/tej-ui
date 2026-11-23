//! IME composition pipeline: pre-edit underline styling, candidate window positioning, and dead keys.

use tej_ui_layout::{Point, Rect, Size};

/// Visual underline style rendered beneath active pre-edit text clusters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreeditUnderline {
    None,
    Single,
    Thick,
    Squiggly,
    Dotted,
}

/// Computes the optimal window screen position for the IME candidate selection popup.
pub fn compute_candidate_window_position(
    cursor_rect: Rect,
    candidate_size: Size,
    screen_bounds: Rect,
) -> Point {
    // Default anchor: directly below the cursor caret
    let mut x = cursor_rect.origin.x;
    let mut y = cursor_rect.origin.y + cursor_rect.size.height + 4.0;

    // Check bottom overflow: flip above cursor if not enough vertical room
    if y + candidate_size.height > screen_bounds.origin.y + screen_bounds.size.height {
        let flipped_y = cursor_rect.origin.y - candidate_size.height - 4.0;
        if flipped_y >= screen_bounds.origin.y {
            y = flipped_y;
        }
    }

    // Check right overflow: clamp within screen bounds
    if x + candidate_size.width > screen_bounds.origin.x + screen_bounds.size.width {
        x = (screen_bounds.origin.x + screen_bounds.size.width - candidate_size.width)
            .max(screen_bounds.origin.x);
    }

    Point::new(x, y)
}

/// Dead-key sequence resolver for European and accented typing.
#[derive(Debug, Clone, Default)]
pub struct DeadKeyResolver {
    pending_dead_key: Option<char>,
}

impl DeadKeyResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds an input character. If preceded by a dead key, returns the combined accented character.
    pub fn feed_char(&mut self, c: char) -> Option<char> {
        match self.pending_dead_key.take() {
            Some('^') => Some(match c {
                'a' => 'â',
                'e' => 'ê',
                'i' => 'î',
                'o' => 'ô',
                'u' => 'û',
                'A' => 'Â',
                'E' => 'Ê',
                'I' => 'Î',
                'O' => 'Ô',
                'U' => 'Û',
                other => other,
            }),
            Some('~') => Some(match c {
                'a' => 'ã',
                'o' => 'õ',
                'n' => 'ñ',
                'A' => 'Ã',
                'O' => 'Õ',
                'N' => 'Ñ',
                other => other,
            }),
            Some('\'') => Some(match c {
                'a' => 'á',
                'e' => 'é',
                'i' => 'í',
                'o' => 'ó',
                'u' => 'ú',
                'c' => 'ç',
                'A' => 'Á',
                'E' => 'É',
                'I' => 'Í',
                'O' => 'Ó',
                'U' => 'Ú',
                'C' => 'Ç',
                other => other,
            }),
            Some('`') => Some(match c {
                'a' => 'à',
                'e' => 'è',
                'i' => 'ì',
                'o' => 'ò',
                'u' => 'ù',
                'A' => 'À',
                'E' => 'È',
                'I' => 'Ì',
                'O' => 'Ò',
                'U' => 'Ù',
                other => other,
            }),
            Some('"') => Some(match c {
                'a' => 'ä',
                'e' => 'ë',
                'i' => 'ï',
                'o' => 'ö',
                'u' => 'ü',
                'A' => 'Ä',
                'E' => 'Ë',
                'I' => 'Ï',
                'O' => 'Ö',
                'U' => 'Ü',
                other => other,
            }),
            Some(other_dead) => {
                if c == other_dead {
                    Some(other_dead)
                } else {
                    Some(c)
                }
            }
            None => {
                // If it's a dead key, store and return None
                if c == '^' || c == '~' || c == '\'' || c == '`' || c == '"' {
                    self.pending_dead_key = Some(c);
                    None
                } else {
                    Some(c)
                }
            }
        }
    }

    /// Returns true if a dead key is waiting for its base character.
    pub fn has_pending_dead_key(&self) -> bool {
        self.pending_dead_key.is_some()
    }

    /// Resets dead-key state (e.g. on blur or escape).
    pub fn reset(&mut self) {
        self.pending_dead_key = None;
    }
}
