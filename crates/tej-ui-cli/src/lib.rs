//! `tej-cli`: Command-line toolchain, scaffolding, and cross-compiler dispatcher for `tej-ui`.

pub mod builder;
pub mod commands;
pub mod scaffold;

pub use builder::BuildConfig;
pub use commands::{CliCommand, TargetPlatform};
pub use scaffold::ProjectScaffolder;
