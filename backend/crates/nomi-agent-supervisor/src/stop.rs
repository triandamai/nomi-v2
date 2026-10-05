//! Stopping agents when the user asks ("stop", "stop the money agent", "stop all agents").
//!
//! Two ways in:
//! - `handle_stop_message`: a short, unambiguous stop command is handled the moment it arrives,
//!   before it would be queued. The turn worker runs one turn at a time, so a queued "stop"
//!   would only be read after the very agent it was meant to stop had finished.
//! - The supervisor agent's `stop_agents` tool, for requests phrased loosely enough to need the
//!   model ("can you call off the coding thing you started?").
//!
//! Both end in `stop_agents`, which cancels the matching work and records an
//! `agent_stop_requests` row. A turn already in flight sees that row at its next step
//! (`nomi_agent_core::stop::is_stop_requested`) and ends without replying.

use std::collections::HashSet;

use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;

use nomi_agent_core::AgentRegistry;

use crate::SUPERVISOR_AGENT_TYPE;

/// What the user asked to stop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopTarget {
    /// Everything running in the current chat.
    ThisChat,
    /// Every agent in every one of the user's chats.
    Everything,
    /// One named agent, wherever it is running.
    Agent { agent_type: String },
}

/// An agent the user can name in a stop command.
#[derive(Debug, Clone)]
pub struct KnownAgent {
    pub agent_type: String,
    pub display_name: String,
    /// Lowercase, space-separated names that refer to this agent.
    pub aliases: Vec<String>,
    pub is_default: bool,
}

const STOP_VERBS: &[&str] = &["stop", "cancel", "abort", "halt", "kill", "terminate", "berhenti", "hentikan", "batalkan", "batal"];
const LEADING_FILLERS: &[&str] =
    &["hey", "hi", "ok", "okay", "oh", "so", "yo", "please", "pls", "plz", "nomi", "can", "could", "would", "will", "you", "u", "just", "tolong"];
const TRAILING_FILLERS: &[&str] =
    &["please", "pls", "plz", "now", "right", "nomi", "thanks", "thank", "you", "asap", "immediately", "dong", "ya", "sekarang", "dulu", "aja", "saja"];
const ALL_WORDS: &[&str] = &["all", "every", "everything", "everyone", "everybody", "semua", "semuanya"];
/// Words that don't change what gets stopped: "stop the running agent" == "stop".
const GENERIC_WORDS: &[&str] = &[
    "the", "a", "an", "this", "that", "it", "them", "those", "these", "current", "currently", "running", "active", "my",
    "your", "of", "agent", "agents", "task", "tasks", "work", "working", "job", "jobs", "crew", "team", "subagent",
    "subagents", "ini", "itu", "agen", "tugas", "kerja", "yang", "sedang", "jalan", "berjalan",
];
/// Longer messages are conversation, not commands ("stop me if I spend too much on coffee").
const MAX_COMMAND_WORDS: usize = 8;

