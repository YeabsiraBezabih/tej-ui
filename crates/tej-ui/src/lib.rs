//! `tej`: Modular, zero-copy, authentic native cross-platform UI framework for Rust.
//!
//! Powered by generational arena signals, static view trees, flat SoA layout,
//! hybrid OS compositors, and direct zero-copy integration with the `mydb` ACID DBMS.

pub mod app;
#[cfg(all(feature = "native-gpu", not(target_arch = "wasm32")))]
pub mod native_runner;
pub mod prelude;

pub use app::App;
#[cfg(all(feature = "native-gpu", not(target_arch = "wasm32")))]
pub use native_runner::{GlyphDrawCommand, GpuSurfaceBridge, NativeWinitApp};
pub use prelude::*;
