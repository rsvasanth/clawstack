//! The main agent loop — accepts, processes, and responds to requests.

use crate::context::ExecutionContext;
use crate::intent::Intent;
use crate::routing::Route;
use clawstack_common::{
    AgentRequest, AgentResponse, Capability, ClawstackError, ContextBuilder,
    ToolCall, ToolResult, Trajectory, TrajectoryOutcome, WorkspacePolicy,
};
use std::sync::Arc;
use tracing::{info, warn, Instrument};

/// Core agent kernel — handles one request end-to-end.
pub struct AgentKernel {
    policy: Arc<dyn PolicyProvider>,
    memory: Arc<dyn MemoryProvider>,
    toolbus: Arc<dyn ToolBus>,
    llm: Arc<dyn LlmClient>,
}

#[async_trait::async_trait]
pub trait PolicyProvider: Send + Sync {
    async fn get_policy(&self, workspace_id: uuid::Uuid) -> Result<WorkspacePolicy, ClawstackError>;
    async fn check_capability(
        &self,
        workspace_id: uuid::Uuid,
        user_id: uuid::Uuid,
        capability: Capability,
    ) -> Result<bool, ClawstackError>;
}

#[async_trait::async_trait]
pub trait MemoryProvider: Send + Sync {
    async fn retrieve(
        &self,
        ctx: &ExecutionContext,
        limit: u32,
    ) -> Result<Vec<crate::context::MemoryEntry>, ClawstackError>;

    async fn store(
        &self,
        trajectory: &Trajectory,
    ) -> Result<(), ClawstackError>;
}

#[async_trait::async_trait]
pub trait ToolBus: Send + Sync {
    async fn execute(
        &self,
        call: ToolCall,
        ctx: &ExecutionContext,
    ) -> Result<ToolResult, ClawstackError>;

    async fn execute_many(
        &self,
        calls: Vec<ToolCall>,
        ctx: &ExecutionContext,
    ) -> Result<Vec<ToolResult>, ClawstackError> {
        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            results.push(self.execute(call, ctx).await?);
        }
        Ok(results)
    }

    /// Get the specification for a tool, including its required capabilities.
    fn get_spec(&self, tool_id: &str) -> Option<crate::ToolSpec>;
}

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(
        &self,
        prompt: &str,
        context: &ExecutionContext,
    ) -> Result<LlmResponse, ClawstackError>;
}

#[derive(Debug)]
pub struct LlmResponse {
    pub text: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: String,
}

impl AgentKernel {
    pub fn new(
        policy: Arc<dyn PolicyProvider>,
        memory: Arc<dyn MemoryProvider>,
        toolbus: Arc<dyn ToolBus>,
        llm: Arc<dyn LlmClient>,
    ) -> Self {
        Self { policy, memory, toolbus, llm }
    }

    /// Main entry point — process one request.
    pub async fn run(&self, req: AgentRequest) -> Result<AgentResponse, ClawstackError> {
        let trace_id = uuid::Uuid::new_v4();
        let span = tracing::info_span!("kernel.run", trace_id = %trace_id);

        async move {
            info!(session_id = %req.session_id, "processing request");

            // 1. Build execution context
            let ctx = self.build_context(&req).await?;

            // 2. Classify intent
            let intent = self.classify_intent(&ctx).await?;

            // 3. Route and execute
            let response = match intent {
                Intent::ChatOnly => self.handle_chat(&ctx).await?,
                Intent::UseTools { tool_ids } => self.handle_tools(&ctx, tool_ids).await?,
                Intent::SpawnWorker(worker) => self.handle_worker(&ctx, worker).await?,
                Intent::DelegateToPython => self.handle_python_delegation(&ctx).await?,
            };

            // 4. Store trajectory (async, don't block response)
            if let Err(e) = self.store_trajectory(&ctx, &response).await {
                warn!("failed to store trajectory: {}", e);
            }

            Ok(response)
        }
        .instrument(span)
        .await
    }

    async fn build_context(&self, req: &AgentRequest) -> Result<ExecutionContext, ClawstackError> {
        let memory = self.memory.retrieve(&ExecutionContext::from(req), 20).await?;

        let policy = self.policy.get_policy(req.workspace_id).await?;

        Ok(ExecutionContext::from_request(req, memory, policy))
    }

    async fn classify_intent(&self, ctx: &ExecutionContext) -> Result<Intent, ClawstackError> {
        let prompt = format!(
            "Classify: {}\nOptions: chat_only, use_tools, spawn_worker, delegate_python",
            ctx.request.text
        );

        let response = self.llm.complete(&prompt, ctx).await?;

        // Parse LLM response into Intent
        // For now, simple pattern matching; later use structured output
        let text = response.text.unwrap_or_default();
        if text.contains("use_tools") {
            Ok(Intent::UseTools { tool_ids: vec![] })
        } else if text.contains("spawn_worker") {
            Ok(Intent::SpawnWorker("code".to_string()))
        } else if text.contains("delegate_python") {
            Ok(Intent::DelegateToPython)
        } else {
            Ok(Intent::ChatOnly)
        }
    }

