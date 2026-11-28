use crate::view::View;
use std::fmt::Debug;

/// Vertical linear stack container.
#[derive(Debug, Clone)]
pub struct VStack<V: View> {
    pub children: V,
    pub spacing: u32,
    pub padding: u32,
    pub width: tej_ui_layout::Dimension,
    pub height: tej_ui_layout::Dimension,
    pub min_width: tej_ui_layout::Dimension,
    pub max_width: tej_ui_layout::Dimension,
    pub align_items: tej_ui_layout::AlignItems,
    pub justify_content: tej_ui_layout::JustifyContent,
    pub flex_wrap: tej_ui_layout::FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub container_query: Option<tej_ui_layout::ContainerQuery>,
}

impl<V: View> VStack<V> {
    /// Create a new vertical stack.
    pub fn new(children: V) -> Self {
        Self {
            children,
            spacing: 0,
            padding: 0,
            width: tej_ui_layout::Dimension::Auto,
            height: tej_ui_layout::Dimension::Auto,
            min_width: tej_ui_layout::Dimension::Auto,
            max_width: tej_ui_layout::Dimension::Auto,
            align_items: tej_ui_layout::AlignItems::FlexStart,
            justify_content: tej_ui_layout::JustifyContent::FlexStart,
            flex_wrap: tej_ui_layout::FlexWrap::NoWrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            container_query: None,
        }
    }

    /// Set spacing between elements in pixels.
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding around stack in pixels.
    #[must_use]
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Set explicit width dimension.
    #[must_use]
    pub fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set explicit height dimension.
    #[must_use]
    pub fn height(mut self, height: tej_ui_layout::Dimension) -> Self {
        self.height = height;
        self
    }

    /// Set minimum width constraint.
    #[must_use]
    pub fn min_width(mut self, min_width: tej_ui_layout::Dimension) -> Self {
        self.min_width = min_width;
        self
    }

    /// Set maximum width constraint.
    #[must_use]
    pub fn max_width(mut self, max_width: tej_ui_layout::Dimension) -> Self {
        self.max_width = max_width;
        self
    }

    /// Set cross-axis item alignment.
    #[must_use]
    pub fn align_items(mut self, align: tej_ui_layout::AlignItems) -> Self {
        self.align_items = align;
        self
    }

    /// Set main-axis item distribution.
    #[must_use]
    pub fn justify_content(mut self, justify: tej_ui_layout::JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and take remaining space).
    #[must_use]
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Set flex shrink factor (default 1.0; set to 0.0 to prevent shrinking).
    #[must_use]
    pub fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.flex_shrink = flex_shrink;
        self
    }

    /// Set flex wrap behavior.
    #[must_use]
    pub fn flex_wrap(mut self, wrap: tej_ui_layout::FlexWrap) -> Self {
        self.flex_wrap = wrap;
        self
    }

    /// Attach a responsive container query for dynamic layout adaptation.
    #[must_use]
    pub fn container_query(mut self, query: tej_ui_layout::ContainerQuery) -> Self {
        self.container_query = Some(query);
        self
    }
}

/// State of a mounted stack.
#[derive(Debug)]
pub struct StackState<S: Debug> {
    pub child_state: S,
    pub spacing: u32,
    pub padding: u32,
    pub node: u32,
}

impl<V: View> View for VStack<V> {
    type State = StackState<V::State>;

    fn build(&self) -> Self::State {
        StackState {
            child_state: self.children.build(),
            spacing: self.spacing,
            padding: self.padding,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.children.rebuild(&mut state.child_state);
        state.spacing = self.spacing;
        state.padding = self.padding;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.children.patch(&mut state.child_state, dirty_bits);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Column,
            align_items: self.align_items,
            justify_content: self.justify_content,
            flex_wrap: self.flex_wrap,
            container_query: self.container_query,
            spacing: f32::from(u16::try_from(self.spacing).unwrap_or(u16::MAX)),
            padding: tej_ui_layout::Insets::all(f32::from(
                u16::try_from(self.padding).unwrap_or(u16::MAX),
            )),
            width: self.width,
            height: self.height,
            min_width: self.min_width,
            max_width: self.max_width,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            ..Default::default()
        };
        let container = arena.allocate_node(style);
        self.children
            .layout_children(&mut state.child_state, arena, container);
        state.node = container;
        container
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        self.children.paint(&state.child_state, ctx);
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        self.children
            .register_events(&state.child_state, dispatcher);
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &tej_ui_layout::LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        let id = tree.allocate_id();
        let bounds = arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let node =
            tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::Generic).with_bounds(bounds);
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
        self.children
            .accessibility(&state.child_state, arena, tree, Some(id));
    }
}

