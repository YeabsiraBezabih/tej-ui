use tej_hal::{Color, PlatformWindow, SurfaceCompositor, UiEvent, WindowConfig};
use tej_layout::{Point, Rect, Size};
use tej_platform_linux::{LinuxSurface, LinuxWindow};

#[test]
fn test_linux_window_creation_and_events() {
    let mut win = LinuxWindow::new(WindowConfig {
        title: "Test Linux Window".into(),
        size: Size::new(800.0, 600.0),
        ..Default::default()
    });

    assert_eq!(win.size().width, 800.0);
    assert_eq!(win.size().height, 600.0);

    win.push_event(UiEvent::MouseMove {
        position: Point::new(50.0, 50.0),
    });

    let events = win.poll_events();
    assert_eq!(events.len(), 1);
    assert!(win.poll_events().is_empty());
}

#[test]
fn test_linux_surface_dirty_clipping() {
    let mut surface = LinuxSurface::new(Size::new(800.0, 600.0));
    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    surface.begin_frame(&[dirty_rect]);

    // Inside dirty rect -> executed
    surface.fill_rect(Rect::new(10.0, 10.0, 20.0, 20.0), Color::BLACK);
    assert_eq!(surface.draw_calls(), 1);

    // Completely outside dirty rect -> skipped
    surface.fill_rect(Rect::new(400.0, 400.0, 50.0, 50.0), Color::BLACK);
    assert_eq!(surface.draw_calls(), 1);

    surface.end_frame();
}

#[test]
fn test_linux_surface_draw_text() {
    let mut surface = LinuxSurface::new(Size::new(200.0, 100.0));
    surface.clear(Color::BLACK);
    surface.begin_frame(&[]);
    surface.draw_text("tej-ui", Point::new(10.0, 10.0), 16.0, Color::WHITE);
    surface.end_frame();

    assert_eq!(surface.draw_calls(), 1);
    // Antialiased text rasterization must draw visible illuminated glyph pixels
    assert!(surface.pixels().iter().any(|&p| (p & 0x00FF_FFFF) > 0));
}
