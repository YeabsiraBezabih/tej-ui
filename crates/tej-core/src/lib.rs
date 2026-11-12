//! `tej-core`: Generational Arena Signals and Glitch-Free Reactive Runtime.
//!
//! Provides 8-byte `Copy` `Signal<T>` handles, generational `SlotMap` arena storage,
//! topological dependency tracking, derived computations (`Memo`), and batched mutations.

pub mod arena;
pub mod batch;
pub mod computed;
pub mod effect;
pub mod error;
pub mod graph;
pub mod id;
pub mod runtime;
pub mod signal;

pub use arena::SignalArena;
pub use batch::batch;
pub use computed::Memo;
pub use effect::{Effect, create_effect};
pub use error::{CoreError, Result};
pub use graph::DependencyGraph;
pub use id::{Epoch, NodeId, SignalId};
pub use runtime::ReactiveRuntime;
pub use signal::{ReadSignal, Signal, WriteSignal};
