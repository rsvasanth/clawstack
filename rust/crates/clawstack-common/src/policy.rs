//! Security and capability policies.

use crate::capability::{Capability, CapabilityGrant, Role};
use crate::skill::RiskLevel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A capability policy for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub version: u32,
    pub rules: Vec<PolicyRule>,
    pub default_grant: CapabilityGrant,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PolicyRule {
    /// Allow a specific capability for a role.
    Allow { role: Role, capabilities: Vec<Capability> },
    /// Deny a specific capability for a role.
    Deny { role: Role, capabilities: Vec<Capability> },
    /// Restrict a skill by risk level.
    RestrictSkill { risk_level: RiskLevel, message: Option<String> },
    /// Require confirmation before high-risk tools.
    ConfirmTool { tool_ids: Vec<String>, message: String },
    /// Block execution if condition is met.
    BlockIf { condition: PolicyCondition, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PolicyCondition {
    /// Block if input contains suspicious patterns.
    InputMatches { pattern: String },
    /// Block if tool output exceeds size threshold.
    OutputExceedsSize { bytes: u64 },
    /// Block if execution exceeds time threshold.
    ExecutionExceedsTime { ms: u64 },
}

/// Secret management policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretPolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    /// Allowed environment variables for tool execution.
    pub allowed_env_vars: Vec<String>,
    /// Blocked environment variables.
    pub blocked_env_vars: Vec<String>,
    /// Whether to inject API keys from the secret store.
    pub inject_secrets: bool,
}
