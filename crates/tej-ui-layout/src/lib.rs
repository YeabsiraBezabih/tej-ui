//! `tej-layout`: Data-oriented flat SoA layout arena and zero-allocation flex engine.
//!
//! Stores widgets in contiguous CPU cache-friendly vectors and computes Flexbox layout
//! without recursive tree allocations.

pub mod container;
pub mod direction;
pub mod engine;
pub mod geometry;
pub mod grid_extended;
pub mod logical;
pub mod scroll_snap;
pub mod storage;
pub mod style;

pub use container::{Breakpoint, ContainerQuery};
pub use direction::LayoutDirection;
pub use engine::LayoutEngine;
pub use geometry::{Insets, Point, Rect, Size};
pub use grid_extended::{
    AnchorPositionArea, GridAutoFlow, JustifyItems, JustifySelf, PlaceContent, ResizeMode,
};
pub use logical::{LogicalCornerRadii, LogicalInsets, LogicalSize};
pub use scroll_snap::{
    OverscrollBehavior, ScrollSnapAlign, ScrollSnapStop, ScrollSnapType, ScrollbarGutter,
    ScrollbarWidth,
};
pub use storage::LayoutArena;
pub use style::{
    AlignItems, Dimension, DisplayMode, FlexDirection, FlexWrap, JustifyContent, LayoutStyle,
};