fn normalize(text: &str) -> String {
    let spaced: String = text.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Cheap first check, so ordinary messages never pay for loading the agent list.
pub fn looks_like_stop_command(text: &str) -> bool {
    let normalized = normalize(text);
    let words: Vec<&str> = normalized.split_whitespace().collect();
    words.len() <= MAX_COMMAND_WORDS && words.iter().any(|w| STOP_VERBS.contains(w))
}

/// Recognizes a stop command, or returns `None` for anything else. Deliberately strict: the
/// message must open with a stop verb (after filler like "hey nomi, please") and every other
/// word must be filler, "all", or one agent's name. "stop telling me jokes" is not a command.
pub fn parse_stop_command(text: &str, agents: &[KnownAgent]) -> Option<StopTarget> {
    let normalized = normalize(text);
    let mut words: Vec<&str> = normalized.split_whitespace().collect();
    if words.is_empty() || words.len() > MAX_COMMAND_WORDS {
        return None;
    }
    while words.len() > 1 && LEADING_FILLERS.contains(&words[0]) {
        words.remove(0);
    }
    while words.len() > 1 && words.last().is_some_and(|w| TRAILING_FILLERS.contains(w)) {
        words.pop();
    }

    let (verb, rest) = words.split_first()?;
    if !STOP_VERBS.contains(verb) {
        return None;
    }

    let everything = rest.iter().any(|w| ALL_WORDS.contains(w));
    let named: Vec<&str> = rest.iter().copied().filter(|w| !ALL_WORDS.contains(w) && !GENERIC_WORDS.contains(w)).collect();
    if named.is_empty() {
        return Some(if everything { StopTarget::Everything } else { StopTarget::ThisChat });
    }

    let name = named.join(" ");
    let agent = agents.iter().find(|a| a.aliases.contains(&name))?;
    // "stop nomi" means stop what's happening here, not just the default agent.
    Some(if agent.is_default { StopTarget::ThisChat } else { StopTarget::Agent { agent_type: agent.agent_type.clone() } })
}

/// The built-in agents plus every active dynamic agent.
pub async fn known_agents(conn: &mut PgConnection, registry: &AgentRegistry) -> Vec<KnownAgent> {
    let mut agents: Vec<KnownAgent> = registry
        .agents()
        .iter()
        .map(|agent| {
            let agent_type = agent.agent_type().into_owned();
            let display_name = agent.display_name().into_owned();
            let mut aliases = vec![normalize(&display_name), normalize(&agent.intent_label()), normalize(&agent_type)];
            aliases.dedup();
            KnownAgent { agent_type, display_name, aliases, is_default: agent.is_default() }
        })
        .collect();

    let dynamic: Vec<(Uuid, String, String)> =
        sqlx::query_as("SELECT id, name, intent_label FROM dynamic_agents WHERE is_active = true")
            .fetch_all(&mut *conn)
            .await
            .unwrap_or_default();
    agents.extend(dynamic.into_iter().map(|(id, name, intent_label)| KnownAgent {
        agent_type: id.to_string(),
        aliases: vec![normalize(&name), normalize(&intent_label)],
        display_name: name,
        is_default: false,
    }));
    agents
}

/// What `stop_agents` stopped.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct StopReport {
    /// Display names of the agents that were stopped, in the order they were found.
    pub stopped: Vec<String>,
    /// How many chats had something stopped.
    pub chats: usize,
    /// Queued messages that were dropped instead of being answered.
    pub dropped_messages: usize,
    /// Approval cards that can no longer be decided (now `cancelled`).
    pub cancelled_approval_message_ids: Vec<Uuid>,
    /// `(delegation_id, session_id)` of every hand-off that was cancelled.
    pub cancelled_delegations: Vec<(Uuid, Uuid)>,
}

impl StopReport {
    pub fn stopped_anything(&self) -> bool {
        !self.stopped.is_empty() || self.dropped_messages > 0
    }
}

pub struct StopRequest {
    pub user_id: Uuid,
    /// The chat the request was made in.
    pub session_id: Uuid,
    pub target: StopTarget,
    /// Set when the supervisor agent's own turn asks for the stop, so that turn (still
    /// running, to report back) isn't stopped or counted.
    pub from_supervisor_turn: bool,
}

