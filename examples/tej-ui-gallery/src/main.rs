//! `tej-gallery` main binary entrypoint.

use std::env;
use tej_ui::prelude::*;
use tej_ui_gallery::{GalleryState, build_gallery_app, run_headless_verification};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Automated CI / test mode
    if args
        .iter()
        .any(|arg| arg == "--test" || arg == "--headless")
    {
        println!("Running Tej Gallery in headless automated verification mode...");
        run_headless_verification()?;
        println!("Tej Gallery headless verification completed successfully!");
        return Ok(());
    }

    println!("Starting Tej Gallery Interactive Showcase...");
    let state = GalleryState::new();

    // Parse CLI arguments for initial tab, subtab, and theme
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tab" if i + 1 < args.len() => {
                match args[i + 1].as_str() {
                    "css" => state.select_tab(tej_ui_gallery::GalleryTab::CssPlayground),
                    "vector" => state.select_tab(tej_ui_gallery::GalleryTab::VectorCanvas),
                    "3d" => state.select_tab(tej_ui_gallery::GalleryTab::Viewport3D),
                    "a11y" => state.select_tab(tej_ui_gallery::GalleryTab::A11yInspector),
                    _ => state.select_tab(tej_ui_gallery::GalleryTab::Widgets),
                }
                i += 1;
            }
            "--widget-tab" if i + 1 < args.len() => {
                match args[i + 1].as_str() {
                    "forms" => state.select_widget_category(1),
                    "feedback" => state.select_widget_category(2),
                    "navigation" => state.select_widget_category(3),
                    "data" => state.select_widget_category(4),
                    _ => state.select_widget_category(0),
                }
                i += 1;
            }
            "--css-tab" if i + 1 < args.len() => {
                match args[i + 1].as_str() {
                    "box" => state.select_css_section(1),
                    "flex" => state.select_css_section(2),
                    "typo" => state.select_css_section(3),
                    "borders" => state.select_css_section(4),
                    "modern" => state.select_css_section(5),
                    "playground" => state.select_css_section(6),
                    _ => state.select_css_section(0),
                }
                i += 1;
            }
            "--theme" if i + 1 < args.len() => {
                match args[i + 1].as_str() {
                    "light" => state.select_theme(tej_ui_gallery::ThemeMode::Light),
                    "contrast" => state.select_theme(tej_ui_gallery::ThemeMode::HighContrast),
                    "cyberpunk" => state.select_theme(tej_ui_gallery::ThemeMode::Cyberpunk),
                    _ => state.select_theme(tej_ui_gallery::ThemeMode::Dark),
                }
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    let root = build_gallery_app(state);

    let app = App::new(root).window(WindowConfig {
        title: "Tej UI Production Showcase & Storybook".into(),
        size: Size::new(1280.0, 800.0),
        resizable: true,
        decorated: true,
        high_dpi: true,
        kind: tej_ui_hal::WindowKind::TopLevel,
        parent: None,
    });

    app.run()
}
