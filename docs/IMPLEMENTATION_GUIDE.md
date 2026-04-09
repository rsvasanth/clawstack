# ClawStack Implementation Guide

> **Status:** Phase 1 Complete — Foundation fixes applied  
> **Last Updated:** April 9, 2026  
> **Branch:** `main` (commit `f331b5d`)

---

## 1. Architecture Overview

ClawStack is a **unified, self-improving, multi-channel agent runtime** built on the principle: **"Rust orchestrates, Python proposes."**

### Core Components

```
Channel (Telegram / CLI / Web / Slack)
    ↓
Gateway (Rust — HTTP/WS/gRPC)
    ↓
Kernel Loop (Rust)
  Intent = classify_intent(request)  →  ChatOnly | UseTools | SpawnWorker | DelegatePython
  Policy = check_capability(workspace, user, cap)  ← enforced HERE, not in tools
  Execute → ToolBus
    ↓              ↓               ↓
  Memory       Workers          Python Learning Service
(Postgres    (Docker/WASM)     (FastAPI — async, offline)
 +pgvector)                          ↓
    ↓                         SkillSynthesizer
  Trajectory stored             proposes SkillDraft
    ↓                               ↓
  Python pulls trajectory    Rust approves/rejects → SkillRegistry
```

### Key Design Decisions

1. **Tool Bus is the ONLY execution interface** — kernel never calls tools directly
2. **Capabilities enforced in Rust kernel, declared by tools** — clean separation
3. **Python never blocks user requests** — always async/offline; proposes, never decides
4. **Trajectories are first-class citizens** — every request stored for learning
5. **Memory is tiered** — short-term (turns), mid-term (summaries), long-term (vectors)

---

## 2. What's Built vs. What's Missing

### ✅ Built (Phase 0 Foundation)

**Rust (`clawstack-common`, `clawstack-kernel`, partial `clawstack-gateway`, `clawstack-memory`):**
- `ToolSpec`, `ToolCall`, `ToolResult` — well-typed, clean
- `Capability` enum + `CapabilityGrant` — solid role-RBAC design
- `Skill`, `SkillDraft`, `DraftStatus` — full lifecycle types
- `MemoryEntry`, `MemoryQuery`, `MemoryTier` (3-tier) — solid
- `Trajectory`, `TrajectoryOutcome` — correct
- `AgentKernel::run()` — the core loop exists with correct 4-step shape:
  ```
  build_context → classify_intent → route/execute → store_trajectory
  ```
- Traits defined: `PolicyProvider`, `MemoryProvider`, `ToolBus`, `LlmClient`
- Intent routing: `ChatOnly | UseTools | SpawnWorker | DelegateToPython`
- `AgentResponse` with tool_calls + tool_results
- **NEW: AgentRequest/AgentResponse types (added in f331b5d)**
- **NEW: Capability enforcement wired in handle_tools() (added in f331b5d)**
- **NEW: Trajectory turns captured properly (added in f331b5d)**

**Python (learning service):**
- `FastAPI` app skeleton with correct endpoints (`/trajectories`, `/skills/synthesize`, `/health`, `/skills/evaluate/{id}`)
- `SkillSynthesizer.run_cycle()` — extracts patterns, generates `SkillDraft`
- `TrajectoryCollector` — stores trajectories
- `SkillEvaluator` — evaluates drafts against benchmarks
- `SkillDraft.to_dict()` — serializes back to Rust-compatible JSON
- **NEW: DATABASE_URL from env var (added in f331b5d)**

**Database (Postgres + pgvector):**
- Full initial migration `001_initial_schema.sql` — ALL tables exist:
  - `users`, `workspaces`, `workspace_members`, `sessions`, `conversations`, `messages`
  - `memories`, `memory_embeddings` (vector(1536), ivfflat index)
  - `trajectories`, `skills`, `skill_versions`, `skill_drafts`
  - `repos`, `jobs`, `workspace_policies`, `audit_events`
- **NEW: Fixed idx_memory_embeddings_embedding (added in f331b5d)**

**Infrastructure:**
- `docker-compose.yml` — Postgres + pgvector, Redis, gateway service, learning service — complete
- `gateway.Dockerfile`, `learning.Dockerfile` — exist

### ❌ Not Yet Built (Gaps)

| Component | Status | Priority |
|-----------|--------|----------|
| `clawstack-gateway` | Stub — no actual HTTP/WS server code | P0 |
| `clawstack-policy` | Missing — no PolicyProvider implementation | P0 |
| `clawstack-toolbus` | Missing — trait defined, no dispatch implementations | P0 |
| `LlmClient` | Undefined — no concrete LLM integration | P0 |
| `clawstack-memory` | Stub — trait defined, no sqlx implementation | P1 |
| `clawstack-channels` | Missing — ChannelAdapter trait not started | P2 |
| `clawstack-scheduler` | Missing — job queue, worker pool | P2 |
| `clawstack-code-runtime` | Missing — coding worker | P3 |
| `clawstack-mcp` | Missing — MCP protocol client | P3 |
| `clawstack-cli` | Missing — CLI client | P3 |
| `clawstack-api` | Missing — gRPC/HTTP API surface | P3 |

