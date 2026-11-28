//! Target compilation dispatcher and asset bundler.

use crate::commands::TargetPlatform;

/// Build configuration for compiling a tej-ui app.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub target: TargetPlatform,
    pub release: bool,
    pub features: Vec<String>,
}

impl BuildConfig {
    pub fn new(target: TargetPlatform) -> Self {
        Self {
            target,
            release: false,
            features: Vec::new(),
        }
    }

    /// Configures the build for release mode.
    #[must_use]
    pub const fn release(mut self) -> Self {
        self.release = true;
        self
    }

    /// Configures the build for debug mode.
    #[must_use]
    pub const fn debug(mut self) -> Self {
        self.release = false;
        self
    }

    /// Constructs the equivalent `cargo build` command invocation arguments.
    pub fn cargo_args(&self) -> Vec<String> {
        let mut args = vec![
            "build".to_string(),
            "--target".to_string(),
            self.target.triple().to_string(),
        ];
        if self.release {
            args.push("--release".to_string());
        }
        if !self.features.is_empty() {
            args.push("--features".to_string());
            args.push(self.features.join(","));
        }
        args
    }
}
