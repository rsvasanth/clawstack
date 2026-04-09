# ClawStack

> Secure, self-improving, multi-channel agent runtime with a built-in coding brain.

**Stack:** Rust (trusted runtime) + Python (learning layer)
**Principle:** "Rust orchestrates, Python proposes."

---

## What is this?

ClawStack unifies four reference architectures into one product:

| Source | Contribution |
|--------|-------------|
| **OpenClaw** | Multi-channel gateway, skill registry, persistent assistant |
| **IronClaw** | Secure Rust runtime, Postgres+pgvector memory, WASM/Docker sandbox |
| **Hermes Agent** | Trajectory-based learning, skill synthesis, self-improvement |
| **Claw Code** | Repo-aware coding harness, edit/test/verify loop |

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│  Channel Ingress (Telegram, CLI, Web, Slack, ...)    │
│  ┌──────────────────┐   ┌──────────────────────────┐ │
│  │  ChannelAdapter   │   │   Session/Identity       │ │
│  │  (trait in Rust)   │   │   (Rust + Postgres)       │ │
│  └────────┬─────────┘   └────────────┬─────────────┘ │
│           │                          │               │
│  ┌────────▼────────────────────────────────────────▼┐│
│  │          Kernel — Rust Core Loop                 ││
│  │  Intent → Policy → Route → Execute → Trajectory  ││
│  └────────┬─────────────────────────────────────────┘│
│           │                                          │
│  ┌────────▼────────┐  ┌──────────────────────────────▼┐│
│  │  Tool Bus (Rust) │  │  Python Learning Service     ││
│  │  WASM / Docker   │  │  Trajectory → Skill Drafts   ││
│  │  MCP / Native     │  │  Evaluator / Synthesizer     ││
│  └──────────────────┘  └─────────────────────────────││
└──────────────────────────────────────────────────────┘
```

---

## Repository Layout

```
clawstack/
├── rust/                     # Rust workspace
│   └── crates/
│       ├── clawstack-common/   # Shared types, errors, schemas
│       ├── clawstack-gateway/  # HTTP/WS API
│       ├── clawstack-kernel/   # Core agent loop
│       ├── clawstack-memory/   # Postgres + pgvector
│       ├── clawstack-policy/   # Capability enforcement
│       ├── clawstack-scheduler/# Job orchestration
│       ├── clawstack-toolbus/  # Unified tool bus
│       ├── clawstack-channels/ # Channel adapters
│       ├── clawstack-code-runtime/ # Repo coding worker
│       ├── clawstack-mcp/     # MCP client
│       ├── clawstack-api/     # gRPC/HTTP API
│       └── clawstack-cli/     # CLI client
│
├── python/                   # Python learning service
│   └── learning/
│       ├── main.py              # FastAPI service
│       ├── collector.py         # Trajectory storage
│       ├── skill_synth.py       # Skill synthesis
│       └── evaluators/          # Benchmark evaluation
│
├── schemas/                  # JSON Schema + Protobuf
│
├── skills/                   # Skill manifests + WASM
│
├── infra/
│   ├── docker/               # Docker configs
│   └── kubernetes/           # K8s manifests
│
├── docs/                    # Architecture, SKILLS, SECURITY
│
└── SPEC.md                  # Full project specification
```

---

## Quick Start

```bash
# Clone
git clone https://github.com/your-org/clawstack.git
cd clawstack

# Start infrastructure
docker compose up -d postgres redis

# Build Rust
cargo build --release

# Run
./target/release/clawstack serve
```

---

## Rust Crates

| Crate | Description |
|-------|-------------|
| `clawstack-common` | Shared types (ToolSpec, Skill, Trajectory, etc.) |
| `clawstack-gateway` | HTTP/WS API surface, channel normalization |
| `clawstack-kernel` | Core agent loop, intent routing, policy enforcement |
| `clawstack-memory` | Three-tier memory with Postgres + pgvector |
| `clawstack-policy` | Capability engine, workspace policies |
| `clawstack-scheduler` | Job queue, worker pool, Docker/WASM runners |
| `clawstack-toolbus` | Unified tool execution interface |
| `clawstack-channels` | Channel adapters (CLI, Telegram, Web) |
| `clawstack-code-runtime` | Repo-aware coding worker |
| `clawstack-mcp` | MCP protocol client |
| `clawstack-api` | Public gRPC + HTTP API |
| `clawstack-cli` | Terminal CLI client |

---

## Capability Model

Every tool and worker declares required capabilities. The kernel enforces them.

| Capability | Description |
|------------|-------------|
| `memory.read` | Read from memory system |
| `memory.write` | Write to memory system |
| `skills.install` | Install new skills |
| `skills.execute` | Execute skills |
| `repo.read` | Read from repositories |
| `repo.write` | Write to repositories |
| `shell.exec` | Execute shell commands |
| `shell.test` | Run tests |
| `browser.use` | Use browser automation |
| `web.fetch` | Fetch web content |
| `secrets.use` | Access secrets |
| `channel.send` | Send messages |
| `admin.approve_skill` | Approve skill drafts |

---

## Skill Lifecycle

1. **Trajectory captured** — Multi-tool execution stored by Rust kernel
2. **Python analyzes** — `SkillSynthesizer` clusters patterns, generates `SkillDraft`
3. **Rust approves** — Kernel validates, checks policies, activates skill
4. **Skill executed** — Available in tool registry for future requests

---

## Development

```bash
# Rust
cargo check
cargo test
cargo run -p clawstack-cli -- serve

# Python
cd python
uv sync
uv run fastapi dev learning/main.py

# Run everything
docker compose up -d
```

---

## License

MIT OR Apache-2.0
