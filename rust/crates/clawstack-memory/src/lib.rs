//! Memory provider implementation using Postgres + pgvector.
//!
//! Implements the `MemoryProvider` trait from `clawstack-kernel`.
//! Provides trajectory storage and retrieval with pgvector similarity search.

use async_trait::async_trait;
use clawstack_common::{
    ClawstackError, MemoryEntry, MemoryTier, MessageRole, Trajectory,
    TrajectoryOutcome, TrajectoryToolCall, TrajectoryTurn,
};
use clawstack_kernel::{
    context::ExecutionContext,
    loop_::MemoryProvider,
};
use sqlx::{postgres::PgPool, PgPool as SqlxPgPool};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Postgres-backed memory provider.
pub struct PostgresMemoryProvider {
    pool: PgPool,
}

impl PostgresMemoryProvider {
    /// Create a new provider with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new provider from a database URL.
    pub async fn from_url(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl MemoryProvider for PostgresMemoryProvider {
    /// Retrieve memories for context building.
    ///
    /// Currently fetches recent short-term and mid-term memories.
    /// TODO: Add pgvector similarity search when embeddings are available.
    async fn retrieve(
        &self,
        ctx: &ExecutionContext,
        limit: u32,
    ) -> Result<Vec<MemoryEntry>, ClawstackError> {
        let rows = sqlx::query_as::<_, MemoryRow>(
            r#"
            SELECT id, workspace_id, user_id, session_id, tier, content,
                   tags, provenance, created_at, accessed_at, expires_at
            FROM memories
            WHERE workspace_id = $1
              AND (tier = 'short_term' OR tier = 'mid_term')
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(ctx.request.workspace_id)
        .bind(limit as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClawstackError::Database(format!("Failed to retrieve memories: {}", e)))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Store a trajectory for learning.
    async fn store(&self, trajectory: &Trajectory) -> Result<(), ClawstackError> {
        let turns_json = serde_json::to_value(&trajectory.turns)
            .map_err(|e| ClawstackError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO trajectories (id, workspace_id, session_id, user_message, outcome, duration_ms, model, turns)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                outcome = EXCLUDED.outcome,
                duration_ms = EXCLUDED.duration_ms,
                turns = EXCLUDED.turns
            "#,
        )
        .bind(trajectory.id)
        .bind(trajectory.workspace_id)
        .bind(trajectory.session_id)
        .bind(&trajectory.user_message)
        .bind(serde_json::to_string(&trajectory.outcome).unwrap_or_default())
        .bind(trajectory.duration_ms as i64)
        .bind(&trajectory.model)
        .bind(turns_json)
        .execute(&self.pool)
        .await
        .map_err(|e| ClawstackError::Database(format!("Failed to store trajectory: {}", e)))?;

        info!(
            trajectory_id = %trajectory.id,
            workspace_id = %trajectory.workspace_id,
            outcome = ?trajectory.outcome,
            "trajectory stored"
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SQLx row types
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct MemoryRow {
    id: Uuid,
    workspace_id: Uuid,
    user_id: Option<Uuid>,
    session_id: Option<Uuid>,
    tier: String,
    content: String,
    tags: Vec<String>,
    provenance: String,
    created_at: chrono::DateTime<chrono::Utc>,
    accessed_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<MemoryRow> for MemoryEntry {
    fn from(row: MemoryRow) -> Self {
        MemoryEntry {
            id: row.id,
            workspace_id: row.workspace_id,
            user_id: row.user_id,
            session_id: row.session_id,
            tier: match row.tier.as_str() {
                "short_term" => MemoryTier::ShortTerm,
                "mid_term" => MemoryTier::MidTerm,
                "long_term" => MemoryTier::LongTerm,
                _ => MemoryTier::ShortTerm,
            },
            content: row.content,
            embedding: None, // TODO: Load from memory_embeddings table
            tags: row.tags,
            provenance: row.provenance,
            created_at: row.created_at,
            accessed_at: row.accessed_at,
            expires_at: row.expires_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Trajectory storage helpers
// ---------------------------------------------------------------------------

/// Convert TrajectoryOutcome to string for DB storage
impl From<TrajectoryOutcome> for String {
    fn from(outcome: TrajectoryOutcome) -> Self {
        match outcome {
            TrajectoryOutcome::CompletedChat => "completed_chat".to_string(),
            TrajectoryOutcome::CompletedTools => "completed_tools".to_string(),
            TrajectoryOutcome::Partial => "partial".to_string(),
            TrajectoryOutcome::Failed => "failed".to_string(),
            TrajectoryOutcome::BlockedPolicy => "blocked_policy".to_string(),
            TrajectoryOutcome::BlockedSafety => "blocked_safety".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory fallback for testing
// ---------------------------------------------------------------------------

/// In-memory memory provider for testing without a database.
pub struct InMemoryMemoryProvider {
    memories: std::sync::RwLock<Vec<MemoryEntry>>,
    trajectories: std::sync::RwLock<Vec<Trajectory>>,
}

impl InMemoryMemoryProvider {
    pub fn new() -> Self {
        Self {
            memories: std::sync::RwLock::new(Vec::new()),
            trajectories: std::sync::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryMemoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MemoryProvider for InMemoryMemoryProvider {
    async fn retrieve(
        &self,
        ctx: &ExecutionContext,
        limit: u32,
    ) -> Result<Vec<MemoryEntry>, ClawstackError> {
        let memories = self.memories.read().unwrap();
        let filtered: Vec<MemoryEntry> = memories
            .iter()
            .filter(|m| m.workspace_id == ctx.request.workspace_id)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn store(&self, trajectory: &Trajectory) -> Result<(), ClawstackError> {
        let mut trajectories = self.trajectories.write().unwrap();
        trajectories.push(trajectory.clone());
        info!(
            trajectory_id = %trajectory.id,
            "trajectory stored (in-memory)"
        );
        Ok(())
    }
}
