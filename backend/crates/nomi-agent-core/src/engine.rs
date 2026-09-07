use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use nomi_embedding::EmbeddingProvider;
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::error::TurnError;
use crate::memory;
use crate::permissions;
use crate::registry::AgentRegistry;
use crate::subagent::SubAgent;

const MAX_TOOL_TURNS: u32 = 10;
pub const COMPLETE_TASK_TOOL_NAME: &str = "complete_task";
pub const DELEGATE_TOOL_NAME: &str = "delegate_to_agent";
pub const SHOW_TABLE_TOOL_NAME: &str = "show_table";
pub const UPDATE_TODOS_TOOL_NAME: &str = "update_todos";

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

fn delegate_tool_definition(targets: &[&str]) -> ToolDefinition {
    ToolDefinition {
        name: DELEGATE_TOOL_NAME.to_string(),
        description: format!(
            "Hand off a task to a specialist agent to work on in the background, and immediately \
             tell the user you'll follow up — do NOT wait for the result before replying. Use this \
             only when the request genuinely needs a specialist; answer anything else yourself. \
             Available specialists: {}.",
            targets.join(", "),
        ),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "target_agent": {"type": "string", "enum": targets, "description": "Which specialist to delegate to"},
                "task": {"type": "string", "description": "What to ask the specialist to do, in your own words"}
            },
            "required": ["target_agent", "task"]
        }),
    }
}

fn show_table_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: SHOW_TABLE_TOOL_NAME.to_string(),
        description: "Show the user a table of structured data — either a plain data table or a side-by-side comparison of a few items across criteria.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "variant": {"type": "string", "enum": ["data", "comparison"]},
                "columns": {
                    "type": "array",
                    "items": {"type": "object", "properties": {"key": {"type": "string"}, "label": {"type": "string"}}, "required": ["key", "label"]}
                },
                "rows": {"type": "array", "items": {"type": "object"}}
            },
            "required": ["variant", "columns", "rows"]
        }),
    }
}

fn update_todos_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: UPDATE_TODOS_TOOL_NAME.to_string(),
        description: "Set or replace your current multi-step task checklist, shown live to the user. Call this again with the full updated list whenever a step's status changes.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "text": {"type": "string"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "done"]}
                        },
                        "required": ["id", "text", "status"]
                    }
                }
            },
            "required": ["items"]
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply { text: String, memory_ids_used: Vec<Uuid>, input_tokens: u32, output_tokens: u32 },
    Completed { status: String, summary: String },
    AwaitingApproval { message_id: Uuid },
}

/// Tools that are never permission-gated: engine-level bookkeeping (complete_task,
/// delegate_to_agent, show_table, update_todos) — none of these touch anything a user would
/// want to approve/deny.
fn is_gateable(tool_name: &str) -> bool {
    tool_name != COMPLETE_TASK_TOOL_NAME
        && tool_name != DELEGATE_TOOL_NAME
        && tool_name != SHOW_TABLE_TOOL_NAME
        && tool_name != UPDATE_TODOS_TOOL_NAME
}

