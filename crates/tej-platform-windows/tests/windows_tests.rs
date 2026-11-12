use tej_hal::{PlatformWindow, UiEvent, WindowConfig};
use tej_layout::Size;
use tej_platform_windows::{
    DirectCompositionSurface, DwmAttributes, WindowBackdrop, WindowsWindow,
};

#[test]
fn test_windows_window_creation_and_dwm() {
    let config = WindowConfig {
        title: "Windows Native App".to_string(),
        size: Size::new(1024.0, 768.0),
        ..Default::default()
    };

    let mut win = WindowsWindow::new(config, DwmAttributes::mica_dark());
    assert_eq!(win.dwm_attributes().backdrop, WindowBackdrop::Mica);
    assert!(win.dwm_attributes().use_dark_mode);
    assert_eq!(win.size(), Size::new(1024.0, 768.0));
    assert_eq!(win.scale_factor(), 1.0);

    // Simulate DPI change to 150% (144 DPI)
    win.handle_dpi_changed(144, Size::new(1536.0, 1152.0));
    assert_eq!(win.scale_factor(), 1.5);
    assert_eq!(win.size(), Size::new(1536.0, 1152.0));

    let events = win.poll_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], UiEvent::ScaleFactorChanged { scale_factor: 1.5 });
    assert_eq!(
        events[1],
        UiEvent::Resize {
            width: 1536.0,
            height: 1152.0
        }
    );
}

#[test]
fn test_direct_composition_surface_clipping() {
    let mut surface = DirectCompositionSurface::new(Size::new(800.0, 600.0), 1.25);
    assert_eq!(surface.scale_factor(), 1.25);

    use tej_hal::{Color, SurfaceCompositor};
    use tej_layout::{Point, Rect};

    surface.begin_frame(&[Rect::from_origin_size(
        Point::new(0.0, 0.0),
        Size::new(100.0, 100.0),
    )]);

    // Draw inside dirty region
    surface.fill_rect(
        Rect::from_origin_size(Point::new(10.0, 10.0), Size::new(50.0, 50.0)),
        Color::rgb(255, 0, 0),
    );
    assert_eq!(surface.draw_calls(), 1);

    // Draw outside dirty region
    surface.fill_rect(
        Rect::from_origin_size(Point::new(300.0, 300.0), Size::new(50.0, 50.0)),
        Color::rgb(0, 255, 0),
    );
    assert_eq!(surface.draw_calls(), 1); // Cull non-dirty draw

    surface.end_frame();
}