---

## 3. Critical Fixes Applied (Commit f331b5d)

### Fix 1: Trajectory Storage Was Empty
**Problem:** `store_trajectory()` stored `turns: vec![]` — the entire learning loop was broken.

**Fix:** Now builds proper `TrajectoryTurn` structs:
```rust
// Turn 0: User message
turns.push(TrajectoryTurn {
    turn_index: 0,
    role: MessageRole::User,
    content: ctx.request.text.clone(),
    tool_calls: vec![],
});
// Subsequent: LLM + tool call + tool result turns
```

**Location:** [`loop_.rs:223-288`](clawstack/rust/crates/clawstack-kernel/src/loop_.rs)

### Fix 2: Capability Enforcement Not Wired
**Problem:** `check_capability()` existed but was never called in `handle_tools()`.

**Fix:** Added capability check before tool dispatch:
```rust
for call in &tool_calls {
    let Some(spec) = self.toolbus.get_spec(&call.tool_id) else {
        return Err(ClawstackError::not_found("tool", &call.tool_id));
    };
    for required_cap in &spec.required_capabilities {
        let granted = self.policy.check_capability(
            ctx.request.workspace_id,
            ctx.request.user_id,
            *required_cap,
        ).await?;
        if !granted {
            return Err(ClawstackError::capability_denied(...));
        }
    }
}
```

**Location:** [`loop_.rs:170-212`](clawstack/rust/crates/clawstack-kernel/src/loop_.rs)

### Fix 3: Missing Types
**Problem:** `AgentRequest` and `AgentResponse` were referenced but not defined.

**Fix:** Added [`agent.rs`](clawstack/rust/crates/clawstack-common/src/agent.rs) with:
- `AgentRequest` — workspace_id, session_id, user_id, channel, text, attachments, metadata
- `AgentResponse` — text, attachments, tool_calls, tool_results, session_id
- `Attachment` — kind, url, content, name, mime_type

### Fix 4: ToolSpec Type Mismatch
**Problem:** `required_capabilities: Vec<CapabilityGrant>` but `PolicyProvider::check_capability()` takes `Capability`.

**Fix:** Changed to `required_capabilities: Vec<Capability>` in [`context.rs:24`](clawstack/rust/crates/clawstack-kernel/src/context.rs)

### Fix 5: ToolBus Missing get_spec()
**Problem:** Couldn't look up tool capabilities without a way to get tool specs.

**Fix:** Added to `ToolBus` trait:
```rust
fn get_spec(&self, tool_id: &str) -> Option<crate::ToolSpec>;
```

### Fix 6: Python Hardcoded DATABASE_URL
**Problem:** `main.py` hardcoded `"postgresql://localhost/clawstack"`.

**Fix:** Now reads from `DATABASE_URL` env var:
```python
db_url = os.environ.get("DATABASE_URL", "postgresql://localhost/clawstack")
```

### Fix 7: SQL Index Name Incomplete
**Problem:** `CREATE INDEX idx_memory_embeddings_ ON ...` — missing column name.

**Fix:** `idx_memory_embeddings_embedding`

---

## 4. Implementation Phases

### 🔴 Phase 0: Foundation (COMPLETED)
- [x] Fix trajectory storage
- [x] Wire capability enforcement
- [x] Add missing types
- [x] Fix Python DATABASE_URL
- [x] Fix SQL index

### 🔴 Phase 1: LLM Integration (P0 — NOTHING WORKS WITHOUT THIS)
**The kernel cannot do anything without a concrete LLM client.**

Tasks:
1. Create `clawstack-llm` crate with `OpenAiLlmClient`
   - Use OpenAI chat completions API
   - Support structured output for tool calls
   - Implement `LlmClient` trait from `loop_.rs`
2. Update `classify_intent()` to use structured JSON parsing instead of string matching
3. Add tool schema passthrough so LLM knows available tools
4. Consider adding Anthropic/Anthropic client later

**Files to create/modify:**
- `rust/crates/clawstack-llm/Cargo.toml`
- `rust/crates/clawstack-llm/src/lib.rs` — `OpenAiLlmClient`
- `rust/crates/clawstack-kernel/src/loop_.rs` — `classify_intent()` improvement

### 🔴 Phase 2: ToolBus Implementation (P0 — Can't Execute Tools)
**The kernel route to `UseTools` will fail without a ToolBus implementation.**

Tasks:
1. Create `clawstack-toolbus` crate
2. Implement `ToolBus` trait with built-in tools:
   - `echo` — returns input text (no capabilities)
   - `memory.read` — requires `MemoryRead` capability
   - `memory.write` — requires `MemoryWrite` capability
   - `web.fetch` — requires `WebFetch` capability
