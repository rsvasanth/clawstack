//! Central error types for ClawStack.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClawstackError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("capability denied: required={required}, granted={granted}")]
    CapabilityDenied { required: String, granted: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("tool execution failed: {tool_id}: {source}")]
    ToolExecutionFailed { tool_id: String, source: String },

    #[error("worker error: {job_id}: {source}")]
    WorkerError { job_id: String, source: String },

    #[error("database error: {0}")]
    Database(String),

    #[error("memory error: {0}")]
    Memory(String),

    #[error("skill error: {skill_id}: {source}")]
    SkillError { skill_id: String, source: String },

    #[error("policy violation: {0}")]
    PolicyViolation(String),

    #[error("channel error: {channel}: {source}")]
    ChannelError { channel: String, source: String },

    #[error("sandbox error: {0}")]
    Sandbox(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("internal: {0}")]
    Internal(String),
}

impl ClawstackError {
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound(format!("{entity} '{id}' not found"))
    }

    pub fn capability_denied(required: impl Into<String>, granted: impl Into<String>) -> Self {
        Self::CapabilityDenied {
            required: required.into(),
            granted: granted.into(),
        }
    }

    pub fn tool_failed(tool_id: impl Into<String>, source: impl Into<String>) -> Self {
        Self::ToolExecutionFailed {
            tool_id: tool_id.into(),
            source: source.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, ClawstackError>;
