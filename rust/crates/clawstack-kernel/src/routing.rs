//! Routing — determines execution path based on intent.

/// Route decision from the kernel.
#[derive(Debug)]
pub enum Route {
    /// Execute in-process with the kernel.
    Kernel,
    /// Dispatch to async job queue.
    AsyncJob { job_type: String },
    /// Delegate to Python learning service over gRPC.
    PythonService { endpoint: String },
    /// Proxy to external MCP server.
    McpServer { server_id: String },
}
