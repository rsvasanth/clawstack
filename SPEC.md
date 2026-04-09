# ClawStack — Specification

> **Status:** Draft — Phase 0
> **Owner:** Vasanth Ranganathan
> **Stack:** Rust (core runtime) + Python (learning layer)

---

## 1. Overview

**Name:** ClawStack
**Tagline:** "Secure, self-improving, multi-channel agent runtime with a built-in coding brain."
**Core principle:** "Rust orchestrates, Python proposes."

One logical product that unifies:
- OpenClaw's multi-channel gateway and skill ecosystem
- IronClaw's secure Rust runtime, Postgres+pgvector memory, and WASM/Docker sandboxing
- Hermes Agent's trajectory-based learning and skill synthesis
- Claw Code's repo-aware coding harness

---

## 2. Product Modes

| Mode | Inspiration | Description |
|------|-------------|-------------|
| Personal Assistant | OpenClaw | Persistent multi-channel presence across Telegram, CLI, web |
| Secure Operations | IronClaw | Capability policies, WASM/Docker isolation, pgvector memory |
| Learning | Hermes | Trajectory capture → skill synthesis → self-improvement |
| Coding | Claw Code | Repo-aware edit/test/verify loop in sandboxed workers |

---

## 3. Language Split

### Rust owns (trusted, authoritative)
- API gateway and channel ingress
- Core agent kernel and execution state machine
- Tool registry and capability enforcement
- Job orchestration and scheduling
- Workspace and filesystem mediation
- Secret management and audit logging
- WASM and Docker/container adapters
- Postgres + pgvector adapters

### Python owns (adaptive, experimental)
- Skill synthesis from trajectories
- Skill evaluation and ranking
- Prompt optimization experiments
- Semantic compression and summarization
- Optional coding/research sub-agent services
- Model experimentation and benchmarking

---

## 4. System Layers

### Layer 1 — Ingress & Identity (Rust)
- `ChannelAdapter` trait: normalize Telegram, Slack, Discord, WhatsApp, CLI, HTTP, WebSocket
- Core entities: User, Workspace, Session, Conversation, Channel, AgentProfile, CapabilityPolicy
- Single identity persists across channels

### Layer 2 — Kernel (Rust)
- Authoritative orchestrator
- Loop: accept → build context → classify intent → route (chat / tool / job / code) → enforce policy → execute → store trajectory
- Deterministic and auditable

### Layer 3 — Tool Bus (Rust)
Unified bus for ALL tool invocations:
- Native Rust tools
- WASM skills
- Docker workers
- MCP-connected external tools
- Python service endpoints

Tool spec schema:
```yaml
id: string
description: string
input_schema: JSONSchema
output_schema: JSONSchema
required_capabilities: [string]
execution_mode: sync | async | worker
timeout_ms: integer
side_effect_class: none | read | write | exec | admin
audit_tags: [string]
```

### Layer 4 — Memory System (Rust + Postgres + pgvector)
Three-tier memory model:
- **Short-term:** recent turns + in-flight state
- **Mid-term:** summaries, session notes, active tasks
- **Long-term:** vectorized facts, preferences, skills, trajectories

Schema groups:
- `conversations`, `messages`, `memories`, `memory_embeddings`
- `trajectories`, `skills`, `skill_versions`
- `workspaces`, `repos`, `jobs`, `audit_events`, `policies`

### Layer 5 — Learning Service (Python)
Consumes completed trajectories from Rust:
- Ignore / summarize / cluster
- Propose `SkillDraft`, `PromptVariant`, `PolicySuggestion`
- Rust validates and activates

### Layer 6 — Code Worker (Rust)
Isolated execution domain (Claw Code patterns):
- `RepoSession` → `CodePlan` → `PatchSet` → `VerificationRun` → `GitAction`
- Sandbox + policy aware at every step

---

## 5. Feature Mapping

