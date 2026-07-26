use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};

use super::subagent::SubAgent;
use super::types::TurnError;

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
    Reply(String),
    Completed { status: String, summary: String },
}

pub async fn run_tool_calling_loop(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    mut messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
    let mut tools = agent.tools();
    tools.push(complete_task_tool_definition());

    for _ in 0..MAX_TOOL_TURNS {
        let request = LlmRequest {
            system: Some(agent.system_prompt().to_string()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
        };

        let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;

        messages.push(LlmMessage { role: LlmRole::Assistant, content: response.content.clone() });

        if response.stop_reason != StopReason::ToolUse {
            let reply_text = response
                .content
                .into_iter()
                .find_map(|block| match block {
                    ContentBlock::Text { text } => Some(text),
                    _ => None,
                })
                .unwrap_or_default();
            return Ok(LoopOutcome::Reply(reply_text));
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
