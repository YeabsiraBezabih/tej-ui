//! Vector graphics styling, fill rules, stroke caps, and dash patterns.

use tej_ui_hal::Color;

/// Rule used to determine the interior of SVG vector shapes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FillRule {
    /// Non-zero winding rule.
    #[default]
    NonZero,
    /// Even-odd winding rule for hollow cutouts.
    EvenOdd,
}

/// Stroke cap shape on open vector path endpoints.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StrokeLineCap {
    /// Flat square cap ending exactly at path coordinates.
    #[default]
    Butt,
    /// Semi-circular rounded cap extending past path endpoints.
    Round,
    /// Square cap extending past path endpoints by half the stroke width.
    Square,
}

/// Shape used to join adjacent stroke path segments.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StrokeLineJoin {
    /// Sharp mitered corners.
    #[default]
    Miter,
    /// Smooth rounded corners.
    Round,
    /// Beveled flat-corner corners.
    Bevel,
}

/// Vector scaling effects under non-uniform transformations.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VectorEffect {
    /// Stroke scales with transformation.
    #[default]
    None,
    /// Stroke thickness remains constant regardless of container scaling.
    NonScalingStroke,
}

/// Styling configuration for vector rendering and SVG graphics.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SvgPaintStyle {
    /// Interior shape fill color.
    pub fill: Option<Color>,
    /// Fill transparency multiplier (0.0 to 1.0).
    pub fill_opacity: f32,
    /// Interior fill rule.
    pub fill_rule: FillRule,
    /// Path outline stroke color.
    pub stroke: Option<Color>,
    /// Outline stroke width in display points.
    pub stroke_width: f32,
    /// Stroke transparency multiplier (0.0 to 1.0).
    pub stroke_opacity: f32,
    /// Dash pattern array: [dash_length, gap_length].
    pub stroke_dasharray: Option<[f32; 2]>,
    /// Phase offset for stroke dash patterns (used in progress ring animations).
    pub stroke_dashoffset: f32,
    /// Line cap style.
    pub stroke_linecap: StrokeLineCap,
    /// Line join style.
    pub stroke_linejoin: StrokeLineJoin,
    /// Non-scaling stroke behavior.
    pub vector_effect: VectorEffect,
}

impl Default for SvgPaintStyle {
    fn default() -> Self {
        Self {
            fill: None,
            fill_opacity: 1.0,
            fill_rule: FillRule::NonZero,
            stroke: None,
            stroke_width: 1.0,
            stroke_opacity: 1.0,
            stroke_dasharray: None,
            stroke_dashoffset: 0.0,
            stroke_linecap: StrokeLineCap::Butt,
            stroke_linejoin: StrokeLineJoin::Miter,
            vector_effect: VectorEffect::None,
        }
    }
}

impl SvgPaintStyle {
    /// Creates a style for solid icon fills.
    #[must_use]
    pub const fn solid_fill(color: Color) -> Self {
        Self {
            fill: Some(color),
            fill_opacity: 1.0,
            fill_rule: FillRule::NonZero,
            stroke: None,
            stroke_width: 0.0,
            stroke_opacity: 1.0,
            stroke_dasharray: None,
            stroke_dashoffset: 0.0,
            stroke_linecap: StrokeLineCap::Butt,
            stroke_linejoin: StrokeLineJoin::Miter,
            vector_effect: VectorEffect::None,
        }
    }

    /// Creates a style for stroked outline icons.
    #[must_use]
    pub const fn stroked(color: Color, width: f32) -> Self {
        Self {
            fill: None,
            fill_opacity: 1.0,
            fill_rule: FillRule::NonZero,
            stroke: Some(color),
            stroke_width: width,
            stroke_opacity: 1.0,
            stroke_dasharray: None,
            stroke_dashoffset: 0.0,
            stroke_linecap: StrokeLineCap::Round,
            stroke_linejoin: StrokeLineJoin::Round,
            vector_effect: VectorEffect::NonScalingStroke,
        }
    }
}
