use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use nomi_embedding::EmbeddingProvider;
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::error::TurnError;
use crate::memory;
use crate::subagent::SubAgent;

const MAX_TOOL_TURNS: u32 = 10;
pub const COMPLETE_TASK_TOOL_NAME: &str = "complete_task";

fn complete_task_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: COMPLETE_TASK_TOOL_NAME.to_string(),
        description: "Call this when you are done helping with this task, whether it succeeded or the user wants to stop.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "status": {"type": "string", "enum": ["completed", "cancelled"]},
                "summary": {"type": "string"}
            },
            "required": ["status", "summary"]
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply { text: String, memory_ids_used: Vec<Uuid>, input_tokens: u32, output_tokens: u32 },
    Completed { status: String, summary: String },
}

/// Runs one agent turn to completion: retrieves memory first if `agent.uses_memory()`,
/// then drives the tool-calling loop with every LLM call streamed (deltas published over
/// `mqtt` when provided — this now happens for every agent, not just a hardcoded chitchat
/// case), then extracts+stores memory after a `Reply` outcome if `agent.uses_memory()`.
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    mut messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
    let mut tools = agent.tools();
    tools.push(complete_task_tool_definition());

    let memories = if agent.uses_memory() {
        let last_user_text = messages
            .iter()
            .rev()
            .find(|m| m.role == LlmRole::User)
            .and_then(|m| m.content.iter().find_map(|b| match b {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            }))
            .unwrap_or_default();
        memory::try_retrieve_memories(conn, embedding_provider, user_id, &last_user_text).await
    } else {
        Vec::new()
    };

    let system_prompt = if memories.is_empty() {
        agent.system_prompt().to_string()
    } else {
        let mut prompt = format!("{}\n\nRelevant things you know about this user:\n", agent.system_prompt());
        for m in &memories {
            prompt.push_str(&format!("- {}\n", m.content));
        }
        prompt
    };

    for _ in 0..MAX_TOOL_TURNS {
        let request = LlmRequest {
            system: Some(system_prompt.clone()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
        };

        // The LLM call happens outside any DB transaction: holding a transaction open across
        // a slow network round trip would needlessly extend how long this connection's locks
        // are held.
        let stream = provider.complete_stream(request).await.map_err(TurnError::LlmCallFailed)?;
        let response = match mqtt {
            Some((publisher, turn_job_id)) => {
                use futures_util::StreamExt;
                // LlmEventStream requires 'static (boxed as `dyn Stream + Send`, no lifetime),
                // so this clones the publisher handle (cheap — wraps rumqttc's AsyncClient,
                // itself a cheap handle clone) rather than capturing the `&MqttPublisher`
                // borrow.
                let publisher = publisher.clone();
                let published = stream.then(move |event_result| {
                    let publisher = publisher.clone();
                    async move {
                        if let Ok(event) = &event_result {
                            let envelope = StreamEnvelope::Delta { turn_job_id, event: event.clone() };
                            // Best-effort: an MQTT publish failure never fails the turn.
                            let _ = publisher.publish(session_id, &envelope).await;
                        }
                        event_result
                    }
                });
                nomi_llm::collect_stream(Box::pin(published)).await.map_err(TurnError::LlmCallFailed)?
            }
            None => nomi_llm::collect_stream(stream).await.map_err(TurnError::LlmCallFailed)?,
        };

        messages.push(LlmMessage { role: LlmRole::Assistant, content: response.content.clone() });

        if response.stop_reason != StopReason::ToolUse {
            let input_tokens = response.input_tokens;
            let output_tokens = response.output_tokens;
            let reply_text = response
                .content
                .into_iter()
                .find_map(|block| match block {
                    ContentBlock::Text { text } => Some(text),
                    _ => None,
                })
                .unwrap_or_default();

            if agent.uses_memory() {
                let last_user_text = messages
                    .iter()
                    .rev()
                    .find(|m| m.role == LlmRole::User)
                    .and_then(|m| m.content.iter().find_map(|b| match b {
                        ContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    }))
                    .unwrap_or_default();
                // Best-effort: never changes the turn's outcome. See the equivalent
                // note that used to live in turn/chitchat.rs before this generalization.
                memory::extract_and_store_memory(conn, provider, embedding_provider, user_id, &last_user_text, &reply_text).await;
            }

            return Ok(LoopOutcome::Reply {
                text: reply_text,
                memory_ids_used: memories.iter().map(|m| m.id).collect(),
                input_tokens,
                output_tokens,
            });
        }

        let mut tool_results = Vec::new();
        for block in &response.content {
            if let ContentBlock::ToolUse { id, name, input } = block {
                let (result_text, is_error) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false)
                } else {
                    match agent.execute_tool(conn, user_id, name, input.clone()).await {
                        Ok(text) => (text, false),
                        Err(err) => (err, true),
                    }
                };

                log_tool_call(conn, session_id, agent_session_id, agent.agent_type(), name, input, &result_text, is_error).await;

                if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
                    let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    return Ok(LoopOutcome::Completed { status, summary });
                }

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: id.clone(),
                    content: result_text,
                    is_error,
                });
            }
        }

        messages.push(LlmMessage { role: LlmRole::User, content: tool_results });
    }

    Err(TurnError::ToolLoopExceeded)
}

async fn log_tool_call(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_type: &str,
    tool_name: &str,
    input: &serde_json::Value,
    result: &str,
    is_error: bool,
) {
    let _ = sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'ToolCalled', $4)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(agent_type)
    .bind(serde_json::json!({"tool_name": tool_name, "input": input, "result": result, "is_error": is_error}))
    .execute(&mut **conn)
    .await;
}
