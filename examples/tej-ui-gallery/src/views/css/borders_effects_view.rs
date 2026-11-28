//! Detailed property cards for Borders, Shadows, Filters & Interaction (55 properties).

use super::meta::CssPropertyMeta;
use super::props_borders_effects::PROPS_BORDERS_EFFECTS;
use tej_ui::prelude::*;

fn prop_card(p: &'static CssPropertyMeta) -> impl View {
    Card::new(format!("{}  ·  [{}]", p.name, p.syntax))
        .description(p.description)
        .flex_grow(1.0)
}

/// Detailed view displaying all 55 Border, Shadow, Filter, and Interaction properties with explanations.
pub fn borders_effects_view() -> impl View {
    VStack::new((
        borders_radii_section(),
        Separator::horizontal(),
        shadows_filters_section(),
        Separator::horizontal(),
        overflow_scrolling_section(),
        Separator::horizontal(),
        interaction_hit_testing_section(),
    ))
    .spacing(12)
    .flex_grow(1.0)
}

fn borders_radii_section() -> impl View {
    VStack::new((
        Text::new("Borders, Corner Radii & Outlines (18 Properties)"),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[0]),
            prop_card(&PROPS_BORDERS_EFFECTS[1]),
            prop_card(&PROPS_BORDERS_EFFECTS[2]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[3]),
            prop_card(&PROPS_BORDERS_EFFECTS[4]),
            prop_card(&PROPS_BORDERS_EFFECTS[5]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[6]),
            prop_card(&PROPS_BORDERS_EFFECTS[7]),
            prop_card(&PROPS_BORDERS_EFFECTS[8]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[9]),
            prop_card(&PROPS_BORDERS_EFFECTS[10]),
            prop_card(&PROPS_BORDERS_EFFECTS[11]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[12]),
            prop_card(&PROPS_BORDERS_EFFECTS[13]),
            prop_card(&PROPS_BORDERS_EFFECTS[14]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[15]),
            prop_card(&PROPS_BORDERS_EFFECTS[16]),
            prop_card(&PROPS_BORDERS_EFFECTS[17]),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn shadows_filters_section() -> impl View {
    VStack::new((
        Text::new("Elevation Shadows, GPU Filters & Blend Modes (15 Properties)"),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[18]),
            prop_card(&PROPS_BORDERS_EFFECTS[19]),
            prop_card(&PROPS_BORDERS_EFFECTS[20]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[21]),
            prop_card(&PROPS_BORDERS_EFFECTS[22]),
            prop_card(&PROPS_BORDERS_EFFECTS[23]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[24]),
            prop_card(&PROPS_BORDERS_EFFECTS[25]),
            prop_card(&PROPS_BORDERS_EFFECTS[26]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[27]),
            prop_card(&PROPS_BORDERS_EFFECTS[28]),
            prop_card(&PROPS_BORDERS_EFFECTS[29]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[30]),
            prop_card(&PROPS_BORDERS_EFFECTS[31]),
            prop_card(&PROPS_BORDERS_EFFECTS[32]),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn overflow_scrolling_section() -> impl View {
    VStack::new((
        Text::new("Overflow Clipping, Snap Points & Scrollbars (11 Properties)"),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[33]),
            prop_card(&PROPS_BORDERS_EFFECTS[34]),
            prop_card(&PROPS_BORDERS_EFFECTS[35]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[36]),
            prop_card(&PROPS_BORDERS_EFFECTS[37]),
            prop_card(&PROPS_BORDERS_EFFECTS[38]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[39]),
            prop_card(&PROPS_BORDERS_EFFECTS[40]),
            prop_card(&PROPS_BORDERS_EFFECTS[41]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[42]),
            prop_card(&PROPS_BORDERS_EFFECTS[43]),
            Card::new("Kinetic Physics Engine")
                .description("Inertial scrolling and magnetic snap point alignment.")
                .flex_grow(1.0),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn interaction_hit_testing_section() -> impl View {
    VStack::new((
        Text::new("Hit-Testing, Gestures & Platform Appearance (11 Properties)"),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[44]),
            prop_card(&PROPS_BORDERS_EFFECTS[45]),
            prop_card(&PROPS_BORDERS_EFFECTS[46]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[47]),
            prop_card(&PROPS_BORDERS_EFFECTS[48]),
            prop_card(&PROPS_BORDERS_EFFECTS[49]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[50]),
            prop_card(&PROPS_BORDERS_EFFECTS[51]),
            prop_card(&PROPS_BORDERS_EFFECTS[52]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BORDERS_EFFECTS[53]),
            prop_card(&PROPS_BORDERS_EFFECTS[54]),
            Card::new("Native Hit-Testing Invariant")
                .description(
                    "Pointer events, touch gestures, and focus rings mapped to native OS windows.",
                )
                .flex_grow(1.0),
        ))
        .spacing(12),
    ))
    .spacing(10)
}
