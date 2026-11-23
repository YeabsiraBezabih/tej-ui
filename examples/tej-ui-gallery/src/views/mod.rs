//! View modules for the Tej Gallery showcase suite.

pub mod a11y_inspector;
pub mod css;
pub mod hud;
pub mod sidebar;
pub mod vector_demo;
pub mod viewport_3d;
pub mod widgets;

pub use a11y_inspector::a11y_inspector_view;
pub use css::css_playground_view;
pub use hud::telemetry_hud_view;
pub use sidebar::gallery_sidebar_view;
pub use vector_demo::vector_demo_view;
pub use viewport_3d::viewport_3d_view;
pub use widgets::widgets_catalog_view;

use crate::state::GalleryState;
use tej_ui::prelude::*;

/// Root gallery view mounting the navigation sidebar, performance HUD, and dynamic category content.
pub fn gallery_main_view(state: GalleryState) -> impl View {
    VStack::new((
        // Top HUD overlay
        telemetry_hud_view(state.telemetry),
        Separator::horizontal(),
        // Main split layout: Sidebar + 1px hairline divider + Responsive Active Tab Content
        HStack::new((
            gallery_sidebar_view(state),
            Separator::vertical(),
            VStack::new(If::new(
                state.is_widgets,
                widgets_catalog_view(state),
                If::new(
                    state.is_css,
                    css_playground_view(state),
                    If::new(
                        state.is_vector,
                        vector_demo_view(),
                        If::new(state.is_viewport, viewport_3d_view(), a11y_inspector_view()),
                    ),
                ),
            ))
            .flex_grow(1.0),
        ))
        .spacing(16)
        .flex_grow(1.0),
    ))
    .padding(16)
    .spacing(12)
}
