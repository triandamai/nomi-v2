//! Plans and todo lists an agent writes during a turn: telling a plan from a reply, saving it,
//! and keeping the posted todo list current.

use super::*;

/// Whether a reply is a plan written out in the chat: sections with steps under them, or a long
/// list of steps.
pub fn looks_like_plan(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().map(str::trim_start).collect();
    let headings = lines.iter().filter(|l| l.starts_with('#')).count();
    let steps = lines
        .iter()
        .filter(|l| {
            l.starts_with("- ")
                || l.starts_with("* ")
                || l.split_once(". ").is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        })
        .count();
    (headings >= 2 && steps >= 4) || (steps >= 6 && text.chars().count() >= 300)
}

/// A short title for a plan written out in chat: its first heading, else its first line.
pub(super) fn plan_title(text: &str) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with('#'))
        .or_else(|| text.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("Plan");
    let title = line.trim_start_matches('#').replace("**", "").trim().trim_end_matches(':').to_string();
    let title = if title.is_empty() { "Plan".to_string() } else { title };
    if title.chars().count() > 60 {
        format!("{}…", title.chars().take(59).collect::<String>().trim_end())
    } else {
        title
    }
}

/// For agents whose plans belong in drafts: a reply that is a plan written out in chat is saved
/// as a plan draft instead, and the reply becomes a pointer to it. Anything else passes through.
#[allow(clippy::too_many_arguments)]
pub(super) async fn keep_plan_in_draft(
    conn: &mut PoolConnection<Postgres>,
    s3: Option<&nomi_storage::S3Config>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    locale: Locale,
    wrote_plan: bool,
    reply: String,
) -> String {
    if wrote_plan || !agent.keeps_plans_in_drafts() || !looks_like_plan(&reply) {
        return reply;
    }
    let input = serde_json::json!({ "title": plan_title(&reply), "content": reply });
    match write_agent_plan(conn, s3, mqtt.map(|(p, _)| p), session_id, agent_session_id, user_id, agent.display_name().as_ref(), &input).await {
        Ok(_) => locale.t("engine.plan_draft"),
        Err(e) => {
            tracing::warn!(error = %e, "couldn't save a written-out plan as a draft; posting it as text");
            reply
        }
    }
}

