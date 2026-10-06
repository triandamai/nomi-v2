use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use nomi_embedding::EmbeddingProvider;
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::error::TurnError;
use crate::locale::Locale;
use crate::memory;
use crate::permissions;
use crate::registry::AgentRegistry;
use crate::subagent::SubAgent;

const MAX_TOOL_TURNS: u32 = 10;

/// Posted when the model hit its output limit before writing any answer.
/// Asked once when a model ends its turn having only thought, with no answer written.
const ANSWER_NOW: &str = "You haven't written a reply yet. Write your answer to the user now.";
// When even that produces nothing, the reply is `engine.no_answer` (or `engine.cut_off` when the
// model ran out of room), so the chat never ends on a silent turn; a plan moved into a draft is
// answered with `engine.plan_draft`. All three in the person's language (nomi-i18n).

/// How agents should think when reasoning is on. Users read the thinking in chat.
pub const REASONING_STYLE: &str = "Your thinking is shown to the person, so keep it brief and on point: at \
     most 3 short bullet points, about 60 words in all, covering what they need, what you'll do, and any \
     catch. Think in the same language you reply in. Don't restate their message, weigh every option, \
     repeat yourself, or draft the reply in your thinking; once you know the next step, stop thinking \
     and act.";
pub const COMPLETE_TASK_TOOL_NAME: &str = "complete_task";
pub const DELEGATE_TOOL_NAME: &str = "delegate_to_agent";
pub const SHOW_TABLE_TOOL_NAME: &str = "show_table";
pub const UPDATE_TODOS_TOOL_NAME: &str = "update_todos";
pub const WRITE_PLAN_TOOL_NAME: &str = "write_plan";
pub const CREATE_REMINDER_TOOL_NAME: &str = "create_reminder";
pub const LIST_REMINDERS_TOOL_NAME: &str = "list_reminders";
pub const CANCEL_REMINDER_TOOL_NAME: &str = "cancel_reminder";

const PHASE_THINKING: &str = "thinking";
const PHASE_CALLING_TOOL: &str = "calling_tool";
const PHASE_WRITING_REPLY: &str = "writing_reply";
const PHASE_WAITING: &str = "waiting";

/// Everything the model wrote in one response, in order. Providers can split a reply across
/// several text blocks; keeping only the first cut answers short.
pub fn reply_text_of(content: &[ContentBlock]) -> String {
    let joined: String = content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    joined.trim().to_string()
}

#[cfg(test)]
mod reply_text_tests {
    use super::reply_text_of;
    use nomi_llm::ContentBlock;

    #[test]
    fn keeps_every_text_block_in_order() {
        let content = vec![
            ContentBlock::Text { text: "Here is your plan:\n\n".into() },
            ContentBlock::Thinking { text: "hidden".into(), signature: None },
            ContentBlock::Text { text: "1. Pack\n2. Go".into() },
        ];
        assert_eq!(reply_text_of(&content), "Here is your plan:\n\n1. Pack\n2. Go");
        assert_eq!(reply_text_of(&[]), "");
    }

