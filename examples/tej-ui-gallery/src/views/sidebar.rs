//! Navigation sidebar and theme selector for the Tej Gallery.

use crate::state::{GalleryState, GalleryTab, ThemeMode};
use tej_ui::prelude::*;

fn nav_button(
    cond: Signal<bool>,
    label: &'static str,
    on_click: impl Fn() + Send + Sync + 'static + Copy,
) -> impl View {
    If::new(
        cond,
        Button::new(Text::new(label))
            .variant(ButtonVariant::Default)
            .width(Dimension::Points(180.0))
            .on_click(on_click),
        Button::new(Text::new(label))
            .variant(ButtonVariant::Ghost)
            .width(Dimension::Points(180.0))
            .on_click(on_click),
    )
}

fn theme_button(
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

/// Sidebar navigation component for selecting gallery categories and themes.
pub fn gallery_sidebar_view(state: GalleryState) -> impl View {
    VStack::new((
        // Header
        VStack::new((
            Text::new("TEJ UI SHOWCASE"),
            Text::new("Production Tier-1 Parity Suite"),
        )),
        Separator::horizontal(),
        // Navigation Buttons with active highlight & click events
        VStack::new((
            nav_button(state.is_widgets, GalleryTab::Widgets.label(), move || {
                state.select_tab(GalleryTab::Widgets);
            }),
            nav_button(state.is_css, GalleryTab::CssPlayground.label(), move || {
                state.select_tab(GalleryTab::CssPlayground);
            }),
            nav_button(
                state.is_vector,
                GalleryTab::VectorCanvas.label(),
                move || {
                    state.select_tab(GalleryTab::VectorCanvas);
                },
            ),
            nav_button(
                state.is_viewport,
                GalleryTab::Viewport3D.label(),
                move || {
                    state.select_tab(GalleryTab::Viewport3D);
                },
            ),
            nav_button(
                state.is_a11y,
                GalleryTab::A11yInspector.label(),
                move || {
                    state.select_tab(GalleryTab::A11yInspector);
                },
            ),
        ))
        .spacing(6),
        Separator::horizontal(),
        // Theme Switcher Section
        VStack::new((
            Text::new("ACTIVE THEME"),
            HStack::new((
                theme_button(state.is_dark, "Dark", move || {
                    state.select_theme(ThemeMode::Dark);
                }),
                theme_button(state.is_light, "Light", move || {
                    state.select_theme(ThemeMode::Light);
                }),
            ))
            .spacing(4),
            HStack::new((
                theme_button(state.is_contrast, "High Contrast", move || {
                    state.select_theme(ThemeMode::HighContrast);
                }),
                theme_button(state.is_cyberpunk, "Cyberpunk", move || {
                    state.select_theme(ThemeMode::Cyberpunk);
                }),
            ))
            .spacing(4),
        ))
        .spacing(8),
    ))
    .width(Dimension::Points(200.0))
    .spacing(16)
}
