//! Radio group selector (shadcn `RadioGroup`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// An option item within a radio group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioItem<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
    pub disabled: bool,
}

impl<T: Clone + PartialEq> RadioItem<T> {
    #[must_use]
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }
}

/// Retained state for a mounted RadioGroup.
#[derive(Debug, Default)]
pub struct RadioGroupState {
    pub bounds: Rect,
    pub selected_index: Option<usize>,
    pub node: u32,
    pub item_nodes: Vec<u32>,
}

/// Callback invoked when selected radio option changes.
pub type RadioCallback<T> = Option<Arc<dyn Fn(&T) + Send + Sync>>;

/// Radio group single-selection container.
#[derive(Clone)]
pub struct RadioGroup<T: Clone + PartialEq + Send + Sync + 'static> {
    selected: Signal<T>,
    items: Vec<RadioItem<T>>,
    on_change: RadioCallback<T>,
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> std::fmt::Debug
    for RadioGroup<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadioGroup")
            .field("selected", &self.selected.get())
            .field("items_count", &self.items.len())
            .finish()
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> RadioGroup<T> {
    /// Creates a new radio group bound to the selected signal.
    #[must_use]
    pub fn new(selected: Signal<T>, items: Vec<RadioItem<T>>) -> Self {
        Self {
            selected,
            items,
            on_change: None,
        }
    }

    /// Sets selection by index.
    pub fn select_index(&self, index: usize) {
        if let Some(item) = self.items.get(index) {
            if !item.disabled {
                self.selected.set(item.value.clone());
                if let Some(cb) = &self.on_change {
                    cb(&item.value);
                }
            }
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> View for RadioGroup<T> {
    type State = RadioGroupState;

    fn build(&self) -> Self::State {
        let current = self.selected.get();
        let selected_index = self.items.iter().position(|it| it.value == current);
        RadioGroupState {
            bounds: Rect::from_origin_size(
                Point::ZERO,
                Size::new(200.0, (self.items.len() as f32) * 28.0),
            ),
            selected_index,
            node: 0,
            item_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let current = self.selected.get();
        state.selected_index = self.items.iter().position(|it| it.value == current);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let root_style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Column,
            spacing: 4.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;
        state.item_nodes.clear();

        for _ in 0..self.items.len() {
            let item_style = tej_ui_layout::LayoutStyle {
                height: tej_ui_layout::Dimension::Points(24.0),
                ..Default::default()
            };
            let child = arena.allocate_node(item_style);
            state.item_nodes.push(child);
            arena.add_child(root, child);
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        for (i, item) in self.items.iter().enumerate() {
            if let Some(&item_node) = state.item_nodes.get(i) {
                let bounds = ctx
                    .arena
                    .computed_bounds
                    .get(item_node as usize)
                    .copied()
                    .unwrap_or(Rect::new(
                        state.bounds.origin.x,
                        state.bounds.origin.y + (i as f32) * 28.0,
                        state.bounds.size.width,
                        24.0,
                    ));

                let circle_r = 8.0;
                let circle_center_x = bounds.origin.x + circle_r;
                let circle_center_y = bounds.origin.y + bounds.size.height * 0.5;
                let circle_rect = Rect::new(
                    circle_center_x - circle_r,
                    circle_center_y - circle_r,
                    circle_r * 2.0,
                    circle_r * 2.0,
                );

                let is_selected = state.selected_index == Some(i);
                let is_hovered = ctx.is_hovered(item_node);

                let ring_color = if item.disabled {
                    ctx.theme.colors.muted
                } else if is_selected {
                    ctx.theme.colors.primary
                } else if is_hovered {
                    ctx.theme.colors.primary.with_alpha(150)
                } else {
                    ctx.theme.colors.input
                };

                ctx.fill_rounded_rect(circle_rect, circle_r, ring_color);

                let inner_bg = Rect::new(
                    circle_rect.origin.x + 2.0,
                    circle_rect.origin.y + 2.0,
                    circle_rect.size.width - 4.0,
                    circle_rect.size.height - 4.0,
                );
                ctx.fill_rounded_rect(inner_bg, circle_r - 2.0, ctx.theme.colors.background);

                if is_selected {
                    let dot_r = 4.0;
                    let dot_rect = Rect::new(
                        circle_center_x - dot_r,
                        circle_center_y - dot_r,
                        dot_r * 2.0,
                        dot_r * 2.0,
                    );
                    let dot_color = if item.disabled {
                        ctx.theme.colors.muted_foreground
                    } else {
                        ctx.theme.colors.primary
                    };
                    ctx.fill_rounded_rect(dot_rect, dot_r, dot_color);
                }

                let text_x = circle_center_x + circle_r + 8.0;
                let text_y = bounds.origin.y + (bounds.size.height - 14.0) * 0.5;
                let text_color = if item.disabled {
                    ctx.theme.colors.muted_foreground
                } else {
                    ctx.theme.colors.foreground
                };
                ctx.draw_text(
                    &item.label,
                    Point::new(text_x, text_y),
                    ctx.theme.typography.body_small.font_size,
                    text_color,
                );
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        for (i, item) in self.items.iter().enumerate() {
            if !item.disabled {
                if let Some(&item_node) = state.item_nodes.get(i) {
                    let val = item.value.clone();
                    let sig = self.selected;
                    let on_change = self.on_change.clone();
                    dispatcher.register_click(
                        item_node,
                        Arc::new(move || {
                            sig.set(val.clone());
                            if let Some(cb) = &on_change {
                                cb(&val);
                            }
                        }),
                    );
                }
            }
        }
    }
}