    #[test]
    fn a_written_out_itinerary_reads_as_a_plan_and_a_short_answer_does_not() {
        let itinerary = "## Bogor day trip\n\n### Morning\n- 08:00 Arrive in Bogor\n- 08:30 Botanical Gardens, walk the paths and see the palace\n- 11:00 Coffee nearby\n### Afternoon\n- 12:30 Lunch: Nasi Timbel at a Sundanese place\n- 14:00 Zoology Museum\n- 16:00 Market walk or a tea house\n### Evening\n- 18:00 Dinner on Suryakencana Street\n- 20:00 Head home";
        assert!(super::looks_like_plan(itinerary));
        assert_eq!(super::plan_title(itinerary), "Bogor day trip");
        assert!(!super::looks_like_plan("Sure! Bogor is lovely in October.\n- Bring an umbrella\n- Go early"));
    }
}

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
fn plan_title(text: &str) -> String {
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
async fn keep_plan_in_draft(
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

fn delegate_tool_definition(targets: &[String]) -> ToolDefinition {
    ToolDefinition {
        name: DELEGATE_TOOL_NAME.to_string(),
        description: format!(
            "Hand the user's request to a specialist. The specialist sees this conversation, takes \
             it over right away and answers the user itself, so don't write a reply of your own \
             when you hand off. Use this only when the request genuinely needs a specialist; \
             answer anything else yourself. Available specialists: {}.",
            targets.join(", "),
        ),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "target_agent": {"type": "string", "enum": targets, "description": "Which specialist to delegate to"},
                "task": {"type": "string", "description": "What the user wants from the specialist, with any details from this conversation it needs"}
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

fn write_plan_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: WRITE_PLAN_TOOL_NAME.to_string(),
        description: "Write a new version of your plan for this task, before or during a \
                       multi-step build. Call this again whenever the plan changes significantly \
                       — each call is a new, separately viewable version, not an edit to the last \
                       one.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Short label for this plan version"},
                "content": {"type": "string", "description": "The plan, in markdown"}
            },
            "required": ["title", "content"]
        }),
    }
}

fn create_reminder_tool_definition(targets: &[String]) -> ToolDefinition {
    ToolDefinition {
        name: CREATE_REMINDER_TOOL_NAME.to_string(),
        description: "Schedule a reminder to fire at a specific future time. Resolve any \
                       relative time the user gives (\"tomorrow\", \"in an hour\") to an \
                       absolute ISO 8601 datetime yourself, using the current date/time and the \
                       user's timezone given in your system prompt.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "run_at": {"type": "string", "description": "Absolute ISO 8601 datetime, e.g. 2026-09-21T11:00:00-04:00"},
                "label": {"type": "string", "description": "Short human label, e.g. \"take a bath\""},
                "prompt": {"type": "string", "description": "The instruction the target agent will act on when this fires"},
                "target_agent": {"type": "string", "enum": targets, "description": "Which agent takes this job when it fires"},
                "recurrence": {"type": "string", "enum": ["daily", "weekly", "monthly"], "description": "Omit for a one-time reminder"},
                "recurrence_weekday": {"type": "integer", "minimum": 0, "maximum": 6, "description": "Required when recurrence is weekly (0 = Sunday)"},
                "recurrence_day_of_month": {"type": "integer", "minimum": 1, "maximum": 31, "description": "Required when recurrence is monthly"}
            },
            "required": ["run_at", "label", "prompt", "target_agent"]
        }),
    }
}

fn list_reminders_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: LIST_REMINDERS_TOOL_NAME.to_string(),
        description: "List your active reminders.".to_string(),
        input_schema: serde_json::json!({"type": "object", "properties": {}}),
    }
}

fn cancel_reminder_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: CANCEL_REMINDER_TOOL_NAME.to_string(),
        description: "Cancel an active reminder by id. Call list_reminders first if you don't \
                       already know the id.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "reminder_id": {"type": "string", "description": "The reminder's id, from a prior list_reminders call"}
            },
            "required": ["reminder_id"]
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply { text: String, memory_ids_used: Vec<Uuid>, input_tokens: u32, output_tokens: u32 },
    Completed { status: String, summary: String },
    AwaitingApproval { message_id: Uuid },
    /// The agent handed the conversation to `target_agent`, which answers the user in this same
    /// turn. Nothing of this agent's is posted: the specialist speaks for itself.
    HandOff { target_agent: String, task: String },
    /// The user stopped this agent mid-turn (see `crate::stop`). Nothing more is posted; whoever
    /// asked for the stop has already told the user.
    Cancelled,
}

/// Tool result for a call the user refused. Says plainly not to retry: models otherwise tend to
/// try the same call again, which only puts the same approval card in front of the user again.
const USER_DENIED_RESULT: &str =
    "The user denied this action. Do not try it again; tell them it wasn't done and ask how they'd like to proceed.";

