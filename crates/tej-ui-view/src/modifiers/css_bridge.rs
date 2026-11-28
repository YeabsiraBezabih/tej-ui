//! Complete bridge mapping all 9 `CssStyleSheet` modules onto `StyleProps`.

use super::props::StyleProps;
use crate::css::{ColorValue, CssStyleSheet, LengthPercentage, Visibility};
use crate::paint::InheritedTextStyle;
use tej_ui_layout::{AlignItems, AlignSelf, Dimension, DisplayMode, LayoutDirection, Overflow};

#[must_use]
fn lp_to_dimension(lp: LengthPercentage) -> Option<Dimension> {
    match lp {
        LengthPercentage::Points(v) => Some(Dimension::Points(v)),
        LengthPercentage::Percent(p)
        | LengthPercentage::Vw(p)
        | LengthPercentage::Vh(p)
        | LengthPercentage::Dvh(p) => Some(Dimension::Percent(p)),
        LengthPercentage::Ch(c) => Some(Dimension::Points(c * 8.0)),
        LengthPercentage::Clamp(_, val, _) => Some(Dimension::Points(val)),
        _ => None,
    }
}

#[must_use]
fn lp_to_points(lp: LengthPercentage) -> Option<f32> {
    match lp {
        LengthPercentage::Points(v) => Some(v),
        LengthPercentage::Ch(c) => Some(c * 8.0),
        LengthPercentage::Clamp(_, val, _) => Some(val),
        _ => None,
    }
}

#[must_use]
fn map_css_overflow(o: crate::css::Overflow) -> Overflow {
    match o {
        crate::css::Overflow::Visible => Overflow::Visible,
        crate::css::Overflow::Hidden | crate::css::Overflow::Clip => Overflow::Hidden,
        crate::css::Overflow::Scroll => Overflow::Scroll,
        crate::css::Overflow::Auto => Overflow::Auto,
    }
}

#[must_use]
fn map_align_self(align: Option<AlignItems>) -> AlignSelf {
    match align {
        Some(AlignItems::FlexStart) => AlignSelf::FlexStart,
        Some(AlignItems::Center) => AlignSelf::Center,
        Some(AlignItems::FlexEnd) => AlignSelf::FlexEnd,
        Some(AlignItems::Stretch) => AlignSelf::Stretch,
        None => AlignSelf::Auto,
    }
}

/// Applies all 9 modules of `sheet` (`box_sizing`, `spacing`, `flex_grid`, `backgrounds`,
/// `borders_effects`, `typography`, `interaction`, `motion`, `modern`) onto `props`.
pub fn apply_css_stylesheet(props: &mut StyleProps, sheet: &CssStyleSheet) {
    apply_box_sizing(props, sheet);
    apply_spacing(props, sheet);
    apply_flex_grid(props, sheet);
    apply_backgrounds(props, sheet);
    apply_borders_effects(props, sheet);
    apply_typography(props, sheet);
    apply_interaction(props, sheet);
    apply_motion(props, sheet);
    apply_modern(props, sheet);
}

fn apply_box_sizing(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let bs = &sheet.box_sizing;
    if let Some(w) = lp_to_dimension(bs.width).or_else(|| lp_to_dimension(bs.inline_size)) {
        props.width = w;
    }
    if let Some(h) = lp_to_dimension(bs.height).or_else(|| lp_to_dimension(bs.block_size)) {
        props.height = h;
    }
    if let Some(w) = lp_to_dimension(bs.min_width).or_else(|| lp_to_dimension(bs.min_inline_size)) {
        props.min_width = w;
    }
    if let Some(w) = lp_to_dimension(bs.max_width).or_else(|| lp_to_dimension(bs.max_inline_size)) {
        props.max_width = w;
    }
    if let Some(h) = lp_to_dimension(bs.min_height).or_else(|| lp_to_dimension(bs.min_block_size)) {
        props.min_height = h;
    }
    if let Some(h) = lp_to_dimension(bs.max_height).or_else(|| lp_to_dimension(bs.max_block_size)) {
        props.max_height = h;
    }
    if let Some(ar) = bs.aspect_ratio
        && ar.height > 0.0
    {
        props.aspect_ratio = Some(ar.width / ar.height);
    }
}

