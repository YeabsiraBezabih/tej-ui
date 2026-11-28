use crate::view::View;

macro_rules! impl_view_for_tuple {
    ($($idx:tt: $T:ident),+) => {
        impl<$($T: View),+> View for ($($T,)+) {
            type State = ($($T::State,)+);

            fn build(&self) -> Self::State {
                ($(self.$idx.build(),)+)
            }

            fn rebuild(&self, state: &mut Self::State) {
                $(self.$idx.rebuild(&mut state.$idx);)+
            }

            fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
                $(self.$idx.patch(&mut state.$idx, dirty_bits);)+
            }

            fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
                let parent = arena.allocate_node(tej_ui_layout::LayoutStyle::default());
                self.layout_children(state, arena, parent);
                parent
            }

            fn layout_children(
                &self,
                state: &mut Self::State,
                arena: &mut tej_ui_layout::LayoutArena,
                parent: u32,
            ) {
                $(
                    self.$idx.layout_children(&mut state.$idx, arena, parent);
                )+
            }

            fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
                $(
                    self.$idx.paint(&state.$idx, ctx);
                )+
            }

            fn register_events(&self, state: &Self::State, dispatcher: &mut crate::events::EventDispatcher) {
                $(
                    self.$idx.register_events(&state.$idx, dispatcher);
                )+
            }

            fn accessibility(
                &self,
                state: &Self::State,
                arena: &tej_ui_layout::LayoutArena,
                tree: &mut tej_ui_a11y::A11yTree,
                parent: Option<tej_ui_a11y::A11yId>,
            ) {
                $(
                    self.$idx.accessibility(&state.$idx, arena, tree, parent);
                )+
            }
        }
    };
}

impl_view_for_tuple!(0: A);
impl_view_for_tuple!(0: A, 1: B);
impl_view_for_tuple!(0: A, 1: B, 2: C);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
