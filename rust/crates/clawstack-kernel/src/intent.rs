//! Intent classification — determines what the agent should do with a request.

/// Classification of a user's intent from their message.
#[derive(Debug)]
pub enum Intent {
    /// Pure chat — no tools needed.
    ChatOnly,
    /// Use tools (identified by tool IDs).
    UseTools { tool_ids: Vec<String> },
    /// Spawn a background worker.
    SpawnWorker(String),
    /// Delegate to Python learning service.
    DelegateToPython,
}
