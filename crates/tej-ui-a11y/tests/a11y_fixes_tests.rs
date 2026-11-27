//! Comprehensive unit tests for `tej-ui-a11y` security, memory safety, and enterprise role fixes.

use tej_ui_a11y::{
    A11yId, A11yNode, A11yRole, A11yTree, WebAriaAdapter, WebAriaShadowNode, convert_a11y_node,
};

#[test]
fn test_web_aria_html_attribute_escaping_prevents_injection() {
    let mut node = A11yNode::new(A11yId(1), A11yRole::Button);
    node.label = Some(r#"Click "me" & <script>alert(1)</script>"#.to_string());

    let mut adapter = WebAriaAdapter::new();
    adapter.sync_node(&node);

    let shadow = adapter.get_node(A11yId(1)).expect("shadow node synced");
    let html = shadow.render_html_opening();

    assert!(html.contains("&amp;"));
    assert!(html.contains("&quot;me&quot;"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn test_web_aria_slider_mapping_and_text_input_no_aria_valuenow() {
    let slider = WebAriaShadowNode::from_role(A11yId(2), A11yRole::Slider);
    assert_eq!(slider.html_tag, "input");
    assert_eq!(slider.role, Some("slider"));

    let mut text_input = A11yNode::new(A11yId(3), A11yRole::TextInput);
    text_input.value = Some("hello world".to_string());

    let mut slider_node = A11yNode::new(A11yId(4), A11yRole::Slider);
    slider_node.value = Some("50".to_string());

    let mut adapter = WebAriaAdapter::new();
    adapter.sync_node(&text_input);
    adapter.sync_node(&slider_node);

    let text_shadow = adapter.get_node(A11yId(3)).unwrap();
    assert!(!text_shadow.attributes.contains_key("aria-valuenow"));
    assert_eq!(
        text_shadow.attributes.get("value").map(String::as_str),
        Some("hello world")
    );

    let slider_shadow = adapter.get_node(A11yId(4)).unwrap();
    assert_eq!(
        slider_shadow
            .attributes
            .get("aria-valuenow")
            .map(String::as_str),
        Some("50")
    );
}

#[test]
fn test_a11y_tree_default_next_id_and_remove_node() {
    let mut default_tree = A11yTree::default();
    let first_id = default_tree.allocate_id();
    assert_eq!(first_id, A11yId(1));

    let root_id = first_id;
    let child_id = default_tree.allocate_id();
    let mut root = A11yNode::new(root_id, A11yRole::Window);
    root.children.push(child_id);
    let child = A11yNode::new(child_id, A11yRole::Button).with_label("Delete Me");

    default_tree.insert_node(root);
    default_tree.insert_node(child);
    let _ = default_tree.drain_dirty();

    let removed = default_tree.remove_node(child_id);
    assert!(removed.is_some());
    assert_eq!(default_tree.len(), 1);
    assert!(default_tree.get_node(child_id).is_none());
    // Parent's children list must no longer reference removed child and parent is marked dirty
    let updated_root = default_tree.get_node(root_id).unwrap();
    assert!(updated_root.children.is_empty());
    assert!(default_tree.has_dirty());

    // Removing root clears root()
    assert!(default_tree.remove_node(root_id).is_some());
    assert_eq!(default_tree.root(), None);
    assert!(default_tree.is_empty());
}

#[test]
fn test_accesskit_bridge_honors_is_hidden_and_enterprise_roles() {
    let mut hidden_node = A11yNode::new(A11yId(10), A11yRole::Alert);
    hidden_node.is_hidden = true;
    let ak_hidden = convert_a11y_node(&hidden_node);
    assert!(ak_hidden.is_hidden());
    assert_eq!(ak_hidden.role(), accesskit::Role::Alert);

    let visible_node = A11yNode::new(A11yId(11), A11yRole::Tooltip);
    let ak_visible = convert_a11y_node(&visible_node);
    assert!(!ak_visible.is_hidden());
    assert_eq!(ak_visible.role(), accesskit::Role::Tooltip);

    let role_pairs = [
        (A11yRole::Menu, accesskit::Role::Menu),
        (A11yRole::MenuItem, accesskit::Role::MenuItem),
        (A11yRole::Tree, accesskit::Role::Tree),
        (A11yRole::TreeItem, accesskit::Role::TreeItem),
        (A11yRole::Grid, accesskit::Role::Grid),
        (A11yRole::ComboBox, accesskit::Role::ComboBox),
        (A11yRole::Alert, accesskit::Role::Alert),
        (A11yRole::Tooltip, accesskit::Role::Tooltip),
    ];

    for (a11y_role, expected_ak_role) in role_pairs {
        assert_eq!(accesskit::Role::from(a11y_role), expected_ak_role);
        let shadow = WebAriaShadowNode::from_role(A11yId(99), a11y_role);
        assert!(shadow.role.is_some());
    }
}
