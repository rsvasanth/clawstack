//! Trajectory tracking — stores multi-tool execution sequences for learning.

use crate::memory::MessageRole;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A trajectory — a complete multi-tool execution sequence from a single user turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub session_id: Uuid,
    /// The user message that initiated this trajectory.
    pub user_message: String,
    /// All turns in the trajectory.
    pub turns: Vec<TrajectoryTurn>,
    /// Final outcome.
    pub outcome: TrajectoryOutcome,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    pub model: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryTurn {
    pub turn_index: u32,
    pub role: MessageRole,
    pub content: String,
    /// Tool calls made in this turn.
    pub tool_calls: Vec<TrajectoryToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryToolCall {
    pub tool_id: String,
    pub parameters: serde_json::Value,
    pub output: Option<String>,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrajectoryOutcome {
    /// Completed successfully without tool use.
    CompletedChat,
    /// Completed successfully using tools.
    CompletedTools,
    /// Partially completed, user provided clarification.
    Partial,
    /// Failed.
    Failed,
    /// Blocked by capability policy.
    BlockedPolicy,
    /// Blocked by safety filter.
    BlockedSafety,
}
