//! Interactive event dispatching, hit-testing, and focus management.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_hal::{MouseButton, UiEvent};
use tej_ui_layout::LayoutArena;

/// Standard platform cursor shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    NotAllowed,
    ResizeCol,
    ResizeRow,
}

/// Execution phase of two-phase event routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// Tunneling/capturing from root down to target ancestor.
    Capturing,
    /// Direct dispatch on the targeted node.
    Target,
    /// Bubbling from target back up to root.
    Bubbling,
}

/// Propagation directive controlling event traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventPropagation {
    /// Continue propagation to next node in the route.
    #[default]
    Continue,
    /// Request surface repaint and continue propagation.
    Repaint,
    /// Stop propagation immediately across remaining phases and nodes.
    Stop,
}

/// Two-phase event listener callback type.
pub type TwoPhaseEventListener =
    Arc<dyn Fn(&UiEvent, EventPhase, u32) -> EventPropagation + Send + Sync>;

/// Outcome of dispatching an event through the view tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventDispatchResult {
    pub handled: bool,
    pub needs_repaint: bool,
    pub cursor: CursorIcon,
}

/// Click callback type for layout nodes.
pub type ClickListener = Arc<dyn Fn() + Send + Sync>;

/// Text input callback type for layout nodes.
pub type TextInputListener = Arc<dyn Fn(&str) + Send + Sync>;

/// Key press callback type for layout nodes.
pub type KeyListener = Arc<dyn Fn(u32) + Send + Sync>;

/// Central interactive event dispatcher routing hardware events to laid-out nodes.
#[derive(Default)]
pub struct EventDispatcher {
    pub focused_node: Option<u32>,
    pub hovered_node: Option<u32>,
    pub pressed_node: Option<u32>,
    click_handlers: HashMap<u32, ClickListener>,
    text_handlers: HashMap<u32, TextInputListener>,
    key_handlers: HashMap<u32, KeyListener>,
    capture_handlers: HashMap<u32, Vec<TwoPhaseEventListener>>,
    bubble_handlers: HashMap<u32, Vec<TwoPhaseEventListener>>,
    tab_order: Vec<u32>,
}

impl Debug for EventDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("focused_node", &self.focused_node)
            .field("hovered_node", &self.hovered_node)
            .field("pressed_node", &self.pressed_node)
            .field("click_listeners", &self.click_handlers.len())
            .field("text_listeners", &self.text_handlers.len())
            .field("capture_listeners", &self.capture_handlers.len())
            .field("bubble_listeners", &self.bubble_handlers.len())
            .finish_non_exhaustive()
    }
}

impl EventDispatcher {
    /// Creates a new event dispatcher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a capturing (tunneling) interceptor invoked from root to target.
    pub fn register_capture(&mut self, node: u32, handler: TwoPhaseEventListener) {
        self.capture_handlers.entry(node).or_default().push(handler);
    }

    /// Registers a bubbling listener invoked from target back to root.
    pub fn register_bubble(&mut self, node: u32, handler: TwoPhaseEventListener) {
        self.bubble_handlers.entry(node).or_default().push(handler);
    }

    /// Registers a click callback for a layout node.
    pub fn register_click(&mut self, node: u32, handler: Arc<dyn Fn() + Send + Sync>) {
        self.click_handlers.insert(node, handler);
        if !self.tab_order.contains(&node) {
            self.tab_order.push(node);
        }
    }

    /// Registers a text input callback for a layout node.
    pub fn register_text_input(&mut self, node: u32, handler: Arc<dyn Fn(&str) + Send + Sync>) {
        self.text_handlers.insert(node, handler);
        if !self.tab_order.contains(&node) {
            self.tab_order.push(node);
        }
    }

    /// Registers a key down callback for a layout node.
    pub fn register_key_down(&mut self, node: u32, handler: Arc<dyn Fn(u32) + Send + Sync>) {
        self.key_handlers.insert(node, handler);
    }