fn apply_spacing(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let sp = &sheet.spacing;
    let is_rtl = sheet.typography.direction == LayoutDirection::Rtl;

    if let Some(t) = lp_to_points(sp.padding_top).or_else(|| lp_to_points(sp.padding_block_start)) {
        props.padding.top = t;
    }
    if let Some(b) = lp_to_points(sp.padding_bottom).or_else(|| lp_to_points(sp.padding_block_end))
    {
        props.padding.bottom = b;
    }
    let (pad_start, pad_end) = if is_rtl {
        (sp.padding_inline_end, sp.padding_inline_start)
    } else {
        (sp.padding_inline_start, sp.padding_inline_end)
    };
    if let Some(l) = lp_to_points(sp.padding_left).or_else(|| lp_to_points(pad_start)) {
        props.padding.left = l;
    }
    if let Some(r) = lp_to_points(sp.padding_right).or_else(|| lp_to_points(pad_end)) {
        props.padding.right = r;
    }

    if let Some(t) = lp_to_points(sp.margin_top).or_else(|| lp_to_points(sp.margin_block_start)) {
        props.margin.top = t;
    }
    if let Some(b) = lp_to_points(sp.margin_bottom).or_else(|| lp_to_points(sp.margin_block_end)) {
        props.margin.bottom = b;
    }
    let (mar_start, mar_end) = if is_rtl {
        (sp.margin_inline_end, sp.margin_inline_start)
    } else {
        (sp.margin_inline_start, sp.margin_inline_end)
    };
    if let Some(l) = lp_to_points(sp.margin_left).or_else(|| lp_to_points(mar_start)) {
        props.margin.left = l;
    }
    if let Some(r) = lp_to_points(sp.margin_right).or_else(|| lp_to_points(mar_end)) {
        props.margin.right = r;
    }

    props.position = sp.position;
    props.z_index = sp.z_index;
    let (inset_start, inset_end) = if is_rtl {
        (sp.inset_inline_end, sp.inset_inline_start)
    } else {
        (sp.inset_inline_start, sp.inset_inline_end)
    };
    let dx = lp_to_points(sp.left)
        .or_else(|| lp_to_points(inset_start))
        .or_else(|| lp_to_points(sp.right).map(|v| -v))
        .or_else(|| lp_to_points(inset_end).map(|v| -v))
        .unwrap_or(0.0);
    let dy = lp_to_points(sp.top)
        .or_else(|| lp_to_points(sp.inset_block_start))
        .or_else(|| lp_to_points(sp.bottom).map(|v| -v))
        .or_else(|| lp_to_points(sp.inset_block_end).map(|v| -v))
        .unwrap_or(0.0);
    props.inset_offset_x = dx;
    props.inset_offset_y = dy;
}

fn apply_flex_grid(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let fg = &sheet.flex_grid;
    props.display = Some(match fg.display {
        crate::css::Display::Grid | crate::css::Display::InlineGrid => DisplayMode::Grid,
        crate::css::Display::None => DisplayMode::None,
        _ => DisplayMode::Flex,
    });
    props.flex_direction = Some(fg.flex_direction);
    props.flex_wrap = Some(fg.flex_wrap);
    props.align_items = Some(fg.align_items);
    props.align_content = fg.align_content;
    props.justify_content = Some(fg.justify_content);
    props.justify_items = fg.justify_items;
    props.justify_self = fg.justify_self;
    props.place_content = fg.place_content;
    if let Some(pc) = fg.place_content {
        props.align_content = Some(pc.align);
        props.justify_content = Some(pc.justify);
    }
    props.align_self = map_align_self(fg.align_self);
    props.order = fg.order;
    props.gap = fg.row_gap.max(fg.column_gap);
    props.grid_row_gap = fg.row_gap;
    props.grid_column_gap = fg.column_gap;
    props.flex_grow = fg.flex_grow;
    if fg.flex_shrink > 0.0 || fg.flex_basis != LengthPercentage::Auto {
        props.flex_shrink = Some(fg.flex_shrink);
    }
    if let Some(basis) = lp_to_dimension(fg.flex_basis) {
        props.flex_basis = basis;
    }
    if !fg.grid_template_columns.is_empty() {
        props.grid_columns = fg.grid_template_columns.len() as u16;
    } else if fg.grid_auto_columns.is_some() {
        props.grid_columns = props.grid_columns.max(1);
    }
    if !fg.grid_template_rows.is_empty() {
        props.grid_rows = fg.grid_template_rows.len() as u16;
    } else if fg.grid_auto_rows.is_some() {
        props.grid_rows = props.grid_rows.max(1);
    }
    props.grid_auto_flow = fg.grid_auto_flow;
    if fg.grid_column.span > 0 {
        props.col_span = fg.grid_column.span;
    }
    if fg.grid_row.span > 0 {
        props.row_span = fg.grid_row.span;
    }
}

