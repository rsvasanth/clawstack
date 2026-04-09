# ClawStack Implementation Plan — 8-Week Roadmap

> **Start from IronClaw core, port one system at a time.**
> Every phase produces a working, testable system.

---

## Week 1 — Foundation & Schemas

### Goal
Monorepo, shared schemas, Postgres migrations, Rust workspace ready.

### Tasks

- [ ] **1.1** Create monorepo root: `Cargo.toml`, `pnpm-workspace.yaml`
- [ ] **1.2** Define shared JSON Schema in `schemas/`:
  - `AgentRequest`, `AgentResponse`
  - `ToolSpec`, `ToolCall`, `ToolResult`
  - `Skill`, `SkillDraft`, `SkillInput`
  - `Trajectory`, `TrajectoryTurn`, `TrajectoryToolCall`
  - `MemoryEntry`, `MemoryQuery`
  - `Job`, `WorkspacePolicy`, `CapabilityGrant`
- [ ] **1.3** Create `clawstack-common` crate with all type definitions
- [ ] **1.4** Write Postgres migrations (`infra/docker/migrations/001_initial_schema.sql`)
  - Users, workspaces, sessions, conversations
  - Messages, memories, memory_embeddings (with pgvector)
  - Trajectories, skills, skill_drafts, skill_versions
  - Repos, jobs, workspace_policies, audit_events
- [ ] **1.5** Set up `clawstack-kernel`, `clawstack-gateway`, `clawstack-memory` stubs
- [ ] **1.6** Verify Docker Compose starts postgres + redis

### Deliverables
✅ Working Postgres schema, Rust workspace builds, Docker Compose up

---

## Week 2 — Gateway, Kernel Loop, Memory

### Goal
Rust gateway with a single `/chat` endpoint, in-memory tool execution, basic memory retrieval.

### Tasks

- [ ] **2.1** `clawstack-gateway`: HTTP server with `/chat` endpoint
- [ ] **2.2** `clawstack-kernel`: Core loop skeleton
  - `Intent::classify()` — simple keyword + pattern matching first
  - `Kernel.run(request)` → routes to chat or tools
- [ ] **2.3** `clawstack-memory`: Postgres adapter
  - Connect via `sqlx`
  - Implement `MemoryProvider` trait
  - `retrieve(context, limit)` — recent messages + pgvector similarity search
- [ ] **2.4** Build minimal CLI client (`clawstack-cli`)
- [ ] **2.5** Wire up `ToolBus` with hardcoded 3-4 tools (echo, add, memory read)
- [ ] **2.6** Audit log: every tool call writes to `audit_events`

### Deliverables
✅ `curl localhost:8080/chat` returns a response; tool calls logged

---

## Week 3 — Tool Bus, Policy Engine

### Goal
Full unified Tool Bus, capability enforcement, policy engine v1.

### Tasks

- [ ] **3.1** Define `ToolSpec` in `clawstack-common` with all fields
- [ ] **3.2** Tool Bus: `execute(call, ctx)` → routes by execution mode
  - `Sync` → spawn blocking task
  - `Async` → queue job
  - `Worker` → Docker spawn
- [ ] **3.3** Implement `PolicyProvider` trait
  - Load `WorkspacePolicy` from Postgres
  - `check_capability(workspace, user, cap)` → bool
- [ ] **3.4** Kernel: enforce capabilities before tool dispatch
  - If capability denied → `CapabilityDenied` error + audit event
- [ ] **3.5** Wire up 5-6 real tools: file read/write, shell exec, web fetch, memory write, http
- [ ] **3.6** Add `required_capabilities` to every ToolSpec

### Deliverables
✅ Every tool has a capability requirement; kernel enforces it

---

## Week 4 — Channel Adapters (CLI + Telegram)

### Goal
Port OpenClaw's channel abstraction; get CLI + Telegram working.

### Tasks

- [ ] **4.1** Define `ChannelAdapter` trait in `clawstack-channels`
  ```rust
  trait ChannelAdapter {
    async fn send(&self, msg: OutboundMessage) -> Result<()>;
    fn inbound_channel(&self) -> impl Stream<Item = ChannelMessage>;
  }
  ```
- [ ] **4.2** CLI adapter: read from stdin, write to stdout
- [ ] **4.3** Telegram adapter:
  - Webhook registration
  - Normalize Telegram updates → `AgentRequest`
  - Send responses back via Telegram Bot API
- [ ] **4.4** Session continuity: same user on Telegram = same workspace + memory
- [ ] **4.5** Identity stitching: `resolve_user(channel, external_id)` → existing or new user
- [ ] **4.6** Operator UI (basic HTML): see active sessions, channel health

### Deliverables
✅ User can chat on CLI and Telegram with the same identity and memory

---

## Week 5 — Python Learning Service, Trajectory Collector

### Goal
Python service consuming trajectories from Rust; session summarization.

### Tasks

