//! Paint helper routines for `Styled<V>` rendering CSS backgrounds, borders, shadows,
//! outlines, transforms, clipping, and inherited typography.

use super::props::StyleProps;
use crate::css::{BoxClip, ColorScheme, PositionScheme, Visibility};
use crate::paint::PaintContext;
use crate::view::View;
use tej_ui_hal::Color;
use tej_ui_layout::{Overflow, Rect};

#[must_use]
fn with_opacity(color: Color, opacity: Option<f32>) -> Color {
    if let Some(op) = opacity {
        let factor = op.clamp(0.0, 1.0);
        Color::rgba(
            color.r,
            color.g,
            color.b,
            (f32::from(color.a) * factor) as u8,
        )
    } else {
        color
    }
}

fn draw_shadow(
    ctx: &mut PaintContext<'_>,
    bounds: Rect,
    radius: f32,
    shadow: crate::theme::Shadow,
    opacity: Option<f32>,
) {
    if shadow.blur > 0.0 || shadow.offset_x != 0.0 || shadow.offset_y != 0.0 {
        let shadow_bounds = Rect::new(
            bounds.origin.x + shadow.offset_x - shadow.spread,
            bounds.origin.y + shadow.offset_y - shadow.spread,
            bounds.size.width + shadow.spread * 2.0,
            bounds.size.height + shadow.spread * 2.0,
        );
        ctx.fill_rounded_rect(
            shadow_bounds,
            radius + shadow.blur * 0.5,
            with_opacity(shadow.color, opacity),
        );
    }
}

fn compute_bg_rect(bounds: Rect, style: &StyleProps) -> Rect {
    match style.background_clip {
        BoxClip::PaddingBox => {
            let bw = style.border_width.max(0.0);
            Rect::new(
                bounds.origin.x + bw,
                bounds.origin.y + bw,
                (bounds.size.width - 2.0 * bw).max(0.0),
                (bounds.size.height - 2.0 * bw).max(0.0),
            )
        }
        BoxClip::ContentBox => {
            let l = style.border_width.max(0.0) + style.padding.left;
            let t = style.border_width.max(0.0) + style.padding.top;
            let r = style.border_width.max(0.0) + style.padding.right;
            let b = style.border_width.max(0.0) + style.padding.bottom;
            Rect::new(
                bounds.origin.x + l,
                bounds.origin.y + t,
                (bounds.size.width - l - r).max(0.0),
                (bounds.size.height - t - b).max(0.0),
            )
        }
        BoxClip::BorderBox | BoxClip::Text => bounds,
    }
}

fn draw_edge_borders(ctx: &mut PaintContext<'_>, bounds: Rect, style: &StyleProps) {
    let [top, right, bottom, left] = style.border_edges;
    let x = bounds.origin.x;
    let y = bounds.origin.y;
    let w = bounds.size.width;
    let h = bounds.size.height;
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    if top.width > 0.0 {
        let bw = top.width.min(h);
        ctx.fill_rect(
            Rect::new(x, y, w, bw),
            with_opacity(top.color, style.opacity),
        );
    }
    if bottom.width > 0.0 {
        let bw = bottom.width.min(h);
        ctx.fill_rect(
            Rect::new(x, y + h - bw, w, bw),
            with_opacity(bottom.color, style.opacity),
        );
    }
    if left.width > 0.0 {
        let bw = left.width.min(w);
        ctx.fill_rect(
            Rect::new(x, y, bw, h),
            with_opacity(left.color, style.opacity),
        );
    }
    if right.width > 0.0 {
        let bw = right.width.min(w);
        ctx.fill_rect(
            Rect::new(x + w - bw, y, bw, h),
            with_opacity(right.color, style.opacity),
        );
    }
}

fn draw_outline(ctx: &mut PaintContext<'_>, bounds: Rect, style: &StyleProps) {
    let ow = style.outline_width;
    if ow <= 0.0 {
        return;
    }
    let Some(outline_col) = style.outline_color else {
        return;
    };
    let col = with_opacity(outline_col, style.opacity);
    let off = style.outline_offset;
    let ox = bounds.origin.x - off - ow;
    let oy = bounds.origin.y - off - ow;
    let ow_total = (bounds.size.width + 2.0 * (off + ow)).max(0.0);
    let oh_total = (bounds.size.height + 2.0 * (off + ow)).max(0.0);
    ctx.fill_rect(Rect::new(ox, oy, ow_total, ow), col);
    ctx.fill_rect(Rect::new(ox, oy + oh_total - ow, ow_total, ow), col);
    ctx.fill_rect(Rect::new(ox, oy, ow, oh_total), col);
    ctx.fill_rect(Rect::new(ox + ow_total - ow, oy, ow, oh_total), col);
}

/// Paints a `Styled<V>` container and its inner child view with full CSS property support.
pub fn paint_styled_node<V: View>(
    inner: &V,
    inner_state: &V::State,
    style: &StyleProps,
    node: u32,
    ctx: &mut PaintContext<'_>,
) {
    if style.visibility == Visibility::Hidden || style.visibility == Visibility::Collapse {
        return;
    }

    let raw_bounds = ctx
        .arena
        .computed_bounds
        .get(node as usize)
        .copied()
        .unwrap_or(Rect::ZERO);

    if style.content_visibility_auto && !ctx.is_dirty(raw_bounds) {
        return;
    }

    let bounds = apply_scale_transform(raw_bounds, style);
    let has_transform = push_styled_translation(style, ctx);

    paint_styled_decorations(ctx, bounds, style);

    let pushed_fg = match style.color_scheme {
        ColorScheme::Light => {
            ctx.push_foreground(Color::from_rgb_u32(0x0009_090B));
            true
        }
        ColorScheme::Dark => {
            ctx.push_foreground(Color::from_rgb_u32(0x00FA_FAFA));
            true
        }
        ColorScheme::LightDark => false,
    };

    if let Some(ts) = style.text_style {
        ctx.push_text_style(ts);
    }

    paint_styled_child_with_clip(inner, inner_state, style, bounds, ctx);

    if style.text_style.is_some() {
        ctx.pop_text_style();
    }
    if pushed_fg {
        ctx.pop_foreground();
    }
    if has_transform {
        ctx.pop_transform();
    }
}

