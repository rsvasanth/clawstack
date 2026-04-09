//! ClawStack CLI — Command-line interface for the agent.
//!
//! Connects to the gateway via HTTP and provides an interactive chat loop.

use anyhow::Result;
use clap::{Parser, Subcommand};
use dialogue::Input;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "clawstack")]
#[command(about = "ClawStack CLI - Chat with the agent", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:8080")]
    gateway_url: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session
    Chat {
        /// Workspace ID to use
        #[arg(short, long)]
        workspace_id: Option<Uuid>,
        /// Session ID to resume (if empty, creates new session)
        #[arg(short, long)]
        session_id: Option<Uuid>,
    },
    /// Send a single message and exit
    Send {
        /// Workspace ID
        #[arg(short, long)]
        workspace_id: Uuid,
        /// Message to send
        #[arg(short, long)]
        message: String,
    },
    /// Check gateway health
    Health,
}

#[derive(Serialize)]
struct AgentRequest {
    workspace_id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    channel: String,
    text: String,
    attachments: Vec<serde_json::Value>,
    metadata: serde_json::Value,
}

#[derive(Deserialize)]
struct AgentResponse {
    text: Option<String>,
    attachments: Vec<serde_json::Value>,
    tool_calls: Vec<serde_json::Value>,
    tool_results: Vec<serde_json::Value>,
    session_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    let client = Client::new();

    match cli.command {
        Some(Commands::Health) => {
            health_check(&client, &cli.gateway_url).await?;
        }
        Some(Commands::Send {
            workspace_id,
            message,
        }) => {
            send_single_message(&client, &cli.gateway_url, workspace_id, message).await?;
        }
        Some(Commands::Chat {
            workspace_id,
            session_id,
        }) => {
            chat_loop(&client, &cli.gateway_url, workspace_id, session_id).await?;
        }
        None => {
            // Default to chat mode
            chat_loop(&client, &cli.gateway_url, None, None).await?;
        }
    }

    Ok(())
}

async fn health_check(client: &Client, gateway_url: &str) -> Result<()> {
    let url = format!("{}/health", gateway_url);
    info!(url = %url, "checking health");

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let health: HealthResponse = response.json().await?;
        println!("✅ {} v{} - {}", health.service, health.version, health.status);
    } else {
        println!("❌ Gateway returned status: {}", response.status());
    }

    Ok(())
}

async fn send_single_message(
    client: &Client,
    gateway_url: &str,
    workspace_id: Uuid,
    message: String,
) -> Result<()> {
    let session_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let request = AgentRequest {
        workspace_id,
        session_id,
        user_id,
        channel: "cli".to_string(),
        text: message,
        attachments: vec![],
        metadata: serde_json::json!({}),
    };

    let response = send_request(client, gateway_url, request).await?;

    if let Some(text) = response.text {
        println!("{}", text);
    }

    if !response.tool_results.is_empty() {
        println!("\n[Tool results: {}]", response.tool_results.len());
    }

    Ok(())
}

async fn chat_loop(
    client: &Client,
    gateway_url: &str,
    workspace_id: Option<Uuid>,
    session_id: Option<Uuid>,
) -> Result<()> {
    // Get workspace ID from user if not provided
    let workspace_id = match workspace_id {
        Some(id) => id,
        None => {
            let input = Input::new()
                .with_prompt("Workspace ID")
                .interact()?;
            Uuid::parse_str(&input)?
        }
    };

    let session_id = session_id.unwrap_or_else(Uuid::new_v4);
    let user_id = Uuid::new_v4();

    println!("\n🐱 ClawStack CLI");
    println!("══════════════════════════════════════════");
    println!("Workspace: {}", workspace_id);
    println!("Session:   {}", session_id);
    println!("Gateway:   {}", gateway_url);
    println!("══════════════════════════════════════════");
    println!("Type 'quit' or 'exit' to end the session.\n");

    loop {
        let input = Input::new()
            .with_prompt("You")
            .interact()?;

        let text = input.trim();

        if text.is_empty() {
            continue;
        }

        if text == "quit" || text == "exit" {
            println!("Goodbye!");
            break;
        }

        print!("Agent: ");
        std::io::stdout().flush()?;

        let request = AgentRequest {
            workspace_id,
            session_id,
            user_id,
            channel: "cli".to_string(),
            text: text.to_string(),
            attachments: vec![],
            metadata: serde_json::json!({}),
        };

        match send_request(client, gateway_url, request).await {
            Ok(response) => {
                if let Some(text) = response.text {
                    println!("{}", text);
                } else {
                    println!("(no response)");
                }

                if !response.tool_results.is_empty() {
                    println!("\n[{} tool(s) executed]", response.tool_results.len());
                }
            }
            Err(e) => {
                error!(error = %e, "request failed");
                println!("❌ Error: {}", e);
            }
        }

        println!();
    }

    Ok(())
}

async fn send_request(
    client: &Client,
    gateway_url: &str,
    request: AgentRequest,
) -> anyhow::Result<AgentResponse> {
    let url = format!("{}/api/v1/agent/request", gateway_url);

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Gateway error {}: {}", status, body));
    }

    let agent_response = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    Ok(agent_response)
}
