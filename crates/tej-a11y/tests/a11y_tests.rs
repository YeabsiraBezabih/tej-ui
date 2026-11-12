use tej_a11y::{A11yNode, A11yRole, A11yTree};
use tej_layout::Rect;

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
