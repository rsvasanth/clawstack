//! Workspace — repo management and workspace-level state.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A git repository attached to a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub remote_url: String,
    pub local_path: Option<String>,
    pub default_branch: String,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Code session for a repo — isolated workspace within a code worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSession {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub job_id: Uuid,
    pub working_dir: String,
    pub git_state: String, // commit hash at session start
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A code plan — proposed changes for a coding task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePlan {
    pub id: Uuid,
    pub session_id: Uuid,
    pub task_description: String,
    pub steps: Vec<CodePlanStep>,
    pub status: CodePlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePlanStep {
    pub step_index: u32,
    pub action: String,
    pub target_files: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodePlanStatus {
    Planned,
    InProgress,
    Applied,
    Verified,
    Failed,
}
