//! Formatter and reporter facade for cross-framework benchmark scoreboard output.

pub mod scoreboard;
pub mod tradeoffs;

pub use scoreboard::{BenchmarkSuiteResults, print_raw_scoreboard};
pub use tradeoffs::print_architectural_tradeoff_matrix;
