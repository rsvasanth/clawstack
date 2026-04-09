//! Three-tier memory model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Memory tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    /// Recent conversation turns and in-flight state.
    ShortTerm,
    /// Summaries, session notes, active tasks.
    MidTerm,
    /// Vectorized facts, preferences, learned skills, trajectories.
    LongTerm,
}

/// A memory entry with optional vector embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub tier: MemoryTier,
    /// Human-readable content.
    pub content: String,
    /// Vector embedding (for long-term tier).
    pub embedding: Option<Vec<f32>>,
    /// Tags for filtering.
    pub tags: Vec<String>,
    /// Source: "user", "assistant", "skill", "trajectory", "summary"
    pub provenance: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A conversation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<Vec<super::tool::ToolCall>>,
    pub tool_results: Option<Vec<super::tool::ToolResult>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Hybrid search query for memory retrieval.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub tags: Option<Vec<String>>,
    pub workspace_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub tier: Option<MemoryTier>,
    pub limit: u32,
    pub offset: u32,
}
