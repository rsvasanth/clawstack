//! OpenAI-compatible LLM client for ClawStack kernel.
//!
//! Implements the `LlmClient` trait from `clawstack-kernel`.
//! Supports tool calls via OpenAI's function calling API.

use async_trait::async_trait;
use clawstack_common::{
    AgentRequest, ClawstackError, ToolCall, ToolResult,
};
use clawstack_kernel::{
    context::ExecutionContext, loop_::LlmClient, loop_::LlmResponse,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

/// OpenAI chat completions API endpoint
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI model to use
const DEFAULT_MODEL: &str = "gpt-4o";

/// OpenAI-compatible LLM client.
pub struct OpenAiLlmClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiLlmClient {
    /// Create a new OpenAI LLM client.
    ///
    /// Reads `OPENAI_API_KEY` from environment.
    /// Optionally set `OPENAI_BASE_URL` for custom endpoints (e.g., Azure, proxies).
    pub fn new() -> anyhow::Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set"))?;

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| OPENAI_API_URL.to_string());

        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
            base_url,
        })
    }

    /// Build the system prompt with tool definitions for the agent.
    fn build_system_prompt(ctx: &ExecutionContext) -> String {
        let mut prompt = String::from(
            "You are ClawStack, a helpful AI assistant with access to tools.\n\n"
        );

        if !ctx.tools.is_empty() {
            prompt.push_str("## Available Tools\n\n");
            for tool in &ctx.tools {
                prompt.push_str(&format!(
                    "### {}\n{}\nCapabilities required: {:?}\n\n",
                    tool.id,
                    tool.description,
                    tool.required_capabilities
                ));
            }
            prompt.push_str("\nUse tools when helpful to answer the user's question.\n");
        }

        // Add memory context if available
        if !ctx.memories.is_empty() {
            prompt.push_str("\n## Relevant Context\n\n");
            for mem in ctx.memories.iter().take(5) {
                prompt.push_str(&format!("- {}\n", mem.content));
            }
        }

        prompt
    }
}

#[async_trait]
impl LlmClient for OpenAiLlmClient {
    async fn complete(
        &self,
        prompt: &str,
        ctx: &ExecutionContext,
    ) -> Result<LlmResponse, ClawstackError> {
        let system_prompt = Self::build_system_prompt(ctx);

        // Build messages
        let messages = serde_json::json!([
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": prompt }
        ]);

        // Build request - include tools if available
        let mut request_body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 2048,
        });

        // Add tools to the request if we have any
        if !ctx.tools.is_empty() {
            let tools: Vec<serde_json::Value> = ctx.tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.id,
                            "description": t.description,
                            "parameters": t.required_capabilities, // TODO: Use proper JSON schema
                        }
                    })
                })
                .collect();

            request_body["tools"] = serde_json::json!(tools);
        }

        info!(model = %self.model, "calling LLM");

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ClawstackError::Internal(format!("LLM request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "LLM request failed");
            return Err(ClawstackError::Internal(format!(
                "LLM API error {}: {}",
                status, body
            )));
        }

        let response_body: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| ClawstackError::Internal(format!("Failed to parse LLM response: {}", e)))?;

        // Extract text response
        let text = response_body
            .choices
            .first()
            .and_then(|c| c.message.content.clone());

        // Extract tool calls if present
        let tool_calls = response_body
            .choices
            .first()
            .and_then(|c| c.message.tool_calls.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|tc| ToolCall {
                call_id: tc.id,
                tool_id: tc.function.name,
                parameters: serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::Null),
                context: Default::default(),
            })
            .collect();

        let finish_reason = response_body
            .choices
            .first()
            .map(|c| c.finish_reason.clone())
            .unwrap_or_else(|| "stop".to_string());

        Ok(LlmResponse {
            text,
            tool_calls,
            finish_reason,
        })
    }
}

// ---------------------------------------------------------------------------
// OpenAI API types (response structures)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    index: u32,
    message: OpenAiMessage,
    finish_reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenAiMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenAiToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAiFunctionCall,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiFunctionCall {
    name: String,
    arguments: String, // JSON string
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Mock LLM client for testing without API key
// ---------------------------------------------------------------------------

/// A mock LLM client that returns predefined responses.
/// Useful for testing and development without API credentials.
pub struct MockLlmClient {
    response_text: Option<String>,
    tool_calls: Vec<ToolCall>,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self {
            response_text: Some("This is a mock response.".to_string()),
            tool_calls: vec![],
        }
    }

    pub fn with_response(mut self, text: impl Into<String>) -> Self {
        self.response_text = Some(text.into());
        self
    }

    pub fn with_tool_call(mut self, tool_id: impl Into<String>, params: serde_json::Value) -> Self {
        self.tool_calls.push(ToolCall {
            call_id: uuid::Uuid::new_v4().to_string(),
            tool_id: tool_id.into(),
            parameters: params,
            context: Default::default(),
        });
        self
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(
        &self,
        _prompt: &str,
        _ctx: &ExecutionContext,
    ) -> Result<LlmResponse, ClawstackError> {
        Ok(LlmResponse {
            text: self.response_text.clone(),
            tool_calls: self.tool_calls.clone(),
            finish_reason: "stop".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// In-memory LLM client for development
// ---------------------------------------------------------------------------

/// Simple in-memory LLM that echoes back the prompt with a prefix.
/// Does not call any external API.
pub struct EchoLlmClient {
    prefix: String,
}

impl EchoLlmClient {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl LlmClient for EchoLlmClient {
    async fn complete(
        &self,
        prompt: &str,
        _ctx: &ExecutionContext,
    ) -> Result<LlmResponse, ClawstackError> {
        Ok(LlmResponse {
            text: Some(format!("{}{}", self.prefix, prompt)),
            tool_calls: vec![],
            finish_reason: "stop".to_string(),
        })
    }
}