/// Tools that are never permission-gated: engine-level bookkeeping (complete_task,
/// delegate_to_agent, show_table, update_todos) — none of these touch anything a user would
/// want to approve/deny.
fn is_gateable(tool_name: &str) -> bool {
    tool_name != COMPLETE_TASK_TOOL_NAME
        && tool_name != DELEGATE_TOOL_NAME
        && tool_name != SHOW_TABLE_TOOL_NAME
        && tool_name != UPDATE_TODOS_TOOL_NAME
        && tool_name != WRITE_PLAN_TOOL_NAME
        && tool_name != CREATE_REMINDER_TOOL_NAME
        && tool_name != LIST_REMINDERS_TOOL_NAME
        && tool_name != CANCEL_REMINDER_TOOL_NAME
        // Record tools only touch the calling agent's own private records.
        && !crate::records::RECORD_TOOL_NAMES.contains(&tool_name)
}

fn describe_pending_action(tool_name: &str, input: &serde_json::Value, locale: Locale) -> String {
    let path = input.get("path").and_then(|v| v.as_str());
    match (tool_name, path) {
        ("delete_file", Some(path)) => locale.tf("engine.approve_delete", &[("path", path)]),
        ("write_file", Some(path)) => locale.tf("engine.approve_overwrite", &[("path", path)]),
        (other, Some(path)) => locale.tf("engine.approve_run_on", &[("tool", other), ("path", path)]),
        (other, None) => locale.tf("engine.approve_run", &[("tool", other)]),
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
    s3: Option<&nomi_storage::S3Config>,
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
        let targets = registry.delegatable_agent_types(agent.agent_type().as_ref());
        if !targets.is_empty() {
            tools.push(delegate_tool_definition(&targets));
        }
    }
    tools.push(show_table_tool_definition());
    if agent.supports_todos() {
        tools.push(update_todos_tool_definition());
    }
    if agent.supports_plans() {
        tools.push(write_plan_tool_definition());
    }
    if agent.uses_records() {
        tools.extend(crate::records::record_tool_definitions());
    }
    if agent.supports_reminders() {
        let targets = registry.reminder_target_agent_types();
        tools.push(create_reminder_tool_definition(&targets));
        tools.push(list_reminders_tool_definition());
        tools.push(cancel_reminder_tool_definition());
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
        let mut prompt = format!(
            "{}\n\nWhat you remember about this person that matters here (use it only where it helps):\n",
            agent.system_prompt()
        );
        for m in &memories {
            prompt.push_str(&format!("- ({}) {}\n", m.kind, m.content));
        }
        prompt
    };

    // Working memory: the older part of this chat, folded into a running summary by the memory
    // worker (only the latest messages are sent in full).
    let summary: Option<String> = sqlx::query_scalar("SELECT summary FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_optional(&mut **conn)
        .await?
        .flatten();
    let system_prompt = match summary.filter(|s| !s.trim().is_empty()) {
        Some(summary) => format!("{system_prompt}\n\nEarlier in this chat (summary):\n{}", summary.trim()),
        None => system_prompt,
    };

    // Every agent answers in the person's language.
    let locale = crate::locale::user_locale(conn, user_id).await;
    let system_prompt = format!("{system_prompt}\n\n{}", locale.reply_instruction());

    let system_prompt = if agent.uses_personality() {
        match crate::personality::get_current_personality(conn, user_id).await {
            Some(p) => format!("{system_prompt}\n\nAdopt this personality in your replies: {p}"),
            None => system_prompt,
        }
    } else {
        system_prompt
    };

    let system_prompt = if agent.supports_reminders() || agent.wants_current_time() {
        let timezone_name = crate::scheduled_jobs::get_user_timezone(conn, user_id).await;
        let tz: chrono_tz::Tz = timezone_name.parse().unwrap_or(chrono_tz::UTC);
        let now = chrono::Utc::now().with_timezone(&tz);
        format!(
            "{system_prompt}\n\nCurrent date/time: {} ({timezone_name}). When scheduling a \
             reminder, resolve relative times against this.",
            now.to_rfc3339(),
        )
    } else {
        system_prompt
    };

    let started_at = crate::stop::database_clock(conn).await?;

    // The chat's thinking level (picked in the composer); delegated turns share their chat's.
    let thinking_level: String = sqlx::query_scalar("SELECT thinking_level FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_optional(&mut **conn)
        .await?
        .unwrap_or_else(|| "medium".to_string());
    let reasoning_effort = match thinking_level.as_str() {
        "low" => nomi_llm::ReasoningEffort::Low,
        "high" => nomi_llm::ReasoningEffort::High,
        _ => nomi_llm::ReasoningEffort::Medium,
    };
    // Thinking is shown in chat, so keep it to the point rather than a running monologue.
    let system_prompt = if thinking_level != "off" {
        format!("{system_prompt}\n\n{REASONING_STYLE}")
    } else {
        system_prompt
    };
    let agent_type = agent.agent_type();

    // Whether this run already saved a plan draft, and already asked once for a missing answer.
    let mut wrote_plan = false;
    let mut asked_for_answer = false;

    for _ in 0..MAX_TOOL_TURNS {
        if crate::stop::is_stop_requested(conn, user_id, session_id, &agent_type, started_at).await {
            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
            return Ok(LoopOutcome::Cancelled);
        }

        update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_THINKING, None).await;

        let request = LlmRequest {
            system: Some(system_prompt.clone()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
            enable_reasoning: thinking_level != "off",
            reasoning_effort,
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

        // The LLM call is the slow step, so this is where a stop most often lands: drop the
        // response instead of replying or running its tools.
        if crate::stop::is_stop_requested(conn, user_id, session_id, &agent_type, started_at).await {
            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
            return Ok(LoopOutcome::Cancelled);
        }

        messages.push(LlmMessage { role: LlmRole::Assistant, content: response.content.clone() });

        // Reasoning is shown for every agent, unlike the tool-call "💭" commentary below (which
        // stays gated behind `surfaces_activity()`) — a provider's own thinking trace is worth
        // seeing regardless of whether this agent normally narrates its tool calls.
        for block in &response.content {
            if let ContentBlock::Thinking { text, .. } = block {
                if !text.trim().is_empty() {
                    let reasoning = crate::content_block::ContentBlock::Reasoning { text: text.trim().to_string() };
                    post_activity_message(
                        conn,
                        mqtt.map(|(p, _)| p),
                        session_id,
                        agent.display_name().as_ref(),
                        &format!("🧠 {}", text.trim()),
                        Some(&reasoning),
                    )
                    .await;
                }
            }
        }

        if response.stop_reason != StopReason::ToolUse {
            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WRITING_REPLY, None).await;

            let input_tokens = response.input_tokens;
            let output_tokens = response.output_tokens;
            let cut_off = response.stop_reason == StopReason::MaxTokens;
            let reply_text = reply_text_of(&response.content);
            // Only thinking, no answer: ask once for the answer itself before giving up.
            if reply_text.is_empty() && !cut_off && !asked_for_answer {
                asked_for_answer = true;
                if response.content.is_empty() {
                    messages.pop();
                }
                messages.push(LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: ANSWER_NOW.to_string() }] });
                continue;
            }
            // The model ran out of room before answering (a half-written plan or tool call is
            // dropped): say so instead of leaving only the thinking in chat.
            let reply_text = match (reply_text.is_empty(), cut_off) {
                (true, true) => locale.t("engine.cut_off"),
                (true, false) => locale.t("engine.no_answer"),
                _ => reply_text,
            };
            let reply_text = keep_plan_in_draft(conn, s3, mqtt, agent, session_id, agent_session_id, user_id, locale, wrote_plan, reply_text).await;

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
                memory::extract_and_store_memory_in(conn, provider, embedding_provider, user_id, Some(session_id), &last_user_text, &reply_text).await;
            }

            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;

            return Ok(LoopOutcome::Reply {
                text: reply_text,
                memory_ids_used: memories.iter().map(|m| m.id).collect(),
                input_tokens,
                output_tokens,
            });
        }

        let pending_tool_use_blocks: Vec<ContentBlock> =
            response.content.iter().filter(|b| matches!(b, ContentBlock::ToolUse { .. })).cloned().collect();

        // Text written alongside complete_task is the agent's answer, not commentary: it becomes
        // the reply instead of a "💭" line (or, for quieter agents, being dropped).
        let written = reply_text_of(&response.content);
        let finishing = pending_tool_use_blocks.iter().any(
            |b| matches!(b, ContentBlock::ToolUse { name, .. } if name.as_str() == COMPLETE_TASK_TOOL_NAME || name.as_str() == DELEGATE_TOOL_NAME),
        );
        if agent.surfaces_activity() && !finishing && !written.is_empty() {
            post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, agent.display_name().as_ref(), &format!("💭 {written}"), None).await;
        }

        wrote_plan |= pending_tool_use_blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolUse { name, .. } if name.as_str() == WRITE_PLAN_TOOL_NAME));

        match resolve_tool_batch(conn, mqtt, s3, registry, agent, session_id, agent_session_id, user_id, &pending_tool_use_blocks, &messages, &[]).await? {
            ToolBatchOutcome::AwaitingApproval { message_id } => return Ok(LoopOutcome::AwaitingApproval { message_id }),
            ToolBatchOutcome::Completed { status, summary } => {
                let summary = if written.is_empty() { summary } else { written };
                let summary = keep_plan_in_draft(conn, s3, mqtt, agent, session_id, agent_session_id, user_id, locale, wrote_plan, summary).await;
                return Ok(LoopOutcome::Completed { status, summary });
            }
            ToolBatchOutcome::HandOff { target_agent, task } => {
                update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
                return Ok(LoopOutcome::HandOff { target_agent, task });
            }
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
    HandOff { target_agent: String, task: String },
}

