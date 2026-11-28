use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_platform_linux::{
    AdwaitaHeaderBar, LibinputScrollPhysics, LinuxDesktopTheme, LinuxSurface, LinuxWindow,
    WindowDecorationMode, XdgColorScheme,
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
    use raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle};
    use tej_ui_hal::{
        CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, UiEvent, WindowConfig,
    };

    // Compile-time soundness check: LinuxWindow remains Send + Sync via parking_lot::Mutex
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LinuxWindow>();

    let config = WindowConfig {
        title: "Linux Test Window".to_string(),
        ..WindowConfig::default()
    };
    let mut window = LinuxWindow::new(config);

    assert_eq!(window.title(), "Linux Test Window");
    assert_eq!(window.scale_factor(), 1.0);

    // Raw OS handles must safely return Err(Unavailable) without fabricating pointers
    assert!(matches!(
        window.window_handle(),
        Err(HandleError::Unavailable)
    ));
    assert!(matches!(
        window.display_handle(),
        Err(HandleError::Unavailable)
    ));

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

#[test]
fn test_linux_surface_partial_dirty_rect_preserves_clean_pixels() {
    let mut surface = LinuxSurface::new(Size::new(64.0, 64.0));

    // Full frame: paint entire surface bright red
    surface.begin_frame(&[]);
    surface.fill_rect(Rect::new(0.0, 0.0, 64.0, 64.0), Color::rgb(255, 0, 0));
    surface.end_frame();
    assert_eq!(surface.pixels()[50 * 64 + 50], 0xFFFF_0000);
    assert_eq!(surface.pixels()[5 * 64 + 5], 0xFFFF_0000);

    // Partial dirty-rect frame: only top-left 16x16 is dirty
    surface.begin_frame(&[Rect::new(0.0, 0.0, 16.0, 16.0)]);
    surface.fill_rect(Rect::new(0.0, 0.0, 16.0, 16.0), Color::rgb(0, 0, 255));
    surface.end_frame();

    // Dirty region was updated to blue
    assert_eq!(surface.pixels()[5 * 64 + 5], 0xFF00_00FF);
    // Clean region outside dirty rect was NOT wiped to background
    assert_eq!(surface.pixels()[50 * 64 + 50], 0xFFFF_0000);
}

#[test]
fn test_linux_surface_draw_text_respects_color_alpha() {
    let mut surface = LinuxSurface::new(Size::new(64.0, 32.0));
    surface.clear(Color::rgb(0, 0, 0));

    // Alpha = 0 must leave background completely untouched
    surface.draw_text(
        "A",
        Point::new(4.0, 4.0),
        16.0,
        Color::rgba(255, 255, 255, 0),
    );
    assert!(surface.pixels().iter().all(|&px| px == 0xFF00_0000));

    // Semi-transparent white (alpha = 128) on black must blend to ~mid-gray, not full white
    surface.draw_text(
        "A",
        Point::new(4.0, 4.0),
        16.0,
        Color::rgba(255, 255, 255, 128),
    );
    let max_red = surface
        .pixels()
        .iter()
        .map(|&px| ((px >> 16) & 0xFF) as u8)
        .max()
        .unwrap_or(0);
    assert!(max_red > 0, "expected some glyph pixels to be drawn");
    assert!(
        max_red <= 140,
        "semi-transparent text should not reach full 255 brightness, got {max_red}"
    );
}

#[test]
fn test_linux_surface_push_pop_clip_prevents_bleed() {
    let mut surface = LinuxSurface::new(Size::new(64.0, 64.0));
    surface.clear(Color::rgb(0, 0, 0));

    // Push outer clip [10, 10, 30, 30] and nested clip [20, 15, 30, 30] -> intersection [20, 15, 20, 25]
    surface.push_clip(Rect::new(10.0, 10.0, 30.0, 30.0), [0.0; 4]);
    surface.push_clip(Rect::new(20.0, 15.0, 30.0, 30.0), [0.0; 4]);

    // Draw full-surface rect, rounded rect, diagonal line, and long text
    surface.fill_rect(Rect::new(0.0, 0.0, 64.0, 64.0), Color::rgb(255, 0, 0));
    surface.fill_rounded_rect(Rect::new(0.0, 0.0, 64.0, 64.0), 4.0, Color::rgb(0, 255, 0));
    surface.draw_line(
        Point::new(0.0, 0.0),
        Point::new(63.0, 63.0),
        3.0,
        Color::rgb(0, 0, 255),
    );
    surface.draw_text(
        "ABCDEFGH",
        Point::new(0.0, 12.0),
        16.0,
        Color::rgb(255, 255, 255),
    );

    // Every pixel outside [20..40, 15..40] must remain untouched black
    for y in 0..64usize {
        for x in 0..64usize {
            let px = surface.pixels()[y * 64 + x];
            let inside = (20..40).contains(&x) && (15..40).contains(&y);
            if !inside {
                assert_eq!(
                    px, 0xFF00_0000,
                    "pixel ({x}, {y}) bled outside intersected clip rect"
                );
            }
        }
    }
    // Pixels inside the intersected clip rect must have been painted
    assert_ne!(surface.pixels()[25 * 64 + 25], 0xFF00_0000);

    // Pop inner clip -> active clip returns to [10..40, 10..40]
    surface.pop_clip();
    surface.fill_rect(Rect::new(0.0, 0.0, 64.0, 64.0), Color::rgb(0, 0, 255));
    assert_eq!(surface.pixels()[12 * 64 + 12], 0xFF00_00FF);
    assert_eq!(surface.pixels()[5 * 64 + 5], 0xFF00_0000);

    // Pop outer clip -> unclipped drawing reaches (5, 5)
    surface.pop_clip();
    surface.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::rgb(0, 255, 0));
    assert_eq!(surface.pixels()[5 * 64 + 5], 0xFF00_FF00);
}

#[test]
fn test_linux_surface_draw_text_styled_works() {
    let mut surface = LinuxSurface::new(Size::new(96.0, 32.0));
    surface.clear(Color::rgb(0, 0, 0));

    surface.draw_text_styled(
        "Hello",
        Point::new(4.0, 4.0),
        16.0,
        700,
        tej_ui_hal::TextFamily::Monospace,
        Color::rgb(255, 255, 255),
    );

    let lit_pixels = surface
        .pixels()
        .iter()
        .filter(|&&px| px != 0xFF00_0000)
        .count();
    assert!(
        lit_pixels > 0,
        "expected draw_text_styled to rasterize visible glyph pixels"
    );
}
