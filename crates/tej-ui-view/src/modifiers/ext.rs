//! Ergonomic extension trait adding composable modifiers to every `View`.

use super::props::StyleProps;
use super::styled::Styled;
use crate::view::View;
use std::fmt::Debug;
use tej_ui_hal::Color;
use tej_ui_layout::{Dimension, Insets};

/// Composable view modifier methods available on every `View`.
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

    /// Add uniform margin on all sides.
    fn margin(self, value: f32) -> Styled<Self> {
        self.styled().margin(value)
    }

    /// Add custom edge margin insets.
    fn margin_insets(self, insets: Insets) -> Styled<Self> {
        self.styled().margin_insets(insets)
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

    /// Enable or configure flex wrapping along the cross axis.
    fn flex_wrap(self, wrap: tej_ui_layout::FlexWrap) -> Styled<Self> {
        self.styled().flex_wrap(wrap)
    }

    /// Set flex grow factor (e.g. 1.0 to expand and fill remaining space).
    fn flex_grow(self, flex_grow: f32) -> Styled<Self> {
        self.styled().flex_grow(flex_grow)
    }

    /// Set column span in CSS Grid layout.
    fn col_span(self, span: u16) -> Styled<Self> {
        self.styled().col_span(span)
    }

    /// Set row span in CSS Grid layout.
    fn row_span(self, span: u16) -> Styled<Self> {
        self.styled().row_span(span)
    }

    /// Attach a responsive container query for dynamic layout adaptation.
    fn container_query(self, query: tej_ui_layout::ContainerQuery) -> Styled<Self> {
        self.styled().container_query(query)
    }

    /// Set layout display mode (Flex or Grid).
    fn display(self, display: tej_ui_layout::DisplayMode) -> Styled<Self> {
        self.styled().display(display)
    }

    /// Set main axis flex layout direction.
    fn flex_direction(self, direction: tej_ui_layout::FlexDirection) -> Styled<Self> {
        self.styled().flex_direction(direction)
    }

    /// Set cross-axis child item alignment.
    fn align_items(self, align: tej_ui_layout::AlignItems) -> Styled<Self> {
        self.styled().align_items(align)
    }

    /// Set main-axis item distribution.
    fn justify_content(self, justify: tej_ui_layout::JustifyContent) -> Styled<Self> {
        self.styled().justify_content(justify)
    }

    /// Set gap spacing between items.
    fn gap(self, gap: f32) -> Styled<Self> {
        self.styled().gap(gap)
    }

    /// Set layer opacity (0.0 to 1.0).
    fn opacity(self, opacity: f32) -> Styled<Self> {
        self.styled().opacity(opacity)
    }

    /// Set elevation drop shadow.
    fn shadow(self, shadow: crate::theme::Shadow) -> Styled<Self> {
        self.styled().shadow(shadow)
    }

    /// Set interactive pointer cursor icon.
    fn cursor(self, cursor: crate::events::CursorIcon) -> Styled<Self> {
        self.styled().cursor(cursor)
    }

    /// Apply styles from a typed CssStyleSheet.
    fn css(self, sheet: &crate::css::CssStyleSheet) -> Styled<Self> {
        self.styled().css(sheet)
    }

    /// Wrap view in a typed ambient context provider.
    fn provide_context<T: std::any::Any + Send + Sync + Clone + Debug>(
        self,
        value: T,
    ) -> crate::context_provider::ContextProvider<T, Self> {
        crate::context_provider::ContextProvider::new(value, self)
    }

    /// Wrap view with an ambient design theme provider.
    fn theme(
        self,
        theme: crate::theme::Theme,
    ) -> crate::context_provider::ContextProvider<crate::theme::Theme, Self> {
        self.provide_context(theme)
    }
}

// Blanket implementation so all types implementing View gain ViewExt modifiers automatically!
impl<V: View> ViewExt for V {}
