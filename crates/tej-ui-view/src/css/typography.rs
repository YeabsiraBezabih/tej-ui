//! Typography, font metrics, text decoration, alignment, and internationalization.

use super::types::ColorValue;
use crate::typography_extended::{CaretStyle, FontVariantNumeric, TextUnderlineStyle, TextWrap};
use std::borrow::Cow;
use tej_ui_layout::LayoutDirection;

/// Font style variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Horizontal text alignment.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextAlign {
    #[default]
    Start,
    Left,
    Center,
    Right,
    End,
    Justify,
}

/// Text decoration lines.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextDecorationLine {
    #[default]
    None,
    Underline,
    LineThrough,
    Overline,
}

/// Text casing transform.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

/// Whitespace handling mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WhiteSpace {
    #[default]
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

/// Single-line text overflow behavior.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
}

/// Word and line break rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WordBreak {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

/// Writing mode block and inline axes flow.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
}

/// Typography and text properties suite.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TypographyProps {
    pub color: ColorValue,
    pub font_family: Cow<'static, str>,
    pub font_size: f32,
    pub font_weight: u16,
    pub font_style: FontStyle,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub text_align: TextAlign,
    pub text_decoration_line: TextDecorationLine,
    pub text_underline: TextUnderlineStyle,
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub text_overflow: TextOverflow,
    pub word_break: WordBreak,
    pub overflow_wrap_anywhere: bool,
    pub writing_mode: WritingMode,
    pub direction: LayoutDirection,
    pub text_wrap: TextWrap,
    pub text_indent: f32,
    pub tab_size: u8,
    pub caret: CaretStyle,
    pub font_variant_numeric: FontVariantNumeric,
    pub font_optical_sizing_auto: bool,
}
