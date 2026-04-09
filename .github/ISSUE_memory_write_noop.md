# Issue: `memory.write` tool is a no-op — ToolBus cannot persist to MemoryProvider

## Problem

The `memory.write` tool in `clawstack-toolbus/src/lib.rs` (lines 301–322) returns a mock success response instead of persisting to the actual memory provider.

```rust
// In a real implementation, this would store to the memory provider.
// For now, we just return success.
Ok(format!(r#"{{"stored": true, "memory_id": "mock-{}", ...}}"#, ...))
```

**Impact:** The ToolBus has no path to call the MemoryProvider. Memory writes via the `memory.write` tool are silently discarded — they never reach Postgres.

## Root Cause

The `ToolBus` trait and `ToolBusImpl` have no reference to the `MemoryProvider`. Tools that need to persist data (like `memory.write`) have no mechanism to do so.

## Proposed Fix

Two options:

### Option A — Add write method to ToolBus trait
Add `fn write_memory(&self, entry: &MemoryEntry) -> Result<(), ClawstackError>` to the `ToolBus` trait, implement it in `ToolBusImpl` by delegating to a `MemoryProvider` reference, and wire it up in `main.rs`.

### Option B — Route memory.write through kernel
The kernel's `handle_tools()` could intercept `memory.write` calls and persist them via `MemoryProvider` before returning tool results — similar to how capability enforcement is already wired there.

**Option B is cleaner** — it keeps the ToolBus focused on execution-only and keeps persistence at the kernel level where `MemoryProvider` is already available.

## Tasks
- [ ] Decide on Option A vs B
- [ ] Implement the chosen approach
- [ ] Add integration test: call memory.write, verify it appears in Postgres `memories` table

## Severity
P2 — Core learning loop depends on memory persistence. Without this, synthesized skills can't be grounded in actual experience.
