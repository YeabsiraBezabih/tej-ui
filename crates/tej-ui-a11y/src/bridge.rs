//! Platform accessibility bridge and event dispatcher for screen readers.

use crate::node::A11yId;
use crate::role::A11yAction;
use crate::tree::A11yTree;

/// Events dispatched across the native OS accessibility bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum A11yBridgeEvent {
    /// Focus moved to a specific accessibility node.
    FocusChanged(A11yId),
    /// Value or text of an accessibility node changed.
    ValueChanged { id: A11yId, new_value: String },
    /// Node was clicked or invoked via assistive technology.
    ActionInvoked { id: A11yId, action: A11yAction },
    /// Tree hierarchy or geometry changed.
    TreeStructureChanged,
}

/// Cross-platform accessibility bridge managing events between UI tree and OS screen readers.
#[derive(Debug, Default)]
pub struct A11yBridge {
    pending_events: Vec<A11yBridgeEvent>,
    focused_node: Option<A11yId>,
    is_screen_reader_active: bool,
}

impl A11yBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks an active screen reader (e.g. Narrator, VoiceOver, Orca) as detected by the OS.
    pub fn activate_screen_reader(&mut self) {
        self.is_screen_reader_active = true;
    }

    /// Marks the screen reader as inactive.
    pub fn deactivate_screen_reader(&mut self) {
        self.is_screen_reader_active = false;
    }

    /// Returns true if a screen reader is actively consuming accessibility events.
    pub fn is_screen_reader_active(&self) -> bool {
        self.is_screen_reader_active
    }

    /// Notifies the bridge that focus shifted to a specific node.
    pub fn notify_focus(&mut self, id: A11yId) {
        self.focused_node = Some(id);
        self.pending_events.push(A11yBridgeEvent::FocusChanged(id));
    }

    /// Notifies the bridge that a node's value changed.
    pub fn notify_value_changed(&mut self, id: A11yId, new_value: impl Into<String>) {
        self.pending_events.push(A11yBridgeEvent::ValueChanged {
            id,
            new_value: new_value.into(),
        });
    }

    /// Dispatches an action initiated by the screen reader into the application.
    pub fn invoke_action(&mut self, id: A11yId, action: A11yAction) {
        self.pending_events
            .push(A11yBridgeEvent::ActionInvoked { id, action });
    }

    /// Notifies the bridge that the accessibility tree structure changed.
    pub fn notify_structure_changed(&mut self) {
        self.pending_events
            .push(A11yBridgeEvent::TreeStructureChanged);
    }

    /// Drains all queued bridge events to be dispatched to the platform OS adapter.
    pub fn drain_events(&mut self) -> Vec<A11yBridgeEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Checks if a node exists and is accessible within the current tree.
    pub fn query_node_label<'a>(&self, tree: &'a A11yTree, id: A11yId) -> Option<&'a str> {
        tree.get_node(id).and_then(|n| n.label.as_deref())
    }
}
