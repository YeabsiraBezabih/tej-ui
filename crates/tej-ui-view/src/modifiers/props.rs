//! Style properties and CSS bridge for view modifiers.

use crate::css::{
    AnimationDirection, AnimationFillMode, AnimationPlayState, BackgroundRepeat, BackgroundSize,
    BlendMode, BorderCollapse, BorderEdge, BorderLineStyle, BoxClip, BreakRule, ColorScheme,
    ObjectFit, PointerEvents, PositionScheme, ScrollBehavior, TableLayout, TimingFunction,
    TransformStyle, Visibility,
};
use crate::paint::InheritedTextStyle;
use crate::svg_paint::SvgPaintStyle;
use tej_ui_hal::Color;
use tej_ui_layout::{
    AlignItems, AlignSelf, AnchorPositionArea, Dimension, GridAutoFlow, Insets, JustifyItems,
    JustifySelf, LayoutDirection, Overflow, OverscrollBehavior, PlaceContent, ResizeMode,
    ScrollSnapAlign, ScrollSnapStop, ScrollSnapType, ScrollbarGutter, ScrollbarWidth,
};

/// Scroll, snap, and user-interaction CSS properties.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct InteractionStyle {
    pub scroll_behavior: ScrollBehavior,
    pub scroll_margin_top: f32,
    pub scroll_padding_top: f32,
    pub scroll_snap_type: ScrollSnapType,
    pub scroll_snap_align: ScrollSnapAlign,
    pub scroll_snap_stop: ScrollSnapStop,
    pub overscroll_behavior: OverscrollBehavior,
    pub scrollbar_width: ScrollbarWidth,
    pub scrollbar_gutter: ScrollbarGutter,
    pub scrollbar_thumb_color: Option<Color>,
    pub user_select: bool,
    pub touch_pan_x: bool,
    pub touch_pan_y: bool,
    pub resize: ResizeMode,
}

/// Transition, keyframe animation, and 2D/3D transform CSS properties.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct MotionStyle {
    pub transition_duration_ms: u32,
    pub transition_delay_ms: u32,
    pub transition_timing: TimingFunction,
    pub transform_rotate_deg: f32,
    pub transform_origin_x: f32,
    pub transform_origin_y: f32,
    pub has_animation: bool,
    pub animation_duration_ms: u32,
    pub animation_delay_ms: u32,
    pub animation_timing: TimingFunction,
    pub animation_iterations: u32,
    pub animation_direction: AnimationDirection,
    pub animation_fill_mode: AnimationFillMode,
    pub animation_play_state: AnimationPlayState,
    pub perspective: Option<f32>,
    pub transform_style: TransformStyle,
    pub backface_visible: bool,
    pub scroll_driven_progress: bool,
}

/// Anchor, media, table, multi-column, SVG, and paged-media CSS properties.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CSS modern style property bag with public boolean CSS flags"
)]
pub struct ModernStyle {
    pub has_anchor_name: bool,
    pub has_position_anchor: bool,
    pub position_area: AnchorPositionArea,
    pub has_view_transition_name: bool,
    pub object_fit: ObjectFit,
    pub object_position_x: f32,
    pub object_position_y: f32,
    pub native_appearance: bool,
    pub has_pseudo_content: bool,
    pub has_container_name: bool,
    pub container_inline_size: bool,
    pub svg: SvgPaintStyle,
    pub table_layout: TableLayout,
    pub border_collapse: BorderCollapse,
    pub border_spacing_h: f32,
    pub border_spacing_v: f32,
    pub caption_side_bottom: bool,
    pub empty_cells_hide: bool,
    pub column_count: Option<u16>,
    pub column_width: Option<f32>,
    pub column_rule_width: f32,
    pub column_span_all: bool,
    pub break_inside: BreakRule,
    pub break_before: BreakRule,
    pub orphans: u16,
    pub widows: u16,
    pub forced_color_adjust_none: bool,
    pub shape_margin: f32,
}

/// Visual style properties applied via view modifiers and `CssStyleSheet`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CSS style property bag with public boolean CSS flags"
)]
pub struct StyleProps {
    pub padding: Insets,
    pub margin: Insets,
    pub background: Option<Color>,
    pub background_size: BackgroundSize,
    pub background_repeat: BackgroundRepeat,
    pub background_clip: BoxClip,
    pub background_origin: BoxClip,
    pub mask_image_gradient: bool,
    pub mask_size: BackgroundSize,
    pub mask_repeat: BackgroundRepeat,
    pub clip_path_inset: Option<[f32; 4]>,
    pub corner_radius: f32,
    pub corner_radii: [f32; 4],
    pub border_width: f32,
    pub border_color: Option<Color>,
    pub border_edges: [BorderEdge; 4], // top, right, bottom, left
    pub outline_width: f32,
    pub outline_style: BorderLineStyle,
    pub outline_color: Option<Color>,
    pub outline_offset: f32,
    pub width: Dimension,
    pub height: Dimension,
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,
    pub flex_wrap: Option<tej_ui_layout::FlexWrap>,
    pub flex_grow: f32,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Dimension,
    pub align_self: AlignSelf,
    pub align_content: Option<AlignItems>,
    pub justify_items: JustifyItems,
    pub justify_self: JustifySelf,
    pub place_content: Option<PlaceContent>,
    pub order: i32,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub col_span: u16,
    pub row_span: u16,
    pub grid_columns: u16,
    pub grid_rows: u16,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_column_gap: f32,
    pub grid_row_gap: f32,
    pub container_query: Option<tej_ui_layout::ContainerQuery>,
    pub display: Option<tej_ui_layout::DisplayMode>,
    pub flex_direction: Option<tej_ui_layout::FlexDirection>,
    pub layout_direction: LayoutDirection,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<tej_ui_layout::JustifyContent>,
    pub gap: f32,
    pub opacity: Option<f32>,
    pub visibility: Visibility,
    pub mix_blend_mode: BlendMode,
    pub filter_grayscale: f32,
    pub isolate: bool,
    pub will_change_transform: bool,
    pub content_visibility_auto: bool,
    pub shadow: Option<crate::theme::Shadow>,
    pub secondary_shadow: Option<crate::theme::Shadow>,
    pub cursor: Option<crate::events::CursorIcon>,
    pub pointer_events: PointerEvents,
    pub aspect_ratio: Option<f32>,
    pub position: PositionScheme,
    pub inset_offset_x: f32,
    pub inset_offset_y: f32,
    pub z_index: Option<i32>,
    pub transform_translate_x: f32,
    pub transform_translate_y: f32,
    pub transform_scale: Option<f32>,
    pub text_style: Option<InheritedTextStyle>,
    pub accent_color: Option<Color>,
    pub color_scheme: ColorScheme,
    pub filter_blur: f32,
    pub backdrop_blur: f32,
    pub interaction: InteractionStyle,
    pub motion: MotionStyle,
    pub modern: ModernStyle,
}

impl StyleProps {
    /// Apply all supported properties across all 9 CSS modules from a `CssStyleSheet`.
    pub fn apply_css(&mut self, sheet: &crate::css::CssStyleSheet) {
        super::css_bridge::apply_css_stylesheet(self, sheet);
    }
}
