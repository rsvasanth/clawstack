//! ToolBus implementation with built-in tools.
//!
//! Provides a concrete implementation of the `ToolBus` trait from `clawstack-kernel`.
//! Includes built-in tools: echo, memory.read, memory.write, web.fetch

use async_trait::async_trait;
use clawstack_common::{
    Capability, ClawstackError, ExecutionMode, SideEffectClass, ToolCall,
    ToolResult, ToolSource, ToolSpec,
};
use clawstack_kernel::{
    context::ExecutionContext,
    loop_::ToolBus,
};
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// ToolBus implementation with built-in tools.
pub struct ToolBusImpl {
    /// Registered tools by ID
    tools: Arc<RwLock<HashMap<String, BuiltInTool>>>,
    /// HTTP client for web.fetch
    client: reqwest::Client,
}

impl ToolBusImpl {
    /// Create a new ToolBus with built-in tools registered.
    pub fn new() -> Self {
        let tools = HashMap::from([
            ("echo".to_string(), BuiltInTool::Echo),
            ("memory.read".to_string(), BuiltInTool::MemoryRead),
            ("memory.write".to_string(), BuiltInTool::MemoryWrite),
            ("web.fetch".to_string(), BuiltInTool::WebFetch),
        ]);

        Self {
            tools: Arc::new(RwLock::new(tools)),
            client: Client::new(),
        }
    }

    /// Register a new tool.
    pub async fn register(&self, tool: BuiltInTool) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.spec().id.clone(), tool);
    }

    /// Get a tool's specification.
    fn get_spec_internal(tool_id: &str) -> Option<ToolSpec> {
        let built_in = match tool_id {
            "echo" => BuiltInTool::Echo,
            "memory.read" => BuiltInTool::MemoryRead,
            "memory.write" => BuiltInTool::MemoryWrite,
            "web.fetch" => BuiltInTool::WebFetch,
            _ => return None,
        };
        Some(built_in.spec())
    }
}

impl Default for ToolBusImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolBus for ToolBusImpl {
    async fn execute(
        &self,
        call: ToolCall,
        ctx: &ExecutionContext,
    ) -> Result<ToolResult, ClawstackError> {
        let start = Instant::now();
        let tool_id = call.tool_id.clone();

        info!(tool_id = %tool_id, "executing tool");

        let result = match tool_id.as_str() {
            "echo" => execute_echo(&call).await,
            "memory.read" => execute_memory_read(&call, ctx).await,
            "memory.write" => execute_memory_write(&call, ctx).await,
            "web.fetch" => execute_web_fetch(&call, &self.client).await,
            _ => Err(ClawstackError::not_found("tool", &tool_id)),
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => Ok(ToolResult {
                call_id: call.call_id,
                tool_id,
                output: serde_json::json!({ "result": output }),
                success: true,
                duration_ms,
                error: None,
                audit_events: vec![],
            }),
            Err(e) => Ok(ToolResult {
                call_id: call.call_id,
                tool_id,
                output: serde_json::Value::Null,
                success: false,
                duration_ms,
                error: Some(e.to_string()),
                audit_events: vec![],
            }),
        }
    }

    async fn execute_many(
        &self,
        calls: Vec<ToolCall>,
        ctx: &ExecutionContext,
    ) -> Result<Vec<ToolResult>, ClawstackError> {
        // Parallel execution using join_all
        let futures = calls
            .into_iter()
            .map(|call| self.execute(call, ctx))
            .collect::<Vec<_>>();

        join_all(futures).await.into_iter().collect()
    }

    fn get_spec(&self, tool_id: &str) -> Option<ToolSpec> {
        Self::get_spec_internal(tool_id)
    }
}

// ---------------------------------------------------------------------------
// Built-in tools
// ---------------------------------------------------------------------------

/// Built-in tool enum.
#[derive(Clone)]
pub enum BuiltInTool {
    Echo,
    MemoryRead,
    MemoryWrite,
    WebFetch,
}

