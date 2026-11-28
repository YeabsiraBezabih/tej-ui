use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_view::View;

/// Virtualized list container that materializes only rows visible in the viewport.
///
/// Keeps memory consumption flat ($O(\text{viewport})$) regardless of whether
/// the backing dataset contains 100 rows or 1,000,000 rows.
pub struct VirtualList<V: View, F>
where
    F: Fn(usize) -> V + 'static,
{
    total_items: usize,
    item_height: f32,
    viewport_height: f32,
    scroll_offset: Signal<f32>,
    row_builder: Arc<F>,
}

impl<V: View, F> Debug for VirtualList<V, F>
where
    F: Fn(usize) -> V + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualList")
            .field("total_items", &self.total_items)
            .field("item_height", &self.item_height)
            .field("viewport_height", &self.viewport_height)
            .finish_non_exhaustive()
    }
}

impl<V: View, F> VirtualList<V, F>
where
    F: Fn(usize) -> V + 'static,
{
    /// Create a new virtual list.
    pub fn new(
        total_items: usize,
        item_height: f32,
        viewport_height: f32,
        scroll_offset: Signal<f32>,
        row_builder: F,
    ) -> Self {
        Self {
            total_items,
            item_height,
            viewport_height,
            scroll_offset,
            row_builder: Arc::new(row_builder),
        }
    }

    /// Calculate visible range `(start_index, end_index)`.
    #[must_use]
    pub fn visible_range(&self) -> (usize, usize) {
        if self.total_items == 0
            || self.item_height <= 0.0
            || !self.item_height.is_finite()
            || !self.viewport_height.is_finite()
        {
            return (0, 0);
        }
        let raw_offset = self.scroll_offset.get();
        let offset = if raw_offset.is_finite() {
            raw_offset.max(0.0)
        } else {
            0.0
        };
        let viewport = self.viewport_height.max(0.0);
        let start = (offset / self.item_height).floor() as usize;
        let visible_count = ((viewport / self.item_height).ceil() as usize).saturating_add(2); // +2 buffer
        let end = start.saturating_add(visible_count).min(self.total_items);
        (start.min(self.total_items), end)
    }
}

/// State retained by a mounted VirtualList.
#[derive(Debug)]
pub struct VirtualListState<S: Debug> {
    pub start_index: usize,
    pub end_index: usize,
    pub visible_states: Vec<(usize, S)>,
}

impl<V: View, F> View for VirtualList<V, F>
where
    F: Fn(usize) -> V + 'static,
{
    type State = VirtualListState<V::State>;

    fn build(&self) -> Self::State {
        let (start, end) = self.visible_range();
        let mut visible_states = Vec::with_capacity(end.saturating_sub(start));
        for idx in start..end {
            let row_view = (self.row_builder)(idx);
            visible_states.push((idx, row_view.build()));
        }
        VirtualListState {
            start_index: start,
            end_index: end,
            visible_states,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let (start, end) = self.visible_range();
        let mut new_states = Vec::with_capacity(end.saturating_sub(start));
        for idx in start..end {
            let row_view = (self.row_builder)(idx);
            if let Some(pos) = state.visible_states.iter().position(|(i, _)| *i == idx) {
                let mut existing = state.visible_states.remove(pos);
                row_view.rebuild(&mut existing.1);
                new_states.push(existing);
            } else {
                new_states.push((idx, row_view.build()));
            }
        }
        state.start_index = start;
        state.end_index = end;
        state.visible_states = new_states;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        let (start, end) = self.visible_range();
        if start != state.start_index || end != state.end_index {
            self.rebuild(state);
        } else {
            for (idx, child_state) in &mut state.visible_states {
                let row_view = (self.row_builder)(*idx);
                row_view.patch(child_state, dirty_bits);
            }
        }
    }
}