fn describe_pending_action(tool_name: &str, input: &serde_json::Value) -> String {
    let path = input.get("path").and_then(|v| v.as_str());
    match (tool_name, path) {
        ("delete_file", Some(path)) => format!("Delete {path}"),
        ("write_file", Some(path)) => format!("Overwrite {path}"),
        (other, Some(path)) => format!("Run {other} on {path}"),
        (other, None) => format!("Run {other}"),
    }
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
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    mut messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
    let mut tools = agent.tools();
    tools.push(complete_task_tool_definition());
    if agent.can_delegate() {
        let targets = registry.delegatable_agent_types(agent.agent_type());
        if !targets.is_empty() {
            tools.push(delegate_tool_definition(&targets));
        }
    }
    tools.push(show_table_tool_definition());
    if agent.supports_todos() {
        tools.push(update_todos_tool_definition());
    }

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

    let system_prompt = if agent.uses_personality() {
        match crate::personality::get_current_personality(conn, user_id).await {
            Some(p) => format!("{system_prompt}\n\nAdopt this personality in your replies: {p}"),
            None => system_prompt,
        }
    } else {
        system_prompt
    };

    for _ in 0..MAX_TOOL_TURNS {
        let request = LlmRequest {
            system: Some(system_prompt.clone()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
            enable_reasoning: true,
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

        // Reasoning is shown for every agent, unlike the tool-call "💭" commentary below (which
        // stays gated behind `surfaces_activity()`) — a provider's own thinking trace is worth
        // seeing regardless of whether this agent normally narrates its tool calls.
        for block in &response.content {
            if let ContentBlock::Thinking { text, .. } = block {
                if !text.trim().is_empty() {
                    post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &format!("🧠 {}", text.trim()), None).await;
                }
            }
        }

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

        if agent.surfaces_activity() {
            if let Some(thought) = response.content.iter().find_map(|b| match b {
                ContentBlock::Text { text } if !text.trim().is_empty() => Some(text.clone()),
                _ => None,
            }) {
                post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &format!("💭 {}", thought.trim()), None).await;
            }
        }

        let pending_tool_use_blocks: Vec<ContentBlock> =
            response.content.iter().filter(|b| matches!(b, ContentBlock::ToolUse { .. })).cloned().collect();

        match resolve_tool_batch(conn, mqtt, registry, agent, session_id, agent_session_id, user_id, &pending_tool_use_blocks, &messages, None).await? {
            ToolBatchOutcome::AwaitingApproval { message_id } => return Ok(LoopOutcome::AwaitingApproval { message_id }),
            ToolBatchOutcome::Completed { status, summary } => return Ok(LoopOutcome::Completed { status, summary }),
            ToolBatchOutcome::Resolved(tool_results) => {
                messages.push(LlmMessage { role: LlmRole::User, content: tool_results });
            }
        }
    }

    Err(TurnError::ToolLoopExceeded)
}

/// The result of resolving one response's worth of tool_use blocks.
#[derive(Debug)]
pub enum ToolBatchOutcome {
    Resolved(Vec<ContentBlock>),
    Completed { status: String, summary: String },
    AwaitingApproval { message_id: Uuid },
}

