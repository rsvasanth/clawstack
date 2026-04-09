//! Unified Tool specification — every tool has the same interface.

use crate::capability::Capability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Side-effect classification for audit and policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffectClass {
    None,  // Pure computation, no side effects
    Read,  // Reads data (filesystem, network GET, etc.)
    Write, // Writes data (filesystem, network POST, etc.)
    Exec,  // Executes code or commands
    Admin, // Administrative actions
}

/// Execution mode for a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Runs synchronously in the kernel process.
    Sync,
    /// Runs asynchronously, returns immediately with a job ID.
    Async,
    /// Runs in an isolated worker (Docker/WASM).
    Worker,
}

/// Unified tool specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Unique tool identifier.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for tool input.
    pub input_schema: serde_json::Value,
    /// JSON Schema for tool output.
    pub output_schema: serde_json::Value,
    /// Capabilities required to execute this tool.
    pub required_capabilities: Vec<Capability>,
    /// How this tool is executed.
    pub execution_mode: ExecutionMode,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Side-effect classification.
    pub side_effect_class: SideEffectClass,
    /// Tags for audit logging.
    pub audit_tags: Vec<String>,
    /// Whether this tool is built-in or a loaded skill.
    pub source: ToolSource,
}

/// Where a tool was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSource {
    /// Native Rust tool.
    Native,
    /// Loaded WASM module.
    Wasm,
    /// Loaded Docker worker image.
    Docker,
    /// MCP external tool.
    Mcp,
    /// Python learning service endpoint.
    Python,
}

/// A tool call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub tool_id: String,
    pub parameters: serde_json::Value,
    pub context: ToolCallContext,
}

/// Context attached to a tool call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCallContext {
    pub workspace_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub requested_capabilities: Vec<Capability>,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_id: String,
    /// Serialized result JSON.
    pub output: serde_json::Value,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub audit_events: Vec<AuditEntry>,
}

/// Entry in the audit log for a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub resource: String,
    pub outcome: String,
}
