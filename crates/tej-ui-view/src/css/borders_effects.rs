//! Borders, rounded radii, shadows, outlines, opacity, and visual filter effects.

use crate::theme::Shadow;
use smallvec::SmallVec;
use tej_ui_hal::Color;

/// Stroke line pattern for borders and outlines.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BorderLineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Double,
    None,
}

/// Single border edge description.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct BorderEdge {
    pub width: f32,
    pub style: BorderLineStyle,
    pub color: Color,
}

/// Visual element visibility mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

/// Blend modes for compositing overlapping visual surfaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Difference,
}

/// Border, radius, elevation shadow, and rasterization effects.
#[derive(Clone, PartialEq, Debug)]
pub struct BordersEffectsProps {
    pub border_top: BorderEdge,
    pub border_bottom: BorderEdge,
    pub border_left: BorderEdge,
    pub border_right: BorderEdge,
    pub border_block_start: BorderEdge,
    pub border_block_end: BorderEdge,
    pub border_inline_start: BorderEdge,
    pub border_inline_end: BorderEdge,
    pub corner_radius_tl: f32,
    pub corner_radius_tr: f32,
    pub corner_radius_br: f32,
    pub corner_radius_bl: f32,
    pub corner_radius_start_start: f32,
    pub corner_radius_end_end: f32,
    pub outline_width: f32,
    pub outline_style: BorderLineStyle,
    pub outline_color: Color,
    pub outline_offset: f32,
    pub box_shadows: SmallVec<[Shadow; 2]>,
    pub opacity: f32,
    pub visibility: Visibility,
    pub mix_blend_mode: BlendMode,
    pub filter_blur: f32,
    pub filter_grayscale: f32,
    pub backdrop_blur: f32,
    pub isolate: bool,
    pub will_change_transform: bool,
    pub content_visibility_auto: bool,
}

impl Default for BordersEffectsProps {
    fn default() -> Self {
        Self {
            border_top: BorderEdge::default(),
            border_bottom: BorderEdge::default(),
            border_left: BorderEdge::default(),
            border_right: BorderEdge::default(),
            border_block_start: BorderEdge::default(),
            border_block_end: BorderEdge::default(),
            border_inline_start: BorderEdge::default(),
            border_inline_end: BorderEdge::default(),
            corner_radius_tl: 0.0,
            corner_radius_tr: 0.0,
            corner_radius_br: 0.0,
            corner_radius_bl: 0.0,
            corner_radius_start_start: 0.0,
            corner_radius_end_end: 0.0,
            outline_width: 0.0,
            outline_style: BorderLineStyle::default(),
            outline_color: Color::default(),
            outline_offset: 0.0,
            box_shadows: SmallVec::new(),
            opacity: 1.0,
            visibility: Visibility::default(),
            mix_blend_mode: BlendMode::default(),
            filter_blur: 0.0,
            filter_grayscale: 0.0,
            backdrop_blur: 0.0,
            isolate: false,
            will_change_transform: false,
            content_visibility_auto: false,
        }
    }
}
