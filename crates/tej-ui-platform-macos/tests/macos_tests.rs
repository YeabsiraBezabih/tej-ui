use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_platform_macos::{
    AppleAccentColor, HoveredTrafficLight, MacAppearance, MacScrollPhysics, MacosAppearanceManager,
    MetalSurfaceCompositor, TrafficLightsController,
};

#[test]
fn test_macos_traffic_lights_and_hover() {
    let mut ctrl = TrafficLightsController::new();

    // Standard button colors
    let (close, min, zoom) = ctrl.button_colors();
    assert_eq!(close, Color::rgb(255, 95, 86)); // Red
    assert_eq!(min, Color::rgb(255, 189, 46)); // Yellow
    assert_eq!(zoom, Color::rgb(39, 201, 63)); // Green

    // Hover Close button (left 20, top 14, size 14)
    assert_eq!(
        ctrl.update_hover(Point::new(25.0, 20.0)),
        HoveredTrafficLight::Close
    );

    // Hover Minimize button (20 + 14 + 8 = 42)
    assert_eq!(
        ctrl.update_hover(Point::new(45.0, 20.0)),
        HoveredTrafficLight::Minimize
    );

    // Inactive window turns buttons gray
    ctrl.set_active_window(false);
    let (gray, _, _) = ctrl.button_colors();
    assert_eq!(gray, Color::rgb(70, 70, 70));
}

#[test]
fn test_macos_elastic_rubber_band_scroll_physics() {
    let mut scroll = MacScrollPhysics::new(1000.0, 400.0);
    assert!(scroll.is_at_rest());

    // Pull downward past top (overscroll)
    scroll.begin_gesture();
    scroll.handle_trackpad_delta(-50.0);
    assert!(scroll.offset().y < 0.0);
    // Rubber-band resistance factor (0.55 * -50 = -27.5)
    assert_eq!(scroll.offset().y, -27.5);

    // Release gesture: restorative spring pulls back to 0.0
    scroll.end_gesture();
    assert!(!scroll.is_at_rest());

    for _ in 0..60 {
        scroll.step(0.016);
    }

    assert_eq!(scroll.offset().y, 0.0);
    assert!(scroll.is_at_rest());
}

#[test]
fn test_macos_system_appearance_and_accent_colors() {
    let mgr = MacosAppearanceManager::new(MacAppearance::DarkAqua, AppleAccentColor::Purple);
    assert!(mgr.is_dark());
    assert!(!mgr.is_high_contrast());
    assert_eq!(mgr.accent.color(), Color::rgb(175, 82, 222));
    assert_eq!(mgr.window_background(), Color::rgb(30, 30, 30));

    let high_contrast = MacosAppearanceManager::new(
        MacAppearance::AccessibilityHighContrastAqua,
        AppleAccentColor::Blue,
    );
    assert!(high_contrast.is_high_contrast());
    assert!(!high_contrast.is_dark());
}

#[test]
fn test_macos_metal_surface_retina_rasterization() {
    let mut surface = MetalSurfaceCompositor::new(Size::new(100.0, 100.0), 2.0);
    assert_eq!(surface.backing_scale_factor(), 2.0);

    let blue = Color::rgb(0, 122, 255);
    surface.begin_frame(&[Rect::new(0.0, 0.0, 50.0, 50.0)]);
    surface.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), blue);
    surface.end_frame();

    assert_eq!(surface.draw_calls(), 1);
    // RGBA blue: 0xFF007AFF
    let pixel = surface.pixel_data()[0];
    assert_eq!((pixel >> 24) & 0xFF, 0xFF); // Alpha = 255
    assert_eq!(pixel & 0xFF, 0); // Red = 0
}

#[test]
fn test_macos_window_host_and_ime() {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use tej_ui_hal::{
        CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig,
    };
    use tej_ui_platform_macos::MacosWindow;

    let config = WindowConfig {
        title: "macOS Test Window".to_string(),
        ..WindowConfig::default()
    };
    let mut window = MacosWindow::from_config(config);

    assert_eq!(window.title(), "macOS Test Window");
    assert_eq!(window.scale_factor(), 2.0);

    let win_handle = window.window_handle().expect("valid appkit window handle");
    assert!(matches!(win_handle.as_raw(), RawWindowHandle::AppKit(_)));

    let disp_handle = window
        .display_handle()
        .expect("valid appkit display handle");
    assert!(matches!(disp_handle.as_raw(), RawDisplayHandle::AppKit(_)));

    window.set_cursor(CursorIcon::Grab);
    assert_eq!(window.cursor(), CursorIcon::Grab);

    window.set_ime_cursor_area(ImeCursorArea::new(30.0, 40.0, 15.0, 25.0));
    assert_eq!(
        window.ime_area(),
        ImeCursorArea::new(30.0, 40.0, 15.0, 25.0)
    );

    window.handle_marked_text("marked", Some((0, 6)));
    window.handle_insert_text("inserted");

    let events = window.poll_events();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], UiEvent::ImePreedit { .. }));
    assert!(matches!(events[1], UiEvent::ImeCommit { .. }));
}
