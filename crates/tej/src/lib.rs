//! `tej`: Modular, zero-copy, authentic native cross-platform UI framework for Rust.
//!
//! Powered by generational arena signals, static view trees, flat SoA layout,
//! hybrid OS compositors, and direct zero-copy integration with the `mydb` ACID DBMS.

pub mod app;
pub mod prelude;

pub use app::App;
pub use prelude::*;
