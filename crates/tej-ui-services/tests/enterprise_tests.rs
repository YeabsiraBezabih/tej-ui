//! Verification tests for Enterprise policies, packaging, telemetry, and sandbox security.

use tej_ui_services::{
    CrashReport, CrashReporter, EnterprisePolicyManager, FlatpakManifest, PluginCapability,
    PluginSandboxHost, SbomComponent, SbomGenerator, TelemetrySpan,
};

#[test]
fn test_enterprise_packaging_and_policies() {
    let mut policy_mgr = EnterprisePolicyManager::new();
    assert!(!policy_mgr.is_telemetry_disabled());
    policy_mgr.set_policy("DisableTelemetry", "true");
    policy_mgr.set_policy("CorporateProxyUrl", "https://proxy.corp.internal:8080");
    assert!(policy_mgr.is_telemetry_disabled());
    assert_eq!(
        policy_mgr.enterprise_proxy(),
        Some("https://proxy.corp.internal:8080")
    );

    let flatpak = FlatpakManifest::default();
    assert!(
        flatpak
            .finish_args
            .contains(&"--socket=wayland".to_string())
    );
}

#[test]
fn test_crash_reporting_and_telemetry() {
    let mut crash_reporter = CrashReporter::new();
    let crash = CrashReport::new("c-99", "SIGSEGV fatal crash in driver", "1.0.0")
        .with_stack_trace(vec!["0x7fff001".to_string(), "0x7fff025".to_string()]);
    crash_reporter.record_crash(crash);
    assert_eq!(crash_reporter.total_crashes(), 1);
    assert_eq!(
        crash_reporter.latest_report().unwrap().error_message,
        "SIGSEGV fatal crash in driver"
    );

    let span = TelemetrySpan::new("render_frame", 3500)
        .with_attr("frame_number", "420")
        .with_attr("thread", "main");
    assert_eq!(span.duration_us, 3500);
    assert_eq!(span.attributes.get("thread"), Some(&"main".to_string()));
}

#[test]
fn test_sbom_and_plugin_sandbox() {
    let mut sbom = SbomGenerator::new();
    sbom.add_component(SbomComponent::new("tej-core", "0.1.0", "MIT OR Apache-2.0"));
    sbom.add_component(SbomComponent::new(
        "parking_lot",
        "0.12.3",
        "MIT/Apache-2.0",
    ));
    let cyclonedx = sbom.export_cyclonedx_json();
    assert!(cyclonedx.contains("\"bomFormat\": \"CycloneDX\""));
    assert!(cyclonedx.contains("pkg:cargo/tej-core@0.1.0"));

    let mut sandbox = PluginSandboxHost::new("weather-widget", 1024 * 1024); // 1MB limit
    assert!(!sandbox.has_capability(PluginCapability::NetworkAccess));
    sandbox.grant_capability(PluginCapability::NetworkAccess);
    assert!(sandbox.has_capability(PluginCapability::NetworkAccess));

    // Memory allocation within budget
    assert!(sandbox.allocate_memory(512 * 1024));
    // Allocation exceeding budget fails
    assert!(!sandbox.allocate_memory(600 * 1024));
}
