//! System events — emitted by all layers for observability.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A system lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClawstackEvent {
    /// Agent started processing a request.
    RequestStart(RequestStartEvent),
    /// Agent finished processing.
    RequestEnd(RequestEndEvent),
    /// Agent error.
    RequestError(RequestErrorEvent),
    /// Tool was called.
    ToolCall(ToolCallEvent),
    /// Tool completed.
    ToolResult(ToolResultEvent),
    /// Job state changed.
    JobStateChange(JobStateChangeEvent),
    /// Skill was installed or updated.
    SkillChange(SkillChangeEvent),
    /// Policy was violated.
    PolicyViolation(PolicyViolationEvent),
    /// Trajectory was completed and stored.
    TrajectoryStored(TrajectoryStoredEvent),
    /// Learning service proposed a skill draft.
    SkillDraftProposed(SkillDraftProposedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStartEvent {
    pub trace_id: Uuid,
    pub session_id: Uuid,
    pub channel: String,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEndEvent {
    pub trace_id: Uuid,
    pub outcome: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestErrorEvent {
    pub trace_id: Uuid,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub trace_id: Uuid,
    pub tool_id: String,
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultEvent {
    pub trace_id: Uuid,
    pub tool_id: String,
    pub call_id: String,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStateChangeEvent {
    pub job_id: Uuid,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChangeEvent {
    pub skill_id: String,
    pub action: String, // "installed", "updated", "removed"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationEvent {
    pub trace_id: Uuid,
    pub workspace_id: Uuid,
    pub rule_id: Uuid,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStoredEvent {
    pub trace_id: Uuid,
    pub trajectory_id: Uuid,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDraftProposedEvent {
    pub draft_id: Uuid,
    pub proposed_by: String,
    pub skill_name: String,
}
