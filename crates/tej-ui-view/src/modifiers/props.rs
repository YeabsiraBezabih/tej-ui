//! Style properties and CSS bridge for view modifiers.

use tej_ui_hal::Color;
use tej_ui_layout::{Dimension, Insets};

/// Visual style properties applied via view modifiers.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct StyleProps {
    pub padding: Insets,
    pub margin: Insets,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Option<Color>,
    pub width: Dimension,
    pub height: Dimension,
    pub flex_wrap: Option<tej_ui_layout::FlexWrap>,
    pub flex_grow: f32,
    pub col_span: u16,
    pub row_span: u16,
    pub container_query: Option<tej_ui_layout::ContainerQuery>,
    pub display: Option<tej_ui_layout::DisplayMode>,
    pub flex_direction: Option<tej_ui_layout::FlexDirection>,
    pub align_items: Option<tej_ui_layout::AlignItems>,
    pub justify_content: Option<tej_ui_layout::JustifyContent>,
    pub gap: f32,
    pub opacity: Option<f32>,
    pub shadow: Option<crate::theme::Shadow>,
    pub cursor: Option<crate::events::CursorIcon>,
    pub aspect_ratio: Option<f32>,
}

impl StyleProps {
    /// Apply core properties from a CssStyleSheet onto StyleProps.
    pub fn apply_css(&mut self, sheet: &crate::css::CssStyleSheet) {
        if let crate::css::LengthPercentage::Points(w) = sheet.box_sizing.width {
            self.width = Dimension::Points(w);
        }
        if let crate::css::LengthPercentage::Points(h) = sheet.box_sizing.height {
            self.height = Dimension::Points(h);
        }
        if let (
            crate::css::LengthPercentage::Points(t),
            crate::css::LengthPercentage::Points(r),
            crate::css::LengthPercentage::Points(b),
            crate::css::LengthPercentage::Points(l),
        ) = (
            sheet.spacing.padding_top,
            sheet.spacing.padding_right,
            sheet.spacing.padding_bottom,
            sheet.spacing.padding_left,
        ) {
            self.padding = Insets::new(t, r, b, l);
        }
        if let (
            crate::css::LengthPercentage::Points(t),
            crate::css::LengthPercentage::Points(r),
            crate::css::LengthPercentage::Points(b),
            crate::css::LengthPercentage::Points(l),
        ) = (
            sheet.spacing.margin_top,
            sheet.spacing.margin_right,
            sheet.spacing.margin_bottom,
            sheet.spacing.margin_left,
        ) {
            self.margin = Insets::new(t, r, b, l);
        }
        if let Some(bg) = sheet.backgrounds.background_color {
            self.background = Some(bg);
        }
        if sheet.borders_effects.corner_radius_tl > 0.0 {
            self.corner_radius = sheet.borders_effects.corner_radius_tl;
        }
        if sheet.borders_effects.border_top.width > 0.0 {
            self.border_width = sheet.borders_effects.border_top.width;
            self.border_color = Some(sheet.borders_effects.border_top.color);
        }
        self.align_items = Some(sheet.flex_grid.align_items);
        self.justify_content = Some(sheet.flex_grid.justify_content);
        self.gap = sheet.flex_grid.row_gap.max(sheet.flex_grid.column_gap);
        self.flex_grow = sheet.flex_grid.flex_grow;
        if sheet.borders_effects.opacity > 0.0 && sheet.borders_effects.opacity < 1.0 {
            self.opacity = Some(sheet.borders_effects.opacity);
        }
        if let Some(first_shadow) = sheet.borders_effects.box_shadows.first() {
            self.shadow = Some(*first_shadow);
        }
        self.cursor = Some(sheet.interaction.cursor);
    }
}
