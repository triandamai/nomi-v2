use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::SUPERVISOR_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub mod stop;

pub const SUPERVISOR_AGENT_TYPE: &str = "supervisor";

/// How many recent delegations `list_recent_agent_activity` reports on.
const RECENT_ACTIVITY_LIMIT: i64 = 10;

/// `max_tokens` for the supervisor's one-shot, tool-free phrasing completions
/// (`phrase_delegation_started`) — a single short sentence,
/// never a multi-step reply, so a small budget is intentional, not an oversight.
const SUPERVISOR_PHRASING_MAX_TOKENS: u32 = 256;

pub fn list_recent_agent_activity_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_recent_agent_activity".to_string(),
        description: "List recent and currently active background delegations for this chat session.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

pub fn stop_agents_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "stop_agents".to_string(),
        description: "Stop agents that are working, at the user's request. Cancels their running and queued work \
                      and any approvals they are waiting on. Use scope \"this_chat\" for \"stop\" or \"cancel that\", \
                      \"everything\" for every agent in every chat, or \"agent\" with the agent's name or type \
                      (e.g. \"money\", \"coding\", \"planning\") to stop one agent."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "scope": {"type": "string", "enum": ["this_chat", "everything", "agent"]},
                "agent": {"type": "string", "description": "Required when scope is \"agent\"."}
            },
            "required": ["scope"]
        }),
    }
}

/// Maps what the model passed as `agent` to an agent type: a dynamic agent by name or intent
/// label, otherwise the lowercased text itself (the built-ins' types are their names).
async fn resolve_agent_type(conn: &mut PoolConnection<Postgres>, agent: &str) -> String {
    let wanted = agent.trim().to_lowercase();
    let dynamic: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM dynamic_agents WHERE is_active = true AND (lower(name) = $1 OR lower(intent_label) = $1) LIMIT 1",
    )
    .bind(&wanted)
    .fetch_optional(&mut **conn)
    .await
    .ok()
    .flatten();
    dynamic.map(|id| id.to_string()).unwrap_or_else(|| wanted.trim_end_matches(" agent").to_string())
}

async fn stop_agents_tool(conn: &mut PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid, input: &Value) -> Result<String, String> {
    use sqlx::Connection;

    let (target, target_name) = match input["scope"].as_str() {
        Some("this_chat") => (stop::StopTarget::ThisChat, None),
        Some("everything") => (stop::StopTarget::Everything, None),
        Some("agent") => {
            let name = input["agent"].as_str().filter(|s| !s.trim().is_empty()).ok_or("scope \"agent\" needs the agent's name")?;
            let agent_type = resolve_agent_type(conn, name).await;
            (stop::StopTarget::Agent { agent_type }, Some(name.to_string()))
        }
        _ => return Err("scope must be this_chat, everything or agent".to_string()),
    };

    let mut tx = (**conn).begin().await.map_err(|e| e.to_string())?;
    let report = stop::stop_agents(&mut tx, &stop::StopRequest { user_id, session_id, target: target.clone(), from_supervisor_turn: true })
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    let locale = nomi_agent_core::user_locale(conn, user_id).await;
    Ok(stop::describe(&report, &target, target_name.as_deref(), locale))
}

pub struct SupervisorAgent;

#[async_trait]
impl SubAgent for SupervisorAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(SUPERVISOR_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(SUPERVISOR_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![list_recent_agent_activity_tool_definition(), stop_agents_tool_definition()]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "list_recent_agent_activity" => list_recent_agent_activity(conn, session_id).await.map(nomi_agent_core::ToolOutcome::text),
            "stop_agents" => stop_agents_tool(conn, session_id, user_id, &input).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("supervisor has no tool named {other}")),
        }
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(SUPERVISOR_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed(
            "The user is asking what the other agents are doing, wants a status update, explicitly asks for a report across \
             multiple specialists, or wants to stop, cancel or call off one or all of the agents' work",
        )
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn is_delegation_target(&self) -> bool {
        false
    }
}

