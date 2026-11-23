use tej_ui_a11y::{
    A11yAction, A11yBridgeEvent, A11yId, A11yNode, A11yRole, A11yTree, AccessKitBridge,
    HeadlessA11yAdapter, PlatformA11yAdapter, WebAriaAdapter, from_accesskit_node_id,
    to_accesskit_node_id,
};
use tej_ui_layout::Rect;

#[test]
fn test_accesskit_id_conversion() {
    for raw in [0, 1, 42, 9999, u32::MAX - 1] {
        let id = A11yId(raw);
        let ak_id = to_accesskit_node_id(id);
        let back = from_accesskit_node_id(ak_id);
        assert_eq!(id, back);
    }
}

#[test]
fn test_accesskit_initial_and_incremental_updates() {
    let mut tree = A11yTree::new();
    let win_id = tree.allocate_id();
    let btn_id = tree.allocate_id();
    let label_id = tree.allocate_id();

    let mut win_node = A11yNode::new(win_id, A11yRole::Window);
    win_node.children.push(btn_id);
    win_node.children.push(label_id);

    let mut btn_node = A11yNode::new(btn_id, A11yRole::Button);
    btn_node.label = Some("Save".into());
    btn_node.bounds = Rect::new(10.0, 10.0, 80.0, 30.0);

    let mut label_node = A11yNode::new(label_id, A11yRole::Label);
    label_node.label = Some("Status: Ready".into());
    label_node.bounds = Rect::new(10.0, 50.0, 200.0, 20.0);

    tree.insert_node(win_node);
    tree.insert_node(btn_node);
    tree.insert_node(label_node);

    let mut bridge = AccessKitBridge::new();

    // 1. Initial update: contains all 3 nodes and Tree structure
    let initial_update = bridge.build_tree_update(&mut tree);
    assert!(initial_update.tree.is_some());
    assert_eq!(initial_update.nodes.len(), 3);
    assert_eq!(tree.dirty_count(), 0);

    // 2. Incremental update with no changes: 0 nodes emitted
    let empty_update = bridge.build_tree_update(&mut tree);
    assert!(empty_update.tree.is_none());
    assert_eq!(empty_update.nodes.len(), 0);

    // 3. Mutate only label_id
    tree.update_value(label_id, "Status: Saving...");
    assert_eq!(tree.dirty_count(), 1);

    // 4. Incremental update: exactly 1 mutated node emitted!
    let incremental_update = bridge.build_tree_update(&mut tree);
    assert!(incremental_update.tree.is_none());
    assert_eq!(incremental_update.nodes.len(), 1);
    assert_eq!(
        incremental_update.nodes[0].0,
        to_accesskit_node_id(label_id)
    );
    assert_eq!(tree.dirty_count(), 0);
}

#[test]
fn test_accesskit_action_handling() {
    let mut bridge = AccessKitBridge::new();
    let target_id = A11yId(42);
    let ak_target = to_accesskit_node_id(target_id);

    // Click request
    let req_click = accesskit::ActionRequest {
        action: accesskit::Action::Click,
        target_tree: accesskit::TreeId::ROOT,
        target_node: ak_target,
        data: None,
    };
    let event = bridge.handle_action_request(req_click);
    assert_eq!(
        event,
        Some(A11yBridgeEvent::ActionInvoked {
            id: target_id,
            action: A11yAction::Click,
        })
    );

    // Focus request
    let req_focus = accesskit::ActionRequest {
        action: accesskit::Action::Focus,
        target_tree: accesskit::TreeId::ROOT,
        target_node: ak_target,
        data: None,
    };
    let event = bridge.handle_action_request(req_focus);
    assert_eq!(event, Some(A11yBridgeEvent::FocusChanged(target_id)));
    assert_eq!(bridge.current_focus(), Some(target_id));

    // SetValue request
    let req_val = accesskit::ActionRequest {
        action: accesskit::Action::SetValue,
        target_tree: accesskit::TreeId::ROOT,
        target_node: ak_target,
        data: Some(accesskit::ActionData::Value("New Input".into())),
    };
    let event = bridge.handle_action_request(req_val);
    assert_eq!(
        event,
        Some(A11yBridgeEvent::ValueChanged {
            id: target_id,
            new_value: "New Input".into(),
        })
    );

    let drained = bridge.drain_events();
    assert_eq!(drained.len(), 3);
}

#[test]
fn test_web_aria_shadow_adapter() {
    let mut adapter = WebAriaAdapter::new();

    let btn_node = A11yNode::new(A11yId(1), A11yRole::Button)
        .with_label("Submit Form")
        .with_bounds(Rect::new(0.0, 0.0, 100.0, 30.0));

    adapter.sync_node(&btn_node);
    assert_eq!(adapter.node_count(), 1);

    let shadow = adapter.get_node(A11yId(1)).unwrap();
    assert_eq!(shadow.html_tag, "button");
    assert_eq!(
        shadow.attributes.get("aria-label").map(String::as_str),
        Some("Submit Form")
    );

    let html = shadow.render_html_opening();
    assert!(html.contains("<button"));
    assert!(html.contains(r#"aria-label="Submit Form""#));
}

#[test]
fn test_headless_adapter_recording() {
    let mut adapter = HeadlessA11yAdapter::new();
    assert!(adapter.is_active());
    assert_eq!(adapter.updates().len(), 0);

    let dummy_update = accesskit::TreeUpdate {
        tree_id: accesskit::TreeId::ROOT,
        nodes: vec![],
        tree: None,
        focus: to_accesskit_node_id(A11yId(1)),
    };

    adapter.push_update(dummy_update);
    assert_eq!(adapter.updates().len(), 1);
    assert!(adapter.last_update().is_some());
}