/// Name shown for an agent type: dynamic agents by their row's name, built-ins the way
/// `SubAgent::display_name` derives it.
async fn display_name_for(conn: &mut PgConnection, agent_type: &str) -> String {
    if let Ok(id) = agent_type.parse::<Uuid>() {
        if let Ok(Some(name)) = sqlx::query_scalar::<_, String>("SELECT name FROM dynamic_agents WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
        {
            return name;
        }
    }
    if agent_type == "chitchat" {
        return "Nomi".to_string();
    }
    let mut chars = agent_type.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Cancels everything `request.target` covers and records the stop for turns already running.
/// The caller decides whether this runs in a transaction.
pub async fn stop_agents(conn: &mut PgConnection, request: &StopRequest) -> Result<StopReport, sqlx::Error> {
    let session_filter = match request.target {
        StopTarget::ThisChat => Some(request.session_id),
        _ => None,
    };
    let agent_filter = match &request.target {
        StopTarget::Agent { agent_type } => Some(agent_type.clone()),
        _ => None,
    };
    let exempt = request.from_supervisor_turn.then_some(SUPERVISOR_AGENT_TYPE);

    sqlx::query("INSERT INTO agent_stop_requests (user_id, session_id, agent_type, exempt_agent_type) VALUES ($1, $2, $3, $4)")
        .bind(request.user_id)
        .bind(session_filter)
        .bind(&agent_filter)
        .bind(exempt)
        .execute(&mut *conn)
        .await?;

    let mut stopped_types: Vec<String> = Vec::new();
    let mut touched_sessions: HashSet<Uuid> = HashSet::new();
    let mut sessions_with_agent: HashSet<Uuid> = HashSet::new();

    // Live and paused agents. Clearing paused_for_approval makes the approval endpoint treat
    // their cards as no longer pending.
    let sessions: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
        "UPDATE agent_sessions a \
         SET status = 'cancelled', ended_at = now(), current_phase = 'waiting', current_phase_detail = NULL, \
             state = a.state || jsonb_build_object('paused_for_approval', false) \
         FROM channel_identities ci \
         WHERE ci.id = a.sender_channel_identity_id AND ci.user_id = $1 \
           AND a.status IN ('active', 'awaiting_approval') \
           AND ($2::uuid IS NULL OR a.session_id = $2) \
           AND ($3::text IS NULL OR a.agent_type = $3) \
           AND ($4::text IS NULL OR a.agent_type <> $4) \
         RETURNING a.session_id, a.agent_type, a.state->>'pending_approval_message_id'",
    )
    .bind(request.user_id)
    .bind(session_filter)
    .bind(&agent_filter)
    .bind(exempt)
    .fetch_all(&mut *conn)
    .await?;

    let mut approval_message_ids: Vec<Uuid> = Vec::new();
    for (session_id, agent_type, pending_message_id) in sessions {
        touched_sessions.insert(session_id);
        sessions_with_agent.insert(session_id);
        if let Some(id) = pending_message_id.and_then(|id| id.parse().ok()) {
            approval_message_ids.push(id);
        }
        stopped_types.push(agent_type);
    }

    let delegations: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
        "UPDATE agent_delegations SET status = 'cancelled', completed_at = now(), error = 'Stopped by the user' \
         WHERE user_id = $1 AND status IN ('pending', 'processing') \
           AND ($2::uuid IS NULL OR session_id = $2) \
           AND ($3::text IS NULL OR target_agent_type = $3) \
         RETURNING id, session_id, target_agent_type",
    )
    .bind(request.user_id)
    .bind(session_filter)
    .bind(&agent_filter)
    .fetch_all(&mut *conn)
    .await?;
    for (_, session_id, target) in &delegations {
        touched_sessions.insert(*session_id);
        stopped_types.push(target.clone());
    }

    let mut dropped_messages = 0;
    if agent_filter.is_none() {
        // A turn in flight with no agent session of its own is Nomi (the default agent) at work.
        let busy_sessions: Vec<Uuid> = sqlx::query_scalar(
            "SELECT DISTINCT tj.session_id FROM turn_jobs tj \
             JOIN channel_identities ci ON ci.id = tj.sender_channel_identity_id \
             WHERE ci.user_id = $1 AND tj.status = 'processing' AND ($2::uuid IS NULL OR tj.session_id = $2)",
        )
        .bind(request.user_id)
        .bind(session_filter)
        .fetch_all(&mut *conn)
        .await?;
        for session_id in busy_sessions {
            let is_the_supervisor_turn = request.from_supervisor_turn && session_id == request.session_id;
            if !is_the_supervisor_turn && !sessions_with_agent.contains(&session_id) {
                touched_sessions.insert(session_id);
                stopped_types.push("chitchat".to_string());
            }
        }

        let dropped: Vec<Uuid> = sqlx::query_scalar(
            "UPDATE turn_jobs tj SET status = 'cancelled', completed_at = now(), error = 'Stopped by the user' \
             FROM channel_identities ci \
             WHERE ci.id = tj.sender_channel_identity_id AND ci.user_id = $1 AND tj.status = 'pending' \
               AND ($2::uuid IS NULL OR tj.session_id = $2) \
             RETURNING tj.session_id",
        )
        .bind(request.user_id)
        .bind(session_filter)
        .fetch_all(&mut *conn)
        .await?;
        dropped_messages = dropped.len();
        touched_sessions.extend(dropped);
    }

    if !approval_message_ids.is_empty() {
        sqlx::query("UPDATE approval_resumes SET status = 'cancelled', completed_at = now() WHERE status = 'pending' AND message_id = ANY($1)")
            .bind(&approval_message_ids)
            .execute(&mut *conn)
            .await?;
        sqlx::query(
            "UPDATE messages SET content_blocks = jsonb_set(content_blocks, '{0,status}', '\"cancelled\"') \
             WHERE id = ANY($1) AND content_blocks->0->>'status' = 'pending'",
        )
        .bind(&approval_message_ids)
        .execute(&mut *conn)
        .await?;
    }

    let mut stopped: Vec<String> = Vec::new();
    for agent_type in stopped_types {
        let name = display_name_for(conn, &agent_type).await;
        if !stopped.contains(&name) {
            stopped.push(name);
        }
    }

    Ok(StopReport {
        stopped,
        chats: touched_sessions.len(),
        dropped_messages,
        cancelled_approval_message_ids: approval_message_ids,
        cancelled_delegations: delegations.into_iter().map(|(id, session_id, _)| (id, session_id)).collect(),
    })
}

fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The supervisor's reply to a stop, written without a model call so it lands instantly.
pub fn describe(report: &StopReport, target: &StopTarget, target_name: Option<&str>) -> String {
    if !report.stopped_anything() {
        return match (target, target_name) {
            (StopTarget::Agent { .. }, Some(name)) => format!("{name} isn't working on anything right now."),
            _ => "Nothing is running right now, so there's nothing to stop.".to_string(),
        };
    }

    let mut reply = if report.stopped.is_empty() {
        "Stopped.".to_string()
    } else {
        format!("Stopped {}.", join_names(&report.stopped))
    };
    if matches!(target, StopTarget::Everything) && report.chats > 1 {
        reply.pop();
        reply.push_str(&format!(" across {} chats.", report.chats));
    }
    if !report.cancelled_approval_message_ids.is_empty() {
        reply.push_str(" Pending approval requests are cancelled too.");
    }
    if report.dropped_messages > 0 {
        let noun = if report.dropped_messages == 1 { "message" } else { "messages" };
        reply.push_str(&format!(" I dropped {} queued {noun} without answering.", report.dropped_messages));
    }
    reply.push_str(" Tell me when you want to pick it back up.");
    reply
}

pub struct StopMessageOutcome {
    pub user_message_id: Uuid,
    pub reply_message_id: Uuid,
    pub reply: String,
    pub report: StopReport,
}

