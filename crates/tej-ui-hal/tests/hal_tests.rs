use tej_ui_hal::{
    Color, DirtyRectTracker, FocusDirection, FocusId, FocusManager, MonitorId, MonitorInfo,
    MonitorManager, MouseButton, PlatformWindow, UiEvent, WindowId, WindowRegistry,
};
use tej_ui_layout::{Point, Rect, Size};

#[test]
fn test_dirty_rect_aggregation() {
    let mut tracker = DirtyRectTracker::new();
    assert!(!tracker.has_dirty());

    tracker.add_rect(Rect::new(10.0, 10.0, 50.0, 50.0));
    assert!(tracker.has_dirty());
    assert_eq!(tracker.rects().len(), 1);

    // Overlapping rect should merge
    tracker.add_rect(Rect::new(30.0, 30.0, 50.0, 50.0));
    assert_eq!(tracker.rects().len(), 1);
    let merged = tracker.rects()[0];
    assert_eq!(merged.origin.x, 10.0);
    assert_eq!(merged.origin.y, 10.0);
    assert_eq!(merged.size.width, 70.0);
    assert_eq!(merged.size.height, 70.0);

    tracker.reset();
    assert!(!tracker.has_dirty());
}

#[test]
fn test_ui_events_model() {
    let evt = UiEvent::MouseDown {
        button: MouseButton::Left,
        position: Point::new(100.0, 200.0),
    };

    if let UiEvent::MouseDown { button, position } = evt {
        assert_eq!(button, MouseButton::Left);
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
    } else {
        panic!("Event pattern mismatch");
    }
}

#[test]
fn test_color_constructors() {
    let c = Color::rgb(10, 20, 30);
    assert_eq!(c.r, 10);
    assert_eq!(c.g, 20);
    assert_eq!(c.b, 30);
    assert_eq!(c.a, 255);
}

#[test]
fn test_linear_space_color_and_blending() {
    use tej_ui_hal::{ColorLinear, ColorSpace};

    // ColorSpace default
    assert_eq!(ColorSpace::default(), ColorSpace::Srgb);

    // Linear conversion of pure white and pure black
    let white = Color::WHITE;
    let white_linear = white.to_linear();
    assert!((white_linear.r - 1.0).abs() < 1e-4);
    assert!((white_linear.g - 1.0).abs() < 1e-4);
    assert!((white_linear.b - 1.0).abs() < 1e-4);

    let black = Color::BLACK;
    let black_linear = black.to_linear();
    assert!(black_linear.r < 1e-4);

    // Roundtrip linear conversion
    let srgb = Color::rgb(128, 64, 200);
    let back = srgb.to_linear().to_srgb_color();
    assert!((i32::from(back.r) - i32::from(srgb.r)).abs() <= 1);
    assert!((i32::from(back.g) - i32::from(srgb.g)).abs() <= 1);
    assert!((i32::from(back.b) - i32::from(srgb.b)).abs() <= 1);

    // Linear-space blending without dark fringes:
    // Blending semi-transparent white (50% alpha) over black in linear space produces ~188 sRGB (not 128!)
    let semi_white = Color::rgba(255, 255, 255, 128);
    let blended = Color::blend_linear(semi_white, Color::BLACK);
    assert!(blended.r > 170 && blended.r < 195);

    // Display P3 gamut conversion
    let lin_white = ColorLinear::new(1.0, 1.0, 1.0, 1.0);
    let p3 = lin_white.to_display_p3();
    assert!(p3.r <= 1.0 && p3.g <= 1.0 && p3.b <= 1.0);
}

#[derive(Debug)]
struct MockWindow {
    id: WindowId,
    title: String,
    size: Size,
}

impl PlatformWindow for MockWindow {
    fn id(&self) -> WindowId {
        self.id
    }
    fn size(&self) -> Size {
        self.size
    }
    fn request_redraw(&self) {}
    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }
    fn poll_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }
}

