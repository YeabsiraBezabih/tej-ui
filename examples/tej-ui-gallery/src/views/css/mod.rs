//! Comprehensive 231 CSS Properties catalog and interactive playground.

pub mod borders_effects_view;
pub mod box_spacing_view;
pub mod flex_grid_view;
pub mod meta;
pub mod modern_view;
pub mod overview_view;
pub mod playground_view;
pub mod props_borders_effects;
pub mod props_box_spacing;
pub mod props_flex_grid;
pub mod props_modern;
pub mod props_typography;
pub mod typography_view;

use crate::state::GalleryState;
use borders_effects_view::borders_effects_view;
use box_spacing_view::box_spacing_view;
use flex_grid_view::flex_grid_view;
use meta::TOTAL_CSS_PROPERTIES;
use modern_view::modern_view;
use overview_view::css_overview_view;
use playground_view::css_interactive_playground_view;
use tej_ui::prelude::*;
use typography_view::typography_view;

fn tab_btn(
    cond: Signal<bool>,
    label: &'static str,
    on_click: impl Fn() + Send + Sync + 'static + Copy,
) -> impl View {
    If::new(
        cond,
        Button::new(Text::new(label))
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Default)
            .on_click(on_click),
        Button::new(Text::new(label))
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Secondary)
            .on_click(on_click),
    )
}

/// Interactive showcase demonstrating all 231 CSS properties supported by `tej-ui`.
pub fn css_playground_view(state: GalleryState) -> impl View {
    VStack::new((
        // Header
        VStack::new((
            HStack::new((
                Text::new("231 CSS Properties Architecture Engine"),
                Badge::new(format!("{TOTAL_CSS_PROPERTIES} PROPERTIES COMPILED"))
                    .variant(BadgeVariant::Success),
                Badge::new("ZERO-COPY BOX MODEL").variant(BadgeVariant::Outline),
            ))
            .spacing(12),
            Text::new(
                "Complete strongly-typed CSS specification with Box Model, Flexbox, Grid, Typography, SDF Shadows, and GPU Transforms.",
            ),
        ))
        .spacing(4),
        Separator::horizontal(),
        // Navigation Tabs across CSS Categories
        HStack::new((
            tab_btn(state.is_css_overview, "Overview (231)", move || {
                state.select_css_section(0);
            }),
            tab_btn(state.is_css_box, "Box & Spacing (45)", move || {
                state.select_css_section(1);
            }),
            tab_btn(state.is_css_flex, "Flexbox & Grid (44)", move || {
                state.select_css_section(2);
            }),
            tab_btn(state.is_css_typo, "Typography (58)", move || {
                state.select_css_section(3);
            }),
            tab_btn(state.is_css_borders, "Borders & Effects (55)", move || {
                state.select_css_section(4);
            }),
            tab_btn(state.is_css_modern, "Modern & SVG (29)", move || {
                state.select_css_section(5);
            }),
            tab_btn(state.is_css_playground, "Interactive Playground", move || {
                state.select_css_section(6);
            }),
        ))
        .spacing(8),
        Separator::horizontal(),
        // Dynamic Active Section View
        If::new(
            state.is_css_overview,
            css_overview_view(),
            If::new(
                state.is_css_box,
                box_spacing_view(),
                If::new(
                    state.is_css_flex,
                    flex_grid_view(),
                    If::new(
                        state.is_css_typo,
                        typography_view(),
                        If::new(
                            state.is_css_borders,
                            borders_effects_view(),
                            If::new(
                                state.is_css_modern,
                                modern_view(),
                                css_interactive_playground_view(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ))
    .spacing(14)
    .flex_grow(1.0)
}
