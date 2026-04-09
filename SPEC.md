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

## 12. Breakthrough Memory Architecture

> From swarm design session — memory expert + ml expert contributions.

### 12.1 Causal Provenance Graph

Every memory node carries a full causal chain back to its source. The system maintains an explicit DAG (directed acyclic graph) of memory provenance. Forgetting is a first-class causal operation — not deletion, but a recorded reason for removal.

**The breakthrough:** "Why do you know this?" returns an actual evidentiary chain, not proximity similarity.

```rust
pub struct MemoryNode {
    id: Uuid,
    embedding: Vec<f32>,
    text: String,
    node_type: NodeType,
    provenance: Vec<ProvenanceLink>,  // causal chain backward
    confidence: f32,                  // 0.0-1.0 evidence strength
    is_active: bool,                 // false = soft-deleted with reason
}

pub enum NodeType {
    UserInput(String),              // raw user utterance
    ToolOutput { tool: String },    // from tool execution
    Derived { rule: String },      // derived from similarity
    Synthesized { skill_ref: Uuid },
}

pub enum ForgetReason {
    Contradicted { newer_id: Uuid },
    Superseded { replacement_id: Uuid },
    DecayedBelowThreshold { score: f32 },
    UserRequested,
    DriftDetected { old: Vec<f32>, new: Vec<f32>, delta: f32 },
}
```

**Postgres schema:**
- `memory_nodes` — DAG nodes with provenance edges
- `provenance_edges` — causal links (supporting, refuting, derivation_step)
- `forget_log` — immutable record of why something was discarded

**Key algorithms:**
- `trace_provenance(memory_id, depth)` — BFS backward chain, returns evidentiary roots
- `find_contradictions(new_embedding)` — pgvector cosine distance check before write; if refutes existing, insert refuting edge instead of overwriting
- `detect_semantic_drift(cluster_id)` — if same "fact" drifts semantically over time, flag and preserve both versions

### 12.2 Attractor-Based Memory Dynamics

Memory space modeled as a **dynamic attractor landscape**, not flat vector store. Each concept/skill occupies an attractor basin. The system continuously learns which memories lead to success and adjusts basin boundaries accordingly.

**The breakthrough:** Not passive retrieval. Active learning about what leads to desired outcomes.

```rust
pub struct AttractorState {
    memory_id: Uuid,
    activation_level: f32,    // EWMA of recent access × similarity
    success_signal: f32,      // EWMA of success outcomes
    failure_signal: f32,      // EWMA of failure outcomes
    attractor_strength: f32,  // how focused vs diffuse
}

pub struct QueryActivation {
    activated_attractors: Vec<(Uuid, f32)>,
    suppressed_attractors: Vec<Uuid>,
    novel_region: bool,
}
```

**Key algorithms:**
- `activate(memory_id, query_embedding)` — on every access, update activation + success/failure EWMA
- `decay_all()` — timer-based decay of activation levels
- `recompute_landscape()` — periodic full recompute of centroids, basin boundaries, repulsion pairs
- `query(embedding)` — activates basins, returns ranked memories weighted by activation × success_signal

### 12.3 Memory That Can Explain Itself

For any memory:
- "Why do you know this?" → full causal chain to user inputs
- "How sure are you?" → joint confidence from all provenance links
- "What contradicts this?" → traversal of refuting edges
- "Has this changed over time?" → semantic drift detection

### 12.4 Implicit Forgetting

Forgetting is NOT deletion. Every removal is a `ForgetRecord` with a reason. Old memory is preserved in the causal chain — you can reconstruct what the agent believed before the correction.

---

## 13. Breakthrough Learning Architecture

> From swarm design session — ml expert contributions.

### 13.1 Skill Lifecycle with Effectiveness Tracking

Skills are living artifacts, not static files:

```rust
pub struct Skill {
    // ... existing fields ...
    effectiveness: EffectivenessMetrics,
    lineage: Vec<SkillId>,      // parent skills this evolved from
    principles: Vec<PrincipleId>,
}

pub struct EffectivenessMetrics {
    use_count: u64,
    success_count: u64,
    failure_count: u64,
    avg_duration_ms: u64,
    last_used: DateTime<Utc>,
    last_refined: DateTime<Utc>,
}
```

On every skill execution: update counters → if failure_rate > 20%, trigger SkillRefiner.

### 13.2 Principle Extraction (Not Just Patterns)

Patterns overfit. Principles generalize. The learning service extracts **principles** from clusters of successful trajectories:

```
Input: 5 successful repo fix trajectories
Output: "When fixing test failures in a PR, first run the failing test in isolation
         to confirm the failure mode, then check git blame on that specific line
         before assuming the change that introduced the failure."
```

Every synthesized skill is annotated with the principles that guided it → skills become explainable.

### 13.3 FTS5 Session Search

Session history indexed with full-text search + structured metadata.

### 13.4 Memory Nudge System

Periodic proactive memory management triggered by:
- Every N turns (e.g., 50)
- Significant decisions detected in conversation
- On session end
- Weekly scheduled review

### 13.5 User Preference Model

Learned model of user preferences, updated continuously:

```rust
pub struct UserPreferenceModel {
    communication: CommunicationPrefs,
    technical: TechnicalPrefs,
    privacy: PrivacyPrefs,
    patterns: PatternPrefs,
}
```

---

## 15. Security Architecture

> From swarm design session — security expert.

### 15.1 Transitive Trust Auditing

The #1 unique threat in agent runtimes: **action graph amplification**. If tool A calls B, and B calls C, the agent may execute an action authorized by A but executed by C — no framework tracks this graph.

