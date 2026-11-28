//! Multi-line word wrapping engine built on [`TextMeasurer`] and UAX #29 grapheme clusters.

use crate::font::SystemFont;
use crate::measurement::TextMeasurer;
use unicode_segmentation::UnicodeSegmentation;

/// Result of laying out wrapped multi-line text.
#[derive(Debug, Clone, PartialEq)]
pub struct WrappedText {
    /// Wrapped visual lines in top-to-bottom order.
    pub lines: Vec<String>,
    /// Maximum measured width across all wrapped lines (in logical pixels).
    pub width: f32,
    /// Total height of the wrapped text block (`lines.len() * line_height`).
    pub height: f32,
    /// Height of a single line in logical pixels.
    pub line_height: f32,
}

/// Wraps `text` into multiple lines constrained by `max_width` using `measurer`.
///
/// - Preserves explicit `\n` line breaks.
/// - Wraps at whitespace boundaries when a word would exceed `max_width`.
/// - Falls back to UAX #29 grapheme-cluster breaking for single words wider than
///   `max_width` so multi-byte characters and ZWJ emoji are never split.
/// - If `max_width <= 0.0` or non-finite, soft wrapping is disabled and only
///   explicit newlines create line breaks.
#[must_use]
pub fn wrap_text(
    text: &str,
    max_width: f32,
    font_size: f32,
    measurer: &TextMeasurer,
) -> WrappedText {
    let font = SystemFont::Body;
    let line_height = measurer.measure_slice(" ", font, font_size).height;

    if text.is_empty() {
        return WrappedText {
            lines: vec![String::new()],
            width: 0.0,
            height: line_height,
            line_height,
        };
    }

    let constrain = max_width.is_finite() && max_width > 0.0;
    let mut lines: Vec<String> = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        if !constrain {
            lines.push(paragraph.to_string());
            continue;
        }

        let mut current_line = String::new();

        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                push_word_or_break(
                    word,
                    max_width,
                    font_size,
                    font,
                    measurer,
                    &mut lines,
                    &mut current_line,
                );
            } else {
                let mut candidate = String::with_capacity(current_line.len() + 1 + word.len());
                candidate.push_str(&current_line);
                candidate.push(' ');
                candidate.push_str(word);

                let candidate_w = measurer.measure_slice(&candidate, font, font_size).width;
                if candidate_w <= max_width {
                    current_line = candidate;
                } else {
                    lines.push(std::mem::take(&mut current_line));
                    push_word_or_break(
                        word,
                        max_width,
                        font_size,
                        font,
                        measurer,
                        &mut lines,
                        &mut current_line,
                    );
                }
            }
        }

        if !current_line.is_empty() || paragraph.chars().all(char::is_whitespace) {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    let width = lines
        .iter()
        .map(|line| measurer.measure_slice(line, font, font_size).width)
        .fold(0.0_f32, f32::max);
    let line_count = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let height = f32::from(line_count) * line_height;

    WrappedText {
        lines,
        width,
        height,
        line_height,
    }
}

fn push_word_or_break(
    word: &str,
    max_width: f32,
    font_size: f32,
    font: SystemFont,
    measurer: &TextMeasurer,
    lines: &mut Vec<String>,
    current_line: &mut String,
) {
    if measurer.measure_slice(word, font, font_size).width <= max_width {
        current_line.push_str(word);
        return;
    }

    // Single word exceeds max_width: break cleanly on UAX #29 grapheme boundaries.
    for grapheme in word.graphemes(true) {
        if current_line.is_empty() {
            current_line.push_str(grapheme);
        } else {
            let mut candidate = String::with_capacity(current_line.len() + grapheme.len());
            candidate.push_str(current_line);
            candidate.push_str(grapheme);
            if measurer.measure_slice(&candidate, font, font_size).width <= max_width {
                *current_line = candidate;
            } else {
                lines.push(std::mem::take(current_line));
                current_line.push_str(grapheme);
            }
        }
    }
}
