//! Unicode Bidirectional text classification and run segmenter (UAX #9).
//!
//! Powered by the `unicode-bidi` crate for spec-compliant UAX #9 paragraph level
//! resolution, embedding level computation, and visual run reordering.

use std::borrow::Cow;
use unicode_bidi::{BidiInfo, Direction, Level};

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

impl TextDirection {
    #[must_use]
    fn to_bidi_level(self) -> Option<Level> {
        match self {
            Self::LeftToRight => Some(Level::ltr()),
            Self::RightToLeft => Some(Level::rtl()),
            Self::Auto => None,
        }
    }

    #[must_use]
    fn from_bidi_level(level: Level) -> Self {
        if level.is_rtl() {
            Self::RightToLeft
        } else {
            Self::LeftToRight
        }
    }
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

/// Bidirectional text analysis and segmentation engine backed by UAX #9 (`unicode-bidi`).
#[derive(Debug, Default)]
pub struct BidiClassifier;

impl BidiClassifier {
    /// Creates a new `BidiClassifier`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Detects the base text direction of a string slice based on the first strong character
    /// according to UAX #9 rules P2/P3.
    #[must_use]
    pub fn detect_base_direction(text: &str) -> TextDirection {
        match unicode_bidi::get_base_direction(text) {
            Direction::Rtl => TextDirection::RightToLeft,
            Direction::Ltr | Direction::Mixed => TextDirection::LeftToRight,
        }
    }

    /// Checks if a character has strong Right-To-Left directionality.
    ///
    /// Excludes weak numeric digits (Arabic-Indic `U+0660..=U+0669` and Extended Arabic-Indic
    /// `U+06F0..=U+06F9`) and the Unicode BOM (`U+FEFF`).
    #[must_use]
    pub const fn is_rtl_char(c: char) -> bool {
        let u = c as u32;
        // Arabic-Indic digits (AN) and Extended Arabic-Indic digits (EN) are weak, not strong RTL
        if (u >= 0x0660 && u <= 0x0669) || (u >= 0x06F0 && u <= 0x06F9) {
            return false;
        }
        // Hebrew: 0x0590 - 0x05FF
        // Arabic: 0x0600 - 0x06FF (excluding digits above), Arabic Supplement 0x0750 - 0x077F
        // Arabic Extended-A: 0x08A0 - 0x08FF
        // Syriac: 0x0700 - 0x074F
        // Thaana: 0x0780 - 0x07BF
        // N'Ko: 0x07C0 - 0x07FF
        // Samaritan: 0x0800 - 0x083F
        // Presentation Forms: 0xFB1D - 0xFDFF, 0xFE70 - 0xFEFC (excluding U+FEFF BOM)
        (u >= 0x0590 && u <= 0x08FF) || (u >= 0xFB1D && u <= 0xFDFF) || (u >= 0xFE70 && u <= 0xFEFC)
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

    /// Computes logical UAX #9 directional runs using `unicode_bidi::BidiInfo` with an explicit
    /// or automatic base paragraph direction.
    #[must_use]
    pub fn analyze<'a>(&self, text: &'a str, base_direction: TextDirection) -> Vec<BidiRun<'a>> {
        if text.is_empty() {
            return Vec::new();
        }

        let bidi_info = BidiInfo::new(text, base_direction.to_bidi_level());
        let mut runs = Vec::new();

        for para in &bidi_info.paragraphs {
            let range = para.range.clone();
            if range.is_empty() {
                continue;
            }
            let mut run_start = range.start;
            let mut current_level = bidi_info.levels[run_start];

            for (rel_idx, _) in text[range.clone()].char_indices() {
                let idx = range.start + rel_idx;
                let lvl = bidi_info.levels[idx];
                if lvl != current_level {
                    runs.push(BidiRun {
                        text: &text[run_start..idx],
                        start: run_start,
                        end: idx,
                        direction: TextDirection::from_bidi_level(current_level),
                        level: current_level.number(),
                    });
                    run_start = idx;
                    current_level = lvl;
                }
            }

            if run_start < range.end {
                runs.push(BidiRun {
                    text: &text[run_start..range.end],
                    start: run_start,
                    end: range.end,
                    direction: TextDirection::from_bidi_level(current_level),
                    level: current_level.number(),
                });
            }
        }