/// Resolves every ToolUse block in `tool_use_blocks`, in order. `already_decided`, when set, is
/// `(tool_use_id, approved)` for a block whose approval was just resolved externally (a user
/// clicked Approve/Deny on its card) — that one block skips the permission check and uses the
/// given decision directly; every other block still goes through the normal
/// check-permission-then-execute-or-pause path, which may itself pause again on a *different*
/// block — handled identically to the very first pause (see nomi-turn's resume path, which calls
/// this same function again when that happens).
#[allow(clippy::too_many_arguments)]
pub async fn resolve_tool_batch(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    tool_use_blocks: &[ContentBlock],
    conversation_so_far: &[LlmMessage],
    already_decided: Option<(&str, bool)>,
) -> Result<ToolBatchOutcome, TurnError> {
    for (index, block) in tool_use_blocks.iter().enumerate() {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            if already_decided.map(|(decided_id, _)| decided_id == id.as_str()).unwrap_or(false) {
                continue;
            }
            if !is_gateable(name) {
                continue;
            }
            let decision = permissions::check_tool_permission(conn, user_id, name, input).await;
            if !matches!(decision, permissions::PermissionDecision::Ask) {
                continue;
            }

            let description = describe_pending_action(name, input);
            let approval_block = crate::content_block::ContentBlock::ApprovalRequest {
                id: Uuid::new_v4(),
                tool_name: name.clone(),
                description: description.clone(),
                input: input.clone(),
                status: crate::content_block::ApprovalStatus::Pending,
                decided_at: None,
            };
            let content_blocks = serde_json::json!([approval_block]);
            let message_id: Option<Uuid> = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(format!("⏳ {description}"))
            .bind(&content_blocks)
            .fetch_one(&mut **conn)
            .await
            .ok();

            let Some(message_id) = message_id else {
                return Err(TurnError::ToolLoopExceeded);
            };

            if let Some((publisher, _)) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
            }

            // Only the not-yet-resolved blocks from this point on need to survive into the next
            // resume — everything before `index` is already resolved by the time this is reached
            // (either during THIS call's own execution pass below, on a later resume, or never
            // executed at all on the very first pass, where nothing runs until the pre-scan
            // finds no more "Ask" blocks).
            let state_patch = serde_json::json!({
                "paused_for_approval": true,
                "pending_approval_message_id": message_id.to_string(),
                "pending_tool_use_id": id,
                "tool_use_blocks": &tool_use_blocks[index..],
                "messages": conversation_so_far,
            });
            let _ = sqlx::query("UPDATE agent_sessions SET state = state || $1 WHERE id = $2")
                .bind(&state_patch)
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await;

            return Ok(ToolBatchOutcome::AwaitingApproval { message_id });
        }
    }

    let mut tool_results = Vec::new();
    for block in tool_use_blocks {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            let (result_text, is_error, rich_block) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false, None)
            } else if name.as_str() == DELEGATE_TOOL_NAME {
                let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
                let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
                let rejection = registry.find(target_agent).and_then(|t| t.validate_delegation_task(task).err());
                let (text, err) = match rejection {
                    Some(reason) => (reason, true),
                    None => match crate::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type(), target_agent, task, user_id).await {
                        Ok(ack) => (ack, false),
                        Err(err) => (err, true),
                    },
                };
                (text, err, None)
            } else if name.as_str() == SHOW_TABLE_TOOL_NAME {
                match parse_table_input(input) {
                    Ok((variant, columns, rows)) => {
                        let text = table_display_text(&variant, rows.len());
                        let block = crate::content_block::ContentBlock::Table { variant, columns, rows };
                        (text, false, Some(block))
                    }
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == UPDATE_TODOS_TOOL_NAME {
                match parse_todo_items(input) {
                    Ok(items) => match upsert_todo_list(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, items).await {
                        Ok(text) => (text, false, None),
                        Err(err) => (err, true, None),
                    },
                    Err(err) => (err, true, None),
                }
            } else if already_decided.map(|(decided_id, _)| decided_id == id.as_str()).unwrap_or(false) {
                let approved = already_decided.unwrap().1;
                if approved {
                    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(outcome) => (outcome.display_text, false, outcome.block),
                        Err(err) => (describe_tool_error(name, input, &err), true, None),
                    }
                } else {
                    ("Denied by your permission rules.".to_string(), true, None)
                }
            } else if matches!(permissions::check_tool_permission(conn, user_id, name, input).await, permissions::PermissionDecision::Deny) {
                ("Denied by your permission rules.".to_string(), true, None)
            } else {
                match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                    Ok(outcome) => (outcome.display_text, false, outcome.block),
                    Err(err) => (describe_tool_error(name, input, &err), true, None),
                }
            };

            log_tool_call(conn, session_id, agent_session_id, agent.agent_type(), name, input, &result_text, is_error).await;

            let should_post = name.as_str() == SHOW_TABLE_TOOL_NAME
                || (agent.surfaces_activity()
                    && name.as_str() != COMPLETE_TASK_TOOL_NAME
                    && name.as_str() != DELEGATE_TOOL_NAME
                    && name.as_str() != UPDATE_TODOS_TOOL_NAME);
            if should_post {
                post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &result_text, rich_block.as_ref()).await;
            }

            if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
                let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                return Ok(ToolBatchOutcome::Completed { status, summary });
            }

            tool_results.push(ContentBlock::ToolResult { tool_use_id: id.clone(), content: result_text, is_error });
        }
    }

    Ok(ToolBatchOutcome::Resolved(tool_results))
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

/// Inserts an activity message (an agent's "thought", or a description of a tool call it just
/// made) so a user watching a long-running build sees it appear like any other chat message —
/// see `SubAgent::surfaces_activity`. Best-effort: never fails the turn.
async fn post_activity_message(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    content: &str,
    block: Option<&crate::content_block::ContentBlock>,
) {
    let content_blocks = block.map(|b| serde_json::json!([b]));
    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3)")
        .bind(session_id)
        .bind(content)
        .bind(&content_blocks)
        .execute(&mut **conn)
        .await;
    if let Some(publisher) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
    }
}

fn parse_todo_items(input: &serde_json::Value) -> Result<Vec<crate::content_block::TodoItem>, String> {
    let items = input.get("items").and_then(|v| v.as_array()).ok_or("items is required")?;
    items
        .iter()
        .map(|item| {
            let id = item.get("id").and_then(|v| v.as_str()).ok_or("each item needs an id")?.to_string();
            let text = item.get("text").and_then(|v| v.as_str()).ok_or("each item needs text")?.to_string();
            let status = match item.get("status").and_then(|v| v.as_str()) {
                Some("pending") => crate::content_block::TodoStatus::Pending,
                Some("in_progress") => crate::content_block::TodoStatus::InProgress,
                Some("done") => crate::content_block::TodoStatus::Done,
                _ => return Err("each item's status must be pending, in_progress, or done".to_string()),
            };
            Ok(crate::content_block::TodoItem { id, text, status })
        })
        .collect()
}