fn apply_scale_transform(bounds: Rect, style: &StyleProps) -> Rect {
    let Some(scale) = style.transform_scale else {
        return bounds;
    };
    let ox = if style.motion.transform_origin_x == 0.0 {
        bounds.size.width * 0.5
    } else {
        style.motion.transform_origin_x
    };
    let oy = if style.motion.transform_origin_y == 0.0 {
        bounds.size.height * 0.5
    } else {
        style.motion.transform_origin_y
    };
    let cx = bounds.origin.x + ox;
    let cy = bounds.origin.y + oy;
    Rect::new(
        cx - ox * scale,
        cy - oy * scale,
        bounds.size.width * scale,
        bounds.size.height * scale,
    )
}

fn push_styled_translation(style: &StyleProps, ctx: &mut PaintContext<'_>) -> bool {
    let (rel_dx, rel_dy) = if style.position == PositionScheme::Relative {
        (style.inset_offset_x, style.inset_offset_y)
    } else {
        (0.0, 0.0)
    };
    let tx = style.transform_translate_x + rel_dx;
    let ty = style.transform_translate_y + rel_dy;
    let has_transform = tx != 0.0 || ty != 0.0;
    if has_transform {
        ctx.push_transform(tx, ty);
    }
    has_transform
}

fn paint_styled_decorations(ctx: &mut PaintContext<'_>, bounds: Rect, style: &StyleProps) {
    if let Some(shadow) = style.shadow {
        draw_shadow(ctx, bounds, style.corner_radius, shadow, style.opacity);
    }
    if let Some(shadow2) = style.secondary_shadow {
        draw_shadow(ctx, bounds, style.corner_radius, shadow2, style.opacity);
    }

    let has_uniform_border = style.border_width > 0.0 && style.border_color.is_some();
    let has_edge_borders = style.border_edges.iter().any(|e| e.width > 0.0);

    if has_uniform_border && style.background.is_some() && style.corner_radius > 0.0 {
        let border_col = with_opacity(
            style.border_color.unwrap_or(Color::TRANSPARENT),
            style.opacity,
        );
        let bg_col = with_opacity(
            style.background.unwrap_or(Color::TRANSPARENT),
            style.opacity,
        );
        let bw = style
            .border_width
            .min(bounds.size.width * 0.5)
            .min(bounds.size.height * 0.5)
            .max(0.0);
        ctx.fill_rounded_rect(bounds, style.corner_radius, border_col);

        let inner_w = (bounds.size.width - 2.0 * bw).max(0.0);
        let inner_h = (bounds.size.height - 2.0 * bw).max(0.0);
        if inner_w > 0.0 && inner_h > 0.0 {
            let inner_bounds =
                Rect::new(bounds.origin.x + bw, bounds.origin.y + bw, inner_w, inner_h);
            ctx.fill_rounded_rect(inner_bounds, (style.corner_radius - bw).max(0.0), bg_col);
        }
    } else {
        if let Some(bg) = style.background {
            let bg_rect = compute_bg_rect(bounds, style);
            let col = with_opacity(bg, style.opacity);
            if style.corner_radius > 0.0 {
                ctx.fill_rounded_rect(bg_rect, style.corner_radius, col);
            } else {
                ctx.fill_rect(bg_rect, col);
            }
        }
        if has_edge_borders {
            draw_edge_borders(ctx, bounds, style);
        } else if has_uniform_border {
            let col = with_opacity(
                style.border_color.unwrap_or(Color::TRANSPARENT),
                style.opacity,
            );
            let bw = style
                .border_width
                .min(bounds.size.width * 0.5)
                .min(bounds.size.height * 0.5)
                .max(0.0);
            let mut tmp = *style;
            let edge = crate::css::BorderEdge {
                width: bw,
                style: crate::css::BorderLineStyle::Solid,
                color: col,
            };
            tmp.border_edges = [edge, edge, edge, edge];
            tmp.opacity = None;
            draw_edge_borders(ctx, bounds, &tmp);
        }
    }

    draw_outline(ctx, bounds, style);
}

fn paint_styled_child_with_clip<V: View>(
    inner: &V,
    inner_state: &V::State,
    style: &StyleProps,
    bounds: Rect,
    ctx: &mut PaintContext<'_>,
) {
    let should_clip_overflow =
        style.overflow_x != Overflow::Visible || style.overflow_y != Overflow::Visible;
    let clip_rect = if let Some([ct, cr, cb, cl]) = style.clip_path_inset {
        Some(Rect::new(
            bounds.origin.x + cl,
            bounds.origin.y + ct,
            (bounds.size.width - cl - cr).max(0.0),
            (bounds.size.height - ct - cb).max(0.0),
        ))
    } else if should_clip_overflow {
        Some(bounds)
    } else {
        None
    };

    if let Some(cr) = clip_rect {
        let radii = if style.corner_radii.iter().any(|&r| r > 0.0) {
            style.corner_radii
        } else {
            [style.corner_radius; 4]
        };
        ctx.push_clip(cr, radii);
        inner.paint(inner_state, ctx);
        ctx.pop_clip();
    } else {
        inner.paint(inner_state, ctx);
    }
}