3. Add tool registry that tracks available tools
4. **BONUS:** Parallelize `execute_many()` using `futures::future::join_all`

**Files to create:**
- `rust/crates/clawstack-toolbus/Cargo.toml`
- `rust/crates/clawstack-toolbus/src/lib.rs` — `ToolBusImpl`, built-in tools

### 🟡 Phase 3: Memory Provider (P1 — Trajectory Storage Won't Persist)
**Currently `MemoryProvider` is a trait with no implementation. Trajectories are lost.**

Tasks:
1. Create sqlx-based `MemoryProviderImpl`
2. Implement `retrieve()` for context building (pgvector similarity search)
3. Implement `store()` for trajectory persistence
4. Add workspace/memory tables queries

**Files to create:**
- `rust/crates/clawstack-memory/src/provider.rs` — `MemoryProviderImpl`

### 🟡 Phase 4: Policy Provider (P1 — Capability Enforcement Won't Work)
**Currently `PolicyProvider` is a trait with no implementation. Capabilities always fail.**

Tasks:
1. Create `WorkspacePolicyProviderImpl` loading from Postgres
2. Implement JSONB rule evaluation
3. Wire into kernel on startup

**Files to create:**
- `rust/crates/clawstack-policy/src/lib.rs` — `WorkspacePolicyProvider`

### 🟢 Phase 5: Python Learning Loop (P2 — Close the Feedback Cycle)
**The Python learning service can analyze but can't submit SkillDrafts back to Rust.**

Tasks:
1. Add gRPC endpoint to Rust gateway for `SubmitSkillDraft`
2. Implement gRPC call from Python `SkillSynthesizer.run_cycle()`
3. Add REST fallback endpoint
4. Fix Python bug: `run_cycle()` passes random UUID instead of real workspace_id

**Files to modify:**
- `python/learning/service/skill_synth.py` — gRPC call (TODO: line 205)
- `rust/crates/clawstack-gateway/src/lib.rs` — skill draft endpoint

### 🟢 Phase 6: Gateway Binary (P2 — Make It Runnable)
**No binary exists to run the gateway server.**

Tasks:
1. Create `clawstack-gateway/src/main.rs`
2. Set up HTTP server (tonic-web or axum)
3. Set up gRPC server
4. Wire all providers together
5. Add health check endpoint

### 🔵 Phase 7: Channel Adapters (P3)
**Only CLI can be built first. Others (Telegram, Slack) need bot tokens.**

Tasks:
1. `clawstack-cli` — CLI client for testing
2. `clawstack-channels/src/telegram.rs` — Telegram adapter
3. `clawstack-channels/src/slack.rs` — Slack adapter

---

## 5. Key Technical Decisions

### Rust Traits (Already Correct)
The traits in `loop_.rs` are well-designed:
- `PolicyProvider` — async, returns `bool` for capability checks
- `MemoryProvider` — async retrieve + store
- `ToolBus` — async execute + execute_many
- `LlmClient` — async complete, returns `LlmResponse`

### Capability System
```rust
pub enum Capability {
    MemoryRead, MemoryWrite,
    SkillsInstall, SkillsExecute,
    RepoRead, RepoWrite,
    ShellExec, ShellTest,
    BrowserUse, BrowserAuth,
    WebFetch, SecretsUse,
    ChannelSend, AdminApproveSkill,
}
```

Roles have default grants:
- `Owner` → all capabilities
- `Admin` → all except `AdminApproveSkill`
- `Developer` → most capabilities, no secrets
- `Guest` → read-only + web fetch + channel send
- `Worker` → memory + repo + shell

### Trajectory Turn Structure
```rust
pub struct TrajectoryTurn {
    pub turn_index: u32,
    pub role: MessageRole, // System, User, Assistant, Tool
    pub content: String,
    pub tool_calls: Vec<TrajectoryToolCall>,
}
```

---

## 6. How to Run (Once Phase 6 is Complete)

```bash
# Start infrastructure
cd infra/docker
docker-compose up -d postgres redis

# Run gateway
cd rust
cargo run -p clawstack-gateway

# Run learning service
cd python
pip install -e .
uvicorn learning.main:app --reload
```

---

## 7. Testing Strategy

1. **Unit tests** — Each crate has its own tests
2. **Integration tests** — `clawstack-integration` crate with test Postgres
3. **Docker compose test** — Full stack in docker
4. **Manual testing** — Use CLI client to send requests

---

## 8. References

- [SPEC.md](clawstack/SPEC.md) — Full product specification
- [Product Head Review](#) — Original analysis by Antigravity
- [Implementation Plan](clawstack/docs/IMPLEMENTATION_PLAN.md) — Week-by-week plan

---

*This guide is maintained by the core team. Update when phases are completed.*