#[test]
fn test_multi_monitor_and_window_registry() {
    // 1. Multi-monitor management
    let mut manager = MonitorManager::default();
    assert_eq!(manager.monitors().len(), 1);

    // Add secondary 4K monitor at 2.0x DPI scale factor
    manager.add_monitor(MonitorInfo::new(
        MonitorId(2),
        "Dell 4K External",
        Rect::from_origin_size(Point::new(1920.0, 0.0), Size::new(3840.0, 2160.0)),
        2.0,
        tej_ui_hal::MonitorRole::Secondary,
    ));

    assert_eq!(manager.monitors().len(), 2);
    let secondary = manager.monitor_at_point(Point::new(2000.0, 500.0)).unwrap();
    assert_eq!(secondary.id, MonitorId(2));
    assert_eq!(secondary.scale_factor, 2.0);
    assert_eq!(secondary.physical_size(), Size::new(7680.0, 4320.0));

    // 2. Window Registry
    let mut registry = WindowRegistry::new();
    assert_eq!(registry.count(), 0);

    let win1 = Box::new(MockWindow {
        id: WindowId(1),
        title: "Main".to_string(),
        size: Size::new(800.0, 600.0),
    });
    let win2 = Box::new(MockWindow {
        id: WindowId(2),
        title: "Inspector".to_string(),
        size: Size::new(400.0, 600.0),
    });

    registry.register(win1);
    registry.register(win2);
    assert_eq!(registry.count(), 2);
    assert_eq!(registry.focused_window_id(), Some(WindowId(1)));

    registry.set_focused_window(WindowId(2));
    assert_eq!(registry.focused_window_id(), Some(WindowId(2)));

    let removed = registry.unregister(WindowId(1));
    assert!(removed.is_some());
    assert_eq!(registry.count(), 1);

    // 3. Modal ownership and parent blocking
    let mut reg2 = WindowRegistry::new();
    let main_win = Box::new(MockWindow {
        id: WindowId(10),
        title: "Main Editor".to_string(),
        size: Size::new(1024.0, 768.0),
    });
    let modal_win = Box::new(MockWindow {
        id: WindowId(11),
        title: "Export File Modal".to_string(),
        size: Size::new(400.0, 300.0),
    });

    let main_id = reg2.register(main_win);
    assert!(!reg2.is_window_blocked(main_id));

    let modal_id = reg2.register_modal(modal_win, main_id);
    assert!(reg2.is_window_blocked(main_id));
    assert_eq!(reg2.parent_of(modal_id), Some(main_id));
    assert_eq!(reg2.modals_for(main_id), &[modal_id]);
    assert_eq!(reg2.focused_window_id(), Some(modal_id));

    // Close modal dialog -> parent unblocks and regains focus
    reg2.unregister(modal_id);
    assert!(!reg2.is_window_blocked(main_id));
    assert_eq!(reg2.focused_window_id(), Some(main_id));
}

#[test]
fn test_focus_manager_tab_cycling() {
    let mut fm = FocusManager::new();
    let id1 = FocusId::next_unique();
    let id2 = FocusId::next_unique();
    let id3 = FocusId::next_unique();

    fm.register_target(id1);
    fm.register_target(id2);
    fm.register_target(id3);

    assert_eq!(fm.current_focus(), None);
    assert!(!fm.is_focused(id1));

    // Tab forward: None -> id1 -> id2 -> id3 -> id1
    assert_eq!(fm.traverse(FocusDirection::Next), Some(id1));
    assert!(fm.is_focused(id1));

    assert_eq!(fm.traverse(FocusDirection::Next), Some(id2));
    assert!(fm.is_focused(id2));

    assert_eq!(fm.traverse(FocusDirection::Next), Some(id3));
    assert!(fm.is_focused(id3));

    assert_eq!(fm.traverse(FocusDirection::Next), Some(id1));
    assert!(fm.is_focused(id1));

    // Shift-Tab backward: id1 -> id3
    assert_eq!(fm.traverse(FocusDirection::Previous), Some(id3));
    assert!(fm.is_focused(id3));

    // Unregister
    fm.unregister_target(id3);
    assert_eq!(fm.current_focus(), None);
}

#[test]
fn test_focus_manager_modal_trap_scope() {
    let mut fm = FocusManager::new();
    let id_bg = FocusId::next_unique();
    let id_modal1 = FocusId::next_unique();
    let id_modal2 = FocusId::next_unique();

    fm.register_target(id_bg);
    fm.register_target(id_modal1);
    fm.register_target(id_modal2);

    // Initial tab focuses id_bg
    assert_eq!(fm.traverse(FocusDirection::Next), Some(id_bg));

    // Push modal trap containing only modal1 and modal2
    fm.push_trap_scope(vec![id_modal1, id_modal2]);
    assert!(fm.is_trapped());
    assert_eq!(fm.current_focus(), Some(id_modal1));

    // Tab within modal: modal1 -> modal2 -> modal1 (id_bg is trapped out)
    assert_eq!(fm.traverse(FocusDirection::Next), Some(id_modal2));
    assert_eq!(fm.traverse(FocusDirection::Next), Some(id_modal1));
    assert_eq!(fm.traverse(FocusDirection::Previous), Some(id_modal2));

    // Pop modal trap -> restored to global traversal
    let popped = fm.pop_trap_scope();
    assert_eq!(popped, Some(vec![id_modal1, id_modal2]));
    assert!(!fm.is_trapped());
}

