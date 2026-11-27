//! Incremental AccessKit tree bridge for platform assistive technology.
//!
//! Subscribes to dirty node epochs, serializing only mutated accessibility nodes into
//! `accesskit::TreeUpdate` diffs for zero full-tree serialization during steady state.

use crate::announcement::AnnouncementPoliteness;
use crate::bridge::A11yBridgeEvent;
use crate::node::{A11yId, A11yNode};
use crate::role::{A11yAction, A11yRole};
use crate::tree::A11yTree;

/// Convert `A11yId` to `accesskit::NodeId`.
#[inline]
#[must_use]
pub fn to_accesskit_node_id(id: A11yId) -> accesskit::NodeId {
    accesskit::NodeId(id.0 as u64 + 1)
}

/// Convert `accesskit::NodeId` to `A11yId`.
#[inline]
#[must_use]
pub fn from_accesskit_node_id(id: accesskit::NodeId) -> A11yId {
    A11yId((id.0.saturating_sub(1)) as u32)
}

/// Convert internal `A11yNode` into an `accesskit::Node`.
#[must_use]
pub fn convert_a11y_node(node: &A11yNode) -> accesskit::Node {
    let mut ak_node = accesskit::Node::default();
    ak_node.set_role(node.role.into());

    if let Some(label) = &node.label {
        ak_node.set_label(label.as_str());
    }
    if let Some(val) = &node.value {
        ak_node.set_value(val.as_str());
    }
    if let Some(desc) = &node.description {
        ak_node.set_description(desc.as_str());
    }

    let b = node.bounds;
    ak_node.set_bounds(accesskit::Rect {
        x0: b.origin.x as f64,
        y0: b.origin.y as f64,
        x1: (b.origin.x + b.size.width) as f64,
        y1: (b.origin.y + b.size.height) as f64,
    });

    if node.is_disabled {
        ak_node.set_disabled();
    }
    if node.is_hidden {
        ak_node.set_hidden();
    }
    if let Some(checked) = node.is_checked {
        ak_node.set_toggled(if checked {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        });
    }
    if let Some(live) = node.live_region {
        match live {
            AnnouncementPoliteness::Polite => ak_node.set_live(accesskit::Live::Polite),
            AnnouncementPoliteness::Assertive => ak_node.set_live(accesskit::Live::Assertive),
        }
    }
    if !node.children.is_empty() {
        ak_node.set_children(
            node.children
                .iter()
                .copied()
                .map(to_accesskit_node_id)
                .collect::<Vec<_>>(),
        );
    }

    // Role-dependent default accessibility actions
    match node.role {
        A11yRole::Button | A11yRole::Link | A11yRole::MenuItem | A11yRole::TreeItem => {
            ak_node.add_action(accesskit::Action::Click);
            ak_node.add_action(accesskit::Action::Focus);
        }
        A11yRole::Checkbox | A11yRole::Switch | A11yRole::RadioButton => {
            ak_node.add_action(accesskit::Action::Click);
            ak_node.add_action(accesskit::Action::Focus);
        }
        A11yRole::TextInput | A11yRole::ComboBox => {
            ak_node.add_action(accesskit::Action::Focus);
            ak_node.add_action(accesskit::Action::SetValue);
        }
        A11yRole::ScrollArea | A11yRole::List | A11yRole::Tree | A11yRole::Grid => {
            ak_node.add_action(accesskit::Action::ScrollDown);
            ak_node.add_action(accesskit::Action::ScrollUp);
        }
        A11yRole::Slider => {
            ak_node.add_action(accesskit::Action::Focus);
            ak_node.add_action(accesskit::Action::SetValue);
        }
        _ => {
            ak_node.add_action(accesskit::Action::Focus);
        }
    }

    ak_node
}

/// Live bridge connecting `A11yTree` with platform AccessKit adapters.
#[derive(Debug, Default)]
pub struct AccessKitBridge {
    initial_tree_sent: bool,
    active_focus: Option<A11yId>,
    pending_events: Vec<A11yBridgeEvent>,
}

impl AccessKitBridge {
    /// Create a new AccessKit bridge.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset bridge state (e.g. on window re-creation).
    pub fn reset(&mut self) {
        self.initial_tree_sent = false;
        self.active_focus = None;
        self.pending_events.clear();
    }