impl BuiltInTool {
    /// Get the tool specification.
    pub fn spec(&self) -> ToolSpec {
        match self {
            Self::Echo => ToolSpec {
                id: "echo".to_string(),
                description: "Echoes back the input text. Useful for testing.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "Text to echo back" }
                    },
                    "required": ["text"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "echoed": { "type": "string" }
                    }
                }),
                required_capabilities: vec![],
                execution_mode: ExecutionMode::Sync,
                timeout_ms: 1000,
                side_effect_class: SideEffectClass::None,
                audit_tags: vec!["test".to_string()],
                source: ToolSource::Native,
            },
            Self::MemoryRead => ToolSpec {
                id: "memory.read".to_string(),
                description: "Reads recent memories from the memory system.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "description": "Max memories to return", "default": 10 }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memories": { "type": "array" }
                    }
                }),
                required_capabilities: vec![Capability::MemoryRead],
                execution_mode: ExecutionMode::Sync,
                timeout_ms: 5000,
                side_effect_class: SideEffectClass::Read,
                audit_tags: vec!["memory".to_string(), "read".to_string()],
                source: ToolSource::Native,
            },
            Self::MemoryWrite => ToolSpec {
                id: "memory.write".to_string(),
                description: "Writes a memory to the memory system.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Memory content to store" },
                        "tier": { "type": "string", "enum": ["short_term", "mid_term", "long_term"], "default": "short_term" },
                        "tags": { "type": "array", "items": { "type": "string" }, "default": [] }
                    },
                    "required": ["content"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "stored": { "type": "boolean" },
                        "memory_id": { "type": "string" }
                    }
                }),
                required_capabilities: vec![Capability::MemoryWrite],
                execution_mode: ExecutionMode::Sync,
                timeout_ms: 5000,
                side_effect_class: SideEffectClass::Write,
                audit_tags: vec!["memory".to_string(), "write".to_string()],
                source: ToolSource::Native,
            },
            Self::WebFetch => ToolSpec {
                id: "web.fetch".to_string(),
                description: "Fetches content from a URL.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "format": "uri", "description": "URL to fetch" },
                        "method": { "type": "string", "enum": ["GET", "POST"], "default": "GET" }
                    },
                    "required": ["url"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": { "type": "integer" },
                        "body": { "type": "string" },
                        "headers": { "type": "object" }
                    }
                }),
                required_capabilities: vec![Capability::WebFetch],
                execution_mode: ExecutionMode::Sync,
                timeout_ms: 30000,
                side_effect_class: SideEffectClass::Read,
                audit_tags: vec!["web".to_string(), "fetch".to_string()],
                source: ToolSource::Native,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tool executors
// ---------------------------------------------------------------------------

async fn execute_echo(call: &ToolCall) -> Result<String, ClawstackError> {
    let text = call
        .parameters
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClawstackError::InvalidInput("echo requires 'text' parameter".to_string()))?;

    Ok(text.to_string())
}

async fn execute_memory_read(
    call: &ToolCall,
    ctx: &ExecutionContext,
) -> Result<String, ClawstackError> {
    let limit = call
        .parameters
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let memories: Vec<String> = ctx
        .memories
        .iter()
        .take(limit)
        .map(|m| m.content.clone())
        .collect();

    Ok(serde_json::to_string(&memories).unwrap_or_else(|_| "[]".to_string()))
}

async fn execute_memory_write(
    call: &ToolCall,
    ctx: &ExecutionContext,
) -> Result<String, ClawstackError> {
    let content = call
        .parameters
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ClawstackError::InvalidInput("memory.write requires 'content' parameter".to_string())
        })?;

    // In a real implementation, this would store to the memory provider.
    // For now, we just return success.
    info!(
        workspace_id = %ctx.request.workspace_id,
        content = %content,
        "memory.write called (no-op - memory provider not yet wired)"
    );

    Ok(format!(
        r#"{{"stored": true, "memory_id": "mock-{}", "content": "{}"}}"#,
        uuid::Uuid::new_v4(),
        content
    ))
}

async fn execute_web_fetch(
    call: &ToolCall,
    client: &reqwest::Client,
) -> Result<String, ClawstackError> {
    let url = call
        .parameters
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ClawstackError::InvalidInput("web.fetch requires 'url' parameter".to_string()))?;

    let method = call
        .parameters
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET");

    let response = match method {
        "POST" => client.post(url).send().await,
        _ => client.get(url).send().await,
    }
    .map_err(|e| ClawstackError::Internal(format!("web.fetch failed: {}", e)))?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| ClawstackError::Internal(format!("web.fetch read failed: {}", e)))?;

    Ok(serde_json::json!({
        "status": status,
        "body": body,
        "headers": {}
    })
    .to_string())
}
