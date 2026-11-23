use tej_ui_layout::Point;
use tej_ui_platform_linux::{
    AdwaitaHeaderBar, LibinputScrollPhysics, LinuxDesktopTheme, WindowDecorationMode,
    XdgColorScheme,
};

#[test]
fn test_linux_adwaita_csd_headerbar() {
    let mut header = AdwaitaHeaderBar::new(800.0);
    assert_eq!(header.mode(), WindowDecorationMode::ClientSide);
    assert_eq!(header.corner_radius(), 12.0);

    // Click in close button area (right side: 800 - 12 - 24 = 764 to 788)
    assert!(header.is_close_button_hit(Point::new(770.0, 20.0)));
    assert!(!header.is_close_button_hit(Point::new(400.0, 20.0)));

    // Drag area
    assert!(header.is_drag_area(Point::new(400.0, 20.0)));
    assert!(!header.is_drag_area(Point::new(770.0, 20.0)));

    header.set_mode(WindowDecorationMode::ServerSide);
    assert_eq!(header.mode(), WindowDecorationMode::ServerSide);
}

#[test]
fn test_linux_xdg_portal_theme_and_color_scheme() {
    assert_eq!(XdgColorScheme::from_u32(1), XdgColorScheme::PreferDark);
    assert_eq!(XdgColorScheme::from_u32(2), XdgColorScheme::PreferLight);
    assert_eq!(XdgColorScheme::from_u32(0), XdgColorScheme::NoPreference);

    let dark_theme = LinuxDesktopTheme::adwaita_dark();
    assert!(dark_theme.is_dark());

    let light_theme = LinuxDesktopTheme::adwaita_light();
    assert!(!light_theme.is_dark());
}

#[test]
fn test_linux_libinput_scroll_physics() {
    let mut scroll = LibinputScrollPhysics::new();
    assert!(scroll.is_at_rest());

    scroll.handle_axis_delta(Point::new(0.0, 5.0));
    assert!(!scroll.is_at_rest());

    let delta = scroll.step(0.016);
    assert!(delta.y > 0.0);

    scroll.stop();
    assert!(scroll.is_at_rest());
}

#[test]
fn test_linux_window_host_and_ime() {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use tej_ui_hal::{
        CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig,
    };
    use tej_ui_platform_linux::LinuxWindow;

    let config = WindowConfig {
        title: "Linux Test Window".to_string(),
        ..WindowConfig::default()
    };
    let mut window = LinuxWindow::new(config);

    assert_eq!(window.title(), "Linux Test Window");
    assert_eq!(window.scale_factor(), 1.0);

    // Handles check
    let win_handle = window.window_handle().expect("valid linux window handle");
    assert!(matches!(win_handle.as_raw(), RawWindowHandle::Xlib(_)));

    let disp_handle = window.display_handle().expect("valid linux display handle");
    assert!(matches!(disp_handle.as_raw(), RawDisplayHandle::Xlib(_)));

    // Cursor
    window.set_cursor(CursorIcon::Text);
    assert_eq!(window.cursor(), CursorIcon::Text);

    // IME
    window.set_ime_cursor_area(ImeCursorArea::new(50.0, 100.0, 10.0, 20.0));
    assert_eq!(
        window.ime_area(),
        ImeCursorArea::new(50.0, 100.0, 10.0, 20.0)
    );

    window.push_ime_preedit("hangul".to_string(), Some((0, 6)));
    window.push_ime_commit("한글".to_string());

    let events = window.poll_events();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], UiEvent::ImePreedit { .. }));
    assert!(matches!(events[1], UiEvent::ImeCommit { .. }));
}
