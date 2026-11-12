use crate::view::View;
use std::fmt::Debug;
use tej_core::Signal;

/// Branch state representing either the Then or Else view state.
#[derive(Debug)]
pub enum EitherState<A: Debug, B: Debug> {
    Then(A),
    Else(B),
}

/// Static conditional view without heap boxing.
#[derive(Debug, Clone)]
pub struct If<Then: View, Else: View> {
    condition: Signal<bool>,
    then_view: Then,
    else_view: Else,
}

impl<Then: View, Else: View> If<Then, Else> {
    /// Create a conditional view.
    pub fn new(condition: Signal<bool>, then_view: Then, else_view: Else) -> Self {
        Self {
            condition,
            then_view,
            else_view,
        }
    }
}

/// State of an If view.
#[derive(Debug)]
pub struct IfState<Then: Debug, Else: Debug> {
    pub current_cond: bool,
    pub branch_state: EitherState<Then, Else>,
}

impl<Then: View, Else: View> View for If<Then, Else> {
    type State = IfState<Then::State, Else::State>;

    fn build(&self) -> Self::State {
        let cond = self.condition.get();
        let branch_state = if cond {
            EitherState::Then(self.then_view.build())
        } else {
            EitherState::Else(self.else_view.build())
        };
        IfState {
            current_cond: cond,
            branch_state,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let cond = self.condition.get();
        if cond != state.current_cond {
            state.current_cond = cond;
            state.branch_state = if cond {
                EitherState::Then(self.then_view.build())
            } else {
                EitherState::Else(self.else_view.build())
            };
        } else {
            match &mut state.branch_state {
                EitherState::Then(s) => self.then_view.rebuild(s),
                EitherState::Else(s) => self.else_view.rebuild(s),
            }
        }
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        let cond = self.condition.get();
        if cond != state.current_cond {
            self.rebuild(state);
        } else {
            match &mut state.branch_state {
                EitherState::Then(s) => self.then_view.patch(s, dirty_bits),
                EitherState::Else(s) => self.else_view.patch(s, dirty_bits),
            }
        }
    }
}