    async fn handle_chat(&self, ctx: &ExecutionContext) -> Result<AgentResponse, ClawstackError> {
        let response = self.llm.complete(&ctx.request.text, ctx).await?;
        Ok(AgentResponse {
            text: response.text,
            attachments: vec![],
            tool_calls: vec![],
            tool_results: vec![],
            session_id: Some(ctx.request.session_id),
        })
    }

    async fn handle_tools(
        &self,
        ctx: &ExecutionContext,
        _tool_ids: Vec<String>,
    ) -> Result<AgentResponse, ClawstackError> {
        // Build tool calls from LLM response
        let response = self.llm.complete(&ctx.request.text, ctx).await?;

        let tool_calls = response.tool_calls;

        // === CAPABILITY ENFORCEMENT ===
        // Check that the user has required capabilities before executing any tool
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
                    return Err(ClawstackError::capability_denied(
                        required_cap.as_str(),
                        "user lacks required capability",
                    ));
                }
            }
        }

        let tool_results = self.toolbus.execute_many(tool_calls.clone(), ctx).await?;

        // Synthesize final response from tool results
        let text = format!(
            "Executed {} tools. Last result: {}",
            tool_results.len(),
            tool_results.last().map(|r| r.output.to_string()).unwrap_or_default()
        );

        Ok(AgentResponse {
            text,
            attachments: vec![],
            tool_calls,
            tool_results,
            session_id: Some(ctx.request.session_id),
        })
    }

    async fn handle_worker(
        &self,
        ctx: &ExecutionContext,
        worker_type: String,
    ) -> Result<AgentResponse, ClawstackError> {
        // Spawn job to scheduler, return job ID
        Ok(AgentResponse {
            text: format!("Spawning {} worker...", worker_type),
            attachments: vec![],
            tool_calls: vec![],
            tool_results: vec![],
            session_id: Some(ctx.request.session_id),
        })
    }

    async fn handle_python_delegation(&self, ctx: &ExecutionContext) -> Result<AgentResponse, ClawstackError> {
        // Call Python learning service for analysis
        Ok(AgentResponse {
            text: "Delegating to Python learning service...".to_string(),
            attachments: vec![],
            tool_calls: vec![],
            tool_results: vec![],
            session_id: Some(ctx.request.session_id),
        })
    }

    async fn store_trajectory(&self, ctx: &ExecutionContext, resp: &AgentResponse) -> Result<(), ClawstackError> {
        use clawstack_common::{MessageRole, TrajectoryToolCall, TrajectoryTurn};

        let mut turns = Vec::new();
        let start_time = chrono::Utc::now();

        // Turn 0: User message
        turns.push(TrajectoryTurn {
            turn_index: 0,
            role: MessageRole::User,
            content: ctx.request.text.clone(),
            tool_calls: vec![],
        });

        // Subsequent turns: LLM response + tool calls/results
        let mut turn_index = 1;
        for (call, result) in resp.tool_calls.iter().zip(resp.tool_results.iter()) {
            // LLM generated this tool call
            turns.push(TrajectoryTurn {
                turn_index,
                role: MessageRole::Assistant,
                content: format!("Called tool: {}", call.tool_id),
                tool_calls: vec![],
            });
            turn_index += 1;

            // Tool execution result
            let tool_call = TrajectoryToolCall {
                tool_id: call.tool_id.clone(),
                parameters: call.parameters.clone(),
                output: result.output.as_ref().map(|o| o.to_string()),
                success: result.success,
                duration_ms: result.duration_ms,
            };
            turns.push(TrajectoryTurn {
                turn_index,
                role: MessageRole::Tool,
                content: result.output.as_ref().map(|o| o.to_string()).unwrap_or_default(),
                tool_calls: vec![tool_call],
            });
            turn_index += 1;
        }

        // If no tools were called, just have the user turn
        let outcome = if resp.tool_calls.is_empty() {
            TrajectoryOutcome::CompletedChat
        } else {
            TrajectoryOutcome::CompletedTools
        };

        let duration_ms = (chrono::Utc::now() - start_time).num_milliseconds() as u64;

        let trajectory = Trajectory {
            id: uuid::Uuid::new_v4(),
            workspace_id: ctx.request.workspace_id,
            session_id: ctx.request.session_id,
            user_message: ctx.request.text.clone(),
            turns,
            outcome,
            duration_ms,
            model: None,
            created_at: chrono::Utc::now(),
        };

        self.memory.store(&trajectory).await
    }
}
