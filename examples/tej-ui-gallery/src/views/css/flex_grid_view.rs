//! Detailed property cards for Flexbox, CSS Grid & Backgrounds (44 properties).

use super::meta::CssPropertyMeta;
use super::props_flex_grid::PROPS_FLEX_GRID;
use tej_ui::prelude::*;

fn prop_card(p: &'static CssPropertyMeta) -> impl View {
    Card::new(format!("{}  ·  [{}]", p.name, p.syntax))
        .description(p.description)
        .flex_grow(1.0)
}

/// Detailed view displaying all 44 Flexbox, Grid, and Background properties with explanations.
pub fn flex_grid_view() -> impl View {
    VStack::new((
        // Section 1A: Flexbox Mechanics (Props 0..15)
        VStack::new((
            Text::new("Flexbox Mechanics & Alignments (15 Properties)"),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[0]),
                prop_card(&PROPS_FLEX_GRID[1]),
                prop_card(&PROPS_FLEX_GRID[2]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[3]),
                prop_card(&PROPS_FLEX_GRID[4]),
                prop_card(&PROPS_FLEX_GRID[5]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[6]),
                prop_card(&PROPS_FLEX_GRID[7]),
                prop_card(&PROPS_FLEX_GRID[8]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[9]),
                prop_card(&PROPS_FLEX_GRID[10]),
                prop_card(&PROPS_FLEX_GRID[11]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[12]),
                prop_card(&PROPS_FLEX_GRID[13]),
                prop_card(&PROPS_FLEX_GRID[14]),
            ))
            .spacing(12),
        ))
        .spacing(10),
        Separator::horizontal(),
        // Section 1B: CSS Grid Layout (Props 15..29)
        VStack::new((
            Text::new("CSS Grid Tracks, Areas & Auto-Flow (14 Properties)"),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[15]),
                prop_card(&PROPS_FLEX_GRID[16]),
                prop_card(&PROPS_FLEX_GRID[17]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[18]),
                prop_card(&PROPS_FLEX_GRID[19]),
                prop_card(&PROPS_FLEX_GRID[20]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[21]),
                prop_card(&PROPS_FLEX_GRID[22]),
                prop_card(&PROPS_FLEX_GRID[23]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[24]),
                prop_card(&PROPS_FLEX_GRID[25]),
                prop_card(&PROPS_FLEX_GRID[26]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[27]),
                prop_card(&PROPS_FLEX_GRID[28]),
                Card::new("Grid Track Auto-Flow")
                    .description(
                        "Implicit tracks allocated without memory overhead via flat SoA arena.",
                    )
                    .flex_grow(1.0),
            ))
            .spacing(12),
        ))
        .spacing(10),
        Separator::horizontal(),
        // Section 2: Backgrounds & Masks (Props 29..44)
        VStack::new((
            Text::new("Backgrounds, Tiling & Masking Engine (15 Properties)"),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[29]),
                prop_card(&PROPS_FLEX_GRID[30]),
                prop_card(&PROPS_FLEX_GRID[31]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[32]),
                prop_card(&PROPS_FLEX_GRID[33]),
                prop_card(&PROPS_FLEX_GRID[34]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[35]),
                prop_card(&PROPS_FLEX_GRID[36]),
                prop_card(&PROPS_FLEX_GRID[37]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[38]),
                prop_card(&PROPS_FLEX_GRID[39]),
                prop_card(&PROPS_FLEX_GRID[40]),
            ))
            .spacing(12),
            HStack::new((
                prop_card(&PROPS_FLEX_GRID[41]),
                prop_card(&PROPS_FLEX_GRID[42]),
                prop_card(&PROPS_FLEX_GRID[43]),
            ))
            .spacing(12),
        ))
        .spacing(10),
    ))
    .spacing(12)
    .flex_grow(1.0)
}
