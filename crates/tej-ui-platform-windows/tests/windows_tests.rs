use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_platform_windows::{
    DirectCompositionSurface, NcHitCode, Win32ScrollPhysics, Win32SystemTheme, Win32ThemeMode,
    Win32WindowFrame,
};

#[test]
fn test_windows_non_client_hit_testing_and_snap_layouts() {
    let frame = Win32WindowFrame::new(1000.0, 700.0);

    // Resizing corners and borders
    assert_eq!(frame.hit_test(Point::new(2.0, 2.0)), NcHitCode::TopLeft);
    assert_eq!(frame.hit_test(Point::new(998.0, 2.0)), NcHitCode::TopRight);
    assert_eq!(frame.hit_test(Point::new(500.0, 2.0)), NcHitCode::Top);
    assert_eq!(frame.hit_test(Point::new(2.0, 350.0)), NcHitCode::Left);
    assert_eq!(frame.hit_test(Point::new(998.0, 350.0)), NcHitCode::Right);
    assert_eq!(frame.hit_test(Point::new(500.0, 698.0)), NcHitCode::Bottom);

    // Titlebar area
    assert_eq!(frame.hit_test(Point::new(100.0, 15.0)), NcHitCode::Caption);

    // Buttons
    assert_eq!(frame.hit_test(Point::new(980.0, 15.0)), NcHitCode::Close);
    assert_eq!(
        frame.hit_test(Point::new(930.0, 15.0)),
        NcHitCode::MaxButton
    );
    assert!(frame.is_snap_layout_hover(Point::new(930.0, 15.0)));
    assert_eq!(
        frame.hit_test(Point::new(880.0, 15.0)),
        NcHitCode::MinButton
    );

    // Client area
    assert_eq!(frame.hit_test(Point::new(200.0, 200.0)), NcHitCode::Client);
}

#[test]
fn test_windows_scroll_physics_and_precision_touchpad() {
    let mut scroll = Win32ScrollPhysics::new();
    assert!(scroll.is_at_rest());

    // Single detent wheel scroll (120 units)
    let delta = scroll.handle_wheel_message(120.0, false);
    assert_eq!(delta, 60.0); // 1.0 * 3 lines * 20px
    assert!(!scroll.is_at_rest());

    // Step physics
    let step1 = scroll.step(0.016);
    assert!(step1.y > 0.0);

    scroll.stop();
    assert!(scroll.is_at_rest());
}

#[test]
fn test_windows_theme_and_colorization() {
    let theme = Win32SystemTheme::windows_11_dark();
    assert_eq!(theme.mode, Win32ThemeMode::Dark);
    assert!(!theme.is_high_contrast());

    // Parse DWM ARGB colorization (e.g. 0xC40078D4)
    let custom = Win32SystemTheme::from_colorization_argb(0xC40078D4, true);
    assert_eq!(custom.accent_color, Color::rgb(0, 120, 212));
    assert_eq!(custom.titlebar_text_color, Color::rgb(255, 255, 255));
}

#[test]
fn test_windows_direct_composition_surface_rasterization() {
    let mut surface = DirectCompositionSurface::new(Size::new(100.0, 100.0), 1.0);
    assert_eq!(surface.draw_calls(), 0);

    let red = Color::rgb(255, 0, 0);
    surface.begin_frame(&[Rect::new(0.0, 0.0, 50.0, 50.0)]);
    surface.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), red);
    surface.end_frame();

    assert_eq!(surface.draw_calls(), 1);
    // BGRA red: 0xFFFF0000
    assert_eq!(surface.pixel_data()[0], 0xFFFF0000);
}

#[test]
fn test_windows_window_host_and_ime() {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use tej_ui_hal::{
        CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig,
    };
    use tej_ui_platform_windows::WindowsWindow;

    let config = WindowConfig {
        title: "Windows Test Window".to_string(),
        ..WindowConfig::default()
    };
    let mut window = WindowsWindow::from_config(config);

    assert_eq!(window.title(), "Windows Test Window");

    let win_handle = window.window_handle().expect("valid win32 window handle");
    assert!(matches!(win_handle.as_raw(), RawWindowHandle::Win32(_)));

    let disp_handle = window
        .display_handle()
        .expect("valid windows display handle");
    assert!(matches!(disp_handle.as_raw(), RawDisplayHandle::Windows(_)));

    window.set_cursor(CursorIcon::Crosshair);
    assert_eq!(window.cursor(), CursorIcon::Crosshair);

    window.set_ime_cursor_area(ImeCursorArea::new(10.0, 20.0, 5.0, 15.0));
    assert_eq!(window.ime_area(), ImeCursorArea::new(10.0, 20.0, 5.0, 15.0));

    window.handle_ime_composition(Some("preedit"), None, Some((0, 7)));
    window.handle_ime_composition(None, Some("commit"), None);

    let events = window.poll_events();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], UiEvent::ImePreedit { .. }));
    assert!(matches!(events[1], UiEvent::ImeCommit { .. }));
}