fn apply_backgrounds(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let bg = &sheet.backgrounds;
    if let Some(col) = bg.background_color {
        props.background = Some(col);
    }
    props.background_size = bg.background_size;
    props.background_repeat = bg.background_repeat;
    props.background_clip = bg.background_clip;
    props.background_origin = bg.background_origin;
    props.mask_image_gradient = bg.mask_image_gradient;
    props.mask_size = bg.mask_size;
    props.mask_repeat = bg.mask_repeat;
    props.clip_path_inset = bg.clip_path_inset;
}

fn apply_borders_effects(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let be = &sheet.borders_effects;
    let is_rtl = sheet.typography.direction == LayoutDirection::Rtl;

    let top = if be.border_top.width > 0.0 {
        be.border_top
    } else {
        be.border_block_start
    };
    let bottom = if be.border_bottom.width > 0.0 {
        be.border_bottom
    } else {
        be.border_block_end
    };
    let left = if be.border_left.width > 0.0 {
        be.border_left
    } else if is_rtl {
        be.border_inline_end
    } else {
        be.border_inline_start
    };
    let right = if be.border_right.width > 0.0 {
        be.border_right
    } else if is_rtl {
        be.border_inline_start
    } else {
        be.border_inline_end
    };
    props.border_edges = [top, right, bottom, left];

    if top.width > 0.0 {
        props.border_width = top.width;
        props.border_color = Some(top.color);
    } else {
        for edge in [right, bottom, left] {
            if edge.width > 0.0 {
                props.border_width = edge.width;
                props.border_color = Some(edge.color);
                break;
            }
        }
    }

    let tl = if be.corner_radius_tl > 0.0 {
        be.corner_radius_tl
    } else {
        be.corner_radius_start_start
    };
    let br = if be.corner_radius_br > 0.0 {
        be.corner_radius_br
    } else {
        be.corner_radius_end_end
    };
    props.corner_radii = [tl, be.corner_radius_tr, br, be.corner_radius_bl];
    let max_r = props.corner_radii.into_iter().fold(0.0f32, f32::max);
    if max_r > 0.0 {
        props.corner_radius = max_r;
    }

    if be.outline_width > 0.0 {
        props.outline_width = be.outline_width;
        props.outline_style = be.outline_style;
        props.outline_color = Some(be.outline_color);
        props.outline_offset = be.outline_offset;
    }

    if be.opacity >= 0.0 && be.opacity < 1.0 {
        props.opacity = Some(be.opacity);
    }
    props.visibility = be.visibility;
    if be.visibility == Visibility::Collapse {
        props.display = Some(DisplayMode::None);
    }
    props.mix_blend_mode = be.mix_blend_mode;
    props.filter_blur = be.filter_blur;
    props.filter_grayscale = be.filter_grayscale;
    props.backdrop_blur = be.backdrop_blur;
    props.isolate = be.isolate;
    props.will_change_transform = be.will_change_transform;
    props.content_visibility_auto = be.content_visibility_auto;

    if let Some(first_shadow) = be.box_shadows.first() {
        props.shadow = Some(*first_shadow);
    }
    if let Some(second_shadow) = be.box_shadows.get(1) {
        props.secondary_shadow = Some(*second_shadow);
    }
}

