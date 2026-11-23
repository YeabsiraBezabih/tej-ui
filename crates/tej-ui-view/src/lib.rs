//! `tej-view`: Compile-time static view trees and primitive widgets.
//!
//! Provides static dispatch, zero-vtable tuple combinators, primitive interactive
//! controls (`Text`, `Button`, `Checkbox`, `Input`, `VStack`, `HStack`), and control flow (`If`, `For`).

pub mod context_provider;
pub mod control_flow;
pub mod css;
pub mod error_boundary;
pub mod events;
pub mod fuzz;
pub mod modifiers;
pub mod paint;
pub mod paint_fx;
pub mod primitives;
pub mod snapshot;
pub mod svg_paint;
pub mod testing;
pub mod theme;
pub mod tuples;
pub mod typography_extended;
pub mod view;

pub use context_provider::ContextProvider;
pub use control_flow::cond::{EitherState, If, IfState};
pub use control_flow::iter::{For, ForState};
pub use css::{CssStyleSheet, TOTAL_CSS_PROPERTIES};
pub use error_boundary::{ErrorBoundary, ErrorBoundaryState};
pub use events::{
    CursorIcon, EventDispatchResult, EventDispatcher, EventPhase, EventPropagation,
    TwoPhaseEventListener,
};
pub use fuzz::{EventStreamFuzzer, MountStressReport};
pub use modifiers::{StyleProps, Styled, StyledState, ViewExt};
pub use paint::PaintContext;
pub use paint_fx::{bayer8, dither_test, erf_approx, gaussian_shadow_alpha, sd_rounded_rect};
pub use primitives::button::{Button, ButtonSize, ButtonState, ButtonVariant, ClickHandler};
pub use primitives::canvas::{Canvas, CanvasPainter, CanvasState};
pub use primitives::checkbox::{Checkbox, CheckboxState, ToggleHandler};
pub use primitives::grid::{Grid, GridState};
pub use primitives::input::{Input, InputState, SearchBar, TextChangeHandler};
pub use primitives::stacks::{HStack, StackState, VStack};
pub use primitives::text::{Text, TextSource, TextState};
pub use snapshot::{ImageBuffer, SnapshotComparator, SnapshotDiffResult};
pub use svg_paint::{FillRule, StrokeLineCap, StrokeLineJoin, SvgPaintStyle, VectorEffect};
pub use testing::{TestHarness, TestNode};
pub use typography_extended::{CaretStyle, FontVariantNumeric, TextUnderlineStyle, TextWrap};

pub use theme::{
    ColorTokens, ElevationTokens, RadiiTokens, Shadow, SpacingTokens, TextStyle, Theme,
    TypographyTokens, provide_theme, use_theme,
};
pub use view::View;
