//! Navigational breadcrumb trail (shadcn `Breadcrumb`).

use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// A single segment in a breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
    pub is_current: bool,
}

impl BreadcrumbItem {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            is_current: false,
        }
    }

    #[must_use]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    #[must_use]
    pub const fn current(mut self, current: bool) -> Self {
        self.is_current = current;
        self
    }
}

/// Retained state for a mounted Breadcrumb.
#[derive(Debug, Default)]
pub struct BreadcrumbState {
    pub bounds: Rect,
    pub count: usize,
}

/// Horizontal navigation trail revealing the current page hierarchy.
#[derive(Debug, Clone, Default)]
pub struct Breadcrumb {
    pub items: Vec<BreadcrumbItem>,
    pub separator: String,
}

impl Breadcrumb {
    /// Creates a breadcrumb trail from a list of items.
    #[must_use]
    pub fn new(items: Vec<BreadcrumbItem>) -> Self {
        Self {
            items,
            separator: "/".to_string(),
        }
    }

    /// Sets custom separator character or icon.
    #[must_use]
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl View for Breadcrumb {
    type State = BreadcrumbState;

    fn build(&self) -> Self::State {
        let width = (self.items.len() as f32) * 80.0;
        BreadcrumbState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 24.0)),
            count: self.items.len(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.count = self.items.len();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
