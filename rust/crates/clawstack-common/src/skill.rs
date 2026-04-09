//! Skill definition — reusable, versioned procedural knowledge.

use serde::{Deserialize, Serialize};

/// A registered skill with metadata and implementation reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub scope: SkillScope,
    pub inputs: Vec<SkillInput>,
    pub steps: Vec<String>,           // Human-readable step descriptions
    pub success_signals: Vec<String>,  // What "done" looks like
    pub risk_level: RiskLevel,
    pub required_capabilities: Vec<String>,
    pub source: SkillSource,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillScope {
    Coding,
    Data,
    Messaging,
    Web,
    System,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub schema: serde_json::Value,
}

/// Source of a skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    /// Built-in skill.
    Builtin,
    /// Installed from the skills registry.
    Installed,
    /// Synthesized by the Python learning service.
    Learned,
    /// Loaded from a WASM module.
    Wasm,
}

/// A draft skill proposed by Python learning service, awaiting approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDraft {
    pub id: Uuid,
    pub proposed_by: String, // Python service identifier
    pub skill: Skill,
    pub evidence_trajectory_ids: Vec<Uuid>,
    pub confidence_score: f32,
    pub status: DraftStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    Pending,
    Approved,
    Rejected,
    Superseded,
}

use uuid::Uuid;
