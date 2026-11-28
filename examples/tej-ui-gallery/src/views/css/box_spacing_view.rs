//! Detailed property cards for Box Model, Sizing, and Spacing (45 properties).

use super::meta::CssPropertyMeta;
use super::props_box_spacing::PROPS_BOX_SPACING;
use tej_ui::prelude::*;

fn prop_card(p: &'static CssPropertyMeta) -> impl View {
    Card::new(format!("{}  ·  [{}]", p.name, p.syntax))
        .description(p.description)
        .flex_grow(1.0)
}

/// Detailed view displaying all 45 Box Model, Sizing, Spacing & Inset properties with explanations.
pub fn box_spacing_view() -> impl View {
    VStack::new((
        box_sizing_section(),
        Separator::horizontal(),
        margins_paddings_section(),
        Separator::horizontal(),
        position_insets_section(),
    ))
    .spacing(12)
    .flex_grow(1.0)
}

fn box_sizing_section() -> impl View {
    VStack::new((
        Text::new("Box Sizing & Logical Dimensions (14 Properties)"),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[0]),
            prop_card(&PROPS_BOX_SPACING[1]),
            prop_card(&PROPS_BOX_SPACING[2]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[3]),
            prop_card(&PROPS_BOX_SPACING[4]),
            prop_card(&PROPS_BOX_SPACING[5]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[6]),
            prop_card(&PROPS_BOX_SPACING[7]),
            prop_card(&PROPS_BOX_SPACING[8]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[9]),
            prop_card(&PROPS_BOX_SPACING[10]),
            prop_card(&PROPS_BOX_SPACING[11]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[12]),
            prop_card(&PROPS_BOX_SPACING[13]),
            Card::new("Box Sizing Invariant")
                .description(
                    "All width and height calculations respect zero-copy points or percent constraints.",
                )
                .flex_grow(1.0),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn margins_paddings_section() -> impl View {
    VStack::new((
        Text::new("Physical & Logical Margins and Paddings (16 Properties)"),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[14]),
            prop_card(&PROPS_BOX_SPACING[15]),
            prop_card(&PROPS_BOX_SPACING[16]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[17]),
            prop_card(&PROPS_BOX_SPACING[18]),
            prop_card(&PROPS_BOX_SPACING[19]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[20]),
            prop_card(&PROPS_BOX_SPACING[21]),
            prop_card(&PROPS_BOX_SPACING[22]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[23]),
            prop_card(&PROPS_BOX_SPACING[24]),
            prop_card(&PROPS_BOX_SPACING[25]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[26]),
            prop_card(&PROPS_BOX_SPACING[27]),
            prop_card(&PROPS_BOX_SPACING[28]),
        ))
        .spacing(12),
    ))
    .spacing(10)
}

fn position_insets_section() -> impl View {
    VStack::new((
        Text::new("Positioning Schemes, Insets & Z-Index (15 Properties)"),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[29]),
            prop_card(&PROPS_BOX_SPACING[30]),
            prop_card(&PROPS_BOX_SPACING[31]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[32]),
            prop_card(&PROPS_BOX_SPACING[33]),
            prop_card(&PROPS_BOX_SPACING[34]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[35]),
            prop_card(&PROPS_BOX_SPACING[36]),
            prop_card(&PROPS_BOX_SPACING[37]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[38]),
            prop_card(&PROPS_BOX_SPACING[39]),
            prop_card(&PROPS_BOX_SPACING[40]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[41]),
            prop_card(&PROPS_BOX_SPACING[42]),
            prop_card(&PROPS_BOX_SPACING[43]),
        ))
        .spacing(12),
        HStack::new((
            prop_card(&PROPS_BOX_SPACING[44]),
            Card::new("Logical Insets Invariant")
                .description(
                    "Supports full internationalization with writing-mode-aware inset resolution.",
                )
                .flex_grow(1.0),
            Card::new("Z-Index Stacking Invariant")
                .description(
                    "GPU compositor maintains strict depth layering matching z-index declarations.",
                )
                .flex_grow(1.0),
        ))
        .spacing(12),
    ))
    .spacing(10)
}