| Feature | Source | Port Strategy |
|---------|--------|---------------|
| Multi-channel gateway | OpenClaw | Adapt channel abstraction; normalize to ChannelAdapter trait |
| Persistent assistant | OpenClaw | Port session model and daemon behavior |
| Skills marketplace | OpenClaw | Adapt skill manifests into unified Tool spec |
| Rust-first runtime | IronClaw | Port directly as foundation |
| Postgres + pgvector | IronClaw | Port migrations and adapter patterns |
| WASM sandbox | IronClaw | Port WASM tool runner |
| Docker isolation | IronClaw | Port Docker worker abstraction |
| Capability/security model | IronClaw | Port directly (core value) |
| Skill documents | Hermes | Adapt into Python service with Rust activation |
| Multi-level memory | Hermes | Port hierarchy into Rust-backed store |
| Self-improving loop | Hermes | Adapt as Python→Rust approval workflow |
| Trajectory-based learning | Hermes | Port collector; Python synthesizes |
| Runtime/session model | Claw Code | Port as foundation for code worker |
| Repo-aware prompts | Claw Code | Port context construction pattern |
| Verification loop | Claw Code | Port as first-class worker step |
| Plugin/hook model | Claw Code | Port selectively; delay full parity |

---

## 6. Shared Schema Design

All schemas defined in `schemas/` as JSON Schema + Protobuf.
Rust and Python code-gen from same schema source.

Key entities:
```json
AgentRequest, AgentResponse, ToolCall, ToolResult,
WorkerJob, WorkerResult, Skill, SkillDraft,
Trajectory, MemoryEntry, Workspace, User,
CapabilityPolicy, AuditEvent, ChannelMessage
```

---

## 7. Capability Model

Capability domains:
```
memory.read       memory.write     skills.install
skills.execute    repo.read        repo.write
shell.exec        shell.test       browser.use
browser.auth      web.fetch        secrets.use
channel.send      admin.approve_skill
```

Every tool/worker/skill declares required capabilities.
Every workspace/user role grants subsets.
**Rust enforces; Python recommends.**

---

## 8. Deployment Model

- Rust core services: long-running containers
- Python learning service: separate internal service
- PostgreSQL + pgvector: canonical store
- Redis/NATS: async queues (optional)
- Docker runner: code jobs and heavy tools
- WASM runner: low-risk sandboxed skills

---

## 9. Milestones

| # | Milestone | Key Deliverables |
|---|-----------|-----------------|
| 0 | Foundation | Monorepo, schemas, Rust workspace, Postgres migrations |
| 1 | Secure Assistant Kernel | Gateway, kernel loop, tool bus, Postgres, CLI |
| 2 | Multi-Channel Assistant | Telegram/Web, unified identity, skill registry v1 |
| 3 | Learning Assistant | Trajectory storage, skill synthesis, approval workflow |
| 4 | Coding Agent | Repo session, patch loop, verification, git audit |
| 5 | Unified Product | Web console, workspace policies, multi-agent, cost controls |

---

## 10. 8-Week Build Sequence

| Week | Focus | Deliverables |
|------|-------|-------------|
| 1 | Schemas, monorepo, migrations | Shared JSON Schema, Rust workspace, Postgres migrations |
| 2 | Gateway, kernel, memory adapter, CLI | Basic chat loop, in-memory + Postgres retrieval |
| 3 | Tool bus, policy engine, audit events | Unified Tool spec, capability checks, audit log |
| 4 | Channel adapters (CLI + Telegram) | ChannelAdapter trait, identity/session model |
| 5 | Python learning service, trajectory collector | Trajectory storage, session summarizer |
| 6 | Skill registry, draft promotion, approval | skill_synth, skill registry, approval workflow |
| 7 | Code worker with repo read/write/search + test | Code worker, verification loop |
| 8 | Web console, hardening, policies | Admin UI, workspace policies, end-to-end test |

---

## 11. Out of Scope (Phase 0–5)

- Full channel parity (WhatsApp, Slack, Discord) — Phase 5+
- Editor integrations (VS Code, Neovim) — Phase 5+
- WASM skill authoring toolchain — Phase 5+
- A/B skill evaluation framework — Phase 5+
- Native mobile apps — Future
- Multi-region deployment — Future

---

## 12. Source Project References

| Project | Path | Role |
|---------|------|------|
| OpenClaw/agent-friday | `~/agent-friday/` | Channel abstraction, skill registry |
| IronClaw | `~/ironclaw/` | Secure runtime, pgvector, WASM, Docker |
| Hermes Agent | `~/.openclaw/hermes-agent/` | Learning loop, skill synthesis |
| Claw Code | `~/claw-code/` | Coding harness, runtime/session model |
