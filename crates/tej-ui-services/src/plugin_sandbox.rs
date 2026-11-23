//! Software Bill of Materials (SBOM) generation and C-ABI / WASM plugin sandbox.

use std::collections::HashSet;

/// A software component record in an SBOM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SbomComponent {
    pub name: String,
    pub version: String,
    pub license: String,
    pub purl: String,
}

impl SbomComponent {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        license: impl Into<String>,
    ) -> Self {
        let n = name.into();
        let v = version.into();
        let purl = format!("pkg:cargo/{}@{}", n, v);
        Self {
            name: n,
            version: v,
            license: license.into(),
            purl,
        }
    }
}

/// Software Bill of Materials (SBOM) metadata generator (CycloneDX/SPDX format).
#[derive(Debug, Clone, Default)]
pub struct SbomGenerator {
    components: Vec<SbomComponent>,
}

impl SbomGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_component(&mut self, component: SbomComponent) {
        self.components.push(component);
    }

    pub fn components(&self) -> &[SbomComponent] {
        &self.components
    }

    /// Serializes components into CycloneDX-compatible JSON string without external dependencies.
    pub fn export_cyclonedx_json(&self) -> String {
        let mut json = String::from(
            "{\n  \"bomFormat\": \"CycloneDX\",\n  \"specVersion\": \"1.4\",\n  \"components\": [\n",
        );
        for (i, c) in self.components.iter().enumerate() {
            let comma = if i + 1 < self.components.len() {
                ","
            } else {
                ""
            };
            json.push_str(&format!(
                "    {{\"type\": \"library\", \"name\": \"{}\", \"version\": \"{}\", \"purl\": \"{}\", \"licenses\": [{{\"license\": {{\"id\": \"{}\"}}}}]}}{}\n",
                c.name, c.version, c.purl, c.license, comma
            ));
        }
        json.push_str("  ]\n}");
        json
    }
}

/// Capabilities granted to a sandboxed plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    NetworkAccess,
    FileSystemRead,
    FileSystemWrite,
    HardwareAcceleration,
}

/// Sandboxed plugin runtime host with memory bounds and capability gating.
#[derive(Debug, Clone)]
pub struct PluginSandboxHost {
    plugin_name: String,
    granted_capabilities: HashSet<PluginCapability>,
    memory_limit_bytes: usize,
    current_memory_usage: usize,
}

impl PluginSandboxHost {
    pub fn new(plugin_name: impl Into<String>, memory_limit_bytes: usize) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            granted_capabilities: HashSet::new(),
            memory_limit_bytes,
            current_memory_usage: 0,
        }
    }

    pub fn grant_capability(&mut self, cap: PluginCapability) {
        self.granted_capabilities.insert(cap);
    }

    pub fn has_capability(&self, cap: PluginCapability) -> bool {
        self.granted_capabilities.contains(&cap)
    }

    /// Allocates memory within sandbox bounds, failing if limit is exceeded.
    pub fn allocate_memory(&mut self, bytes: usize) -> bool {
        if self.current_memory_usage + bytes <= self.memory_limit_bytes {
            self.current_memory_usage += bytes;
            true
        } else {
            false
        }
    }

    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    pub fn current_memory(&self) -> usize {
        self.current_memory_usage
    }
}