#[test]
fn test_focus_manager_spatial_navigation() {
    use tej_ui_hal::SpatialDirection;

    let mut fm = FocusManager::new();
    let id_top = FocusId::next_unique();
    let id_bottom = FocusId::next_unique();
    let id_left = FocusId::next_unique();
    let id_right = FocusId::next_unique();

    // Register 2D layout bounds (card grid layout)
    fm.register_spatial_bounds(id_top, Rect::new(100.0, 0.0, 50.0, 50.0));
    fm.register_spatial_bounds(id_bottom, Rect::new(100.0, 100.0, 50.0, 50.0));
    fm.register_spatial_bounds(id_left, Rect::new(0.0, 50.0, 50.0, 50.0));
    fm.register_spatial_bounds(id_right, Rect::new(200.0, 50.0, 50.0, 50.0));

    // Start at top
    fm.request_focus(id_top);
    assert_eq!(fm.current_focus(), Some(id_top));

    // Arrow Down -> navigates to bottom
    assert_eq!(fm.navigate_spatial(SpatialDirection::Down), Some(id_bottom));
    assert_eq!(fm.current_focus(), Some(id_bottom));

    // Arrow Left from bottom -> navigates to left
    assert_eq!(fm.navigate_spatial(SpatialDirection::Left), Some(id_left));
    assert_eq!(fm.current_focus(), Some(id_left));

    // Arrow Right from left -> navigates to right
    assert_eq!(fm.navigate_spatial(SpatialDirection::Right), Some(id_right));
    assert_eq!(fm.current_focus(), Some(id_right));

    // Arrow Up from right -> navigates to top
    assert_eq!(fm.navigate_spatial(SpatialDirection::Up), Some(id_top));
    assert_eq!(fm.current_focus(), Some(id_top));
}

#[test]
fn test_lifecycle_state_machine_and_close_veto() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use tej_ui_hal::{AppLifecycleState, LifecycleEvent, LifecycleManager, MemoryPressureLevel};

    let mut manager = LifecycleManager::new();
    assert_eq!(manager.state(), AppLifecycleState::Active);

    // 1. Transition listeners
    let transition_recorded = std::sync::Arc::new(AtomicBool::new(false));
    let t_clone = std::sync::Arc::clone(&transition_recorded);
    manager.on_transition(move |prev, next| {
        if prev == AppLifecycleState::Active && next == AppLifecycleState::Inactive {
            t_clone.store(true, Ordering::SeqCst);
        }
    });

    manager.dispatch(LifecycleEvent::ForegroundInactive);
    assert_eq!(manager.state(), AppLifecycleState::Inactive);
    assert!(transition_recorded.load(Ordering::SeqCst));

    // 2. Memory pressure hook
    let memory_alert = std::sync::Arc::new(AtomicBool::new(false));
    let m_clone = std::sync::Arc::clone(&memory_alert);
    manager.on_low_memory(move |level| {
        if level == MemoryPressureLevel::Critical {
            m_clone.store(true, Ordering::SeqCst);
        }
    });

    manager.dispatch(LifecycleEvent::LowMemory(MemoryPressureLevel::Critical));
    assert!(memory_alert.load(Ordering::SeqCst));

    // 3. Unsaved work close interception
    let has_unsaved_changes = std::sync::Arc::new(AtomicBool::new(true));
    let u_clone = std::sync::Arc::clone(&has_unsaved_changes);
    manager.on_close_requested(move |token| {
        if u_clone.load(Ordering::SeqCst) {
            token.veto("Draft document has unsaved edits");
        }
    });

    // Close attempt is vetoed
    let close_res = manager.request_close();
    assert!(close_res.is_err());
    assert_eq!(
        close_res.unwrap_err(),
        vec!["Draft document has unsaved edits".to_string()]
    );

    // Save changes and try again
    has_unsaved_changes.store(false, Ordering::SeqCst);
    let close_res_ok = manager.request_close();
    assert!(close_res_ok.is_ok());
}