        runs
    }

    /// Computes visual-ordered UAX #9 runs directly using `unicode_bidi::BidiInfo::visual_runs`.
    #[must_use]
    pub fn visual_runs<'a>(
        &self,
        text: &'a str,
        base_direction: TextDirection,
    ) -> Vec<BidiRun<'a>> {
        if text.is_empty() {
            return Vec::new();
        }

        let bidi_info = BidiInfo::new(text, base_direction.to_bidi_level());
        let mut result = Vec::new();

        for para in &bidi_info.paragraphs {
            let range = para.range.clone();
            if range.is_empty() {
                continue;
            }
            let (levels, visual_ranges) = bidi_info.visual_runs(para, range);
            for run_range in visual_ranges {
                if run_range.is_empty() {
                    continue;
                }
                let lvl = levels[run_range.start];
                result.push(BidiRun {
                    text: &text[run_range.clone()],
                    start: run_range.start,
                    end: run_range.end,
                    direction: TextDirection::from_bidi_level(lvl),
                    level: lvl.number(),
                });
            }
        }

        result
    }

    /// Reorders a single line of text into visual display order according to UAX #9.
    #[must_use]
    pub fn reorder_line<'a>(&self, text: &'a str, base_direction: TextDirection) -> Cow<'a, str> {
        if text.is_empty() {
            return Cow::Borrowed(text);
        }
        let bidi_info = BidiInfo::new(text, base_direction.to_bidi_level());
        if let Some(para) = bidi_info.paragraphs.first() {
            bidi_info.reorder_line(para, para.range.clone())
        } else {
            Cow::Borrowed(text)
        }
    }

    /// Segments a text slice into directional runs without allocating string copies,
    /// using UAX #9 (`unicode_bidi::BidiInfo`) level resolution.
    #[must_use]
    pub fn segment<'a>(&self, text: &'a str) -> Vec<BidiRun<'a>> {
        let mut runs = self.analyze(text, TextDirection::Auto);

        // Preserve trailing ASCII space attachment on an RTL run followed by an LTR run
        // so existing callers expecting `"مرحبا "` + `"World"` remain backward-compatible.
        for i in 1..runs.len() {
            if runs[i - 1].direction == TextDirection::RightToLeft
                && runs[i].direction == TextDirection::LeftToRight
                && runs[i].text.starts_with(' ')
                && runs[i].text.len() > 1
            {
                let split_at = runs[i].start + 1;
                runs[i - 1].end = split_at;
                runs[i - 1].text = &text[runs[i - 1].start..split_at];
                runs[i].start = split_at;
                runs[i].text = &text[split_at..runs[i].end];
            }
        }

        runs
    }

    /// Reorders segmented runs into visual display order using UAX #9 Rule L2
    /// (reversing contiguous sequences of runs from the highest level down to the
    /// lowest odd embedding level).
    #[must_use]
    pub fn visual_order<'a>(&self, mut runs: Vec<BidiRun<'a>>) -> Vec<BidiRun<'a>> {
        if runs.len() <= 1 {
            return runs;
        }

        let max_level = runs.iter().map(|r| r.level).max().unwrap_or(0);
        let Some(min_odd_level) = runs.iter().map(|r| r.level).filter(|&l| l % 2 == 1).min() else {
            return runs;
        };

        for level in (min_odd_level..=max_level).rev() {
            let mut i = 0;
            while i < runs.len() {
                if runs[i].level >= level {
                    let start = i;
                    while i < runs.len() && runs[i].level >= level {
                        i += 1;
                    }
                    runs[start..i].reverse();
                } else {
                    i += 1;
                }
            }
        }

        runs
    }
}
