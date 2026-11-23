//! Modular 210-component suite catalog view.

pub mod buttons_badges;
pub mod data_display;
pub mod feedback_status;
pub mod forms_inputs;
pub mod navigation_layout;

pub use buttons_badges::buttons_badges_view;
pub use data_display::data_display_view;
pub use feedback_status::feedback_status_view;
pub use forms_inputs::forms_inputs_view;
pub use navigation_layout::navigation_layout_view;

use crate::state::GalleryState;
use tej_ui::prelude::*;

fn category_button(
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

/// Root widget catalog storybook view mounting all component suites.
pub fn widgets_catalog_view(state: GalleryState) -> impl View {
    VStack::new((
        // Catalog Title & Parity Badges
        VStack::new((
            HStack::new((
                Text::new("210 Component Suite Catalog"),
                Badge::new("TIER 1 PARITY").variant(BadgeVariant::Success),
                Badge::new("210 PRIMITIVES READY").variant(BadgeVariant::Outline),
            ))
            .spacing(12),
            Text::new(
                "Zero-copy, authentic native OS controls with AccessKit semantic accessibility metadata.",
            ),
        ))
        .spacing(4),
        Separator::horizontal(),
        // Sub-category Navigation Filter Tabs
        HStack::new((
            category_button(state.is_widget_btn, "Buttons & Badges", move || {
                state.select_widget_category(0);
            }),
            category_button(state.is_widget_form, "Forms & Inputs", move || {
                state.select_widget_category(1);
            }),
            category_button(state.is_widget_feed, "Feedback & Status", move || {
                state.select_widget_category(2);
            }),
            category_button(state.is_widget_nav, "Navigation & Layout", move || {
                state.select_widget_category(3);
            }),
            category_button(state.is_widget_data, "DataGrid & Tables", move || {
                state.select_widget_category(4);
            }),
        ))
        .spacing(8),
        Separator::horizontal(),
        // Dynamic Active Category View
        If::new(
            state.is_widget_btn,
            buttons_badges_view(),
            If::new(
                state.is_widget_form,
                forms_inputs_view(),
                If::new(
                    state.is_widget_feed,
                    feedback_status_view(),
                    If::new(
                        state.is_widget_nav,
                        navigation_layout_view(),
                        data_display_view(),
                    ),
                ),
            ),
        ),
    ))
    .spacing(14)
    .flex_grow(1.0)
}
