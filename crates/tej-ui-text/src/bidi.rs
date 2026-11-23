//! Unicode Bidirectional text classification and run segmenter (UAX #9).

/// Direction of text flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDirection {
    /// Left-to-right (Latin, Cyrillic, Greek, CJK, etc.).
    #[default]
    LeftToRight,
    /// Right-to-left (Arabic, Hebrew, Persian, Urdu, etc.).
    RightToLeft,
    /// Automatic detection based on the first strong directional character.
    Auto,
}

/// A contiguous segment of text with uniform directionality and embedding level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiRun<'a> {
    /// Zero-copy slice of the original text.
    pub text: &'a str,
    /// Byte start offset in the original string.
    pub start: usize,
    /// Byte end offset in the original string.
    pub end: usize,
    /// Direction of this specific run.
    pub direction: TextDirection,
    /// Unicode embedding level (even = LTR, odd = RTL).
    pub level: u8,
}

/// Bidirectional text analysis and segmentation engine.
#[derive(Debug, Default)]
pub struct BidiClassifier;

impl BidiClassifier {
    /// Creates a new `BidiClassifier`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Detects the base text direction of a string slice based on the first strong character.
    #[must_use]
    pub fn detect_base_direction(text: &str) -> TextDirection {
        for c in text.chars() {
            if Self::is_rtl_char(c) {
                return TextDirection::RightToLeft;
            }
            if Self::is_ltr_char(c) {
                return TextDirection::LeftToRight;
            }
        }
        TextDirection::LeftToRight
    }

    /// Checks if a character has strong Right-To-Left directionality.
    #[must_use]
    pub const fn is_rtl_char(c: char) -> bool {
        let u = c as u32;
        // Hebrew: 0x0590 - 0x05FF
        // Arabic: 0x0600 - 0x06FF, Arabic Supplement 0x0750 - 0x077F
        // Arabic Extended-A: 0x08A0 - 0x08FF
        // Syriac: 0x0700 - 0x074F
        // Thaana: 0x0780 - 0x07BF
        // N'Ko: 0x07C0 - 0x07FF
        // Samaritan: 0x0800 - 0x083F
        // Presentation Forms: 0xFB1D - 0xFB4F, 0xFB50 - 0xFDFF, 0xFE70 - 0xFEFF
        (u >= 0x0590 && u <= 0x08FF) || (u >= 0xFB1D && u <= 0xFDFF) || (u >= 0xFE70 && u <= 0xFEFF)
    }

    /// Checks if a character has strong Left-To-Right directionality.
    #[must_use]
    pub const fn is_ltr_char(c: char) -> bool {
        let u = c as u32;
        // Basic Latin letters
        (u >= 0x0041 && u <= 0x005A)
            || (u >= 0x0061 && u <= 0x007A)
            // Latin-1 Supplement letters
            || (u >= 0x00C0 && u <= 0x00D6)
            || (u >= 0x00D8 && u <= 0x00F6)
            || (u >= 0x00F8 && u <= 0x02B8)
            // Greek and Coptic
            || (u >= 0x0370 && u <= 0x03FF)
            // Cyrillic
            || (u >= 0x0400 && u <= 0x052F)
            // CJK scripts
            || (u >= 0x2E80 && u <= 0x9FFF)
            || (u >= 0xAC00 && u <= 0xD7AF)
    }

    /// Segments a text slice into directional runs without allocating string copies.
    #[must_use]
    pub fn segment<'a>(&self, text: &'a str) -> Vec<BidiRun<'a>> {
        if text.is_empty() {
            return Vec::new();
        }

        let base_dir = Self::detect_base_direction(text);
        let mut runs = Vec::new();
        let mut run_start = 0;
        let mut current_dir = base_dir;
        let mut has_started = false;

        for (idx, c) in text.char_indices() {
            let char_is_rtl = Self::is_rtl_char(c);
            let char_is_ltr = Self::is_ltr_char(c);

            let char_dir = if char_is_rtl {
                TextDirection::RightToLeft
            } else if char_is_ltr {
                TextDirection::LeftToRight
            } else {
                // Neutral character (space, punctuation, digits); inherits current run dir
                current_dir
            };

            if !has_started {
                current_dir = char_dir;
                has_started = true;
            } else if (char_is_rtl || char_is_ltr) && char_dir != current_dir {
                // Direction changed, close previous run
                runs.push(BidiRun {
                    text: &text[run_start..idx],
                    start: run_start,
                    end: idx,
                    direction: current_dir,
                    level: if current_dir == TextDirection::RightToLeft {
                        1
                    } else {
                        0
                    },
                });
                run_start = idx;
                current_dir = char_dir;
            }
        }

        // Close trailing run
        if run_start < text.len() {
            runs.push(BidiRun {
                text: &text[run_start..],
                start: run_start,
                end: text.len(),
                direction: current_dir,
                level: if current_dir == TextDirection::RightToLeft {
                    1
                } else {
                    0
                },
            });
        }

        runs
    }

    /// Reorders segmented runs into visual display order for RTL contexts.
    #[must_use]
    pub fn visual_order<'a>(&self, mut runs: Vec<BidiRun<'a>>) -> Vec<BidiRun<'a>> {
        if runs.len() <= 1 {
            return runs;
        }

        // If the base direction is RTL, reverse the sequence of runs
        let has_rtl = runs
            .iter()
            .any(|r| r.direction == TextDirection::RightToLeft);
        if has_rtl
            && runs
                .first()
                .is_some_and(|r| r.direction == TextDirection::RightToLeft)
        {
            runs.reverse();
        }

        runs
    }
}