/// The latest plan drafted in this chat, as the agent should read it.
pub(crate) async fn read_latest_plan(conn: &mut PoolConnection<Postgres>, s3: Option<&nomi_storage::S3Config>, session_id: Uuid) -> String {
    let row: Option<(String, i32, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT title, version, content, content_s3_key FROM agent_plans WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(&mut **conn)
    .await
    .ok()
    .flatten();
    let Some((title, version, content, key)) = row else {
        return "No plan has been drafted in this chat yet. Work from the conversation and the task you were given.".to_string();
    };
    let content = match (content, key, s3) {
        (Some(inline), _, _) => Some(inline),
        (None, Some(key), Some(s3)) => s3.get_object(&key).await.ok().flatten(),
        _ => None,
    };
    match content {
        Some(content) => format!("Plan \"{title}\" (version {version}):\n\n{content}"),
        None => format!("The plan \"{title}\" (version {version}) couldn't be loaded right now."),
    }
}

pub(super) fn parse_todo_items(input: &serde_json::Value) -> Result<Vec<crate::content_block::TodoItem>, String> {
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

pub(super) fn todo_list_display_text(items: &[crate::content_block::TodoItem]) -> String {
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

/// Inserts a new plan version (S3-backed when `s3` is configured and the upload succeeds, inline
/// in Postgres otherwise — storage never fails just because S3 is unavailable or erroring) and
/// posts a new chat message for it. Unlike `upsert_todo_list`, this always creates a NEW message
/// per call (a version history, not an in-place edit) — matching what the plan design calls for.
/// The posted message's plain-text `content` carries the FULL plan body, not a summary, since
/// that's what keeps the plan in the LLM's own context on later turns via `fetch_recent_messages`.
pub(super) async fn write_agent_plan(
    conn: &mut PoolConnection<Postgres>,
    s3: Option<&nomi_storage::S3Config>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    agent_display_name: &str,
    input: &serde_json::Value,
) -> Result<String, String> {
    let title = input.get("title").and_then(|v| v.as_str()).ok_or("title is required")?.to_string();
    let content = input.get("content").and_then(|v| v.as_str()).ok_or("content is required")?.to_string();
    if content.trim().is_empty() {
        return Err("content must not be empty".to_string());
    }

    let version: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM agent_plans WHERE agent_session_id = $1")
        .bind(agent_session_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    let s3_key = format!("plans/{agent_session_id}/{version}.md");
    let stored_in_s3 = match s3 {
        Some(s3) => s3.put_object(&s3_key, &content, "text/markdown").await.is_ok(),
        None => false,
    };

    let plan_id: Uuid = if stored_in_s3 {
        sqlx::query_scalar(
            "INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content_s3_key, version) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(session_id)
        .bind(agent_session_id)
        .bind(user_id)
        .bind(&title)
        .bind(&s3_key)
        .bind(version)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_scalar(
            "INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(session_id)
        .bind(agent_session_id)
        .bind(user_id)
        .bind(&title)
        .bind(&content)
        .bind(version)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    };

    let block = crate::content_block::ContentBlock::Plan { plan_id, agent_session_id, title: title.clone(), version };
    let content_blocks = serde_json::json!([block]);
    let message_text = format!("📋 {title}\n\n{content}");

    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) VALUES ($1, NULL, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(&message_text)
    .bind(&content_blocks)
    .bind(agent_display_name)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(publisher) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
    }

    Ok(format!("plan v{version} saved"))
}

/// Finds the most recent todo-list message for this agent session (tracked via
/// `agent_sessions.state->>'todo_message_id'`) and updates it in place; inserts a fresh message
/// and records its id into `state` the first time this agent session ever calls update_todos.
/// Unlike every other block-producing tool, this posts/updates the message itself rather than
/// returning a block for engine.rs's generic post-at-the-bottom-of-the-loop path, since that
/// path only ever inserts — it has no notion of "update this existing row instead".
pub(super) async fn upsert_todo_list(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_display_name: &str,
    conversation_so_far: &[LlmMessage],
    items: Vec<crate::content_block::TodoItem>,
) -> Result<String, String> {
    let display_text = todo_list_display_text(&items);
    let block = crate::content_block::ContentBlock::TodoList { items };
    let content_blocks = serde_json::json!([block]);

    let tracked: Option<Option<Uuid>> = sqlx::query_scalar(
        "SELECT (state->>'todo_message_id')::uuid FROM agent_sessions WHERE id = $1",
    )
    .bind(agent_session_id)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;
    let existing_id = match tracked {
        Some(tracked) => tracked,
        // Delegated and default-agent runs have no agent_sessions row (their agent_session_id is
        // the chat's own id): the list is this run's, so it's the one it already posted, if any.
        None if posted_todos_this_run(conversation_so_far) => sqlx::query_scalar(
            "SELECT id FROM messages WHERE session_id = $1 AND agent_display_name = $2 \
             AND content_blocks @> '[{\"kind\": \"todo_list\"}]' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(session_id)
        .bind(agent_display_name)
        .fetch_optional(&mut **conn)
        .await
        .map_err(|e| e.to_string())?,
        None => None,
    };

    match existing_id {
        Some(message_id) => {
            sqlx::query("UPDATE messages SET content = $1, content_blocks = $2 WHERE id = $3")
                .bind(&display_text)
                .bind(&content_blocks)
                .bind(message_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(publisher) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::MessageUpdated { message_id }).await;
            }
        }
        None => {
            let message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) VALUES ($1, NULL, $2, $3, $4) RETURNING id",
            )
            .bind(session_id)
            .bind(&display_text)
            .bind(&content_blocks)
            .bind(agent_display_name)
            .fetch_one(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query("UPDATE agent_sessions SET state = state || jsonb_build_object('todo_message_id', $1::text) WHERE id = $2")
                .bind(message_id.to_string())
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(publisher) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
            }
        }
    }

    Ok("todo list updated".to_string())
}

/// Whether an earlier update_todos call in this run (its tool calls are only in the run's own
/// conversation, never in the chat history it started from) succeeded.
pub(super) fn posted_todos_this_run(conversation: &[LlmMessage]) -> bool {
    let calls: Vec<&String> = conversation
        .iter()
        .flat_map(|m| m.content.iter())
        .filter_map(|b| match b {
            ContentBlock::ToolUse { id, name, .. } if name.as_str() == UPDATE_TODOS_TOOL_NAME => Some(id),
            _ => None,
        })
        .collect();
    conversation.iter().flat_map(|m| m.content.iter()).any(|b| {
        matches!(b, ContentBlock::ToolResult { tool_use_id, is_error: false, .. } if calls.contains(&tool_use_id))
    })
}
