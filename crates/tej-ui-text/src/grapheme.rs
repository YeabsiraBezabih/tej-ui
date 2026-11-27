//! UAX #29 Unicode extended grapheme cluster and word boundary helpers.
//!
//! Powered by `unicode-segmentation` so caret navigation, Backspace/Delete, and
//! word-jump editing never split multi-codepoint grapheme clusters (such as ZWJ
//! emoji sequences, regional flags, or combining diacritical marks).

use unicode_segmentation::UnicodeSegmentation;

/// Returns the number of extended grapheme clusters (UAX #29) in `text`.
#[must_use]
pub fn grapheme_count(text: &str) -> usize {
    text.graphemes(true).count()
}

/// Returns the byte offset of the extended grapheme cluster boundary strictly
/// before `byte_offset`, or `0` if `byte_offset` is at the start of `text`.
#[must_use]
pub fn prev_grapheme_boundary(text: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(text.len());
    if clamped == 0 {
        return 0;
    }

    let mut prev = 0;
    for (idx, _) in text.grapheme_indices(true) {
        if idx >= clamped {
            break;
        }
        prev = idx;
    }
    prev
}

/// Returns the byte offset of the extended grapheme cluster boundary strictly
/// after `byte_offset`, or `text.len()` if `byte_offset` is at or past the end.
#[must_use]
pub fn next_grapheme_boundary(text: &str, byte_offset: usize) -> usize {
    if byte_offset >= text.len() {
        return text.len();
    }

    for (idx, _) in text.grapheme_indices(true) {
        if idx > byte_offset {
            return idx;
        }
    }
    text.len()
}

/// Removes and returns the final extended grapheme cluster from `text`,
/// or `None` if `text` is empty.
pub fn pop_grapheme(text: &mut String) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let start = prev_grapheme_boundary(text, text.len());
    Some(text.split_off(start))
}

/// Returns the byte offset of the previous UAX #29 word boundary before `byte_offset`.
///
/// Skips whitespace-only segments immediately preceding `byte_offset` so cursor
/// navigation jumps cleanly to the start of the previous word.
#[must_use]
pub fn prev_word_boundary(text: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(text.len());
    if clamped == 0 {
        return 0;
    }

    let mut last_non_ws_start = None;
    for (idx, segment) in text.split_word_bound_indices() {
        if idx >= clamped {
            break;
        }
        if !segment.chars().all(char::is_whitespace) {
            last_non_ws_start = Some(idx);
        }
    }
    last_non_ws_start.unwrap_or(0)
}

/// Returns the byte offset of the next UAX #29 word boundary after `byte_offset`.
///
/// Advances past any leading whitespace to the end of the next word segment.
#[must_use]
pub fn next_word_boundary(text: &str, byte_offset: usize) -> usize {
    if byte_offset >= text.len() {
        return text.len();
    }

    for (idx, segment) in text.split_word_bound_indices() {
        let end = idx + segment.len();
        if end <= byte_offset {
            continue;
        }
        if !segment.chars().all(char::is_whitespace) {
            return end;
        }
    }
    text.len()
}
