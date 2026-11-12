use tej_hal::{PlatformWindow, UiEvent, WindowConfig};
use tej_layout::Size;
use tej_platform_macos::{
    MacosWindow, MacosWindowAttributes, MacosWindowStyle, MetalSurfaceCompositor,
};

#[test]
fn test_macos_window_creation_and_attributes() {
    let config = WindowConfig {
        title: "Linear for macOS".to_string(),
        size: Size::new(1280.0, 800.0),
        ..Default::default()
    };

    let mut win = MacosWindow::new(config, MacosWindowAttributes::modern_flat());
    assert_eq!(
        win.attributes().style,
        MacosWindowStyle::FullSizeContentView
    );
    assert!(win.attributes().titlebar_transparent);
    assert!(win.attributes().title_hidden);
    assert_eq!(win.scale_factor(), 2.0); // Retina 2x

    // Move to external standard-DPI display
    win.set_backing_scale_factor(1.0);
    assert_eq!(win.scale_factor(), 1.0);

    let events = win.poll_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], UiEvent::ScaleFactorChanged { scale_factor: 1.0 });
}

#[test]
fn test_metal_surface_compositor_culling() {
    let mut surface = MetalSurfaceCompositor::new(Size::new(1280.0, 800.0), 2.0);
    assert_eq!(surface.backing_scale_factor(), 2.0);

    use tej_hal::{Color, SurfaceCompositor};
    use tej_layout::{Point, Rect};

    surface.begin_frame(&[Rect::from_origin_size(
        Point::new(50.0, 50.0),
        Size::new(100.0, 100.0),
    )]);

    // Hit dirty rect
    surface.fill_rect(
        Rect::from_origin_size(Point::new(60.0, 60.0), Size::new(20.0, 20.0)),
        Color::rgb(0, 0, 255),
    );
    assert_eq!(surface.draw_calls(), 1);

    // Miss dirty rect
    surface.fill_rect(
        Rect::from_origin_size(Point::new(500.0, 500.0), Size::new(50.0, 50.0)),
        Color::rgb(255, 255, 0),
    );
    assert_eq!(surface.draw_calls(), 1);

    surface.end_frame();
}
