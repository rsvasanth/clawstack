//! ClawStack Kernel — the authoritative orchestrator.
//!
//! Core loop:
//!  1. Accept request
//!  2. Build context (memory + identity)
//!  3. Classify intent
//!  4. Route: chat / tool / job / code-worker
//!  5. Enforce policy
//!  6. Execute through tool-bus
//!  7. Store trajectory
//!  8. Synthesize response

pub mod intent;
pub mod context;
pub mod routing;
pub mod loop_;

pub use loop_::*;