fn apply_typography(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let ty = &sheet.typography;
    props.layout_direction = ty.direction;
    let color = match ty.color {
        ColorValue::Color(c) => Some(c),
        _ => None,
    };
    let font_size = (ty.font_size > 0.0).then_some(ty.font_size);
    let font_weight = (ty.font_weight > 0).then_some(ty.font_weight);
    let italic = (ty.font_style != crate::css::FontStyle::Normal).then_some(true);
    let family_lower = ty.font_family.as_ref();
    let monospace = if family_lower.eq_ignore_ascii_case("monospace")
        || family_lower.contains("Mono")
        || family_lower.contains("mono")
    {
        Some(true)
    } else {
        None
    };
    let line_height = (ty.line_height > 0.0).then_some(ty.line_height);
    let letter_spacing = (ty.letter_spacing != 0.0).then_some(ty.letter_spacing);
    let word_spacing = (ty.word_spacing != 0.0).then_some(ty.word_spacing);
    let text_align = (ty.text_align != crate::css::TextAlign::Start).then_some(ty.text_align);
    let text_decoration = (ty.text_decoration_line != crate::css::TextDecorationLine::None)
        .then_some(ty.text_decoration_line);
    let text_underline = (ty.text_underline
        != crate::typography_extended::TextUnderlineStyle::default())
    .then_some(ty.text_underline);
    let text_transform =
        (ty.text_transform != crate::css::TextTransform::None).then_some(ty.text_transform);
    let white_space = (ty.white_space != crate::css::WhiteSpace::Normal).then_some(ty.white_space);
    let text_overflow =
        (ty.text_overflow != crate::css::TextOverflow::Clip).then_some(ty.text_overflow);
    let word_break = (ty.word_break != crate::css::WordBreak::Normal).then_some(ty.word_break);
    let overflow_wrap_anywhere = ty.overflow_wrap_anywhere.then_some(true);
    let writing_mode =
        (ty.writing_mode != crate::css::WritingMode::HorizontalTb).then_some(ty.writing_mode);
    let text_wrap =
        (ty.text_wrap != crate::typography_extended::TextWrap::Wrap).then_some(ty.text_wrap);
    let text_indent = (ty.text_indent != 0.0).then_some(ty.text_indent);
    let tab_size = (ty.tab_size > 0).then_some(ty.tab_size);
    let caret = (ty.caret != crate::typography_extended::CaretStyle::default()).then_some(ty.caret);
    let font_variant_numeric = (ty.font_variant_numeric
        != crate::typography_extended::FontVariantNumeric::Normal)
        .then_some(ty.font_variant_numeric);
    let font_optical_sizing_auto = ty.font_optical_sizing_auto.then_some(true);

    if color.is_some()
        || font_size.is_some()
        || font_weight.is_some()
        || italic.is_some()
        || monospace.is_some()
        || line_height.is_some()
        || letter_spacing.is_some()
        || word_spacing.is_some()
        || text_align.is_some()
        || text_decoration.is_some()
        || text_underline.is_some()
        || text_transform.is_some()
        || white_space.is_some()
        || text_overflow.is_some()
        || word_break.is_some()
        || overflow_wrap_anywhere.is_some()
        || writing_mode.is_some()
        || text_wrap.is_some()
        || text_indent.is_some()
        || tab_size.is_some()
        || caret.is_some()
        || font_variant_numeric.is_some()
        || font_optical_sizing_auto.is_some()
    {
        props.text_style = Some(InheritedTextStyle {
            color,
            font_size,
            font_weight,
            italic,
            monospace,
            line_height,
            letter_spacing,
            word_spacing,
            text_align,
            text_decoration,
            text_underline,
            text_transform,
            white_space,
            text_overflow,
            word_break,
            overflow_wrap_anywhere,
            writing_mode,
            text_wrap,
            text_indent,
            tab_size,
            caret,
            font_variant_numeric,
            font_optical_sizing_auto,
        });
    }
}