    /// Clears registered handlers before rebuilding the event tree.
    pub fn clear_handlers(&mut self) {
        self.click_handlers.clear();
        self.text_handlers.clear();
        self.key_handlers.clear();
        self.capture_handlers.clear();
        self.bubble_handlers.clear();
        self.tab_order.clear();
    }

    /// Sets explicit keyboard tab traversal order.
    pub fn set_tab_order(&mut self, order: Vec<u32>) {
        self.tab_order = order;
    }

    /// Sets keyboard focus directly to a node.
    pub fn focus(&mut self, node: Option<u32>) {
        self.focused_node = node;
    }

    /// Advances keyboard focus to the next interactive node.
    pub fn focus_next(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }
        let next_idx = match self.focused_node {
            Some(curr) => {
                if let Some(pos) = self.tab_order.iter().position(|&n| n == curr) {
                    (pos + 1) % self.tab_order.len()
                } else {
                    0
                }
            }
            None => 0,
        };
        self.focused_node = Some(self.tab_order[next_idx]);
    }

    /// Moves keyboard focus to the previous interactive node.
    pub fn focus_prev(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }
        let prev_idx = match self.focused_node {
            Some(curr) => {
                if let Some(pos) = self.tab_order.iter().position(|&n| n == curr) {
                    if pos == 0 {
                        self.tab_order.len().saturating_sub(1)
                    } else {
                        pos - 1
                    }
                } else {
                    0
                }
            }
            None => self.tab_order.len().saturating_sub(1),
        };
        self.focused_node = Some(self.tab_order[prev_idx]);
    }

    /// Returns `true` if `node` has keyboard focus.
    #[must_use]
    pub fn is_focused(&self, node: u32) -> bool {
        self.focused_node == Some(node)
    }

    /// Returns `true` if `node` or any of its descendants is hovered.
    #[must_use]
    pub fn is_hovered(&self, arena: &LayoutArena, node: u32) -> bool {
        if let Some(mut curr) = self.hovered_node {
            loop {
                if curr == node {
                    return true;
                }
                if let Some(Some(parent)) = arena.parents.get(curr as usize) {
                    curr = *parent;
                } else {
                    break;
                }
            }
        }
        false
    }

    /// Returns `true` if `node` is currently pressed.
    #[must_use]
    pub fn is_pressed(&self, node: u32) -> bool {
        self.pressed_node == Some(node)
    }

    /// Determines cursor shape based on currently hovered node and its ancestors.
    #[must_use]
    pub fn current_cursor(&self, arena: &LayoutArena) -> CursorIcon {
        if let Some(mut node) = self.hovered_node {
            loop {
                if self.text_handlers.contains_key(&node) {
                    return CursorIcon::Text;
                }
                if self.click_handlers.contains_key(&node) {
                    return CursorIcon::Pointer;
                }
                if let Some(Some(parent)) = arena.parents.get(node as usize) {
                    node = *parent;
                } else {
                    break;
                }
            }
        }
        CursorIcon::Default
    }

    /// Finds the nearest ancestor (or self) that handles click events.
    #[must_use]
    pub fn find_click_target(&self, arena: &LayoutArena, mut node: u32) -> Option<u32> {
        loop {
            if self.click_handlers.contains_key(&node) {
                return Some(node);
            }
            if let Some(Some(parent)) = arena.parents.get(node as usize) {
                node = *parent;
            } else {
                return None;
            }
        }
    }

    /// Executes the capturing phase from root down to target.
    fn dispatch_capture(&self, event: &UiEvent, path: &[u32]) -> bool {
        for &node in path {
            if let Some(handlers) = self.capture_handlers.get(&node) {
                for handler in handlers {
                    if handler(event, EventPhase::Capturing, node) == EventPropagation::Stop {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Executes the bubbling phase from target back up to root.
    fn dispatch_bubble(
        &self,
        event: &UiEvent,
        path: &[u32],
        result: &mut EventDispatchResult,
    ) -> bool {
        for &node in path.iter().rev() {
            if let Some(handlers) = self.bubble_handlers.get(&node) {
                for handler in handlers {
                    match handler(event, EventPhase::Bubbling, node) {
                        EventPropagation::Repaint => {
                            result.needs_repaint = true;
                            result.handled = true;
                        }
                        EventPropagation::Stop => {
                            result.handled = true;
                            return false;
                        }
                        EventPropagation::Continue => {}
                    }
                }
            }
        }
        true
    }

    /// Dispatches an incoming hardware `UiEvent` into the laid-out view hierarchy
    /// using two-phase event routing (Capturing -> Target -> Bubbling).
    pub fn dispatch(
        &mut self,
        event: &UiEvent,
        arena: &LayoutArena,
        root: u32,
    ) -> EventDispatchResult {
        let mut result = EventDispatchResult::default();

        // 1. Identify target node and compute root-to-target path
        let target_node = match event {
            UiEvent::MouseMove { position }
            | UiEvent::MouseDown { position, .. }
            | UiEvent::MouseUp { position, .. }
            | UiEvent::Scroll { position, .. } => arena.hit_test(root, *position),
            UiEvent::TextInput { .. } | UiEvent::KeyDown { .. } | UiEvent::KeyUp { .. } => {
                self.focused_node
            }
            _ => None,
        };

        let path = target_node.map(|t| arena.path_from_root(t));

        // 2. Phase 1: Capturing (Tunneling) from root down to target
        if let Some(ref p) = path
            && !self.dispatch_capture(event, p)
        {
            result.handled = true;
            result.cursor = self.current_cursor(arena);
            return result;
        }

        // 3. Phase 2: Target invocation
        match event {
            UiEvent::MouseMove { position } => {
                let hit = arena.hit_test(root, *position);
                if hit != self.hovered_node {
                    self.hovered_node = hit;
                    result.needs_repaint = true;
                }
                result.cursor = self.current_cursor(arena);
            }
            UiEvent::MouseDown {
                button: MouseButton::Left,
                position,
            } => {
                let hit = arena.hit_test(root, *position);
                let target = hit.and_then(|n| self.find_click_target(arena, n)).or(hit);
                self.pressed_node = target;
                if target != self.focused_node {
                    self.focused_node = target;
                    result.needs_repaint = true;
                }
                result.handled = target.is_some();
            }
            UiEvent::MouseUp {
                button: MouseButton::Left,
                position,
            } => {
                let hit = arena.hit_test(root, *position);
                let target = hit.and_then(|n| self.find_click_target(arena, n));
                if let Some(pressed) = self.pressed_node
                    && Some(pressed) == target
                    && let Some(handler) = self.click_handlers.get(&pressed)
                {
                    handler();
                    result.handled = true;
                    result.needs_repaint = true;
                }
                self.pressed_node = None;
                result.needs_repaint = true;
            }
            UiEvent::TextInput { text } => {
                if let Some(focused) = self.focused_node
                    && let Some(handler) = self.text_handlers.get(&focused)
                {
                    handler(text);
                    result.handled = true;
                    result.needs_repaint = true;
                }
            }
            UiEvent::KeyDown { key_code } => match *key_code {
                9 => {
                    // Tab key
                    self.focus_next();
                    result.handled = true;
                    result.needs_repaint = true;
                }
                13 | 32 => {
                    // Enter (13) or Space (32) activates focused clickable control
                    if let Some(focused) = self.focused_node
                        && let Some(handler) = self.click_handlers.get(&focused)
                    {
                        handler();
                        result.handled = true;
                        result.needs_repaint = true;
                    }
                }
                code => {
                    let target = self.focused_node.or(Some(root));
                    if let Some(node) = target
                        && let Some(handler) = self.key_handlers.get(&node)
                    {
                        handler(code);
                        result.handled = true;
                        result.needs_repaint = true;
                    }
                }
            },
            _ => {}
        }

        // 4. Phase 3: Bubbling from target back up to root
        if let Some(ref p) = path {
            self.dispatch_bubble(event, p, &mut result);
        }

        result.cursor = self.current_cursor(arena);
        result
    }
}
