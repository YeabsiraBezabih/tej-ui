//! Modern CSS modules: Anchor positioning, View Transitions, SVG, Tables, and Paged Media.

use crate::svg_paint::SvgPaintStyle;
use std::borrow::Cow;
use tej_ui_hal::Color;
use tej_ui_layout::AnchorPositionArea;

/// Object fitting mode for replaced media elements (images/videos).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ObjectFit {
    #[default]
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

/// Table column layout algorithm.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TableLayout {
    #[default]
    Auto,
    Fixed,
}

/// Border collapse mode in tables.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BorderCollapse {
    #[default]
    Collapse,
    Separate,
}

/// System color scheme adaptation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ColorScheme {
    #[default]
    LightDark,
    Light,
    Dark,
}

/// Page / column break avoidance rules for paged media.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BreakRule {
    #[default]
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
}

/// Advanced modern modules and specialized specifications.
#[derive(Clone, PartialEq, Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CSS modern properties struct with public boolean CSS flags"
)]
pub struct ModernProps {
    // Anchor Positioning
    pub anchor_name: Option<Cow<'static, str>>,
    pub position_anchor: Option<Cow<'static, str>>,
    pub position_area: AnchorPositionArea,

    // View Transitions
    pub view_transition_name: Option<Cow<'static, str>>,

    // Media & Form Styling
    pub object_fit: ObjectFit,
    pub object_position_x: f32,
    pub object_position_y: f32,
    pub accent_color: Option<Color>,
    pub native_appearance: bool,
    pub pseudo_content: Option<Cow<'static, str>>,

    // Container Queries
    pub container_name: Option<Cow<'static, str>>,
    pub container_inline_size: bool,

    // SVG Vector Graphics
    pub svg: SvgPaintStyle,

    // Tables
    pub table_layout: TableLayout,
    pub border_collapse: BorderCollapse,
    pub border_spacing_h: f32,
    pub border_spacing_v: f32,
    pub caption_side_bottom: bool,
    pub empty_cells_hide: bool,

    // Multi-Column
    pub column_count: Option<u16>,
    pub column_width: Option<f32>,
    pub column_rule_width: f32,
    pub column_span_all: bool,

    // Paged Media & Print
    pub break_inside: BreakRule,
    pub break_before: BreakRule,
    pub orphans: u16,
    pub widows: u16,

    // Color Scheme
    pub color_scheme: ColorScheme,
    pub forced_color_adjust_none: bool,

    // Shapes
    pub shape_margin: f32,
}
