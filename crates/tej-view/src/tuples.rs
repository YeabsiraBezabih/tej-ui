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
        }
    };
}

impl_view_for_tuple!(0: A, 1: B);
impl_view_for_tuple!(0: A, 1: B, 2: C);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_view_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
