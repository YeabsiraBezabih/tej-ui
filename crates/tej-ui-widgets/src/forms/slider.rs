//! Continuous range slider input (shadcn `Slider`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Slider.
#[derive(Debug, Default)]
pub struct SliderState {
    pub bounds: Rect,
    pub current_value: f32,
    pub normalized_ratio: f32,
    pub node: u32,
}

/// Horizontal range slider widget.
#[derive(Clone)]
pub struct Slider {
    value: Signal<f32>,
    min: f32,
    max: f32,
    step: f32,
    on_change: Option<Arc<dyn Fn(f32) + Send + Sync>>,
    disabled: bool,
    width: Option<tej_ui_layout::Dimension>,
    flex_grow: f32,
}

impl std::fmt::Debug for Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slider")
            .field("value", &self.value.get())
            .field("min", &self.min)
            .field("max", &self.max)
            .finish()
    }
}

impl Slider {
    /// Creates a new slider bound to a reactive signal with min and max bounds.
    #[must_use]
    pub fn new(value: Signal<f32>, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            step: 1.0,
            on_change: None,
            disabled: false,
            width: None,
            flex_grow: 0.0,
        }
    }

    /// Sets explicit slider width constraint.
    #[must_use]
    pub const fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets flex-grow factor in flex containers.
    #[must_use]
    pub const fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Sets step increment.
    #[must_use]
    pub const fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Sets on_change callback.
    #[must_use]
    pub fn on_change<F>(mut self, cb: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(cb));
        self
    }

    /// Updates slider value from a normalized click/drag ratio [0.0 .. 1.0].
    pub fn set_normalized(&self, ratio: f32) {
        if self.disabled {
            return;
        }
        let clamped = ratio.clamp(0.0, 1.0);
        let raw = self.min + clamped * (self.max - self.min);
        let stepped = (raw / self.step).round() * self.step;
        let final_val = stepped.clamp(self.min, self.max);
        self.value.set(final_val);
        if let Some(cb) = &self.on_change {
            cb(final_val);
        }
    }
}

impl View for Slider {
    type State = SliderState;

    fn build(&self) -> Self::State {
        let val = self.value.get();
        let range = (self.max - self.min).max(0.0001);
        let ratio = ((val - self.min) / range).clamp(0.0, 1.0);

        SliderState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(200.0, 20.0)),
            current_value: val,
            normalized_ratio: ratio,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let val = self.value.get();
        let range = (self.max - self.min).max(0.0001);
        state.current_value = val;
        state.normalized_ratio = ((val - self.min) / range).clamp(0.0, 1.0);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let width = self
            .width
            .unwrap_or(tej_ui_layout::Dimension::Points(200.0));
        let style = tej_ui_layout::LayoutStyle {
            width,
            height: tej_ui_layout::Dimension::Points(20.0),
            flex_grow: self.flex_grow,
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

        let track_h = 6.0;
        let track_y = bounds.origin.y + (bounds.size.height - track_h) * 0.5;
        let track_rect = Rect::new(bounds.origin.x, track_y, bounds.size.width, track_h);

        // Track background
        let bg_color = ctx.theme.colors.muted;
        ctx.fill_rounded_rect(track_rect, 3.0, bg_color);

        // Filled progress
        let fill_w = bounds.size.width * state.normalized_ratio;
        let fill_rect = Rect::new(bounds.origin.x, track_y, fill_w, track_h);
        let fill_color = if self.disabled {
            ctx.theme.colors.muted_foreground
        } else {
            ctx.theme.colors.primary
        };
        ctx.fill_rounded_rect(fill_rect, 3.0, fill_color);

        // Thumb
        let thumb_radius = 8.0;
        let thumb_x = bounds.origin.x + fill_w;
        let thumb_y = bounds.origin.y + bounds.size.height * 0.5;
        let thumb_rect = Rect::new(
            thumb_x - thumb_radius,
            thumb_y - thumb_radius,
            thumb_radius * 2.0,
            thumb_radius * 2.0,
        );

        // Focus ring
        if ctx.is_focused(state.node) && !self.disabled {
            let ring = Rect::new(
                thumb_rect.origin.x - 2.0,
                thumb_rect.origin.y - 2.0,
                thumb_rect.size.width + 4.0,
                thumb_rect.size.height + 4.0,
            );
            ctx.fill_rounded_rect(ring, thumb_radius + 2.0, ctx.theme.colors.ring);
        }

        let thumb_color = if self.disabled {
            ctx.theme.colors.muted
        } else if ctx.is_pressed(state.node) {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.background
        };
        ctx.fill_rounded_rect(thumb_rect, thumb_radius, thumb_color);
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        if !self.disabled {
            let val_sig = self.value;
            let min = self.min;
            let max = self.max;
            let step = self.step;
            let on_change = self.on_change.clone();
            dispatcher.register_click(
                state.node,
                Arc::new(move || {
                    let current = val_sig.get();
                    let next = if current + step > max {
                        min
                    } else {
                        current + step
                    };
                    val_sig.set(next);
                    if let Some(cb) = &on_change {
                        cb(next);
                    }
                }),
            );
            let on_change_key = self.on_change.clone();
            dispatcher.register_key_down(
                state.node,
                Arc::new(move |code| {
                    let current = val_sig.get();
                    let next = match code {
                        37 | 40 => (current - step).max(min),
                        38 | 39 => (current + step).min(max),
                        _ => current,
                    };
                    if (next - current).abs() > f32::EPSILON {
                        val_sig.set(next);
                        if let Some(cb) = &on_change_key {
                            cb(next);
                        }
                    }
                }),
            );
        }
    }
}
