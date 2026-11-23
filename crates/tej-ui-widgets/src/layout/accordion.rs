//! Vertically stacked collapsible panels (shadcn `Accordion`).

use std::collections::HashSet;
use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// An individual collapsible section within an Accordion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccordionItem {
    pub id: String,
    pub title: String,
    pub content: String,
}

impl AccordionItem {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
        }
    }
}

/// Retained state for a mounted Accordion.
#[derive(Debug, Default)]
pub struct AccordionState {
    pub bounds: Rect,
    pub open_items: HashSet<String>,
}

/// Interactive accordion revealing and collapsing content sections.
#[derive(Debug, Clone)]
pub struct Accordion {
    open_items: Signal<HashSet<String>>,
    items: Vec<AccordionItem>,
    allow_multiple: bool,
}

impl Accordion {
    /// Creates a new accordion with open items tracked by signal.
    #[must_use]
    pub fn new(open_items: Signal<HashSet<String>>, items: Vec<AccordionItem>) -> Self {
        Self {
            open_items,
            items,
            allow_multiple: false,
        }
    }

    /// Allows multiple sections to be open simultaneously.
    #[must_use]
    pub const fn multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }

    /// Toggles open state for an item.
    pub fn toggle_item(&self, item_id: &str) {
        let mut current = self.open_items.get();
        if current.contains(item_id) {
            current.remove(item_id);
        } else {
            if !self.allow_multiple {
                current.clear();
            }
            current.insert(item_id.to_string());
        }
        self.open_items.set(current);
    }
}

impl View for Accordion {
    type State = AccordionState;

    fn build(&self) -> Self::State {
        let open_items = self.open_items.get();
        let total_height = self
            .items
            .iter()
            .map(|it| {
                if open_items.contains(&it.id) {
                    120.0
                } else {
                    48.0
                }
            })
            .sum();

        AccordionState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(400.0, total_height)),
            open_items,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.open_items = self.open_items.get();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