pub async fn list_recent_agent_activity(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<String, String> {
    let rows: Vec<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT target_agent_type, task, status, result, error FROM agent_delegations \
         WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(session_id)
    .bind(RECENT_ACTIVITY_LIMIT)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No background delegations for this session yet.".to_string());
    }

    let mut out = String::new();
    for (target, task, status, result, error) in rows {
        out.push_str(&format!("- {target} ({status}): asked to \"{task}\""));
        if let Some(r) = result {
            out.push_str(&format!(" — result: {r}"));
        }
        if let Some(e) = error {
            out.push_str(&format!(" — error: {e}"));
        }
        out.push('\n');
    }
    Ok(out)
}

/// Resolves what to call the user in a notification: display_name, falling back to username,
/// falling back to the login email — the same precedence the rest of the app uses wherever a
/// human-readable name is shown (see Sidebar.svelte's accountLabel on the frontend).
async fn get_user_identity(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Option<String> {
    let (display_name, username, email): (Option<String>, Option<String>, String) = sqlx::query_as(
        "SELECT up.display_name, up.username, wc.email \
         FROM web_credentials wc \
         LEFT JOIN user_profiles up ON up.user_id = wc.user_id \
         WHERE wc.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&mut **conn)
    .await
    .ok()
    .flatten()?;

    display_name
        .filter(|s| !s.trim().is_empty())
        .or_else(|| username.filter(|s| !s.trim().is_empty()))
        .or(Some(email))
}

/// Appends identity and personality context to a base system prompt, shared by every one-shot,
/// tool-free supervisor completion below (duplicated relative to run_agent_turn's own personality
/// injection — not shared with it — because this path intentionally doesn't go through
/// run_agent_turn's full tool loop; this is a deliberate, documented design decision, not an
/// oversight, see the design spec). Sharing between the functions in this file, unlike sharing
/// with run_agent_turn, is a plain within-crate dedup with no such tradeoff.
async fn personalize_system_prompt(conn: &mut PoolConnection<Postgres>, user_id: Uuid, base: String) -> String {
    let mut system = base;
    if let Some(identity) = get_user_identity(conn, user_id).await {
        system = format!("{system}\n\nYou're speaking with {identity}.");
    }
    if let Some(p) = nomi_agent_core::personality::get_current_personality(conn, user_id).await {
        system = format!("{system}\n\nAdopt this personality in your reply: {p}");
    }
    let locale = nomi_agent_core::user_locale(conn, user_id).await;
    format!("{system}\n\n{}", locale.reply_instruction())
}

/// One-shot, tool-free completion that tells the user work has just begun on a delegated task, in
/// the supervisor's voice, folding in the user's identity and personality. It fires at claim-time,
/// so the user sees something happen right away; the agent posts its own answer when done.
pub async fn phrase_delegation_started(
    provider: &dyn nomi_llm::LlmProvider,
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    target_agent_type: &str,
    task: &str,
) -> Result<String, String> {
    let base = format!(
        "{SUPERVISOR_SYSTEM_PROMPT}\n\nYou've just handed a task off to the {target_agent_type} agent. Tell the \
         user, in one short sentence, that you're on it and will update them here once it's done. Don't invent \
         details about what the agent will find — just that work has started. Keep it to one sentence no matter \
         how expressive your personality is; save the personality for the follow-up report.\n\n\
         What was asked: {task}"
    );
    let system = personalize_system_prompt(conn, user_id, base).await;
    let locale = nomi_agent_core::user_locale(conn, user_id).await;

    let request = nomi_llm::LlmRequest {
        system: Some(system),
        messages: vec![nomi_llm::LlmMessage {
            role: nomi_llm::LlmRole::User,
            content: vec![nomi_llm::ContentBlock::Text { text: "Let the user know you've started.".to_string() }],
        }],
        tools: vec![],
        max_tokens: SUPERVISOR_PHRASING_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let response = nomi_llm::complete(provider, request).await.map_err(|e| e.to_string())?;
    Ok(response
        .content
        .into_iter()
        .find_map(|block| match block {
            nomi_llm::ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_else(|| locale.tf("turn.working_with", &[("agent", target_agent_type)])))
}
