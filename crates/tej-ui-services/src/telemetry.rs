//! Out-of-process crash reporting, minidumps, and OpenTelemetry trace exporter.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Structured crash metadata generated on fatal signals or unhandled panics.
#[derive(Debug, Clone, PartialEq)]
pub struct CrashReport {
    pub crash_id: String,
    pub timestamp_secs: u64,
    pub error_message: String,
    pub stack_trace: Vec<String>,
    pub memory_rss_mb: f64,
    pub app_version: String,
    pub os_info: String,
}

impl CrashReport {
    pub fn new(
        crash_id: impl Into<String>,
        error_message: impl Into<String>,
        app_version: impl Into<String>,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        Self {
            crash_id: crash_id.into(),
            timestamp_secs: ts,
            error_message: error_message.into(),
            stack_trace: Vec::new(),
            memory_rss_mb: 0.0,
            app_version: app_version.into(),
            os_info: std::env::consts::OS.to_string(),
        }
    }

    pub fn with_stack_trace(mut self, trace: Vec<String>) -> Self {
        self.stack_trace = trace;
        self
    }
}

/// In-memory / on-disk crash report writer.
#[derive(Debug, Clone, Default)]
pub struct CrashReporter {
    reports: Vec<CrashReport>,
}

impl CrashReporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_crash(&mut self, report: CrashReport) {
        self.reports.push(report);
    }

    pub fn latest_report(&self) -> Option<&CrashReport> {
        self.reports.last()
    }

    pub fn total_crashes(&self) -> usize {
        self.reports.len()
    }
}

/// OpenTelemetry-compatible tracing span for UI distributed observability.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetrySpan {
    pub name: String,
    pub duration_us: u64,
    pub attributes: HashMap<String, String>,
}

impl TelemetrySpan {
    pub fn new(name: impl Into<String>, duration_us: u64) -> Self {
        Self {
            name: name.into(),
            duration_us,
            attributes: HashMap::new(),
        }
    }

    pub fn with_attr(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), val.into());
        self
    }
}
