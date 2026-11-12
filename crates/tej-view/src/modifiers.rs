use crate::view::View;
use std::fmt::Debug;
use tej_hal::Color;
use tej_layout::{Dimension, Insets};

/// Visual style properties applied via view modifiers.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct StyleProps {
    pub padding: Insets,
    pub margin: Insets,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Option<Color>,
    pub width: Dimension,
    pub height: Dimension,
}

/// A wrapper view that decorates an inner view with layout and visual style properties.
#[derive(Clone, Debug)]
pub struct Styled<V: View> {
    pub inner: V,
    pub style: StyleProps,
}

impl<V: View> Styled<V> {
    /// Add padding around all four edges.
    pub fn padding(mut self, value: f32) -> Self {
        self.style.padding = Insets::all(value);
        self
    }

    /// Add custom edge padding insets.
    pub fn padding_insets(mut self, insets: Insets) -> Self {
        self.style.padding = insets;
        self
    }

    /// Set solid background color.
    pub fn background(mut self, color: Color) -> Self {
        self.style.background = Some(color);
        self
    }

    /// Set rounded corner radius.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.style.corner_radius = radius;
        self
    }

    /// Set stroke border.
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style.border_width = width;
        self.style.border_color = Some(color);
        self
    }

    /// Set width constraint.
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.width = width;
        self
    }

    /// Set height constraint.
    pub fn height(mut self, height: Dimension) -> Self {
        self.style.height = height;
        self
    }
}

/// State of a mounted Styled view.
#[derive(Debug)]
pub struct StyledState<S: Debug> {
    pub inner_state: S,
    pub style: StyleProps,
}

impl<V: View> View for Styled<V> {
    type State = StyledState<V::State>;

    fn build(&self) -> Self::State {
        StyledState {
            inner_state: self.inner.build(),
            style: self.style,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.inner.rebuild(&mut state.inner_state);
        state.style = self.style;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.inner.patch(&mut state.inner_state, dirty_bits);
    }
}

/// Ergonomic extension trait adding composable modifiers to every `View`.
pub trait ViewExt: View + Sized {
    /// Wrap view in a styled container.
    fn styled(self) -> Styled<Self> {
        Styled {
            inner: self,
            style: StyleProps::default(),
        }
    }

    /// Add padding on all sides.
    fn padding(self, value: f32) -> Styled<Self> {
        self.styled().padding(value)
    }

    /// Add custom edge insets.
    fn padding_insets(self, insets: Insets) -> Styled<Self> {
        self.styled().padding_insets(insets)
    }

    /// Add a solid background color.
    fn background(self, color: Color) -> Styled<Self> {
        self.styled().background(color)
    }

    /// Add corner rounding.
    fn corner_radius(self, radius: f32) -> Styled<Self> {
        self.styled().corner_radius(radius)
    }

    /// Add border stroke and color.
    fn border(self, width: f32, color: Color) -> Styled<Self> {
        self.styled().border(width, color)
    }

    /// Set explicit width.
    fn width(self, width: Dimension) -> Styled<Self> {
        self.styled().width(width)
    }

    /// Set explicit height.
    fn height(self, height: Dimension) -> Styled<Self> {
        self.styled().height(height)
    }
}

// Blanket implementation so all types implementing View gain ViewExt modifiers automatically!
impl<V: View> ViewExt for V {}
