//! `tej-view`: Compile-time static view trees and primitive widgets.
//!
//! Provides static dispatch, zero-vtable tuple combinators, primitive interactive
//! controls (`Text`, `Button`, `Checkbox`, `Input`, `VStack`, `HStack`), and control flow (`If`, `For`).

pub mod control_flow;
pub mod modifiers;
pub mod primitives;
pub mod tuples;
pub mod view;

pub use control_flow::cond::{EitherState, If, IfState};
pub use control_flow::iter::{For, ForState};
pub use modifiers::{StyleProps, Styled, StyledState, ViewExt};
pub use primitives::button::{Button, ButtonState, ClickHandler};
pub use primitives::canvas::{Canvas, CanvasPainter, CanvasState};
pub use primitives::checkbox::{Checkbox, CheckboxState, ToggleHandler};
pub use primitives::input::{Input, InputState, SearchBar, TextChangeHandler};
pub use primitives::stacks::{HStack, StackState, VStack};
pub use primitives::text::{Text, TextSource, TextState};
pub use view::View;
