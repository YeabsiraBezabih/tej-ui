use tej_ui_a11y::{A11yNode, A11yRole, A11yTree};
use tej_ui_layout::Rect;

#[test]
fn test_a11y_tree_construction() {
    let mut tree = A11yTree::new();
    let win_id = tree.allocate_id();
    let btn_id = tree.allocate_id();

    let mut win_node = A11yNode::new(win_id, A11yRole::Window);
    win_node.label = Some("Main Window".into());
    win_node.children.push(btn_id);

    let mut btn_node = A11yNode::new(btn_id, A11yRole::Button);
    btn_node.label = Some("Submit".into());

    tree.insert_node(win_node);
    tree.insert_node(btn_node);

    assert_eq!(tree.len(), 2);
    assert_eq!(tree.root(), Some(win_id));

    tree.update_bounds(btn_id, Rect::new(10.0, 10.0, 100.0, 40.0));
    assert_eq!(tree.get_node(btn_id).unwrap().bounds.size.width, 100.0);
}

#[test]
fn test_a11y_live_region_announcements() {
    use tej_ui_a11y::{AnnouncementPoliteness, AnnouncementQueue};

    let queue = AnnouncementQueue::new();
    assert!(!queue.has_pending());

    queue.announce_polite("File saved successfully.");
    queue.announce_assertive("Network connection lost!");

    assert_eq!(queue.pending_count(), 2);
    assert!(queue.has_pending());

    let pending = queue.drain_pending();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].message, "File saved successfully.");
    assert_eq!(pending[0].politeness, AnnouncementPoliteness::Polite);
    assert_eq!(pending[1].message, "Network connection lost!");
    assert_eq!(pending[1].politeness, AnnouncementPoliteness::Assertive);

    // After draining, queue must be empty
    assert!(!queue.has_pending());
}

#[test]
fn test_a11y_os_preferences_reactive_bindings() {
    use tej_ui_a11y::A11yPreferences;

    let prefs = A11yPreferences::standard();
    assert!(!prefs.is_reduced_motion());
    assert!(!prefs.is_high_contrast());
    assert_eq!(prefs.font_scale(), 1.0);
    assert_eq!(prefs.scale_font_size(16.0), 16.0);

    // Toggle reduced motion
    prefs.enable_reduced_motion();
    assert!(prefs.is_reduced_motion());

    // Toggle high contrast
    prefs.enable_high_contrast();
    assert!(prefs.is_high_contrast());

    // Scale font size (e.g. 1.5x Dynamic Type)
    prefs.set_dynamic_type_scale(1.5);
    assert_eq!(prefs.font_scale(), 1.5);
    assert_eq!(prefs.scale_font_size(16.0), 24.0);
}

#[test]
fn test_a11y_bridge_events() {
    use tej_ui_a11y::{A11yAction, A11yBridge, A11yBridgeEvent, A11yNode, A11yRole, A11yTree};

    let mut bridge = A11yBridge::new();
    assert!(!bridge.is_screen_reader_active());

    bridge.activate_screen_reader();
    assert!(bridge.is_screen_reader_active());

    let mut tree = A11yTree::new();
    let btn_id = tree.allocate_id();
    let mut btn_node = A11yNode::new(btn_id, A11yRole::Button);
    btn_node.label = Some("OK Button".into());
    tree.insert_node(btn_node);

    assert_eq!(bridge.query_node_label(&tree, btn_id), Some("OK Button"));

    // Event dispatches
    bridge.notify_focus(btn_id);
    bridge.notify_value_changed(btn_id, "Clicked");
    bridge.invoke_action(btn_id, A11yAction::Click);

    let events = bridge.drain_events();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0], A11yBridgeEvent::FocusChanged(btn_id));
    assert_eq!(
        events[1],
        A11yBridgeEvent::ValueChanged {
            id: btn_id,
            new_value: "Clicked".into()
        }
    );
    assert_eq!(
        events[2],
        A11yBridgeEvent::ActionInvoked {
            id: btn_id,
            action: A11yAction::Click
        }
    );

    // Queue drained
    assert_eq!(
        bridge.drain_events(),
        [] as [tej_ui_a11y::A11yBridgeEvent; 0]
    );
}