    /// Set current focused node ID.
    pub fn set_focus(&mut self, id: A11yId) {
        self.active_focus = Some(id);
        self.pending_events.push(A11yBridgeEvent::FocusChanged(id));
    }

    /// Get current focused node ID.
    #[must_use]
    pub const fn current_focus(&self) -> Option<A11yId> {
        self.active_focus
    }

    /// Builds a full or incremental `accesskit::TreeUpdate`.
    ///
    /// On initial call, sends the complete tree structure with `TreeInfo::new(root)`.
    /// On subsequent calls, drains dirty nodes from `tree` and only serializes mutated nodes.
    pub fn build_tree_update(&mut self, tree: &mut A11yTree) -> accesskit::TreeUpdate {
        let root_id = tree.root().unwrap_or(A11yId(1));
        let focus_id = self.active_focus.unwrap_or(root_id);

        if !self.initial_tree_sent {
            self.initial_tree_sent = true;
            let mut nodes = Vec::with_capacity(tree.len());
            for (&id, node) in tree.iter() {
                nodes.push((to_accesskit_node_id(id), convert_a11y_node(node)));
            }
            // Clear dirty set since all nodes have been sent
            let _ = tree.drain_dirty();

            accesskit::TreeUpdate {
                tree_id: accesskit::TreeId::ROOT,
                nodes,
                tree: Some(accesskit::TreeInfo::new(to_accesskit_node_id(root_id))),
                focus: to_accesskit_node_id(focus_id),
            }
        } else {
            let dirty = tree.drain_dirty();
            let mut nodes = Vec::with_capacity(dirty.len());
            for id in dirty {
                if let Some(node) = tree.get_node(id) {
                    nodes.push((to_accesskit_node_id(id), convert_a11y_node(node)));
                }
            }

            accesskit::TreeUpdate {
                tree_id: accesskit::TreeId::ROOT,
                nodes,
                tree: None,
                focus: to_accesskit_node_id(focus_id),
            }
        }
    }

    /// Explicitly build an update for a specific set of node IDs.
    #[must_use]
    pub fn build_update_for_nodes(
        &self,
        tree: &A11yTree,
        dirty_ids: &[A11yId],
    ) -> accesskit::TreeUpdate {
        let root_id = tree.root().unwrap_or(A11yId(1));
        let focus_id = self.active_focus.unwrap_or(root_id);

        let mut nodes = Vec::with_capacity(dirty_ids.len());
        for &id in dirty_ids {
            if let Some(node) = tree.get_node(id) {
                nodes.push((to_accesskit_node_id(id), convert_a11y_node(node)));
            }
        }

        accesskit::TreeUpdate {
            tree_id: accesskit::TreeId::ROOT,
            nodes,
            tree: None,
            focus: to_accesskit_node_id(focus_id),
        }
    }

    /// Handles an incoming `accesskit::ActionRequest` from platform assistive technologies
    /// and maps it to a native `A11yBridgeEvent`.
    pub fn handle_action_request(
        &mut self,
        req: accesskit::ActionRequest,
    ) -> Option<A11yBridgeEvent> {
        let target_id = from_accesskit_node_id(req.target_node);

        let event = match req.action {
            accesskit::Action::Click => Some(A11yBridgeEvent::ActionInvoked {
                id: target_id,
                action: A11yAction::Click,
            }),
            accesskit::Action::Focus => {
                self.active_focus = Some(target_id);
                Some(A11yBridgeEvent::FocusChanged(target_id))
            }
            accesskit::Action::ScrollDown => Some(A11yBridgeEvent::ActionInvoked {
                id: target_id,
                action: A11yAction::ScrollForward,
            }),
            accesskit::Action::ScrollUp => Some(A11yBridgeEvent::ActionInvoked {
                id: target_id,
                action: A11yAction::ScrollBackward,
            }),
            accesskit::Action::SetValue => {
                if let Some(accesskit::ActionData::Value(val)) = req.data {
                    Some(A11yBridgeEvent::ValueChanged {
                        id: target_id,
                        new_value: val.to_string(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(ref ev) = event {
            self.pending_events.push(ev.clone());
        }

        event
    }

    /// Drain all queued bridge events.
    pub fn drain_events(&mut self) -> Vec<A11yBridgeEvent> {
        core::mem::take(&mut self.pending_events)
    }
}
