//! `tej-layout`: Data-oriented flat SoA layout arena and zero-allocation flex engine.
//!
//! Stores widgets in contiguous CPU cache-friendly vectors and computes Flexbox layout
//! without recursive tree allocations.

pub mod engine;
pub mod geometry;
pub mod storage;
pub mod style;

pub use engine::LayoutEngine;
pub use geometry::{Insets, Point, Rect, Size};
pub use storage::LayoutArena;
pub use style::{AlignItems, Dimension, FlexDirection, JustifyContent, LayoutStyle};
