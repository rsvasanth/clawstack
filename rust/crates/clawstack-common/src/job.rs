//! Job orchestration — for async, worker, and scheduled tasks.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job submitted to the scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub priority: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Async tool execution.
    ToolAsync,
    /// Code worker job (repo task).
    CodeWorker,
    /// Browser automation job.
    BrowserWorker,
    /// ETL / reporting job.
    EtlWorker,
    /// Scheduled routine.
    Scheduled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    DeadLetter,
}

/// A scheduled routine definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledRoutine {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub enabled: bool,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
