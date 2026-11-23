//! Enterprise installer formats (MSI/MSIX, DMG/PKG, Flatpak) and MDM compliance.

use std::collections::HashMap;

/// Windows MSI / MSIX Group Policy and installation manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct MsiConfig {
    pub product_name: String,
    pub product_code: String,
    pub upgrade_code: String,
    pub manufacturer: String,
    pub per_machine: bool,
    pub registry_keys: HashMap<String, String>,
}

/// macOS Application Bundle & Notarization configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct MacAppBundle {
    pub bundle_identifier: String,
    pub display_name: String,
    pub team_id: String,
    pub hardened_runtime: bool,
    pub entitlements: Vec<String>,
}

/// Linux Flatpak distribution manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FlatpakManifest {
    pub app_id: String,
    pub runtime: String,
    pub runtime_version: String,
    pub finish_args: Vec<String>,
}

impl Default for FlatpakManifest {
    fn default() -> Self {
        Self {
            app_id: "dev.tej.Application".to_string(),
            runtime: "org.freedesktop.Platform".to_string(),
            runtime_version: "23.08".to_string(),
            finish_args: vec![
                "--share=ipc".to_string(),
                "--socket=wayland".to_string(),
                "--socket=fallback-x11".to_string(),
                "--device=dri".to_string(),
            ],
        }
    }
}

/// Mobile Device Management (MDM) enterprise policy controller.
#[derive(Debug, Clone, Default)]
pub struct EnterprisePolicyManager {
    policies: HashMap<String, String>,
}

impl EnterprisePolicyManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads policies from enterprise configuration (e.g. Windows GPO or macOS MDM payload).
    pub fn set_policy(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.policies.insert(key.into(), value.into());
    }

    /// Retrieves an enterprise policy value.
    pub fn get_policy(&self, key: &str) -> Option<&str> {
        self.policies.get(key).map(|s| s.as_str())
    }

    /// Checks if telemetry is disabled by enterprise administrative decree.
    pub fn is_telemetry_disabled(&self) -> bool {
        self.get_policy("DisableTelemetry") == Some("true")
            || self.get_policy("DisableTelemetry") == Some("1")
    }

    /// Checks if enterprise corporate proxy is enforced.
    pub fn enterprise_proxy(&self) -> Option<&str> {
        self.get_policy("CorporateProxyUrl")
    }
}
