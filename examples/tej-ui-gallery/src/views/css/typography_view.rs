//! Detailed property cards for Typography & Motion Transforms (58 properties).

use super::meta::CssPropertyMeta;
use super::props_typography::PROPS_TYPOGRAPHY;
use tej_ui::prelude::*;

fn prop_card(p: &'static CssPropertyMeta) -> impl View {
    Card::new(format!("{}  ·  [{}]", p.name, p.syntax))
        .description(p.description)
        .flex_grow(1.0)
}

/// Detailed view displaying all 58 Typography and Motion properties with explanations.
pub fn typography_view() -> impl View {
    VStack::new((
        core_typography_section(),
        Separator::horizontal(),
        opentype_i18n_section(),
        Separator::horizontal(),
        transitions_transforms_section(),
        Separator::horizontal(),
        animations_projections_section(),
    ))
    .spacing(12)
    .flex_grow(1.0)
}

fn core_typography_section() -> impl View {
    VStack::new((
        Text::new("Core Typography, Metrics & Line Height (18 Properties)"),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[0]),
            prop_card(&PROPS_TYPOGRAPHY[1]),
            prop_card(&PROPS_TYPOGRAPHY[2]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[3]),
            prop_card(&PROPS_TYPOGRAPHY[4]),
            prop_card(&PROPS_TYPOGRAPHY[5]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[6]),
            prop_card(&PROPS_TYPOGRAPHY[7]),
            prop_card(&PROPS_TYPOGRAPHY[8]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[9]),
            prop_card(&PROPS_TYPOGRAPHY[10]),
            prop_card(&PROPS_TYPOGRAPHY[11]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[12]),
            prop_card(&PROPS_TYPOGRAPHY[13]),
            prop_card(&PROPS_TYPOGRAPHY[14]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[15]),
            prop_card(&PROPS_TYPOGRAPHY[16]),
            prop_card(&PROPS_TYPOGRAPHY[17]),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn opentype_i18n_section() -> impl View {
    VStack::new((
        Text::new("OpenType, BiDi Direction & Modern Wrapping (16 Properties)"),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[18]),
            prop_card(&PROPS_TYPOGRAPHY[19]),
            prop_card(&PROPS_TYPOGRAPHY[20]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[21]),
            prop_card(&PROPS_TYPOGRAPHY[22]),
            prop_card(&PROPS_TYPOGRAPHY[23]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[24]),
            prop_card(&PROPS_TYPOGRAPHY[25]),
            prop_card(&PROPS_TYPOGRAPHY[26]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[27]),
            prop_card(&PROPS_TYPOGRAPHY[28]),
            prop_card(&PROPS_TYPOGRAPHY[29]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[30]),
            prop_card(&PROPS_TYPOGRAPHY[31]),
            prop_card(&PROPS_TYPOGRAPHY[32]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[33]),
            Card::new("Unicode BiDi Invariant")
                .description("Hardware rasterizer resolves complex RTL ligature clusters and contextual forms.")
                .flex_grow(1.0),
            Card::new("Variable Font Optical Sizing")
                .description("Automatic adjustments along weight, width, and optical size axes.")
                .flex_grow(1.0),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn transitions_transforms_section() -> impl View {
    VStack::new((
        Text::new("Transitions & GPU Affine Transforms (12 Properties)"),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[34]),
            prop_card(&PROPS_TYPOGRAPHY[35]),
            prop_card(&PROPS_TYPOGRAPHY[36]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[37]),
            prop_card(&PROPS_TYPOGRAPHY[38]),
            prop_card(&PROPS_TYPOGRAPHY[39]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[40]),
            prop_card(&PROPS_TYPOGRAPHY[41]),
            prop_card(&PROPS_TYPOGRAPHY[42]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[43]),
            prop_card(&PROPS_TYPOGRAPHY[44]),
            prop_card(&PROPS_TYPOGRAPHY[45]),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn animations_projections_section() -> impl View {
    VStack::new((
        Text::new("Keyframe Animations & 3D Projections (12 Properties)"),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[46]),
            prop_card(&PROPS_TYPOGRAPHY[47]),
            prop_card(&PROPS_TYPOGRAPHY[48]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[49]),
            prop_card(&PROPS_TYPOGRAPHY[50]),
            prop_card(&PROPS_TYPOGRAPHY[51]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[52]),
            prop_card(&PROPS_TYPOGRAPHY[53]),
            prop_card(&PROPS_TYPOGRAPHY[54]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_TYPOGRAPHY[55]),
            prop_card(&PROPS_TYPOGRAPHY[56]),
            prop_card(&PROPS_TYPOGRAPHY[57]),
        ))
        .spacing(12),
    ))
    .spacing(10)
}
