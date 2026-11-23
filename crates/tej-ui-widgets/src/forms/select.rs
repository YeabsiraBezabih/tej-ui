//! Select dropdown picker (shadcn `Select`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// An option item inside a Select dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
}

impl<T: Clone + PartialEq> SelectOption<T> {
    #[must_use]
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

/// Retained state for a mounted Select.
#[derive(Debug, Default)]
pub struct SelectState {
    pub bounds: Rect,
    pub is_open: bool,
    pub selected_label: String,
    pub node: u32,
    pub option_nodes: Vec<u32>,
}

/// Callback invoked when selected option changes.
pub type SelectCallback<T> = Option<Arc<dyn Fn(&T) + Send + Sync>>;

/// Dropdown select component with trigger and options menu.
#[derive(Clone)]
pub struct Select<T: Clone + PartialEq + Send + Sync + 'static> {
    selected: Signal<T>,
    is_open: Signal<bool>,
    options: Vec<SelectOption<T>>,
    placeholder: String,
    on_change: SelectCallback<T>,
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> std::fmt::Debug for Select<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Select")
            .field("selected", &self.selected.get())
            .field("is_open", &self.is_open.get())
            .field("options_count", &self.options.len())
            .finish()
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    /// Creates a new Select component.
    #[must_use]
    pub fn new(
        selected: Signal<T>,
        is_open: Signal<bool>,
        options: Vec<SelectOption<T>>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            selected,
            is_open,
            options,
            placeholder: placeholder.into(),
            on_change: None,
        }
    }

    /// Selects an option by index and closes dropdown.
    pub fn choose_index(&self, index: usize) {
        if let Some(opt) = self.options.get(index) {
            self.selected.set(opt.value.clone());
            self.is_open.set(false);
            if let Some(cb) = &self.on_change {
                cb(&opt.value);
            }
        }
    }

    /// Toggles the dropdown menu.
    pub fn toggle_open(&self) {
        self.is_open.set(!self.is_open.get());
    }
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> View for Select<T> {
    type State = SelectState;

    fn build(&self) -> Self::State {
        let val = self.selected.get();
        let selected_label = self
            .options
            .iter()
            .find(|o| o.value == val)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.clone());

        SelectState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(180.0, 36.0)),
            is_open: self.is_open.get(),
            selected_label,
            node: 0,
            option_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let val = self.selected.get();
        state.selected_label = self
            .options
            .iter()
            .find(|o| o.value == val)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.clone());
        state.is_open = self.is_open.get();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let trigger_style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(180.0),
            height: tej_ui_layout::Dimension::Points(36.0),
            ..Default::default()
        };
        let root = arena.allocate_node(trigger_style);
        state.node = root;
        state.option_nodes.clear();

        if state.is_open {
            for _ in 0..self.options.len() {
                let opt_style = tej_ui_layout::LayoutStyle {
                    width: tej_ui_layout::Dimension::Points(180.0),
                    height: tej_ui_layout::Dimension::Points(32.0),
                    ..Default::default()
                };
                let opt_node = arena.allocate_node(opt_style);
                state.option_nodes.push(opt_node);
                arena.add_child(root, opt_node);
            }
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        // Trigger focus ring
        if ctx.is_focused(state.node) {
            let ring = Rect::new(
                bounds.origin.x - 2.0,
                bounds.origin.y - 2.0,
                bounds.size.width + 4.0,
                bounds.size.height + 4.0,
            );
            ctx.fill_rounded_rect(ring, 8.0, ctx.theme.colors.ring);
        }

        // Trigger background & border
        ctx.fill_rounded_rect(bounds, 6.0, ctx.theme.colors.input);
        let inner_trigger = Rect::new(
            bounds.origin.x + 1.0,
            bounds.origin.y + 1.0,
            bounds.size.width - 2.0,
            bounds.size.height - 2.0,
        );
        let bg = if ctx.is_hovered(state.node) {
            ctx.theme.colors.accent.with_alpha(50)
        } else {
            ctx.theme.colors.background
        };
        ctx.fill_rounded_rect(inner_trigger, 5.0, bg);

        // Text & indicator
        let text_pt = Point::new(
            bounds.origin.x + 12.0,
            bounds.origin.y + (bounds.size.height - 14.0) * 0.5,
        );
        ctx.draw_text(
            &state.selected_label,
            text_pt,
            ctx.theme.typography.body_small.font_size,
            ctx.theme.colors.foreground,
        );

        let chevron_pt = Point::new(
            bounds.origin.x + bounds.size.width - 18.0,
            bounds.origin.y + (bounds.size.height - 12.0) * 0.5,
        );
        ctx.draw_text(
            if state.is_open { "^" } else { "v" },
            chevron_pt,
            ctx.theme.typography.caption.font_size,
            ctx.theme.colors.muted_foreground,
        );

        // Dropdown popover
        if state.is_open && !self.options.is_empty() {
            let menu_h = (self.options.len() as f32) * 32.0 + 8.0;
            let menu_rect = Rect::new(
                bounds.origin.x,
                bounds.origin.y + bounds.size.height + 4.0,
                bounds.size.width,
                menu_h,
            );
            ctx.fill_rounded_rect(menu_rect, 6.0, ctx.theme.colors.border);
            let inner_menu = Rect::new(
                menu_rect.origin.x + 1.0,
                menu_rect.origin.y + 1.0,
                menu_rect.size.width - 2.0,
                menu_rect.size.height - 2.0,
            );
            ctx.fill_rounded_rect(inner_menu, 5.0, ctx.theme.colors.popover);

            for (i, opt) in self.options.iter().enumerate() {
                let opt_node = state.option_nodes.get(i).copied().unwrap_or(0);
                let opt_y = menu_rect.origin.y + 4.0 + (i as f32) * 32.0;
                let opt_rect = Rect::new(
                    menu_rect.origin.x + 4.0,
                    opt_y,
                    menu_rect.size.width - 8.0,
                    28.0,
                );

                let is_hovered = ctx.is_hovered(opt_node);
                if is_hovered {
                    ctx.fill_rounded_rect(opt_rect, 4.0, ctx.theme.colors.accent);
                }

                let opt_text_pt = Point::new(opt_rect.origin.x + 8.0, opt_rect.origin.y + 7.0);
                let text_color = if is_hovered {
                    ctx.theme.colors.accent_foreground
                } else {
                    ctx.theme.colors.popover_foreground
                };
                ctx.draw_text(
                    &opt.label,
                    opt_text_pt,
                    ctx.theme.typography.body_small.font_size,
                    text_color,
                );
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        let is_open_sig = self.is_open;
        dispatcher.register_click(
            state.node,
            Arc::new(move || {
                is_open_sig.set(!is_open_sig.get());
            }),
        );

        if state.is_open {
            for (i, opt) in self.options.iter().enumerate() {
                if let Some(&opt_node) = state.option_nodes.get(i) {
                    let val = opt.value.clone();
                    let sig = self.selected;
                    let is_open = self.is_open;
                    let on_change = self.on_change.clone();
                    dispatcher.register_click(
                        opt_node,
                        Arc::new(move || {
                            sig.set(val.clone());
                            is_open.set(false);
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
