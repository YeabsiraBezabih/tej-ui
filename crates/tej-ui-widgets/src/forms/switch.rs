//! Toggle switch control (shadcn `Switch`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_hal::Color;
use tej_ui_layout::{Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Switch widget.
#[derive(Debug, Default)]
pub struct SwitchState {
    pub bounds: Rect,
    pub is_checked: bool,
    pub thumb_offset_x: f32,
    pub node: u32,
}

/// Two-state toggle switch control.
#[derive(Clone)]
pub struct Switch {
    checked: Signal<bool>,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    disabled: bool,
    active_color: Color,
    inactive_color: Color,
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("checked", &self.checked.get())
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl Switch {
    /// Creates a new switch bound to a reactive boolean signal.
    #[must_use]
    pub fn new(checked: Signal<bool>) -> Self {
        Self {
            checked,
            on_change: None,
            disabled: false,
            active_color: Color::rgb(37, 99, 235), // Primary blue
            inactive_color: Color::rgb(229, 231, 235), // Neutral gray
        }
    }

    /// Callback invoked when toggle state changes.
    #[must_use]
    pub fn on_change<F>(mut self, cb: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(cb));
        self
    }

    /// Disables user interaction.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Primary active color when switch is checked.
    #[must_use]
    pub const fn active_color(&self) -> Color {
        self.active_color
    }

    /// Background color when switch is inactive.
    #[must_use]
    pub const fn inactive_color(&self) -> Color {
        self.inactive_color
    }

    /// Toggles the switch state.
    pub fn toggle(&self) {
        if !self.disabled {
            let next = !self.checked.get();
            self.checked.set(next);
            if let Some(cb) = &self.on_change {
                cb(next);
            }
        }
    }
}

impl View for Switch {
    type State = SwitchState;

    fn build(&self) -> Self::State {
        let is_checked = self.checked.get();
        SwitchState {
            bounds: Rect::from_origin_size(tej_ui_layout::Point::ZERO, Size::new(44.0, 24.0)),
            is_checked,
            thumb_offset_x: if is_checked { 20.0 } else { 2.0 },
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.is_checked = self.checked.get();
        state.thumb_offset_x = if state.is_checked { 20.0 } else { 2.0 };
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(44.0),
            height: tej_ui_layout::Dimension::Points(24.0),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        let track_color = if self.disabled {
            ctx.theme.colors.muted
        } else if state.is_checked {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.input
        };

        if ctx.is_focused(state.node) && !self.disabled {
            let ring = Rect::new(
                bounds.origin.x - 2.0,
                bounds.origin.y - 2.0,
                bounds.size.width + 4.0,
                bounds.size.height + 4.0,
            );
            ctx.fill_rounded_rect(ring, 14.0, ctx.theme.colors.ring);
        }

        ctx.fill_rounded_rect(bounds, 12.0, track_color);

        let thumb_rect = Rect::new(
            bounds.origin.x + state.thumb_offset_x,
            bounds.origin.y + 2.0,
            20.0,
            20.0,
        );
        ctx.fill_rounded_rect(thumb_rect, 10.0, ctx.theme.colors.primary_foreground);
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        if !self.disabled {
            let sig = self.checked;
            let on_change = self.on_change.clone();
            dispatcher.register_click(
                state.node,
                Arc::new(move || {
                    let next = !sig.get();
                    sig.set(next);
                    if let Some(cb) = &on_change {
                        cb(next);
                    }
                }),
            );
        }
    }
}
