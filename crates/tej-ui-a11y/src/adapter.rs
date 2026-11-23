//! Platform accessibility adapters and semantic shadow tree bridges.

use crate::node::A11yId;
use crate::role::A11yRole;
use std::collections::HashMap;

/// Common trait implemented by platform accessibility adapters.
pub trait PlatformA11yAdapter: Send + Sync {
    /// Dispatches an incremental or initial tree update to the platform accessibility subsystem.
    fn push_update(&mut self, update: accesskit::TreeUpdate);

    /// Whether assistive technology is actively listening to this adapter.
    fn is_active(&self) -> bool;
}

/// Headless accessibility adapter used for headless rendering and deterministic regression testing.
#[derive(Debug, Default)]
pub struct HeadlessA11yAdapter {
    updates: Vec<accesskit::TreeUpdate>,
    is_active: bool,
}

impl HeadlessA11yAdapter {
    /// Create a new headless adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
            is_active: true,
        }
    }

    /// Set whether the adapter simulates an active screen reader.
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// History of all updates dispatched to this adapter.
    #[must_use]
    pub fn updates(&self) -> &[accesskit::TreeUpdate] {
        &self.updates
    }

    /// Latest update dispatched, if any.
    #[must_use]
    pub fn last_update(&self) -> Option<&accesskit::TreeUpdate> {
        self.updates.last()
    }

    /// Clear recorded update history.
    pub fn clear(&mut self) {
        self.updates.clear();
    }
}

impl PlatformA11yAdapter for HeadlessA11yAdapter {
    fn push_update(&mut self, update: accesskit::TreeUpdate) {
        self.updates.push(update);
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

/// A node in the web ARIA shadow DOM layer.
#[derive(Debug, Clone, PartialEq)]
pub struct WebAriaShadowNode {
    pub id: A11yId,
    pub html_tag: &'static str,
    pub role: Option<&'static str>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<A11yId>,
}

impl WebAriaShadowNode {
    /// Create a shadow node from an `A11yRole`.
    #[must_use]
    pub fn from_role(id: A11yId, role: A11yRole) -> Self {
        let (html_tag, aria_role) = match role {
            A11yRole::Button => ("button", None),
            A11yRole::TextInput => ("input", None),
            A11yRole::Checkbox => ("input", Some("checkbox")),
            A11yRole::Switch => ("button", Some("switch")),
            A11yRole::RadioButton => ("input", Some("radio")),
            A11yRole::Label => ("label", None),
            A11yRole::List => ("ul", None),
            A11yRole::ListItem => ("li", None),
            A11yRole::Header => ("h2", None),
            A11yRole::Link => ("a", None),
            A11yRole::Image => ("img", None),
            A11yRole::Dialog => ("dialog", None),
            A11yRole::ProgressBar => ("div", Some("progressbar")),
            A11yRole::Tab => ("button", Some("tab")),
            A11yRole::TabList => ("div", Some("tablist")),
            A11yRole::ScrollArea => ("div", Some("region")),
            A11yRole::Window | A11yRole::Generic | A11yRole::Slider => ("div", None),
        };

        Self {
            id,
            html_tag,
            role: aria_role,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Render shadow node to an HTML tag opening string.
    #[must_use]
    pub fn render_html_opening(&self) -> String {
        let mut out = format!("<{}", self.html_tag);
        if let Some(r) = self.role {
            out.push_str(&format!(r#" role="{}""#, r));
        }
        for (k, v) in &self.attributes {
            out.push_str(&format!(r#" {}="{}""#, k, v));
        }
        out.push('>');
        out
    }
}

/// Web ARIA shadow DOM adapter synchronizing accessibility updates with web browsers.
#[derive(Debug, Default)]
pub struct WebAriaAdapter {
    nodes: HashMap<A11yId, WebAriaShadowNode>,
    is_active: bool,
}

impl WebAriaAdapter {
    /// Create a new Web ARIA adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            is_active: true,
        }
    }

    /// Total shadow DOM nodes tracked.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Set whether web accessibility is active.
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Whether web accessibility is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_active
    }

    /// Look up shadow node.
    #[must_use]
    pub fn get_node(&self, id: A11yId) -> Option<&WebAriaShadowNode> {
        self.nodes.get(&id)
    }

    /// Synchronize shadow node with an `A11yNode`.
    pub fn sync_node(&mut self, node: &crate::node::A11yNode) {
        let mut shadow = WebAriaShadowNode::from_role(node.id, node.role);
        if let Some(ref l) = node.label {
            shadow.attributes.insert("aria-label".into(), l.clone());
        }
        if let Some(ref v) = node.value {
            shadow.attributes.insert("aria-valuenow".into(), v.clone());
        }
        if node.is_disabled {
            shadow
                .attributes
                .insert("aria-disabled".into(), "true".into());
        }
        if let Some(checked) = node.is_checked {
            shadow.attributes.insert(
                "aria-checked".into(),
                if checked {
                    "true".into()
                } else {
                    "false".into()
                },
            );
        }
        if let Some(live) = node.live_region {
            shadow.attributes.insert(
                "aria-live".into(),
                match live {
                    crate::announcement::AnnouncementPoliteness::Polite => "polite".into(),
                    crate::announcement::AnnouncementPoliteness::Assertive => "assertive".into(),
                },
            );
        }
        shadow.children = node.children.to_vec();
        self.nodes.insert(node.id, shadow);
    }
}
