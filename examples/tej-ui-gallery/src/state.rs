//! Reactive gallery state management and telemetry tracking.

use tej_ui::prelude::*;

/// Active gallery showcase section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalleryTab {
    #[default]
    Widgets,
    CssPlayground,
    VectorCanvas,
    Viewport3D,
    A11yInspector,
}

impl GalleryTab {
    /// Human-readable tab label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Widgets => "210 Widgets Catalog",
            Self::CssPlayground => "231 CSS Properties",
            Self::VectorCanvas => "2D Vector & Canvas",
            Self::Viewport3D => "3D Hybrid Viewport",
            Self::A11yInspector => "Accessibility Inspector",
        }
    }
}

/// Available theme modes for the showcase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    HighContrast,
    Cyberpunk,
}

impl ThemeMode {
    /// Convert to `tej_ui_view::Theme`.
    #[must_use]
    pub fn to_theme(self) -> Theme {
        match self {
            Self::Dark => Theme::dark(),
            Self::Light => Theme::light(),
            Self::HighContrast => Theme::high_contrast(),
            Self::Cyberpunk => {
                let mut t = Theme::dark();
                t.colors.primary = Color::rgb(0, 255, 204);
                t.colors.accent = Color::rgb(255, 0, 128);
                t.colors.background = Color::rgb(10, 10, 20);
                t.colors.card = Color::rgb(20, 20, 35);
                t
            }
        }
    }
}

/// Real-time frame performance and memory metrics.
#[derive(Debug, Clone, Copy)]
pub struct TelemetryMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub instance_count: usize,
    pub dirty_rects: usize,
    pub steady_state_allocations: usize,
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self {
            fps: 120.0,
            frame_time_ms: 8.33,
            instance_count: 1420,
            dirty_rects: 1,
            steady_state_allocations: 0,
        }
    }
}

/// Dynamic CSS property values tweaked in the live playground.
#[derive(Debug, Clone)]
pub struct CssPlaygroundProps {
    pub width: f32,
    pub height: f32,
    pub padding: f32,
    pub margin: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub opacity: f32,
    pub gap: f32,
    pub is_flex_column: bool,
    pub show_shadow: bool,
}

impl Default for CssPlaygroundProps {
    fn default() -> Self {
        Self {
            width: 240.0,
            height: 140.0,
            padding: 16.0,
            margin: 8.0,
            border_radius: 12.0,
            font_size: 14.0,
            opacity: 1.0,
            gap: 12.0,
            is_flex_column: true,
            show_shadow: true,
        }
    }
}

/// Root reactive state bundle for the Tej Gallery.
#[derive(Debug, Clone, Copy)]
pub struct GalleryState {
    pub active_tab: Signal<GalleryTab>,
    pub is_widgets: Signal<bool>,
    pub is_css: Signal<bool>,
    pub is_vector: Signal<bool>,
    pub is_viewport: Signal<bool>,
    pub is_a11y: Signal<bool>,
    pub theme_mode: Signal<ThemeMode>,
    pub is_dark: Signal<bool>,
    pub is_light: Signal<bool>,
    pub is_contrast: Signal<bool>,
    pub is_cyberpunk: Signal<bool>,
    pub is_widget_btn: Signal<bool>,
    pub is_widget_form: Signal<bool>,
    pub is_widget_feed: Signal<bool>,
    pub is_widget_nav: Signal<bool>,
    pub is_widget_data: Signal<bool>,
    pub is_css_overview: Signal<bool>,
    pub is_css_box: Signal<bool>,
    pub is_css_flex: Signal<bool>,
    pub is_css_typo: Signal<bool>,
    pub is_css_borders: Signal<bool>,
    pub is_css_modern: Signal<bool>,
    pub is_css_playground: Signal<bool>,
    pub telemetry: Signal<TelemetryMetrics>,
}

impl GalleryState {
    /// Initialize gallery signals.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_tab: Signal::new(GalleryTab::Widgets),
            is_widgets: Signal::new(true),
            is_css: Signal::new(false),
            is_vector: Signal::new(false),
            is_viewport: Signal::new(false),
            is_a11y: Signal::new(false),
            theme_mode: Signal::new(ThemeMode::Dark),
            is_dark: Signal::new(true),
            is_light: Signal::new(false),
            is_contrast: Signal::new(false),
            is_cyberpunk: Signal::new(false),
            is_widget_btn: Signal::new(true),
            is_widget_form: Signal::new(false),
            is_widget_feed: Signal::new(false),
            is_widget_nav: Signal::new(false),
            is_widget_data: Signal::new(false),
            is_css_overview: Signal::new(true),
            is_css_box: Signal::new(false),
            is_css_flex: Signal::new(false),
            is_css_typo: Signal::new(false),
            is_css_borders: Signal::new(false),
            is_css_modern: Signal::new(false),
            is_css_playground: Signal::new(false),
            telemetry: Signal::new(TelemetryMetrics::default()),
        }
    }

    /// Select an active gallery tab and update all visibility signals.
    pub fn select_tab(&self, tab: GalleryTab) {
        self.active_tab.set(tab);
        self.is_widgets.set(tab == GalleryTab::Widgets);
        self.is_css.set(tab == GalleryTab::CssPlayground);
        self.is_vector.set(tab == GalleryTab::VectorCanvas);
        self.is_viewport.set(tab == GalleryTab::Viewport3D);
        self.is_a11y.set(tab == GalleryTab::A11yInspector);
    }

    /// Select an active widget catalog subcategory.
    pub fn select_widget_category(&self, cat: usize) {
        self.is_widget_btn.set(cat == 0);
        self.is_widget_form.set(cat == 1);
        self.is_widget_feed.set(cat == 2);
        self.is_widget_nav.set(cat == 3);
        self.is_widget_data.set(cat == 4);
    }

    /// Select an active CSS catalog sub-section.
    pub fn select_css_section(&self, sec: usize) {
        self.is_css_overview.set(sec == 0);
        self.is_css_box.set(sec == 1);
        self.is_css_flex.set(sec == 2);
        self.is_css_typo.set(sec == 3);
        self.is_css_borders.set(sec == 4);
        self.is_css_modern.set(sec == 5);
        self.is_css_playground.set(sec == 6);
    }

    /// Select an active theme mode, update theme signals, and inject into ambient context.
    pub fn select_theme(&self, mode: ThemeMode) {
        self.theme_mode.set(mode);
        self.is_dark.set(mode == ThemeMode::Dark);
        self.is_light.set(mode == ThemeMode::Light);
        self.is_contrast.set(mode == ThemeMode::HighContrast);
        self.is_cyberpunk.set(mode == ThemeMode::Cyberpunk);
        tej_ui_view::provide_theme(mode.to_theme());
    }
}

impl Default for GalleryState {
    fn default() -> Self {
        Self::new()
    }
}
