//! Data pagination navigation (shadcn `Pagination`).

use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Retained state for a mounted Pagination.
#[derive(Debug, Default)]
pub struct PaginationState {
    pub bounds: Rect,
    pub current_page: usize,
    pub total_pages: usize,
}

/// Paginated data set navigation with prev/next buttons.
#[derive(Debug, Clone)]
pub struct Pagination {
    current_page: Signal<usize>,
    total_pages: usize,
}

impl Pagination {
    /// Creates a pagination control bound to a current page signal.
    #[must_use]
    pub const fn new(current_page: Signal<usize>, total_pages: usize) -> Self {
        Self {
            current_page,
            total_pages: if total_pages > 0 { total_pages } else { 1 },
        }
    }

    /// Moves to the next page if available.
    pub fn next_page(&self) {
        let cur = self.current_page.get();
        if cur < self.total_pages {
            self.current_page.set(cur + 1);
        }
    }

    /// Moves to the previous page if available.
    pub fn prev_page(&self) {
        let cur = self.current_page.get();
        if cur > 1 {
            self.current_page.set(cur - 1);
        }
    }

    /// Sets explicit page number.
    pub fn set_page(&self, page: usize) {
        let clamped = page.clamp(1, self.total_pages);
        self.current_page.set(clamped);
    }
}

impl View for Pagination {
    type State = PaginationState;

    fn build(&self) -> Self::State {
        PaginationState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(320.0, 36.0)),
            current_page: self.current_page.get(),
            total_pages: self.total_pages,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.current_page = self.current_page.get();
        state.total_pages = self.total_pages;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
