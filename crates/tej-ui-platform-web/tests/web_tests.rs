//! Integration tests for Web canvas window shell, DOM event translation, and `TejWindowHost`.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use tej_ui_hal::event::{MouseButton, UiEvent};
use tej_ui_hal::{CursorIcon, ImeCursorArea, PlatformWindow, TejWindowHost, WindowConfig};
use tej_ui_layout::Size;
use tej_ui_platform_web::{WebEventBridge, WebSurfaceCompositor, WebWindow};

#[test]
fn test_web_window_creation_and_host_contract() {
    let config = WindowConfig {
        title: "Tej Web App".to_string(),
        size: Size::new(1024.0, 768.0),
        ..WindowConfig::default()
    };

    let mut window = WebWindow::new(config, "app-canvas");
    assert_eq!(window.canvas_id(), "app-canvas");
    assert_eq!(window.title(), "Tej Web App");
    assert_eq!(window.size(), Size::new(1024.0, 768.0));
    assert_eq!(window.scale_factor(), 1.0);
    assert!(window.is_open());

    // Window handle check
    let win_handle = window.window_handle().expect("valid web window handle");
    match win_handle.as_raw() {
        RawWindowHandle::WebCanvas(_) => {}
        _ => panic!("Expected WebCanvasWindowHandle"),
    }

    // Display handle check
    let disp_handle = window.display_handle().expect("valid web display handle");
    assert!(matches!(disp_handle.as_raw(), RawDisplayHandle::Web(_)));

    // Cursor switching
    window.set_cursor(CursorIcon::Pointer);
    assert_eq!(window.cursor(), CursorIcon::Pointer);

    // IME cursor area & composition
    window.set_ime_cursor_area(ImeCursorArea::new(100.0, 200.0, 20.0, 30.0));
    assert_eq!(
        window.ime_area(),
        ImeCursorArea::new(100.0, 200.0, 20.0, 30.0)
    );

    window.handle_composition_start();
    window.handle_composition_update("nihon");
    window.handle_composition_end("日本");

    let events = window.poll_events();
    assert_eq!(events.len(), 3);
    assert!(matches!(events[0], UiEvent::ImePreedit { .. }));
    assert!(matches!(events[1], UiEvent::ImePreedit { .. }));
    assert!(matches!(events[2], UiEvent::ImeCommit { .. }));
}

#[test]
fn test_web_event_bridge_translations() {
    let move_ev = WebEventBridge::on_mouse_move(150.0, 250.0);
    assert!(matches!(
        move_ev,
        UiEvent::MouseMove {
            position
        } if (position.x - 150.0).abs() < 1e-4 && (position.y - 250.0).abs() < 1e-4
    ));

    let down_ev = WebEventBridge::on_mouse_down(0, 50.0, 60.0);
    assert!(matches!(
        down_ev,
        UiEvent::MouseDown {
            button: MouseButton::Left,
            ..
        }
    ));

    let right_down = WebEventBridge::on_mouse_down(2, 50.0, 60.0);
    assert!(matches!(
        right_down,
        UiEvent::MouseDown {
            button: MouseButton::Right,
            ..
        }
    ));

    let scroll_ev = WebEventBridge::on_wheel(-10.0, 20.0, 100.0, 100.0);
    assert!(matches!(
        scroll_ev,
        UiEvent::Scroll {
            delta_x,
            delta_y,
            ..
        } if (delta_x - -10.0).abs() < 1e-4 && (delta_y - 20.0).abs() < 1e-4
    ));

    let dpr_ev = WebEventBridge::on_dpr_change(2.0);
    assert!(matches!(
        dpr_ev,
        UiEvent::ScaleFactorChanged { scale_factor } if (scale_factor - 2.0).abs() < 1e-4
    ));
}

#[test]
fn test_web_surface_compositor() {
    let mut comp = WebSurfaceCompositor::new(64, 64);
    assert_eq!(comp.width(), 64);
    assert_eq!(comp.height(), 64);
    assert_eq!(comp.pixels().len(), 64 * 64);

    comp.resize_dims(128, 128);
    assert_eq!(comp.width(), 128);
    assert_eq!(comp.height(), 128);
    assert_eq!(comp.pixels().len(), 128 * 128);
}