/// If `text` is a stop command, stops the matching agents and records both the user's message
/// and the supervisor's reply in the chat. `Ok(None)` means it isn't one: queue it as usual.
pub async fn handle_stop_message(
    pool: &PgPool,
    registry: &AgentRegistry,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<Option<StopMessageOutcome>, sqlx::Error> {
    if !looks_like_stop_command(text) {
        return Ok(None);
    }
    let mut conn = pool.acquire().await?;
    let agents = known_agents(&mut conn, registry).await;
    let Some(target) = parse_stop_command(text, &agents) else {
        return Ok(None);
    };
    let target_name = match &target {
        StopTarget::Agent { agent_type } => agents.iter().find(|a| &a.agent_type == agent_type).map(|a| a.display_name.clone()),
        _ => None,
    };

    let mut tx = conn.begin().await?;
    let user_message_id: Uuid =
        sqlx::query_scalar("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, $3) RETURNING id")
            .bind(session_id)
            .bind(sender_channel_identity_id)
            .bind(text)
            .fetch_one(&mut *tx)
            .await?;

    let report = stop_agents(&mut tx, &StopRequest { user_id, session_id, target: target.clone(), from_supervisor_turn: false }).await?;
    let reply = describe(&report, &target, target_name.as_deref());

    // clock_timestamp() so the reply sorts after the user's message inside this transaction.
    let reply_message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name, created_at) \
         VALUES ($1, NULL, $2, 'Supervisor', clock_timestamp()) RETURNING id",
    )
    .bind(session_id)
    .bind(&reply)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Some(StopMessageOutcome { user_message_id, reply_message_id, reply, report }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agents() -> Vec<KnownAgent> {
        vec![
            KnownAgent { agent_type: "chitchat".into(), display_name: "Nomi".into(), aliases: vec!["nomi".into(), "chitchat".into()], is_default: true },
            KnownAgent { agent_type: "money".into(), display_name: "Money".into(), aliases: vec!["money".into()], is_default: false },
            KnownAgent {
                agent_type: "6f1c0e8e-0000-0000-0000-000000000000".into(),
                display_name: "Travel Planner".into(),
                aliases: vec!["travel planner".into(), "travel".into()],
                is_default: false,
            },
        ]
    }

    #[test]
    fn plain_stop_words_stop_this_chat() {
        for text in ["stop", "Stop!", "stop it", "cancel that", "please stop", "hey nomi, stop the running agent", "stop working now", "stop nomi", "berhenti"] {
            assert_eq!(parse_stop_command(text, &agents()), Some(StopTarget::ThisChat), "{text}");
        }
    }

    #[test]
    fn all_words_stop_everything() {
        for text in ["stop all agents", "stop all agent", "Stop everything.", "cancel all the tasks please", "stop semua agen"] {
            assert_eq!(parse_stop_command(text, &agents()), Some(StopTarget::Everything), "{text}");
        }
    }

    #[test]
    fn a_named_agent_stops_only_that_agent() {
        assert_eq!(parse_stop_command("stop the money agent", &agents()), Some(StopTarget::Agent { agent_type: "money".into() }));
        assert_eq!(
            parse_stop_command("cancel travel planner", &agents()),
            Some(StopTarget::Agent { agent_type: "6f1c0e8e-0000-0000-0000-000000000000".into() })
        );
    }

    #[test]
    fn ordinary_messages_are_not_commands() {
        for text in [
            "stop telling me jokes",
            "how do I stop overspending on coffee?",
            "can you cancel my dentist reminder",
            "what are the agents doing",
            "the bus stop near my house is closed and I need another route to work",
            "",
        ] {
            assert_eq!(parse_stop_command(text, &agents()), None, "{text}");
        }
    }

    #[test]
    fn describe_reports_what_happened() {
        let none = StopReport::default();
        assert_eq!(describe(&none, &StopTarget::ThisChat, None), "Nothing is running right now, so there's nothing to stop.");
        assert_eq!(
            describe(&none, &StopTarget::Agent { agent_type: "money".into() }, Some("Money")),
            "Money isn't working on anything right now."
        );

        let report = StopReport { stopped: vec!["Money".into(), "Coding".into(), "Nomi".into()], chats: 2, dropped_messages: 1, ..Default::default() };
        assert_eq!(
            describe(&report, &StopTarget::Everything, None),
            "Stopped Money, Coding and Nomi across 2 chats. I dropped 1 queued message without answering. Tell me when you want to pick it back up."
        );
    }
}
