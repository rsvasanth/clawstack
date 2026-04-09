//! Core identity entities: User, Workspace, Session, Channel.

use crate::capability::{CapabilityGrant, Role};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A workspace groups users, sessions, and policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_id: Uuid,
    pub default_role: Role,
    pub capability_grants: HashMap<Uuid, CapabilityGrant>, // user_id -> grants
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A channel (Telegram, Slack, CLI, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelKind {
    Telegram,
    Slack,
    Discord,
    WhatsApp,
    WebSocket,
    Http,
    Cli,
}

/// A channel binding for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub kind: ChannelKind,
    pub external_id: String, // Telegram chat ID, Slack channel ID, etc.
    pub bot_token_encrypted: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A session — a single conversational context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub current_workspace_slug: Option<String>, // For multi-repo sessions
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

/// An active conversation (a thread within a session).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub session_id: Uuid,
    pub channel: ChannelKind,
    pub thread_external_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
