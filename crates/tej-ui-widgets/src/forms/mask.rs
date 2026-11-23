//! Input masking engine for structured form inputs (phone numbers, dates, currency, cards).

use std::fmt::Debug;

/// Pattern token in an input mask.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MaskToken {
    /// Digit placeholder (`#` or `0`): accepts '0'..='9'.
    Digit,
    /// Alpha placeholder (`A` or `a`): accepts 'a'..='z', 'A'..='Z'.
    Alpha,
    /// Alphanumeric placeholder (`*`): accepts digits and letters.
    Alphanumeric,
    /// Fixed literal character that must be displayed as-is (e.g. `(`, `)`, `-`, ` `).
    Literal(char),
}

/// Input mask defining formatting structure and validation rules for text inputs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InputMask {
    tokens: Vec<MaskToken>,
}

impl InputMask {
    /// Create a custom mask from a pattern string.
    ///
    /// - `#` = digit
    /// - `A` = alphabetic character
    /// - `*` = alphanumeric character
    /// - `\` = escape next character as literal
    /// - Any other character = literal delimiter
    #[must_use]
    pub fn parse(pattern: &str) -> Self {
        let mut tokens = Vec::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        tokens.push(MaskToken::Literal(escaped));
                    }
                }
                '#' | '0' => tokens.push(MaskToken::Digit),
                'A' | 'a' => tokens.push(MaskToken::Alpha),
                '*' => tokens.push(MaskToken::Alphanumeric),
                literal => tokens.push(MaskToken::Literal(literal)),
            }
        }

        Self { tokens }
    }

    /// Standard US phone number mask: `(###) ###-####`.
    #[must_use]
    pub fn phone_us() -> Self {
        Self::parse("(###) ###-####")
    }

    /// ISO 8601 date mask: `YYYY-MM-DD` (`####-##-##`).
    #[must_use]
    pub fn date_iso() -> Self {
        Self::parse("####-##-##")
    }

    /// Standard 16-digit credit card mask: `#### #### #### ####`.
    #[must_use]
    pub fn credit_card() -> Self {
        Self::parse("#### #### #### ####")
    }

    /// Total number of placeholder tokens (excluding fixed literals).
    #[must_use]
    pub fn placeholder_count(&self) -> usize {
        self.tokens
            .iter()
            .filter(|t| !matches!(t, MaskToken::Literal(_)))
            .count()
    }

    /// Format a raw unmasked string into the masked representation.
    #[must_use]
    pub fn format(&self, raw: &str) -> String {
        let mut formatted = String::with_capacity(self.tokens.len());
        let mut raw_chars = raw.chars().filter(|c| c.is_alphanumeric()).peekable();

        for token in &self.tokens {
            match token {
                MaskToken::Literal(lit) => {
                    if raw_chars.peek().is_some() {
                        formatted.push(*lit);
                    } else {
                        break;
                    }
                }
                MaskToken::Digit => {
                    for c in raw_chars.by_ref() {
                        if c.is_ascii_digit() {
                            formatted.push(c);
                            break;
                        }
                    }
                }
                MaskToken::Alpha => {
                    for c in raw_chars.by_ref() {
                        if c.is_alphabetic() {
                            formatted.push(c);
                            break;
                        }
                    }
                }
                MaskToken::Alphanumeric => {
                    if let Some(c) = raw_chars.next() {
                        formatted.push(c);
                    }
                }
            }
            if raw_chars.peek().is_none() {
                break;
            }
        }

        formatted
    }

    /// Extract raw unmasked characters (stripping fixed literals).
    #[must_use]
    pub fn unmask(&self, formatted: &str) -> String {
        let mut unmasked = String::new();
        let mut format_chars = formatted.chars();

        for token in &self.tokens {
            let Some(fc) = format_chars.next() else {
                break;
            };

            match token {
                MaskToken::Literal(_) => {
                    // Skip literal delimiter
                }
                MaskToken::Digit => {
                    if fc.is_ascii_digit() {
                        unmasked.push(fc);
                    }
                }
                MaskToken::Alpha => {
                    if fc.is_alphabetic() {
                        unmasked.push(fc);
                    }
                }
                MaskToken::Alphanumeric => {
                    if fc.is_alphanumeric() {
                        unmasked.push(fc);
                    }
                }
            }
        }

        unmasked
    }

    /// Apply a typed character at the given cursor position, returning (new_formatted_text, new_cursor_pos).
    pub fn apply_keystroke(
        &self,
        current_formatted: &str,
        cursor_pos: usize,
        typed: char,
    ) -> (String, usize) {
        let mut raw = self.unmask(current_formatted);
        // Find how many raw chars were before the cursor
        let mut raw_prefix_count = 0;
        for (i, c) in current_formatted.chars().enumerate() {
            if i >= cursor_pos {
                break;
            }
            if let Some(token) = self.tokens.get(i) {
                if !matches!(token, MaskToken::Literal(_)) && c.is_alphanumeric() {
                    raw_prefix_count += 1;
                }
            }
        }

        // Insert typed character if valid
        if typed.is_alphanumeric() && raw_prefix_count <= raw.len() {
            raw.insert(raw_prefix_count, typed);
        }

        let new_formatted = self.format(&raw);
        let new_cursor = self.calculate_cursor_position(&new_formatted, raw_prefix_count + 1);

        (new_formatted, new_cursor)
    }

    /// Check if the masked text fills all required placeholder positions.
    #[must_use]
    pub fn is_complete(&self, formatted: &str) -> bool {
        let unmasked = self.unmask(formatted);
        unmasked.chars().count() == self.placeholder_count()
    }

    fn calculate_cursor_position(&self, formatted: &str, target_raw_chars: usize) -> usize {
        let mut seen_raw = 0;
        let mut cursor = 0;

        for (i, ch) in formatted.chars().enumerate() {
            if seen_raw == target_raw_chars {
                return i;
            }
            if let Some(token) = self.tokens.get(i) {
                if !matches!(token, MaskToken::Literal(_)) && ch.is_alphanumeric() {
                    seen_raw += 1;
                }
            }
            cursor = i + 1;
        }

        cursor
    }
}
