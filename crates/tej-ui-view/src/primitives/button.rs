use crate::view::View;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// Click callback type.
pub type ClickHandler = Arc<dyn Fn() + Send + Sync>;

/// Visual presentation variant for a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

/// Sizing specification for a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Sm,
    #[default]
    Default,
    Lg,
    Icon,
}

/// Interactive push button widget.
#[derive(Clone)]
pub struct Button<V: View> {
    child: V,
    on_click: Option<ClickHandler>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    width: Option<tej_ui_layout::Dimension>,
}

impl<V: View> fmt::Debug for Button<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Button")
            .field("child", &self.child)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("disabled", &self.disabled)
            .field("has_click_handler", &self.on_click.is_some())
            .finish()
    }
}

/// State retained by a mounted Button.
pub struct ButtonState<S: Debug> {
    pub child_state: S,
    pub on_click: Option<ClickHandler>,
    pub node: u32,
}

impl<S: Debug> fmt::Debug for ButtonState<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ButtonState")
            .field("child_state", &self.child_state)
            .field("has_click_handler", &self.on_click.is_some())
            .finish()
    }
}

impl<V: View> Button<V> {
    /// Create a button wrapping child view `child`.
    pub fn new(child: V) -> Self {
        Self {
            child,
            on_click: None,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            width: None,
        }
    }

    /// Sets explicit button width.
    #[must_use]
    pub const fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the visual style variant.
    #[must_use]
    pub const fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the button sizing.
    #[must_use]
    pub const fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Sets whether the button is disabled.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Register a click event callback.
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, handler: F) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl<V: View> View for Button<V> {
    type State = ButtonState<V::State>;

    fn build(&self) -> Self::State {
        ButtonState {
            child_state: self.child.build(),
            on_click: self.on_click.clone(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.child.rebuild(&mut state.child_state);
        state.on_click = self.on_click.clone();
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.child.patch(&mut state.child_state, dirty_bits);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let (padding, min_height) = match self.size {
            ButtonSize::Sm => (tej_ui_layout::Insets::symmetric(4.0, 10.0), 28.0),
            ButtonSize::Default => (tej_ui_layout::Insets::symmetric(8.0, 16.0), 36.0),
            ButtonSize::Lg => (tej_ui_layout::Insets::symmetric(10.0, 24.0), 44.0),
            ButtonSize::Icon => (tej_ui_layout::Insets::all(8.0), 36.0),
        };
        let child_node = self.child.layout(&mut state.child_state, arena);
        let child_w = arena.styles[child_node as usize].width;
        let width = if let Some(w) = self.width {
            w
        } else {
            match child_w {
                tej_ui_layout::Dimension::Points(p) => {
                    tej_ui_layout::Dimension::Points(p + padding.left + padding.right)
                }
                other => other,
            }
        };
        let style = tej_ui_layout::LayoutStyle {
            width,
            padding,
            height: tej_ui_layout::Dimension::Points(min_height),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        arena.add_child(node, child_node);
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);

        let is_hovered = ctx.is_hovered(state.node);
        let is_pressed = ctx.is_pressed(state.node);
        let is_focused = ctx.is_focused(state.node);

        let radius = match self.size {
            ButtonSize::Sm => ctx.theme.radii.sm,
            ButtonSize::Default | ButtonSize::Lg | ButtonSize::Icon => ctx.theme.radii.md,
        };

        let bg = match self.variant {
            ButtonVariant::Default => {
                if is_pressed {
                    ctx.theme.colors.accent
                } else if is_hovered {
                    tej_ui_hal::Color::from_rgb_u32(0x2563EB)
                } else {
                    ctx.theme.colors.primary
                }
            }
            ButtonVariant::Secondary => {
                if is_hovered {
                    ctx.theme.colors.muted
                } else {
                    ctx.theme.colors.secondary
                }
            }
            ButtonVariant::Destructive => {
                if is_hovered {
                    tej_ui_hal::Color::from_rgb_u32(0xB91C1C)
                } else {
                    ctx.theme.colors.destructive
                }
            }
            ButtonVariant::Outline => {
                ctx.fill_rounded_rect(bounds, radius, ctx.theme.colors.border);
                let inner = tej_ui_layout::Rect::new(
                    bounds.origin.x + 1.0,
                    bounds.origin.y + 1.0,
                    (bounds.size.width - 2.0).max(0.0),
                    (bounds.size.height - 2.0).max(0.0),
                );
                let inner_bg = if is_hovered {
                    ctx.theme.colors.secondary
                } else {
                    ctx.theme.colors.card
                };
                ctx.fill_rounded_rect(inner, (radius - 1.0).max(0.0), inner_bg);
                tej_ui_hal::Color::TRANSPARENT
            }
            ButtonVariant::Ghost => {
                if is_hovered {
                    ctx.theme.colors.secondary
                } else {
                    tej_ui_hal::Color::TRANSPARENT
                }
            }
            ButtonVariant::Link => tej_ui_hal::Color::TRANSPARENT,
        };

        if is_focused && !self.disabled {
            let ring = tej_ui_layout::Rect::new(
                bounds.origin.x - 2.0,
                bounds.origin.y - 2.0,
                bounds.size.width + 4.0,
                bounds.size.height + 4.0,
            );
            ctx.fill_rounded_rect(ring, radius + 2.0, ctx.theme.colors.ring);
        }

        if bg != tej_ui_hal::Color::TRANSPARENT {
            ctx.fill_rounded_rect(bounds, radius, bg);
        }
        self.child.paint(&state.child_state, ctx);
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        if !self.disabled {
            if let Some(handler) = &self.on_click {
                dispatcher.register_click(state.node, Arc::clone(handler));
            }
        }
        self.child.register_events(&state.child_state, dispatcher);
    }
}