fn apply_interaction(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let inter = &sheet.interaction;
    props.overflow_x = map_css_overflow(inter.overflow_x);
    props.overflow_y = map_css_overflow(inter.overflow_y);
    props.cursor = Some(inter.cursor);
    props.pointer_events = inter.pointer_events;
    props.interaction = super::props::InteractionStyle {
        scroll_behavior: inter.scroll_behavior,
        scroll_margin_top: inter.scroll_margin_top,
        scroll_padding_top: inter.scroll_padding_top,
        scroll_snap_type: inter.scroll_snap_type,
        scroll_snap_align: inter.scroll_snap_align,
        scroll_snap_stop: inter.scroll_snap_stop,
        overscroll_behavior: inter.overscroll_behavior,
        scrollbar_width: inter.scrollbar_width,
        scrollbar_gutter: inter.scrollbar_gutter,
        scrollbar_thumb_color: inter.scrollbar_thumb_color,
        user_select: inter.user_select,
        touch_pan_x: inter.touch_pan_x,
        touch_pan_y: inter.touch_pan_y,
        resize: inter.resize,
    };
}

fn apply_motion(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let mo = &sheet.motion;
    props.transform_translate_x = mo.transform_translate_x;
    props.transform_translate_y = mo.transform_translate_y;
    if (mo.transform_scale - 1.0).abs() > f32::EPSILON && mo.transform_scale > 0.0 {
        props.transform_scale = Some(mo.transform_scale);
    }
    props.motion = super::props::MotionStyle {
        transition_duration_ms: mo.transition_duration_ms,
        transition_delay_ms: mo.transition_delay_ms,
        transition_timing: mo.transition_timing,
        transform_rotate_deg: mo.transform_rotate_deg,
        transform_origin_x: mo.transform_origin_x,
        transform_origin_y: mo.transform_origin_y,
        has_animation: mo.animation_name.is_some(),
        animation_duration_ms: mo.animation_duration_ms,
        animation_delay_ms: mo.animation_delay_ms,
        animation_timing: mo.animation_timing,
        animation_iterations: mo.animation_iterations,
        animation_direction: mo.animation_direction,
        animation_fill_mode: mo.animation_fill_mode,
        animation_play_state: mo.animation_play_state,
        perspective: mo.perspective,
        transform_style: mo.transform_style,
        backface_visible: mo.backface_visible,
        scroll_driven_progress: mo.scroll_driven_progress,
    };
}

fn apply_modern(props: &mut StyleProps, sheet: &CssStyleSheet) {
    let md = &sheet.modern;
    props.accent_color = md.accent_color;
    props.color_scheme = md.color_scheme;
    props.modern = super::props::ModernStyle {
        has_anchor_name: md.anchor_name.is_some(),
        has_position_anchor: md.position_anchor.is_some(),
        position_area: md.position_area,
        has_view_transition_name: md.view_transition_name.is_some(),
        object_fit: md.object_fit,
        object_position_x: md.object_position_x,
        object_position_y: md.object_position_y,
        native_appearance: md.native_appearance,
        has_pseudo_content: md.pseudo_content.is_some(),
        has_container_name: md.container_name.is_some(),
        container_inline_size: md.container_inline_size,
        svg: md.svg,
        table_layout: md.table_layout,
        border_collapse: md.border_collapse,
        border_spacing_h: md.border_spacing_h,
        border_spacing_v: md.border_spacing_v,
        caption_side_bottom: md.caption_side_bottom,
        empty_cells_hide: md.empty_cells_hide,
        column_count: md.column_count,
        column_width: md.column_width,
        column_rule_width: md.column_rule_width,
        column_span_all: md.column_span_all,
        break_inside: md.break_inside,
        break_before: md.break_before,
        orphans: md.orphans,
        widows: md.widows,
        forced_color_adjust_none: md.forced_color_adjust_none,
        shape_margin: md.shape_margin,
    };
}