- [ ] **5.1** Set up Python service with FastAPI + SQLAlchemy + asyncpg
- [ ] **5.2** `TrajectoryCollector.store(trajectory)`:
  - Rust calls `POST /trajectories` after every request
  - Python stores in analysis database
- [ ] **5.3** Session summarizer:
  - Fetch recent messages per session
  - Generate summary via LLM API call
  - Write back to memory as `mid_term` entry
- [ ] **5.4** `TrajectoryCollector.fetch_recent(workspace, limit)` for synthesizer
- [ ] **5.5** Trajectory schema validation on ingestion
- [ ] **5.6** Health check endpoint + graceful shutdown

### Deliverables
✅ `/trajectories` endpoint stores data; summarizer runs every N turns

---

## Week 6 — Skill Registry, Skill Synthesis, Approval

### Goal
Skill drafts flow from Python → Rust → active skill.

### Tasks

- [ ] **6.1** `SkillSynthesizer.run_cycle()`:
  - Fetch recent trajectories
  - Extract tool-call sequences with frequency ≥ 5
  - Generate `SkillDraft` for each pattern
- [ ] **6.2** `POST /skills/synthesize` endpoint in Python
- [ ] **6.3** Rust: call Python synthesize endpoint periodically (or on schedule)
- [ ] **6.4** `SkillRegistry` in Rust:
  - Load skills from Postgres
  - Version tracking (`skill_versions`)
  - `list_skills()`, `get_skill()`, `install_skill()`
- [ ] **6.5** Skill approval workflow:
  - Rust inspects `skill_drafts` table
  - Runs policy check (risk level × required capabilities)
  - Promotes to active or rejects with reason
- [ ] **6.6** Skill manifest format (SKILL.md → `Skill` struct)

### Deliverables
✅ Python synthesizes drafts; Rust approves/rejects; approved skills appear in tool registry

---

## Week 7 — Code Worker (Repo Read/Write + Verification)

### Goal
Claw Code-inspired repo worker: file map → plan → patch → verify.

### Tasks

- [ ] **7.1** `CodeWorker` in `clawstack-code-runtime`
  - `RepoSession`: clone/fetch repo into isolated dir
  - `FileMap`: build index of files, imports, deps
- [ ] **7.2** Tool: `code_worker.plan(task, repo_id)` → `CodePlan`
  - Read relevant files
  - LLM generates edit plan
- [ ] **7.3** Tool: `code_worker.apply(plan_id, patches)` → patch files
  - Validate patches before applying
  - Git commit with audit message
- [ ] **7.4** Tool: `code_worker.verify(plan_id)` → run tests/lint
  - Parse test output
  - Return pass/fail + output
- [ ] **7.5** Docker isolation: code worker runs in container with restricted filesystem
- [ ] **7.6** Audit trail: every file read/write/git action logged to `audit_events`

### Deliverables
✅ `implement_feature` tool that reads repo, plans changes, applies patches, runs tests

---

## Week 8 — Web Console, Hardening, Policies, E2E

### Goal
Production-quality web console, workspace policies, end-to-end test.

### Tasks

- [ ] **8.1** Web admin console (`ui/web-console/`)
  - Session monitor (active sessions, messages, tools used)
  - Job monitor (pending/running/completed)
  - Skill registry view
  - Trajectory browser
- [ ] **8.2** Workspace policies UI:
  - View/edit capability grants per role
  - Configure policy rules
- [ ] **8.3** Policy engine v2:
  - `ConfirmTool` rule → require user confirmation before execution
  - `BlockIf` condition → pattern-based blocking
- [ ] **8.4** Multi-agent delegation:
  - Kernel can spawn multiple code workers
  - Aggregate results
- [ ] **8.5** End-to-end test: Telegram → kernel → tool → trajectory stored → skill synthesized → skill approved → tool used
- [ ] **8.6** Security hardening:
  - Audit every secret access
  - WASM sandbox for low-risk tools
  - Rate limiting on channel ingress

### Deliverables
✅ Shippable product: CLI + Telegram chat, memory, skills, code worker, web console

---

## Phase 1 Milestones Summary

| Week | Milestone | Key Metric |
|------|-----------|------------|
| 1 | Foundation | Postgres schema + Rust builds |
| 2 | Kernel + Memory | `curl /chat` works |
| 3 | Tool Bus + Policy | Capabilities enforced |
| 4 | Channels | CLI + Telegram working |
| 5 | Learning Service | Trajectories stored |
| 6 | Skill Synthesis | Auto-generated skills activate |
| 7 | Code Worker | Repo tasks complete |
| 8 | Product | Web console + hardening |

---

## Phase 2+ (Future)

- WASM sandboxing for all tool execution
- Slack, Discord, WhatsApp channel adapters
- Full skill evaluation benchmarks
- Prompt self-evolution (GPA/GEPA)
- Editor integrations (VS Code, Neovim)
- Multi-region deployment
