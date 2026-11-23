//! Developer Tools & Diagnostics Component Suite.
//!
//! Provides virtualized FlameGraphs, distributed Trace Waterfalls, Git commit DAG graphs,
//! CI/CD pipeline visualizers, interactive Regex testers, and system Uptime bars.

pub mod diagnostics;
pub mod flame_graph;
pub mod git_graph;
pub mod pipeline;
pub mod regex_tester;
pub mod trace_waterfall;

pub use diagnostics::{DiagnosticProblem, DiagnosticSeverity, KeyValueEntry, UptimeBar, UptimeDay};
pub use flame_graph::{FlameGraph, FlameGraphState, FlameNode};
pub use git_graph::{GitCommit, GitGraph, GitGraphState};
pub use pipeline::{PipelineGraph, PipelineStage, PipelineState, PipelineStatus};
pub use regex_tester::{RegexMatch, RegexTester, RegexTesterState};
pub use trace_waterfall::{TimingPhase, TraceSpan, TraceWaterfall, TraceWaterfallState};
