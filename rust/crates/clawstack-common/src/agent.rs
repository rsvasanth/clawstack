//! Agent request/response types — the primary interface to the kernel.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An incoming request to the agent kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub workspace_id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    pub text: String,
    pub attachments: Vec<Attachment>,
    pub metadata: serde_json::Value,
}

/// An outgoing response from the agent kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub text: Option<String>,
    pub attachments: Vec<serde_json::Value>,
    pub tool_calls: Vec<super::ToolCall>,
    pub tool_results: Vec<super::ToolResult>,
    pub session_id: Option<Uuid>,
}

/// An attachment (file, image, etc.) in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub kind: String,
    pub url: Option<String>,
    pub content: Option<String>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
}
