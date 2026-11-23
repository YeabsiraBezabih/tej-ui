//! CLI command parser and target platform configurations.

/// Supported cross-compilation target platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    LinuxX64,
    LinuxArm64,
    WindowsX64,
    MacOsArm64,
    MacOsX64,
    Wasm,
    Android,
    Ios,
}

impl TargetPlatform {
    pub fn parse(target: &str) -> Option<Self> {
        match target.to_lowercase().as_str() {
            "linux-x64" | "x86_64-unknown-linux-gnu" => Some(Self::LinuxX64),
            "linux-arm64" | "aarch64-unknown-linux-gnu" => Some(Self::LinuxArm64),
            "windows-x64" | "x86_64-pc-windows-msvc" => Some(Self::WindowsX64),
            "macos-arm64" | "aarch64-apple-darwin" => Some(Self::MacOsArm64),
            "macos-x64" | "x86_64-apple-darwin" => Some(Self::MacOsX64),
            "wasm" | "wasm32-unknown-unknown" => Some(Self::Wasm),
            "android" | "aarch64-linux-android" => Some(Self::Android),
            "ios" | "aarch64-apple-ios" => Some(Self::Ios),
            _ => None,
        }
    }

    pub fn triple(&self) -> &'static str {
        match self {
            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
            Self::LinuxArm64 => "aarch64-unknown-linux-gnu",
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
            Self::MacOsArm64 => "aarch64-apple-darwin",
            Self::MacOsX64 => "x86_64-apple-darwin",
            Self::Wasm => "wasm32-unknown-unknown",
            Self::Android => "aarch64-linux-android",
            Self::Ios => "aarch64-apple-ios",
        }
    }
}

/// CLI execution command options.
#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    New {
        project_name: String,
    },
    Build {
        target: TargetPlatform,
        release: bool,
    },
    Bundle {
        output_dir: String,
    },
    Dev {
        port: u16,
    },
}
