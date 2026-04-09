//! Execution context — assembled for every request.

use clawstack_common::{
    AgentRequest, CapabilityGrant, MemoryEntry, ToolCall, ToolResult, WorkspacePolicy,
};
use serde::{Deserialize, Serialize};

/// Context assembled during kernel execution.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub request: AgentRequest,
    /// Retrieved memories for context window.
    pub memories: Vec<MemoryEntry>,
    /// Active workspace policy.
    pub policy: WorkspacePolicy,
    /// Available tools.
    pub tools: Vec<crate::ToolSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub id: String,
    pub description: String,
    pub required_capabilities: Vec<CapabilityGrant>,
}

impl ExecutionContext {
    pub fn from_request(
        req: &AgentRequest,
        memories: Vec<MemoryEntry>,
        policy: WorkspacePolicy,
    ) -> Self {
        Self {
            request: req.clone(),
            memories,
            policy,
            tools: vec![],
        }
    }

    pub fn with_tools(mut self, tools: Vec<crate::ToolSpec>) -> Self {
        self.tools = tools;
        self
    }
}
