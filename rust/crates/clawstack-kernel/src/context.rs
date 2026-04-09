//! Execution context — assembled for every request.

use clawstack_common::{
    AgentRequest, MemoryEntry, ToolSpec, WorkspacePolicy,
};

/// Context assembled during kernel execution.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub request: AgentRequest,
    /// Retrieved memories for context window.
    pub memories: Vec<MemoryEntry>,
    /// Active workspace policy.
    pub policy: WorkspacePolicy,
    /// Available tools.
    pub tools: Vec<ToolSpec>,
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

    pub fn with_tools(mut self, tools: Vec<ToolSpec>) -> Self {
        self.tools = tools;
        self
    }
}
