//! Capability system — defines what actions are allowed in what contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Capability domains that tools, workers, and skills can require.
/// Rust enforces these; Python proposes grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // Memory
    MemoryRead,
    MemoryWrite,
    // Skills
    SkillsInstall,
    SkillsExecute,
    // Repo
    RepoRead,
    RepoWrite,
    // Shell
    ShellExec,
    ShellTest,
    // Browser
    BrowserUse,
    BrowserAuth,
    // Network
    WebFetch,
    // Secrets
    SecretsUse,
    // Channel
    ChannelSend,
    // Admin
    AdminApproveSkill,
}

impl Capability {
    pub fn all() -> HashSet<Capability> {
        HashSet::from([
            Capability::MemoryRead,
            Capability::MemoryWrite,
            Capability::SkillsInstall,
            Capability::SkillsExecute,
            Capability::RepoRead,
            Capability::RepoWrite,
            Capability::ShellExec,
            Capability::ShellTest,
            Capability::BrowserUse,
            Capability::BrowserAuth,
            Capability::WebFetch,
            Capability::SecretsUse,
            Capability::ChannelSend,
            Capability::AdminApproveSkill,
        ])
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::MemoryRead => "memory.read",
            Capability::MemoryWrite => "memory.write",
            Capability::SkillsInstall => "skills.install",
            Capability::SkillsExecute => "skills.execute",
            Capability::RepoRead => "repo.read",
            Capability::RepoWrite => "repo.write",
            Capability::ShellExec => "shell.exec",
            Capability::ShellTest => "shell.test",
            Capability::BrowserUse => "browser.use",
            Capability::BrowserAuth => "browser.auth",
            Capability::WebFetch => "web.fetch",
            Capability::SecretsUse => "secrets.use",
            Capability::ChannelSend => "channel.send",
            Capability::AdminApproveSkill => "admin.approve_skill",
        }
    }
}

/// A set of capability grants assigned to a role or user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub allow: HashSet<Capability>,
    pub deny: HashSet<Capability>,
}

impl CapabilityGrant {
    pub fn new(allow: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            allow: allow.into_iter().collect(),
            deny: HashSet::new(),
        }
    }

    pub fn deny(mut self, caps: impl IntoIterator<Item = Capability>) -> Self {
        self.deny.extend(caps);
        self
    }

    pub fn grants(&self, cap: Capability) -> bool {
        self.allow.contains(&cap) && !self.deny.contains(&cap)
    }
}

/// Roles that can have capability grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Admin,
    Developer,
    Guest,
    Worker,
}

impl Role {
    pub fn default_grants(&self) -> CapabilityGrant {
        match self {
            Role::Owner => CapabilityGrant::new(Capability::all()),
            Role::Admin => CapabilityGrant::new([
                Capability::MemoryRead,
                Capability::MemoryWrite,
                Capability::SkillsInstall,
                Capability::SkillsExecute,
                Capability::RepoRead,
                Capability::RepoWrite,
                Capability::ShellExec,
                Capability::ShellTest,
                Capability::BrowserUse,
                Capability::BrowserAuth,
                Capability::WebFetch,
                Capability::SecretsUse,
                Capability::ChannelSend,
            ]),
            Role::Developer => CapabilityGrant::new([
                Capability::MemoryRead,
                Capability::SkillsExecute,
                Capability::RepoRead,
                Capability::RepoWrite,
                Capability::ShellExec,
                Capability::ShellTest,
                Capability::BrowserUse,
                Capability::WebFetch,
                Capability::ChannelSend,
            ]),
            Role::Guest => CapabilityGrant::new([
                Capability::MemoryRead,
                Capability::SkillsExecute,
                Capability::RepoRead,
                Capability::WebFetch,
                Capability::ChannelSend,
            ]),
            Role::Worker => CapabilityGrant::new([
                Capability::MemoryRead,
                Capability::MemoryWrite,
                Capability::RepoRead,
                Capability::ShellExec,
                Capability::ShellTest,
            ]),
        }
    }
}
