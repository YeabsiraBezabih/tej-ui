//! AI Assistant & Autonomous Agent Interface Primitives.
//!
//! Provides production-ready widgets for reasoning/thinking blocks, tool execution trays,
//! agent plan DAG cards, prompt composers with model pickers, and context token meters.

pub mod citations;
pub mod composer;
pub mod plan;
pub mod reasoning;
pub mod tool_call;

pub use citations::{ContextMeter, ContextMeterState, SourceCitation};
pub use composer::{ComposerState, ModelChoice, PromptComposer};
pub use plan::{AgentPlanCard, AgentPlanState, PlanStep, StepStatus};
pub use reasoning::{ReasoningDrawer, ReasoningDrawerState};
pub use tool_call::{ToolCallState, ToolCallTray, ToolExecutionStatus};
