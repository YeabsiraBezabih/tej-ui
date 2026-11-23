//! Overview summary cards for all 231 CSS properties in `tej-ui`.

use tej_ui::prelude::*;

/// Category overview card summarizing a CSS property group.
fn category_card(
    title: &'static str,
    count: usize,
    sample: &'static str,
    desc: &'static str,
) -> impl View {
    Card::new(format!("{title} ({count} Props)"))
        .description(format!("{sample}\n\n{desc}"))
        .flex_grow(1.0)
}

/// Overview section providing architectural summary of all 231 CSS properties.
pub fn css_overview_view() -> impl View {
    VStack::new((
        HStack::new((
            category_card(
                "1. Box Sizing & Dimensions",
                14,
                "width, height, min/max, inline-size, block-size, aspect-ratio",
                "Zero-copy physical and logical sizing engine with aspect-ratio constraints.",
            ),
            category_card(
                "2. Spacing, Margins & Insets",
                31,
                "margin, padding, top, right, bottom, left, inset, z-index",
                "Multi-axis physical and logical margins, paddings, and positioned offsets.",
            ),
            category_card(
                "3. Flexbox & CSS Grid",
                29,
                "display, flex-direction, justify-content, align-items, grid-template, gap",
                "Two-pass Flexbox engine and CSS Grid track resolution with auto-flow.",
            ),
        ))
        .spacing(12),
        HStack::new((
            category_card(
                "4. Backgrounds & Alpha Masks",
                15,
                "background-color, background-size, background-repeat, mask-image, clip-path",
                "GPU surface fills, texture tiling, alpha masks, and vector clip boundaries.",
            ),
            category_card(
                "5. Typography & Text Bidi",
                34,
                "font-family, font-size, font-weight, line-height, text-align, text-wrap",
                "System font discovery, OpenType features, RTL bidi, and text wrapping.",
            ),
            category_card(
                "6. Motion & GPU Transforms",
                24,
                "transition, transform, translate, scale, rotate, animation, perspective",
                "Hardware-accelerated 2D/3D affine matrix transforms and keyframe timelines.",
            ),
        ))
        .spacing(12),
        HStack::new((
            category_card(
                "7. Borders, Radii & Shadows",
                33,
                "border, border-radius, box-shadow, outline, opacity, backdrop-filter",
                "Subpixel SDF anti-aliased rounded borders, elevations, and Gaussian blurs.",
            ),
            category_card(
                "8. Interaction & Hit-Testing",
                22,
                "overflow, scroll-behavior, scroll-snap, cursor, pointer-events, user-select",
                "Kinetic scrolling momentum, magnetic snap stops, and OS cursor binding.",
            ),
            category_card(
                "9. Modern Modules & SVG",
                29,
                "anchor-name, position-area, view-transition-name, object-fit, SVG stroke/fill",
                "CSS Anchor Positioning, View Transitions, SVG paint, and table columns.",
            ),
        ))
        .spacing(12),
    ))
    .spacing(12)
    .flex_grow(1.0)
}