/// Resolves every ToolUse block in `tool_use_blocks`, in order. `already_decided` is the
/// ACCUMULATING set of `(tool_use_id, approved)` decisions for blocks whose approval was resolved
/// externally across earlier resumes of this same paused batch (each a user clicking Approve/Deny
/// on a card) — every block in that set skips the permission check and uses its recorded decision
/// directly; every other block still goes through the normal check-permission-then-execute-or-pause
/// path, which may itself pause again on a *different* block — handled identically to the very
/// first pause (see nomi-turn's resume path, which calls this same function again with the grown
/// accumulator when that happens). Accumulating (rather than tracking a single most-recent
/// decision) is what lets a batch with two-or-more gated tool calls converge: without it, each
/// resume forgets the previous card's decision and re-pauses on an already-decided block forever.
#[allow(clippy::too_many_arguments)]
pub async fn resolve_tool_batch(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    tool_use_blocks: &[ContentBlock],
    conversation_so_far: &[LlmMessage],
    already_decided: &[(String, bool)],
) -> Result<ToolBatchOutcome, TurnError> {
    let locale = crate::locale::user_locale(conn, user_id).await;
    for block in tool_use_blocks {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            if already_decided.iter().any(|(decided_id, _)| decided_id == id) {
                continue;
            }
            if !is_gateable(name) || !agent.tool_needs_approval(name) {
                continue;
            }
            let decision = permissions::check_tool_permission_in_session(conn, user_id, session_id, name, input).await;
            if !matches!(decision, permissions::PermissionDecision::Ask) {
                continue;
            }

            let description = agent.describe_action(name, input, locale).unwrap_or_else(|| describe_pending_action(name, input, locale));
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
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) VALUES ($1, NULL, $2, $3, $4) RETURNING id",
            )
            .bind(session_id)
            .bind(format!("⏳ {description}"))
            .bind(&content_blocks)
            .bind(agent.display_name().as_ref())
            .fetch_one(&mut **conn)
            .await
            .ok();

            let Some(message_id) = message_id else {
                return Err(TurnError::ToolLoopExceeded);
            };

            if let Some((publisher, _)) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
            }

            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;

            // The FULL batch must survive into the next resume, not just the blocks from this
            // point on: this pre-scan loop only checks permissions, it never executes anything —
            // execution only happens in the pass below, and only once the pre-scan clears with
            // zero remaining "Ask" blocks. So every block before this one has been scanned but
            // NOT yet executed; dropping them here would silently lose their results forever
            // (and leave the eventual ToolResult message missing entries for ToolUse blocks the
            // assistant turn actually declared).
            let state_patch = serde_json::json!({
                "paused_for_approval": true,
                "pending_approval_message_id": message_id.to_string(),
                "pending_tool_use_id": id,
                "tool_use_blocks": tool_use_blocks,
                "messages": conversation_so_far,
                "decided_tool_use_ids": already_decided,
            });
            let patched = sqlx::query("UPDATE agent_sessions SET state = state || $1 WHERE id = $2")
                .bind(&state_patch)
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await?
                .rows_affected();

            // The default agent and delegated turns run with a sentinel agent_session_id (the
            // chat's own session_id) that has no agent_sessions row, so the patch above lands
            // nowhere — the approval card would exist but could never be resolved ("this action
            // is no longer pending"). Give the paused state a row of its own. Status
            // 'awaiting_approval' (not 'active') keeps it clear of the one-active-agent-per-
            // speaker index, which the chat's live agent session may already hold.
            if patched == 0 {
                let sender_channel_identity_id: Uuid =
                    sqlx::query_scalar("SELECT id FROM channel_identities WHERE user_id = $1 ORDER BY created_at LIMIT 1")
                        .bind(user_id)
                        .fetch_one(&mut **conn)
                        .await?;
                sqlx::query(
                    "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, state) \
                     VALUES ($1, $2, $3, 'awaiting_approval', $4)",
                )
                .bind(session_id)
                .bind(sender_channel_identity_id)
                .bind(agent.agent_type().as_ref())
                .bind(&state_patch)
                .execute(&mut **conn)
                .await?;
            }

            return Ok(ToolBatchOutcome::AwaitingApproval { message_id });
        }
    }

    let mut tool_results = Vec::new();
    for block in tool_use_blocks {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_CALLING_TOOL, Some(name)).await;

            let (result_text, is_error, rich_block) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false, None)
            } else if name.as_str() == DELEGATE_TOOL_NAME {
                let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
                let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
                let target = registry.find(target_agent);
                let rejection = target.as_ref().and_then(|t| t.validate_delegation_task(task).err());
                // A specialist that answers right away takes the conversation over in this turn.
                // Work split across several specialists at once still runs in the background.
                let hand_offs_in_batch = tool_use_blocks
                    .iter()
                    .filter(|b| matches!(b, ContentBlock::ToolUse { name, .. } if name.as_str() == DELEGATE_TOOL_NAME))
                    .count();
                if let (Some(target), None, 1) = (&target, &rejection, hand_offs_in_batch) {
                    if !target.works_in_background() {
                        log_tool_call(conn, session_id, agent_session_id, agent.agent_type().as_ref(), name, input, "handed off", false).await;
                        return Ok(ToolBatchOutcome::HandOff { target_agent: target_agent.to_string(), task: task.to_string() });
                    }
                }
                let (text, err) = match rejection {
                    Some(reason) => (reason, true),
                    None => match crate::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type().as_ref(), target_agent, task, user_id).await {
                        Ok(ack) => (ack, false),
                        Err(err) => (err, true),
                    },
                };
                (text, err, None)
            } else if name.as_str() == SHOW_TABLE_TOOL_NAME {
                match parse_table_input(input) {
                    Ok((variant, columns, rows)) => {
                        let text = table_display_text(&variant, rows.len(), locale);
                        let block = crate::content_block::ContentBlock::Table { variant, columns, rows };
                        (text, false, Some(block))
                    }
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == UPDATE_TODOS_TOOL_NAME {
                match parse_todo_items(input) {
                    Ok(items) => match upsert_todo_list(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, agent.display_name().as_ref(), items).await {
                        Ok(text) => (text, false, None),
                        Err(err) => (err, true, None),
                    },
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == WRITE_PLAN_TOOL_NAME && agent.supports_plans() {
                match write_agent_plan(conn, s3, mqtt.map(|(p, _)| p), session_id, agent_session_id, user_id, agent.display_name().as_ref(), input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if agent.uses_records() && crate::records::RECORD_TOOL_NAMES.contains(&name.as_str()) {
                match crate::records::execute(conn, agent.agent_type().as_ref(), user_id, name, input).await {
                    Some(Ok(text)) => (text, false, None),
                    Some(Err(err)) => (err, true, None),
                    None => (format!("unknown tool: {name}"), true, None),
                }
            } else if name.as_str() == CREATE_REMINDER_TOOL_NAME && agent.supports_reminders() {
                match crate::scheduled_jobs::create_reminder(conn, session_id, user_id, agent.agent_type().as_ref(), input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == LIST_REMINDERS_TOOL_NAME && agent.supports_reminders() {
                match crate::scheduled_jobs::list_reminders(conn, user_id).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == CANCEL_REMINDER_TOOL_NAME && agent.supports_reminders() {
                match crate::scheduled_jobs::cancel_reminder(conn, user_id, input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if let Some((_, approved)) = already_decided.iter().find(|(decided_id, _)| decided_id == id) {
                if *approved {
                    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(outcome) => (outcome.display_text, false, outcome.block),
                        Err(err) => (describe_tool_error(name, input, &err, locale), true, None),
                    }
                } else {
                    (USER_DENIED_RESULT.to_string(), true, None)
                }
            } else if matches!(
                permissions::check_tool_permission_in_session(conn, user_id, session_id, name, input).await,
                permissions::PermissionDecision::Deny
            ) {
                (USER_DENIED_RESULT.to_string(), true, None)
            } else {
                match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                    Ok(outcome) => (outcome.display_text, false, outcome.block),
                    Err(err) => (describe_tool_error(name, input, &err, locale), true, None),
                }
            };

            log_tool_call(conn, session_id, agent_session_id, agent.agent_type().as_ref(), name, input, &result_text, is_error).await;

            // A tool that returns a block (a table, a file change, a Connect card) means it to be
            // seen, whether or not the agent narrates the rest of its tool calls.
            let should_post = name.as_str() == SHOW_TABLE_TOOL_NAME
                || rich_block.is_some()
                || (agent.surfaces_activity()
                    && name.as_str() != COMPLETE_TASK_TOOL_NAME
                    && name.as_str() != DELEGATE_TOOL_NAME
                    && name.as_str() != UPDATE_TODOS_TOOL_NAME
                    && name.as_str() != WRITE_PLAN_TOOL_NAME
                    && name.as_str() != CREATE_REMINDER_TOOL_NAME
                    && name.as_str() != LIST_REMINDERS_TOOL_NAME
                    && name.as_str() != CANCEL_REMINDER_TOOL_NAME
                    && !crate::records::RECORD_TOOL_NAMES.contains(&name.as_str()));
            if should_post {
                post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, agent.display_name().as_ref(), &result_text, rich_block.as_ref()).await;
            }

            if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
                let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
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
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, (SELECT id FROM agent_sessions WHERE id = $2), $3, 'ToolCalled', $4)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(agent_type)
    .bind(serde_json::json!({"tool_name": tool_name, "input": input, "result": result, "is_error": is_error}))
    .execute(&mut **conn)
    .await;
}

/// Best-effort (never fails the turn, matches `post_activity_message`'s convention): updates
/// `agent_sessions.current_phase`/`current_phase_detail` and, when `mqtt` is available, pushes
/// the same change live over `StreamEnvelope::AgentPhaseChanged`. Silently a no-op for the
/// default agent's turns, where `agent_session_id` is the `session_id` sentinel (see
/// `nomi_turn::run_locked_turn`'s call site comment) — there is no real `agent_sessions` row
/// with that id, so the UPDATE affects zero rows and nothing is published.
async fn update_agent_phase(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    phase: &str,
    detail: Option<&str>,
) {
    let result = sqlx::query("UPDATE agent_sessions SET current_phase = $1, current_phase_detail = $2 WHERE id = $3")
        .bind(phase)
        .bind(detail)
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await;

    if let (Ok(outcome), Some(publisher)) = (&result, mqtt) {
        if outcome.rows_affected() > 0 {
            let envelope = StreamEnvelope::AgentPhaseChanged {
                agent_session_id,
                phase: phase.to_string(),
                detail: detail.map(|d| d.to_string()),
            };
            let _ = publisher.publish(session_id, &envelope).await;
        }
    }
}

/// Inserts an activity message (an agent's "thought", or a description of a tool call it just
/// made) so a user watching a long-running build sees it appear like any other chat message —
/// see `SubAgent::surfaces_activity`. Best-effort: never fails the turn.
async fn post_activity_message(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_display_name: &str,
    content: &str,
    block: Option<&crate::content_block::ContentBlock>,
) -> Option<Uuid> {
    let content_blocks = block.map(|b| serde_json::json!([b]));
    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) VALUES ($1, NULL, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(content)
    .bind(&content_blocks)
    .bind(agent_display_name)
    .fetch_one(&mut **conn)
    .await
    .ok();

    if let (Some(id), Some(publisher)) = (message_id, mqtt) {
        let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id: id }).await;
    }
    message_id
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

/// Inserts a new plan version (S3-backed when `s3` is configured and the upload succeeds, inline
/// in Postgres otherwise — storage never fails just because S3 is unavailable or erroring) and
/// posts a new chat message for it. Unlike `upsert_todo_list`, this always creates a NEW message
/// per call (a version history, not an in-place edit) — matching what the plan design calls for.
/// The posted message's plain-text `content` carries the FULL plan body, not a summary, since
/// that's what keeps the plan in the LLM's own context on later turns via `fetch_recent_messages`.
async fn write_agent_plan(
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
async fn upsert_todo_list(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_display_name: &str,
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

fn table_display_text(variant: &crate::content_block::TableVariant, row_count: usize, locale: Locale) -> String {
    use crate::content_block::TableVariant;
    let count = row_count.to_string();
    match variant {
        TableVariant::Data => locale.tf("engine.table_data", &[("count", &count)]),
        TableVariant::Comparison => locale.tf("engine.table_comparison", &[("count", &count)]),
    }
}

/// Templated, non-LLM description of a failed tool call — kept generic in the engine (unlike a
/// success's display_text, which each tool builds itself) since failures never carry a block and
/// the "⚠️ couldn't X — {reason}" shape is the same regardless of which crate the tool lives in.
fn describe_tool_error(tool_name: &str, input: &serde_json::Value, result: &str, locale: Locale) -> String {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("?");
    let args = [("path", path), ("reason", result), ("tool", tool_name)];
    let key = match tool_name {
        "write_file" => "engine.failed_write",
        "read_file" => "engine.failed_read",
        "delete_file" => "engine.failed_delete",
        "list_files" => "engine.failed_list",
        "create_project" => "engine.failed_project",
        _ => "engine.failed_tool",
    };
    locale.tf(key, &args)
}
