//! Panic-safe Error Boundary view combinator.
//!
//! Catches runtime panics occurring within view lifecycle methods (`build`, `layout`, `paint`)
//! using [`std::panic::catch_unwind`], rendering a fallback view and reporting telemetry
//! without crashing the application process.

use crate::paint::PaintContext;
use crate::view::View;
use std::fmt::Debug;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use tej_ui_layout::LayoutArena;

/// Telemetry reporter callback for caught widget panics.
pub type ErrorTelemetryCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// State representation for an [`ErrorBoundary`].
#[derive(Debug)]
pub enum ErrorBoundaryState<S, FS> {
    /// Normal operational child state.
    Normal(S),
    /// Caught panic error message and active fallback view state.
    Recovered {
        error_message: String,
        fallback_state: FS,
    },
}

/// An error boundary wrapper providing panic isolation and recovery for view subtrees.
pub struct ErrorBoundary<V: View, F, Fallback: View>
where
    F: Fn(&str) -> Fallback + 'static,
{
    child: V,
    fallback_builder: Arc<F>,
    telemetry: Option<ErrorTelemetryCallback>,
}

impl<V: View + Debug, F, Fallback: View + Debug> Debug for ErrorBoundary<V, F, Fallback>
where
    F: Fn(&str) -> Fallback + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorBoundary")
            .field("child", &self.child)
            .field("has_telemetry", &self.telemetry.is_some())
            .finish()
    }
}

impl<V: View, F, Fallback: View> ErrorBoundary<V, F, Fallback>
where
    F: Fn(&str) -> Fallback + 'static,
{
    /// Creates an error boundary around `child` with a `fallback_builder`.
    pub fn new(child: V, fallback_builder: F) -> Self {
        Self {
            child,
            fallback_builder: Arc::new(fallback_builder),
            telemetry: None,
        }
    }

    /// Attaches an error telemetry logging callback.
    #[must_use]
    pub fn on_error<T: Fn(&str) + Send + Sync + 'static>(mut self, callback: T) -> Self {
        self.telemetry = Some(Arc::new(callback));
        self
    }

    fn extract_panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
        if let Some(msg) = payload.downcast_ref::<&str>() {
            (*msg).to_string()
        } else if let Some(msg) = payload.downcast_ref::<String>() {
            msg.clone()
        } else {
            String::from("Unknown component panic occurred")
        }
    }

    fn notify_telemetry(&self, msg: &str) {
        if let Some(ref tel) = self.telemetry {
            tel(msg);
        }
    }
}

impl<V: View, F, Fallback: View> View for ErrorBoundary<V, F, Fallback>
where
    F: Fn(&str) -> Fallback + 'static,
{
    type State = ErrorBoundaryState<V::State, Fallback::State>;

    fn build(&self) -> Self::State {
        let child_res = catch_unwind(AssertUnwindSafe(|| self.child.build()));
        match child_res {
            Ok(state) => ErrorBoundaryState::Normal(state),
            Err(payload) => {
                let msg = Self::extract_panic_message(payload);
                self.notify_telemetry(&msg);
                let fallback_view = (self.fallback_builder)(&msg);
                let fallback_state = fallback_view.build();
                ErrorBoundaryState::Recovered {
                    error_message: msg,
                    fallback_state,
                }
            }
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        match state {
            ErrorBoundaryState::Normal(child_state) => {
                let res = catch_unwind(AssertUnwindSafe(|| self.child.rebuild(child_state)));
                if let Err(payload) = res {
                    let msg = Self::extract_panic_message(payload);
                    self.notify_telemetry(&msg);
                    let fallback_view = (self.fallback_builder)(&msg);
                    let fallback_state = fallback_view.build();
                    *state = ErrorBoundaryState::Recovered {
                        error_message: msg,
                        fallback_state,
                    };
                }
            }
            ErrorBoundaryState::Recovered {
                error_message,
                fallback_state,
            } => {
                let fallback_view = (self.fallback_builder)(error_message);
                fallback_view.rebuild(fallback_state);
            }
        }
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        match state {
            ErrorBoundaryState::Normal(child_state) => {
                let res = catch_unwind(AssertUnwindSafe(|| {
                    self.child.patch(child_state, dirty_bits)
                }));
                if let Err(payload) = res {
                    let msg = Self::extract_panic_message(payload);
                    self.notify_telemetry(&msg);
                    let fallback_view = (self.fallback_builder)(&msg);
                    let fallback_state = fallback_view.build();
                    *state = ErrorBoundaryState::Recovered {
                        error_message: msg,
                        fallback_state,
                    };
                }
            }
            ErrorBoundaryState::Recovered {
                error_message,
                fallback_state,
            } => {
                let fallback_view = (self.fallback_builder)(error_message);
                fallback_view.patch(fallback_state, dirty_bits);
            }
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        match state {
            ErrorBoundaryState::Normal(child_state) => {
                let res = catch_unwind(AssertUnwindSafe(|| self.child.layout(child_state, arena)));
                match res {
                    Ok(node) => node,
                    Err(payload) => {
                        let msg = Self::extract_panic_message(payload);
                        self.notify_telemetry(&msg);
                        let fallback_view = (self.fallback_builder)(&msg);
                        let mut fallback_state = fallback_view.build();
                        let node = fallback_view.layout(&mut fallback_state, arena);
                        *state = ErrorBoundaryState::Recovered {
                            error_message: msg,
                            fallback_state,
                        };
                        node
                    }
                }
            }
            ErrorBoundaryState::Recovered {
                error_message,
                fallback_state,
            } => {
                let fallback_view = (self.fallback_builder)(error_message);
                fallback_view.layout(fallback_state, arena)
            }
        }
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        match state {
            ErrorBoundaryState::Normal(child_state) => {
                let res = catch_unwind(AssertUnwindSafe(|| self.child.paint(child_state, ctx)));
                if let Err(payload) = res {
                    let msg = Self::extract_panic_message(payload);
                    self.notify_telemetry(&msg);
                    let fallback_view = (self.fallback_builder)(&msg);
                    let fallback_state = fallback_view.build();
                    fallback_view.paint(&fallback_state, ctx);
                }
            }
            ErrorBoundaryState::Recovered {
                error_message,
                fallback_state,
            } => {
                let fallback_view = (self.fallback_builder)(error_message);
                fallback_view.paint(fallback_state, ctx);
            }
        }
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        match state {
            ErrorBoundaryState::Normal(child_state) => {
                let res = catch_unwind(AssertUnwindSafe(|| {
                    self.child.register_events(child_state, dispatcher);
                }));
                if let Err(payload) = res {
                    let msg = Self::extract_panic_message(payload);
                    self.notify_telemetry(&msg);
                }
            }
            ErrorBoundaryState::Recovered {
                error_message,
                fallback_state,
            } => {
                let fallback_view = (self.fallback_builder)(error_message);
                fallback_view.register_events(fallback_state, dispatcher);
            }
        }
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        match state {
            ErrorBoundaryState::Normal(child_state) => {
                let res = catch_unwind(AssertUnwindSafe(|| {
                    self.child.accessibility(child_state, arena, tree, parent);
                }));
                if let Err(payload) = res {
                    let msg = Self::extract_panic_message(payload);
                    self.notify_telemetry(&msg);
                }
            }
            ErrorBoundaryState::Recovered {
                error_message,
                fallback_state,
            } => {
                let fallback_view = (self.fallback_builder)(error_message);
                fallback_view.accessibility(fallback_state, arena, tree, parent);
            }
        }
    }
}
