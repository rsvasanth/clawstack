//! ClawStack Gateway — HTTP/WS API surface.
//!
//! Provides REST API endpoints for:
//! - Agent request/response
//! - Skill draft submission (from Python learning service)
//! - Health checks

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clawstack_common::ClawstackError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

/// Application state shared across handlers.
pub struct AppState {
    /// Reference to the agent kernel (wired up in main.rs)
    pub kernel: Arc<dyn crate::kernel::Kernel>,
}

/// Create the gateway router with all routes.
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any());

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/agent/request", post(agent_request_handler))
        .route("/api/v1/skills/drafts", post(skill_draft_handler))
        .with_state(state)
        .layer(cors)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Health check endpoint.
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "clawstack-gateway".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

/// Agent request endpoint — main entry point for agent requests.
async fn agent_request_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<crate::AgentRequest>,
) -> Result<Json<crate::AgentResponse>, (StatusCode, String)> {
    info!(session_id = %request.session_id, "received agent request");

    match state.kernel.run(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            warn!(error = %e, "agent request failed");
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// Skill draft submission endpoint — receives proposed skills from Python learning service.
async fn skill_draft_handler(
    State(state): State<Arc<AppState>>,
    Json(draft): Json<SkillDraftRequest>,
) -> Result<(StatusCode, Json<SkillDraftResponse>), (StatusCode, String)> {
    info!(
        draft_id = %draft.id,
        skill_name = %draft.skill.name,
        "received skill draft"
    );

    // In a full implementation, this would:
    // 1. Validate the draft
    // 2. Store it in the database
    // 3. Emit an event for approval workflow
    // 4. Return the draft with approved/pending status

    Ok((
        StatusCode::CREATED,
        Json(SkillDraftResponse {
            id: draft.id,
            status: "pending".to_string(),
            message: "Draft received and queued for review".to_string(),
        }),
    ))
}

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

/// Skill draft submission request from Python learning service.
#[derive(Debug, Deserialize)]
pub struct SkillDraftRequest {
    pub id: String,
    pub proposed_by: String,
    pub skill: SkillInfo,
    pub evidence_trajectory_ids: Vec<String>,
    pub confidence_score: f32,
    pub status: String,
}

/// Simplified skill info from draft.
#[derive(Debug, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub scope: String,
    pub steps: Vec<String>,
    pub risk_level: String,
    pub required_capabilities: Vec<String>,
}

/// Response after submitting a skill draft.
#[derive(Debug, Serialize)]
pub struct SkillDraftResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Kernel trait (to be implemented by main.rs wiring)
// ---------------------------------------------------------------------------

pub mod kernel {
    use super::*;
    use clawstack_common::AgentRequest;

    /// Kernel trait for testability and dependency injection.
    pub trait Kernel: Send + Sync {
        async fn run(&self, request: AgentRequest) -> Result<AgentResponse, ClawstackError>;
    }
}