/// Horizontal linear stack container.
#[derive(Debug, Clone)]
pub struct HStack<V: View> {
    pub children: V,
    pub spacing: u32,
    pub padding: u32,
    pub width: tej_ui_layout::Dimension,
    pub height: tej_ui_layout::Dimension,
    pub min_width: tej_ui_layout::Dimension,
    pub max_width: tej_ui_layout::Dimension,
    pub align_items: tej_ui_layout::AlignItems,
    pub justify_content: tej_ui_layout::JustifyContent,
    pub flex_wrap: tej_ui_layout::FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub container_query: Option<tej_ui_layout::ContainerQuery>,
}

impl<V: View> HStack<V> {
    /// Create a new horizontal stack.
    pub fn new(children: V) -> Self {
        Self {
            children,
            spacing: 0,
            padding: 0,
            width: tej_ui_layout::Dimension::Auto,
            height: tej_ui_layout::Dimension::Auto,
            min_width: tej_ui_layout::Dimension::Auto,
            max_width: tej_ui_layout::Dimension::Auto,
            align_items: tej_ui_layout::AlignItems::FlexStart,
            justify_content: tej_ui_layout::JustifyContent::FlexStart,
            flex_wrap: tej_ui_layout::FlexWrap::NoWrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            container_query: None,
        }
    }

    /// Set spacing between elements in pixels.
    #[must_use]
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding around stack in pixels.
    #[must_use]
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Set explicit width dimension.
    #[must_use]
    pub fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set explicit height dimension.
    #[must_use]
    pub fn height(mut self, height: tej_ui_layout::Dimension) -> Self {
        self.height = height;
        self
    }

    /// Set minimum width constraint.
    #[must_use]
    pub fn min_width(mut self, min_width: tej_ui_layout::Dimension) -> Self {
        self.min_width = min_width;
        self
    }

    /// Set maximum width constraint.
    #[must_use]
    pub fn max_width(mut self, max_width: tej_ui_layout::Dimension) -> Self {
        self.max_width = max_width;
        self
    }

    /// Set cross-axis item alignment.
    #[must_use]
    pub fn align_items(mut self, align: tej_ui_layout::AlignItems) -> Self {
        self.align_items = align;
        self
    }

    /// Set main-axis item distribution.
    #[must_use]
    pub fn justify_content(mut self, justify: tej_ui_layout::JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and take remaining space).
    #[must_use]
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Set flex shrink factor (default 1.0; set to 0.0 to prevent shrinking).
    #[must_use]
    pub fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.flex_shrink = flex_shrink;
        self
    }

    /// Set flex wrap behavior.
    #[must_use]
    pub fn flex_wrap(mut self, wrap: tej_ui_layout::FlexWrap) -> Self {
        self.flex_wrap = wrap;
        self
    }

    /// Attach a responsive container query for dynamic layout adaptation.
    #[must_use]
    pub fn container_query(mut self, query: tej_ui_layout::ContainerQuery) -> Self {
        self.container_query = Some(query);
        self
    }
}

impl<V: View> View for HStack<V> {
    type State = StackState<V::State>;

    fn build(&self) -> Self::State {
        StackState {
            child_state: self.children.build(),
            spacing: self.spacing,
            padding: self.padding,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.children.rebuild(&mut state.child_state);
        state.spacing = self.spacing;
        state.padding = self.padding;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.children.patch(&mut state.child_state, dirty_bits);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Row,
            align_items: self.align_items,
            justify_content: self.justify_content,
            flex_wrap: self.flex_wrap,
            container_query: self.container_query,
            spacing: f32::from(u16::try_from(self.spacing).unwrap_or(u16::MAX)),
            padding: tej_ui_layout::Insets::all(f32::from(
                u16::try_from(self.padding).unwrap_or(u16::MAX),
            )),
            width: self.width,
            height: self.height,
            min_width: self.min_width,
            max_width: self.max_width,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            ..Default::default()
        };
        let container = arena.allocate_node(style);
        self.children
            .layout_children(&mut state.child_state, arena, container);
        state.node = container;
        container
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        self.children.paint(&state.child_state, ctx);
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        self.children
            .register_events(&state.child_state, dispatcher);
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &tej_ui_layout::LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        let id = tree.allocate_id();
        let bounds = arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let node =
            tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::Generic).with_bounds(bounds);
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
        self.children
            .accessibility(&state.child_state, arena, tree, Some(id));
    }
}