#[test]
fn test_gesture_arena_and_kinetic_deceleration() {
    use tej_ui_hal::{
        GestureArena, GestureState, KineticDeceleration, TouchId, TouchPhase, TouchPoint,
    };
    use tej_ui_layout::Point;

    // 1. Single touch pan gesture
    let mut arena = GestureArena::new();
    let t1 = TouchPoint {
        id: TouchId(1),
        position: Point::new(100.0, 100.0),
        start_position: Point::new(100.0, 100.0),
        timestamp_millis: 1000,
        phase: TouchPhase::Started,
    };
    arena.handle_touch(t1);
    assert_eq!(arena.touch_count(), 1);
    assert_eq!(arena.pan().state, GestureState::Began);

    let t1_move = TouchPoint {
        id: TouchId(1),
        position: Point::new(150.0, 120.0),
        start_position: Point::new(100.0, 100.0),
        timestamp_millis: 1050,
        phase: TouchPhase::Moved,
    };
    arena.handle_touch(t1_move);
    assert_eq!(arena.pan().state, GestureState::Changed);
    assert_eq!(arena.pan().translation.x, 50.0);
    assert_eq!(arena.pan().translation.y, 20.0);

    // 2. Multi-touch pinch-to-zoom
    let t2 = TouchPoint {
        id: TouchId(2),
        position: Point::new(250.0, 120.0),
        start_position: Point::new(250.0, 120.0),
        timestamp_millis: 1050,
        phase: TouchPhase::Started,
    };
    arena.handle_touch(t2);
    assert_eq!(arena.touch_count(), 2);
    assert_eq!(arena.pinch().state, GestureState::Began);
    assert_eq!(arena.pinch().scale, 1.0);

    // Pinch spread apart
    let t2_spread = TouchPoint {
        id: TouchId(2),
        position: Point::new(350.0, 120.0),
        start_position: Point::new(250.0, 120.0),
        timestamp_millis: 1100,
        phase: TouchPhase::Moved,
    };
    arena.handle_touch(t2_spread);
    assert!(arena.pinch().scale > 1.5);

    // 3. Kinetic scroll deceleration
    let mut kinetic = KineticDeceleration::new(Point::new(1000.0, 0.0), 5.0);
    assert!(!kinetic.is_stopped());

    let delta1 = kinetic.step(0.016); // ~1 frame at 60Hz
    assert!(delta1.x > 0.0);
    assert!(kinetic.current_velocity.x < 1000.0);

    // Run until stop
    for _ in 0..100 {
        kinetic.step(0.05);
    }
    assert!(kinetic.is_stopped());
}

#[test]
fn test_stylus_and_palm_rejection() {
    use tej_ui_hal::{PalmRejectionFilter, StylusPoint, StylusToolType};
    use tej_ui_layout::Point;

    // 1. Stylus point properties
    let stylus = StylusPoint::new(Point::new(400.0, 300.0), 0.85).with_tilt(15.0, -10.0);
    assert_eq!(stylus.position.x, 400.0);
    assert_eq!(stylus.pressure, 0.85);
    assert_eq!(stylus.tilt_x, 15.0);
    assert_eq!(stylus.tilt_y, -10.0);
    assert_eq!(stylus.tool_type, StylusToolType::Pen);

    // 2. Palm rejection
    let filter = PalmRejectionFilter::new(20.0, 80.0);

    // Normal finger tap away from stylus -> allowed
    let finger_pos = Point::new(100.0, 100.0);
    assert!(!filter.is_palm(finger_pos, 10.0, Some(stylus.position)));

    // Large contact area (> 20px radius) -> rejected as palm!
    let large_palm_pos = Point::new(100.0, 100.0);
    assert!(filter.is_palm(large_palm_pos, 35.0, None));

    // Small contact within 80px exclusion radius of stylus tip -> rejected as palm!
    let near_stylus_touch = Point::new(420.0, 310.0);
    assert!(filter.is_palm(near_stylus_touch, 8.0, Some(stylus.position)));
}

