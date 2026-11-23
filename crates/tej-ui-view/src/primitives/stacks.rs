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
    pub flex_wrap: tej_ui_layout::FlexWrap,
    pub flex_grow: f32,
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
            flex_wrap: tej_ui_layout::FlexWrap::NoWrap,
            flex_grow: 0.0,
            container_query: None,
        }
    }

    /// Set spacing between elements in pixels.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding around stack in pixels.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Set explicit width dimension.
    pub fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set explicit height dimension.
    pub fn height(mut self, height: tej_ui_layout::Dimension) -> Self {
        self.height = height;
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and take remaining space).
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Set flex wrap behavior.
    pub fn flex_wrap(mut self, wrap: tej_ui_layout::FlexWrap) -> Self {
        self.flex_wrap = wrap;
        self
    }

    /// Attach a responsive container query for dynamic layout adaptation.
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
        let child_node = self.children.layout(&mut state.child_state, arena);
        arena.styles[child_node as usize].direction = tej_ui_layout::FlexDirection::Column;
        arena.styles[child_node as usize].flex_wrap = self.flex_wrap;
        arena.styles[child_node as usize].container_query = self.container_query;
        arena.styles[child_node as usize].spacing = self.spacing as f32;
        arena.styles[child_node as usize].padding = tej_ui_layout::Insets::all(self.padding as f32);
        if self.width != tej_ui_layout::Dimension::Auto {
            arena.styles[child_node as usize].width = self.width;
        }
        if self.height != tej_ui_layout::Dimension::Auto {
            arena.styles[child_node as usize].height = self.height;
        }
        if self.flex_grow > 0.0 {
            arena.styles[child_node as usize].flex_grow = self.flex_grow;
        }
        state.node = child_node;
        child_node
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
}

/// Horizontal linear stack container.
#[derive(Debug, Clone)]
pub struct HStack<V: View> {
    pub children: V,
    pub spacing: u32,
    pub padding: u32,
    pub width: tej_ui_layout::Dimension,
    pub height: tej_ui_layout::Dimension,
    pub flex_wrap: tej_ui_layout::FlexWrap,
    pub flex_grow: f32,
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
            flex_wrap: tej_ui_layout::FlexWrap::NoWrap,
            flex_grow: 0.0,
            container_query: None,
        }
    }

    /// Set spacing between elements in pixels.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding around stack in pixels.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Set explicit width dimension.
    pub fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set explicit height dimension.
    pub fn height(mut self, height: tej_ui_layout::Dimension) -> Self {
        self.height = height;
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and take remaining space).
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Set flex wrap behavior.
    pub fn flex_wrap(mut self, wrap: tej_ui_layout::FlexWrap) -> Self {
        self.flex_wrap = wrap;
        self
    }

    /// Attach a responsive container query for dynamic layout adaptation.
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
        let child_node = self.children.layout(&mut state.child_state, arena);
        arena.styles[child_node as usize].direction = tej_ui_layout::FlexDirection::Row;
        arena.styles[child_node as usize].flex_wrap = self.flex_wrap;
        arena.styles[child_node as usize].container_query = self.container_query;
        arena.styles[child_node as usize].spacing = self.spacing as f32;
        arena.styles[child_node as usize].padding = tej_ui_layout::Insets::all(self.padding as f32);
        if self.width != tej_ui_layout::Dimension::Auto {
            arena.styles[child_node as usize].width = self.width;
        }
        if self.height != tej_ui_layout::Dimension::Auto {
            arena.styles[child_node as usize].height = self.height;
        }
        if self.flex_grow > 0.0 {
            arena.styles[child_node as usize].flex_grow = self.flex_grow;
        }
        state.node = child_node;
        child_node
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
}
