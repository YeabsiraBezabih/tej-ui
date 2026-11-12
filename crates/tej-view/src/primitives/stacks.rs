use crate::view::View;
use std::fmt::Debug;

/// Vertical linear stack container.
#[derive(Debug, Clone)]
pub struct VStack<V: View> {
    pub children: V,
    pub spacing: u32,
    pub padding: u32,
}

impl<V: View> VStack<V> {
    /// Create a new vertical stack.
    pub fn new(children: V) -> Self {
        Self {
            children,
            spacing: 0,
            padding: 0,
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
}

/// State of a mounted VStack.
#[derive(Debug)]
pub struct StackState<S: Debug> {
    pub child_state: S,
    pub spacing: u32,
    pub padding: u32,
}

impl<V: View> View for VStack<V> {
    type State = StackState<V::State>;

    fn build(&self) -> Self::State {
        StackState {
            child_state: self.children.build(),
            spacing: self.spacing,
            padding: self.padding,
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
}

/// Horizontal linear stack container.
#[derive(Debug, Clone)]
pub struct HStack<V: View> {
    pub children: V,
    pub spacing: u32,
    pub padding: u32,
}

impl<V: View> HStack<V> {
    /// Create a new horizontal stack.
    pub fn new(children: V) -> Self {
        Self {
            children,
            spacing: 0,
            padding: 0,
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
}

impl<V: View> View for HStack<V> {
    type State = StackState<V::State>;

    fn build(&self) -> Self::State {
        StackState {
            child_state: self.children.build(),
            spacing: self.spacing,
            padding: self.padding,
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
}