fn todo_list_display_text(items: &[crate::content_block::TodoItem]) -> String {
    use crate::content_block::TodoStatus;
    let mut out = String::from("📋 To-do list:\n");
    for item in items {
        let mark = match item.status {
            TodoStatus::Done => "[x]",
            TodoStatus::InProgress => "[~]",
            TodoStatus::Pending => "[ ]",
        };
        out.push_str(&format!("{mark} {}\n", item.text));
    }
    out
}

/// Finds the most recent todo-list message for this agent session (tracked via
/// `agent_sessions.state->>'todo_message_id'`) and updates it in place; inserts a fresh message
/// and records its id into `state` the first time this agent session ever calls update_todos.
/// Unlike every other block-producing tool, this posts/updates the message itself rather than
/// returning a block for engine.rs's generic post-at-the-bottom-of-the-loop path, since that
/// path only ever inserts — it has no notion of "update this existing row instead".
async fn upsert_todo_list(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    items: Vec<crate::content_block::TodoItem>,
) -> Result<String, String> {
    let display_text = todo_list_display_text(&items);
    let block = crate::content_block::ContentBlock::TodoList { items };
    let content_blocks = serde_json::json!([block]);

    let existing_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT (state->>'todo_message_id')::uuid FROM agent_sessions WHERE id = $1",
    )
    .bind(agent_session_id)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    match existing_id {
        Some(message_id) => {
            sqlx::query("UPDATE messages SET content = $1, content_blocks = $2 WHERE id = $3")
                .bind(&display_text)
                .bind(&content_blocks)
                .bind(message_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
        }
        None => {
            let message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(&display_text)
            .bind(&content_blocks)
            .fetch_one(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query("UPDATE agent_sessions SET state = state || jsonb_build_object('todo_message_id', $1::text) WHERE id = $2")
                .bind(message_id.to_string())
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    if let Some(publisher) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
    }

    Ok("todo list updated".to_string())
}

fn parse_table_input(
    input: &serde_json::Value,
) -> Result<(crate::content_block::TableVariant, Vec<crate::content_block::TableColumn>, Vec<serde_json::Value>), String> {
    use crate::content_block::{TableColumn, TableVariant};
    let variant = match input.get("variant").and_then(|v| v.as_str()) {
        Some("data") => TableVariant::Data,
        Some("comparison") => TableVariant::Comparison,
        _ => return Err("variant must be 'data' or 'comparison'".to_string()),
    };
    let columns: Vec<TableColumn> = input
        .get("columns")
        .and_then(|v| v.as_array())
        .ok_or("columns is required")?
        .iter()
        .map(|c| {
            let key = c.get("key").and_then(|v| v.as_str()).ok_or("each column needs a key")?.to_string();
            let label = c.get("label").and_then(|v| v.as_str()).ok_or("each column needs a label")?.to_string();
            Ok(TableColumn { key, label })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let rows = input.get("rows").and_then(|v| v.as_array()).ok_or("rows is required")?.clone();
    Ok((variant, columns, rows))
}

fn table_display_text(variant: &crate::content_block::TableVariant, row_count: usize) -> String {
    use crate::content_block::TableVariant;
    match variant {
        TableVariant::Data => format!("📊 Showed a table with {row_count} row(s)"),
        TableVariant::Comparison => format!("📊 Compared {row_count} item(s)"),
    }
}

/// Templated, non-LLM description of a failed tool call — kept generic in the engine (unlike a
/// success's display_text, which each tool builds itself) since failures never carry a block and
/// the "⚠️ couldn't X — {reason}" shape is the same regardless of which crate the tool lives in.
fn describe_tool_error(tool_name: &str, input: &serde_json::Value, result: &str) -> String {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("the file");
    match tool_name {
        "write_file" => format!("⚠️ Couldn't write `{path}` — {result}"),
        "read_file" => format!("⚠️ Couldn't read `{path}` — {result}"),
        "delete_file" => format!("⚠️ Couldn't delete `{path}` — {result}"),
        "list_files" => format!("⚠️ Couldn't list project files — {result}"),
        "create_project" => format!("⚠️ Couldn't create the project — {result}"),
        "write_plan" => format!("⚠️ Couldn't save the plan — {result}"),
        other => format!("⚠️ `{other}` failed — {result}"),
    }
}
