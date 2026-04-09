# Contributing to ClawStack

## Core Principle

**Rust orchestrates, Python proposes.**

- Rust owns: the kernel, tool bus, policy enforcement, channel routing, memory, job scheduling, and all security boundaries.
- Python owns: learning, skill synthesis, trajectory analysis, and evaluation.

If your change puts Python in the critical path of request handling, that's a design conversation, not a PR.

## Project Structure

```
clawstack/
  rust/          # Rust workspace (see rust/Cargo.toml for crate list)
  python/        # Python learning service
  schemas/       # JSON Schema sources (generated into both rust and python)
  skills/        # Skill manifests
  infra/         # Docker, Kubernetes, migrations
  docs/          # Architecture docs
```

## Development Setup

```bash
# Prerequisites
cargo 1.82+
python 3.12+
docker (for postgres + redis)

# Clone and init
git clone https://github.com/your-org/clawstack.git
cd clawstack

# Start infrastructure
docker compose -f infra/docker/docker-compose.yml up -d

# Build Rust
cargo build --release

# Build Python
cd python && uv sync
```

## Code Generation

Shared types are defined in `schemas/` as JSON Schema. Rust types are hand-written in `clawstack-common`. Python types are generated:

```bash
# Generate Python from schema
cd python && uv run gen-from-schema ../schemas/
```

## Style

- **Rust**: `cargo fmt` + `cargo clippy`
- **Python**: `ruff format` + `ruff check`
- **Commits**: conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`)

## Testing

```bash
# Rust
cargo test

# Python
cd python && uv run pytest

# E2E
docker compose -f infra/docker/docker-compose.yml up --abort-on-container-exit
```

## Adding a New Tool

1. Define `ToolSpec` in `clawstack-common/src/tool.rs`
2. Implement tool in `clawstack-toolbus/`
3. Register in tool registry with required capabilities
4. Add tests
5. Update docs

## Adding a Channel

1. Implement `ChannelAdapter` trait in `clawstack-channels/`
2. Register adapter in gateway
3. Add to Docker Compose for local dev