**Breakthrough:** Runtime traverses the full call graph before sensitive actions. Each tool logs its *caller context* (not just its own policy) into an immutable audit log.

### 15.2 Linear Logic Capabilities

Capabilities are not boolean flags — they are **proof obligations**.

```rust
// Capabilities as Ed25519-signed JWTs with consumption counter
// Double-spend or replay = kernel rejection
struct CapabilityToken {
    claims: Claims,
    signature: Ed25519Signature,
    consumed: AtomicU64,  // linear: must be exactly 1
}
```

Rust kernel issues; Python learning layer presents as proofs; WASM executor verifies atomically. TLA+-verifiable because linear logic has complete formal semantics.

### 15.3 Tamper-Evident Action Provenance Trees

Not logs — **cryptographically chained state snapshots**.

```rust
// After each tool execution, compute:
H = SHA3-256(parent_H || tool_name || tool_output_summary || memory_pointer)
// Stored in Postgres alongside serialized state snapshot
```

Changing any past node invalidates all downstream hashes. Like Git's Merkle trees but for agent cognition. Dedicated audit service can verify chain integrity and replay any decision subtree offline.

---

## 16. Rust-Native Tool Architecture

> From swarm design session — rust expert.

### 16.1 Typed WASM Tool Contracts

Tool implementations compiled to WASM with typed contracts in the module's custom section. Kernel validates at **load time**, not dispatch.

```rust
pub struct ToolManifest {
    pub name: String,
    pub permissions: Permissions,
    pub memory_pages: u32,
}

fn load_tool(wasm_bytes: &[u8], policy: &Policy) -> Result<Arc<dyn Tool>> {
    let manifest = extract_manifest(wasm_bytes)?;  // from WASM custom section
    ensure!(policy.allows(&manifest.permissions))?;  // static check at load
    Ok(instantiate_wasm(wasm_bytes)?)
}
```

### 16.2 Borrow-Based Kernel/Tool Boundary

Zero serialization crossing the boundary — shared address space with typed references.

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn execute<'_ctx>(&self, ctx: ToolCtx<'_ctx>, input: Cow<'_, str>) -> Result<Cow<'ctx, str>, ToolError>;
}

pub struct ToolCtx<'ctx> {
    pub memory: &'ctx Memory,       // pgvector-backed, borrowed
    pub policy: &'ctx Policy,        // read-only at execute time
    pub grants: &'ctx ToolGrants,   // scoped resources
}
```

### 16.3 Ownership-Guaranteed Resource Grants

Lifetime-encoded resource scope — the borrow checker *physically prevents* accessing resources outside the grant.

```rust
pub struct FileHandle<'ctx> {
    _marker: PhantomData<&'ctx ()>,
    fd: RawFd,
    range: Option<Range<u64>>,
}

pub struct MemoryRegion<'ctx> {
    _marker: PhantomData<&'ctx ()>,
    ptr: NonNull<u8>,
    len: usize,
}
```

Drop `ToolCtx` → `FileHandle` and `MemoryRegion` become invalid. No capability tokens to forge. No revocation logic. The lifetime *is* the enforcement.

---

## 17. UX Architecture

> From swarm design session — ux expert.

### 17.1 Conversation Replay & Undo

After each task: "Here's what I did" accordion. Each step is a card (icon + action label + timestamp). Tap any card to undo that specific action. **Version control for your life.**

### 17.2 Permission-First Skill Cards

Skills show **"what they need" before "what they do"** — icon pills (📧 Email, 📅 Calendar, 📍 Location). Install button disabled until user scrolls to "I understand." Trust through ritual, not education.

### 17.3 The Hesitation Pause

If user sends 3+ messages within 60s rephrasing the same intent → surface a "Clarifying question" card: "It sounds like you want X — is that right?"

Users rarely say "I don't understand." They just try again, differently. That rephrasing effort is the most honest confusion signal — currently ignored by every framework.

---

## 18. Economics & Token Optimization

> From swarm design session — economics expert.

### 18.1 Learned Cost-Quality Pacing

Adaptive token budgeting via RL policy gradient:

```
Algorithm: Thompson Sampling + Linear UCB
- Logistic regression on task embeddings predicts tokens needed per quality threshold
- Early termination when marginal quality gain drops below learned threshold
- Result: ~40% token savings on easy tasks, >95% quality retained
```

### 18.2 Activity-Based Costing Per Workspace

```
Cost/workspace/month = Σ (compute_minutes × $0.002)
                     + Σ (memory_gb_seconds × $0.0001)
                     + Σ (input_tokens × $0.001/1K + output_tokens × $0.003/1K)

Overage tiers: 5% / 15% / 30% discounts at $10 / $50 / $200 / month committed
```

### 18.3 Learned Memory Compression

Distilled compression as an optimizable ML problem:

```
Encoder: sentence-transformer → dense vector
Policy Net (LSTM): predicts "retain / compress-to-X%" per sentence
Training: minimize reconstruction error at retrieval time
Result: 60-80% token reduction, <5% recall degradation on held-out queries
```

Served as async Python microservice behind Rust kernel. ~0.1ms latency overhead for 3-5× memory density gain in pgvector.

---

## 19. Source Project References

| Project | Path | Role |
|---------|------|------|
| OpenClaw/agent-friday | `~/agent-friday/` | Channel abstraction, skill registry |
| IronClaw | `~/ironclaw/` | Secure runtime, pgvector, WASM, Docker |
| Hermes Agent | `~/.openclaw/hermes-agent/` | Learning loop, skill synthesis |
| Claw Code | `~/claw-code/` | Coding harness, runtime/session model |
