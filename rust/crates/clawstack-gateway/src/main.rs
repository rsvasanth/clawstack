//! ClawStack Gateway binary.
//!
//! Wires together all components:
//! - Agent kernel
//! - LLM client (OpenAI or mock)
//! - ToolBus (built-in tools)
//! - Memory provider (Postgres or in-memory)
//! - Policy provider (Postgres or in-memory)

use anyhow::Result;
use clawstack_gateway::{create_router, AppState};
use clawstack_kernel::loop_::AgentKernel;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("starting ClawStack gateway");

    // Build components
    // Note: In production, these would be loaded from config/environment
    let policy = build_policy_provider().await?;
    let memory = build_memory_provider().await?;
    let toolbus = Arc::new(build_toolbus().await);
    let llm = build_llm_client().await;

    // Create kernel
    let kernel = AgentKernel::new(policy, memory, toolbus, llm);

    // Create app state
    let state = Arc::new(AppState { kernel });

    // Create router
    let app = create_router(state).layer(TraceLayer::newForHttp());

    // Start server
    let addr = std::env::var("GW_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(&addr).await?;
    info!(addr = %addr, "gateway listening");

    axum::serve(listener, app).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Component builders (swap implementations for testing/production)
// ---------------------------------------------------------------------------

async fn build_policy_provider() -> Result<Arc<dyn clawstack_kernel::loop_::PolicyProvider>> {
    let database_url = std::env::var("DATABASE_URL").ok();

    if let Some(url) = database_url {
        info!("using Postgres policy provider");
        let pool = sqlx::PgPool::connect(&url).await?;
        Ok(Arc::new(clawstack_policy::PostgresPolicyProvider::new(pool)))
    } else {
        info!("using in-memory policy provider (no DATABASE_URL)");
        Ok(Arc::new(clawstack_policy::InMemoryPolicyProvider::new()))
    }
}

async fn build_memory_provider() -> Result<Arc<dyn clawstack_kernel::loop_::MemoryProvider>> {
    let database_url = std::env::var("DATABASE_URL").ok();

    if let Some(url) = database_url {
        info!("using Postgres memory provider");
        let pool = sqlx::PgPool::connect(&url).await?;
        Ok(Arc::new(clawstack_memory::PostgresMemoryProvider::new(pool)))
    } else {
        info!("using in-memory memory provider (no DATABASE_URL)");
        Ok(Arc::new(clawstack_memory::InMemoryMemoryProvider::new()))
    }
}

async fn build_toolbus() -> clawstack_toolbus::ToolBusImpl {
    clawstack_toolbus::ToolBusImpl::new()
}

async fn build_llm_client() -> Arc<dyn clawstack_kernel::loop_::LlmClient> {
    // Try to create OpenAI client, fall back to echo client
    match clawstack_llm::OpenAiLlmClient::new() {
        Ok(client) => {
            info!("using OpenAI LLM client");
            Arc::new(client)
        }
        Err(e) => {
            info!(error = %e, "OpenAI not configured, using echo client");
            Arc::new(clawstack_llm::EchoLlmClient::new("[Echo] "))
        }
    }
}
