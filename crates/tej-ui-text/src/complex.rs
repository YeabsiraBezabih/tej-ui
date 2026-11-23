//! Complex script shaping, OpenType layout features, and cursive script joiners.

/// Standard OpenType layout feature flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTypeFeatures {
    /// Standard ligatures (`liga`) - e.g. fi, fl.
    pub standard_ligatures: bool,
    /// Discretionary ligatures (`dlig`) - e.g. st, ct.
    pub discretionary_ligatures: bool,
    /// Contextual alternates (`calt`) - e.g. programming arrows `->`, `=>`.
    pub contextual_alternates: bool,
    /// Kerning adjustments (`kern`).
    pub kerning: bool,
}

impl Default for OpenTypeFeatures {
    fn default() -> Self {
        Self {
            standard_ligatures: true,
            discretionary_ligatures: false,
            contextual_alternates: true,
            kerning: true,
        }
    }
}

/// Arabic script cursive joining form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArabicJoiningForm {
    Isolated,
    Initial,
    Medial,
    Final,
}

/// Cursive script analyzer for Arabic / Persian / Urdu script shaping.
#[derive(Debug, Default)]
pub struct CursiveScriptJoiner;

impl CursiveScriptJoiner {
    /// Determines whether a Unicode codepoint belongs to the Arabic script block.
    pub fn is_arabic(c: char) -> bool {
        let u = c as u32;
        (0x0600..=0x06FF).contains(&u)
            || (0x0750..=0x077F).contains(&u)
            || (0x08A0..=0x08FF).contains(&u)
    }

    /// Determines whether a character can join with subsequent letters on the left.
    pub fn joins_left(c: char) -> bool {
        if !Self::is_arabic(c) {
            return false;
        }
        // Non-joining Arabic letters (e.g. Alef, Dal, Dhal, Reh, Zain, Waw)
        let u = c as u32;
        !matches!(
            u,
            0x0627 // Alef
            | 0x062F // Dal
            | 0x0630 // Thal
            | 0x0631 // Reh
            | 0x0632 // Zain
            | 0x0648 // Waw
        )
    }

    /// Determines the contextual joining form for a character based on its neighbors.
    pub fn resolve_form(
        prev: Option<char>,
        current: char,
        next: Option<char>,
    ) -> ArabicJoiningForm {
        if !Self::is_arabic(current) {
            return ArabicJoiningForm::Isolated;
        }

        let joins_prev = prev.map(Self::joins_left).unwrap_or(false);
        let joins_next = next.map(Self::is_arabic).unwrap_or(false) && Self::joins_left(current);

        match (joins_prev, joins_next) {
            (true, true) => ArabicJoiningForm::Medial,
            (true, false) => ArabicJoiningForm::Final,
            (false, true) => ArabicJoiningForm::Initial,
            (false, false) => ArabicJoiningForm::Isolated,
        }
    }
}

/// Emoji presentation variation selector classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiPresentation {
    /// Default text presentation or explicitly requested via VS15 (U+FE0E).
    Text,
    /// Color emoji presentation or explicitly requested via VS16 (U+FE0F).
    ColorEmoji,
}

impl EmojiPresentation {
    pub fn detect(c: char, variation_selector: Option<char>) -> Self {
        match variation_selector {
            Some('\u{FE0E}') => EmojiPresentation::Text,
            Some('\u{FE0F}') => EmojiPresentation::ColorEmoji,
            _ => {
                let u = c as u32;
                if (0x1F300..=0x1FAFF).contains(&u) || (0x2600..=0x27BF).contains(&u) {
                    EmojiPresentation::ColorEmoji
                } else {
                    EmojiPresentation::Text
                }
            }
        }
    }
}