#[test]
fn test_dpi_and_pacing_and_compositor_recovery() {
    use std::time::Duration;
    use tej_ui_hal::{
        CustomTitlebar, DeviceHealth, DeviceLossCoordinator, DirtyRegion, DpiChangeEvent, DpiScale,
        FramePacer, RefreshRateMode, TitlebarHitRegion,
    };
    use tej_ui_layout::{Point, Rect};

    // 1. DPI Fractional Scaling
    let dpi_150 = DpiScale::scale_150();
    assert_eq!(dpi_150.factor(), 1.5);
    assert_eq!(dpi_150.logical_to_physical(100.0), 150.0);
    assert_eq!(dpi_150.physical_to_logical(150.0), 100.0);
    assert_eq!(
        dpi_150.logical_to_physical_point(Point::new(10.0, 20.0)),
        Point::new(15.0, 30.0)
    );

    let event = DpiChangeEvent::new(DpiScale::default_100(), DpiScale::scale_200());
    assert_eq!(event.ratio(), 2.0);

    // 2. VSync & Refresh Rate Pacing (60Hz -> 120Hz)
    let mut pacer = FramePacer::new(RefreshRateMode::PROMOTION_120);
    assert_eq!(pacer.target_hz(), 120.0);
    assert!(pacer.frame_budget() <= Duration::from_millis(9));

    // Fast frame: passes budget
    assert!(pacer.record_frame_execution(Duration::from_millis(5)));
    // Jank frame: exceeds budget
    assert!(!pacer.record_frame_execution(Duration::from_millis(15)));
    assert_eq!(pacer.dropped_frames(), 1);
    assert_eq!(pacer.total_frames(), 2);

    // 3. Multi-rect Dirty Region Tracking
    let mut dirty = DirtyRegion::new(4);
    assert!(dirty.is_empty());
    dirty.mark_dirty(Rect::new(10.0, 10.0, 50.0, 50.0));
    assert!(!dirty.is_empty());
    assert_eq!(
        dirty.bounding_box(),
        Some(Rect::new(10.0, 10.0, 50.0, 50.0))
    );

    // Adjacent/overlapping rect merges
    dirty.mark_dirty(Rect::new(40.0, 40.0, 60.0, 60.0));
    assert_eq!(dirty.rects().len(), 1);
    dirty.clear();
    assert!(dirty.is_empty());

    // 4. GPU Device Loss Recovery & Software Fallback
    let mut recovery = DeviceLossCoordinator::new(2);
    assert_eq!(recovery.health(), DeviceHealth::Healthy);

    // First crash -> enter recovering
    recovery.notify_device_lost();
    assert_eq!(recovery.health(), DeviceHealth::DeviceLost);
    let token = recovery.begin_recovery().expect("Should begin recovery");
    assert!(recovery.complete_recovery(token));
    assert_eq!(recovery.health(), DeviceHealth::Healthy);

    // Repeated crashes -> fallback to CPU software rasterization
    recovery.notify_device_lost();
    let _ = recovery.begin_recovery();
    recovery.notify_device_lost();
    let _ = recovery.begin_recovery();
    recovery.notify_device_lost();
    assert_eq!(recovery.health(), DeviceHealth::SoftwareFallback);
    assert!(recovery.is_software_fallback());

    // 5. Custom Titlebar & Snap Layout Hit Testing
    let titlebar = CustomTitlebar::new(800.0, 32.0);
    assert_eq!(
        titlebar.hit_test(Point::new(100.0, 15.0)),
        TitlebarHitRegion::CaptionDrag
    );
    assert_eq!(
        titlebar.hit_test(Point::new(780.0, 15.0)),
        TitlebarHitRegion::CloseButton
    );
    assert_eq!(
        titlebar.hit_test(Point::new(730.0, 15.0)),
        TitlebarHitRegion::SnapLayoutZone
    );
}

#[test]
fn test_native_view_embedding() {
    use tej_ui_hal::{NativeViewHandle, NativeViewHost};
    use tej_ui_layout::Rect;

    let handle = NativeViewHandle::Win32Hwnd(0x1234_5678);
    let mut host = NativeViewHost::new(handle, Rect::new(50.0, 50.0, 640.0, 480.0));

    assert_eq!(host.handle(), NativeViewHandle::Win32Hwnd(0x1234_5678));
    assert!(host.is_visible());
    assert_eq!(host.clip_rect(), None);

    // Update bounds
    let changed = host.update_bounds(Rect::new(50.0, 50.0, 800.0, 600.0));
    assert!(changed);
    assert_eq!(host.bounds().width(), 800.0);

    // Set clipping rectangle
    host.set_clip_rect(Some(Rect::new(50.0, 50.0, 400.0, 400.0)));
    assert_eq!(host.clip_rect(), Some(Rect::new(50.0, 50.0, 400.0, 400.0)));

    host.hide();
    assert!(!host.is_visible());
}
