//! Policy provider implementation using Postgres.
//!
//! Implements the `PolicyProvider` trait from `clawstack-kernel`.
//! Evaluates capability grants against workspace policies.

use async_trait::async_trait;
use clawstack_common::{
    Capability, CapabilityGrant, ClawstackError, Role, WorkspacePolicy,
};
use clawstack_kernel::{
    context::ExecutionContext,
    loop_::{PolicyProvider, PolicyRule},
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Postgres-backed policy provider.
pub struct PostgresPolicyProvider {
    pool: PgPool,
    /// Cache of workspace_id -> (policy, user_roles)
    cache: std::sync::RwLock<HashMap<Uuid, CachedPolicy>>,
}

struct CachedPolicy {
    policy: WorkspacePolicy,
    user_roles: HashMap<Uuid, Role>,
    cached_at: std::time::Instant,
}

const CACHE_TTL_SECS: u64 = 60;

impl PostgresPolicyProvider {
    /// Create a new provider with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Create a new provider from a database URL.
    pub async fn from_url(database_url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::PgPool::connect(database_url).await?;
        Ok(Self::new(pool))
    }

    /// Get the workspace policy, with caching.
    async fn get_policy_cached(
        &self,
        workspace_id: Uuid,
    ) -> Result<(WorkspacePolicy, HashMap<Uuid, Role>), ClawstackError> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(&workspace_id) {
                if cached.cached_at.elapsed().as_secs() < CACHE_TTL_SECS {
                    return Ok((cached.policy.clone(), cached.user_roles.clone()));
                }
            }
        }

        // Fetch from database
        let policy = self.fetch_policy(workspace_id).await?;
        let user_roles = self.fetch_user_roles(workspace_id).await?;

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(workspace_id, CachedPolicy {
                policy: policy.clone(),
                user_roles: user_roles.clone(),
                cached_at: std::time::Instant::now(),
            });
        }

        Ok((policy, user_roles))
    }

    /// Fetch policy from database.
    async fn fetch_policy(&self, workspace_id: Uuid) -> Result<WorkspacePolicy, ClawstackError> {
        let row = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, workspace_id, version, rules, default_grant, created_at, updated_at
            FROM workspace_policies
            WHERE workspace_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ClawstackError::Database(format!("Failed to fetch policy: {}", e)))?;

        match row {
            Some(row) => Ok(WorkspacePolicy {
                id: row.id,
                workspace_id: row.workspace_id,
                version: row.version,
                rules: serde_json::from_value(row.rules)
                    .unwrap_or_default(),
                default_grant: serde_json::from_value(row.default_grant)
                    .unwrap_or_default(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            }),
            None => {
                // Return default policy if none exists
                info!(workspace_id = %workspace_id, "no policy found, using default");
                Ok(WorkspacePolicy {
                    id: Uuid::new_v4(),
                    workspace_id,
                    version: 1,
                    rules: vec![],
                    default_grant: CapabilityGrant::default(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
            }
        }
    }

    /// Fetch user roles from database.
    async fn fetch_user_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<HashMap<Uuid, Role>, ClawstackError> {
        let rows = sqlx::query_as::<_, UserRoleRow>(
            r#"
            SELECT user_id, role
            FROM workspace_members
            WHERE workspace_id = $1
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ClawstackError::Database(format!("Failed to fetch user roles: {}", e)))?;

        let mut roles = HashMap::new();
        for row in rows {
            roles.insert(row.user_id, Role::from_str(&row.role));
        }
        Ok(roles)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyRow {
    id: Uuid,
    workspace_id: Uuid,
    version: u32,
    rules: serde_json::Value,
    default_grant: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct UserRoleRow {
    user_id: Uuid,
    role: String,
}

#[async_trait]
impl PolicyProvider for PostgresPolicyProvider {
    async fn get_policy(
        &self,
        workspace_id: Uuid,
    ) -> Result<WorkspacePolicy, ClawstackError> {
        let (policy, _) = self.get_policy_cached(workspace_id).await?;
        Ok(policy)
    }

    async fn check_capability(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        capability: Capability,
    ) -> Result<bool, ClawstackError> {
        let (policy, user_roles) = self.get_policy_cached(workspace_id).await?;

        // Get user's role
        let role = user_roles.get(&user_id).copied().unwrap_or(Role::Guest);

        // Get default grants for the role
        let role_grants = role.default_grants();

        // Check if role grants the capability
        if !role_grants.grants(capability) {
            warn!(
                workspace_id = %workspace_id,
                user_id = %user_id,
                role = ?role,
                capability = ?capability,
                "capability denied by role default"
            );
            return Ok(false);
        }

        // Evaluate policy rules
        for rule in &policy.rules {
            match rule {
                PolicyRule::Allow { role: rule_role, capabilities } => {
                    if *rule_role == role && capabilities.contains(&capability) {
                        info!(
                            workspace_id = %workspace_id,
                            user_id = %user_id,
                            capability = ?capability,
                            "capability allowed by policy rule"
                        );
                        return Ok(true);
                    }
                }
                PolicyRule::Deny { role: rule_role, capabilities } => {
                    if *rule_role == role && capabilities.contains(&capability) {
                        warn!(
                            workspace_id = %workspace_id,
                            user_id = %user_id,
                            capability = ?capability,
                            "capability denied by policy rule"
                        );
                        return Ok(false);
                    }
                }
                _ => {} // Other rules don't affect capability checks
            }
        }

        // Fall back to role default grants
        Ok(role_grants.grants(capability))
    }
}

// ---------------------------------------------------------------------------
// Role extension
// ---------------------------------------------------------------------------

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s {
            "owner" => Role::Owner,
            "admin" => Role::Admin,
            "developer" => Role::Developer,
            "guest" => Role::Guest,
            "worker" => Role::Worker,
            _ => Role::Guest,
        }
    }
}

// ---------------------------------------------------------------------------
// Default implementation for CapabilityGrant
// ---------------------------------------------------------------------------

impl Default for CapabilityGrant {
    fn default() -> Self {
        Self {
            allow: Capability::all(),
            deny: std::collections::HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory fallback for testing
// ---------------------------------------------------------------------------

/// In-memory policy provider for testing without a database.
pub struct InMemoryPolicyProvider {
    policies: std::sync::RwLock<HashMap<Uuid, WorkspacePolicy>>,
    user_roles: std::sync::RwLock<HashMap<Uuid, HashMap<Uuid, Role>>>,
}

impl InMemoryPolicyProvider {
    pub fn new() -> Self {
        Self {
            policies: std::sync::RwLock::new(HashMap::new()),
            user_roles: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub fn with_policy(mut self, workspace_id: Uuid, policy: WorkspacePolicy) -> Self {
        self.policies.write().unwrap().insert(workspace_id, policy);
        self
    }

    pub fn with_user_role(
        mut self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: Role,
    ) -> Self {
        let mut workspace_roles = self.user_roles.write().unwrap();
        workspace_roles
            .entry(workspace_id)
            .or_default()
            .insert(user_id, role);
        self
    }
}

impl Default for InMemoryPolicyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PolicyProvider for InMemoryPolicyProvider {
    async fn get_policy(
        &self,
        workspace_id: Uuid,
    ) -> Result<WorkspacePolicy, ClawstackError> {
        let policies = self.policies.read().unwrap();
        Ok(policies
            .get(&workspace_id)
            .cloned()
            .unwrap_or_else(|| WorkspacePolicy {
                id: Uuid::new_v4(),
                workspace_id,
                version: 1,
                rules: vec![],
                default_grant: CapabilityGrant::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
    }

    async fn check_capability(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        capability: Capability,
    ) -> Result<bool, ClawstackError> {
        let user_roles = self.user_roles.read().unwrap();
        let role = user_roles
            .get(&workspace_id)
            .and_then(|roles| roles.get(&user_id))
            .copied()
            .unwrap_or(Role::Guest);

        Ok(role.default_grants().grants(capability))
    }
}
