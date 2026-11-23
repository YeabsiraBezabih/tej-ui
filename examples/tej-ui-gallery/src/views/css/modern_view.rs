//! Detailed property cards for Modern Modules, Anchor Positioning & SVG (29 properties).

use super::meta::CssPropertyMeta;
use super::props_modern::PROPS_MODERN;
use tej_ui::prelude::*;

fn prop_card(p: &'static CssPropertyMeta) -> impl View {
    Card::new(format!("{}  ·  [{}]", p.name, p.syntax))
        .description(p.description)
        .flex_grow(1.0)
}

/// Detailed view displaying all 29 Modern CSS, Anchor Positioning & SVG properties with explanations.
pub fn modern_view() -> impl View {
    VStack::new((
        // Section 1: Modern Modules & Anchors (Props 0..15)
        VStack::new((
            Text::new("Anchor Positioning & Container Queries (15 Properties)"),
            HStack::new((
                prop_card(&PROPS_MODERN[0]),
                prop_card(&PROPS_MODERN[1]),
                prop_card(&PROPS_MODERN[2]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[3]),
                prop_card(&PROPS_MODERN[4]),
                prop_card(&PROPS_MODERN[5]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[6]),
                prop_card(&PROPS_MODERN[7]),
                prop_card(&PROPS_MODERN[8]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[9]),
                prop_card(&PROPS_MODERN[10]),
                prop_card(&PROPS_MODERN[11]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[12]),
                prop_card(&PROPS_MODERN[13]),
                prop_card(&PROPS_MODERN[14]),
            ))
            .spacing(12),
        ))
        .spacing(10),
        Separator::horizontal(),
        // Section 2: SVG Vector Geometry & Tables (Props 15..29)
        VStack::new((
            Text::new("SVG Vector Strokes, Tables & Multi-Column (14 Properties)"),
            HStack::new((
                prop_card(&PROPS_MODERN[15]),
                prop_card(&PROPS_MODERN[16]),
                prop_card(&PROPS_MODERN[17]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[18]),
                prop_card(&PROPS_MODERN[19]),
                prop_card(&PROPS_MODERN[20]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[21]),
                prop_card(&PROPS_MODERN[22]),
                prop_card(&PROPS_MODERN[23]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[24]),
                prop_card(&PROPS_MODERN[25]),
                prop_card(&PROPS_MODERN[26]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_MODERN[27]),
                prop_card(&PROPS_MODERN[28]),
                Card::new("Modern CSS Parity Guarantee")
                    .description("Every modern CSS specification is compiled ahead-of-time with zero runtime string parsing.")
                    .flex_grow(1.0),
            ))
            .spacing(12),
        ))
        .spacing(10),
    ))
    .spacing(12)
    .flex_grow(1.0)
}
