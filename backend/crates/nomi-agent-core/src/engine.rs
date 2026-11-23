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
use crate::personal_crew::Level;
use crate::registry::AgentRegistry;
use crate::subagent::SubAgent;

mod activity;
mod offer;
mod plans;
mod tool_batch;
mod tool_definitions;

pub use activity::*;
pub use offer::*;
pub use plans::*;
pub use tool_batch::*;
pub use tool_definitions::*;

/// Posted when the model hit its output limit before writing any answer.
/// Asked once when a model ends its turn having only thought, with no answer written.
const ANSWER_NOW: &str = "You haven't written a reply yet. Write your answer to the user now.";
// When even that produces nothing, the reply is `engine.no_answer` (or `engine.cut_off` when the
// model ran out of room), so the chat never ends on a silent turn; a plan moved into a draft is
// answered with `engine.plan_draft`. All three in the person's language (nomi-i18n).

/// How many times one run retries after being cut off before telling the person.
const MAX_CUT_OFF_RETRIES: u32 = 3;

/// How agents should think when reasoning is on. Users read the thinking in chat.
pub const REASONING_STYLE: &str = "Keep your private thinking brief and on point: at most 3 short fragments \
     (about 40 words) on what the person needs, what you'll do, and any catch, in the language you reply in. No \
     preamble, no restating their message, no weighing every option, no drafting the reply there. Your reply is only the answer for \
     the person: never write your thinking, your plan of action, or notes about what they asked in it.";
pub const COMPLETE_TASK_TOOL_NAME: &str = "complete_task";
pub const DELEGATE_TOOL_NAME: &str = "delegate_to_agent";
pub const RENAME_CHAT_TOOL_NAME: &str = "rename_chat";
/// What the model hears back after `show`: the view is in the chat, so it needn't say it again.
const VIEW_SHOWN: &str = "Shown in the chat. Don't repeat what it shows; add only what it doesn't say.";
pub const UPDATE_TODOS_TOOL_NAME: &str = "update_todos";
pub const WRITE_PLAN_TOOL_NAME: &str = "write_plan";
pub const ASK_USER_TOOL_NAME: &str = "ask_user";
/// Most questions one `ask_user` card holds; the person steps through them with Next.
pub const MAX_QUESTIONS_PER_CARD: usize = 5;
pub const READ_PLAN_TOOL_NAME: &str = "read_plan";
pub const CREATE_REMINDER_TOOL_NAME: &str = "create_reminder";
pub const LIST_REMINDERS_TOOL_NAME: &str = "list_reminders";
pub const CANCEL_REMINDER_TOOL_NAME: &str = "cancel_reminder";
pub const FIND_TOOLS_TOOL_NAME: &str = "find_tools";
pub const CREATE_AGENT_TOOL_NAME: &str = "create_agent";

const PHASE_THINKING: &str = "thinking";
const PHASE_CALLING_TOOL: &str = "calling_tool";
const PHASE_WRITING_REPLY: &str = "writing_reply";
const PHASE_WAITING: &str = "waiting";
/// The provider couldn't be reached: the detail is `{"attempt", "max", "retry_at"}`.
const PHASE_RETRYING: &str = "retrying";
/// A command runs in the project: the detail is `{"command", "started_at"}`.
const PHASE_RUNNING_COMMAND: &str = "running_command";
/// How many times a model call is tried before the turn gives up.
pub const LLM_MAX_ATTEMPTS: u32 = 10;

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

/// Makes the request end on the person's side: when the chat's newest message is the crew's,
/// adds a note in the person's turn pointing back at what they asked.
pub(crate) fn end_on_the_person(messages: &mut Vec<LlmMessage>) {
    if messages.last().is_some_and(|m| m.role == LlmRole::Assistant) {
        messages.push(LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: crate::prompts::ANSWER_THE_PERSON_NOTE.to_string() }] });
    }
}

#[cfg(test)]
mod ends_on_the_person_tests {
    use super::end_on_the_person;
    use nomi_llm::{ContentBlock, LlmMessage, LlmRole};

    fn said(role: LlmRole, text: &str) -> LlmMessage {
        LlmMessage { role, content: vec![ContentBlock::Text { text: text.into() }] }
    }

    #[test]
    fn a_chat_whose_newest_message_is_the_crews_gets_a_note_from_the_person_last() {
        // The person wrote, then a crew member posted (a progress note, a question to Rena) before
        // the turn ran: Gemini refuses a request whose last turn is the model's.
        let mut messages = vec![said(LlmRole::User, "add a dark mode"), said(LlmRole::Assistant, "Koda: Rena, which colours?")];
        end_on_the_person(&mut messages);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[2].role, LlmRole::User);
        assert_eq!(messages[2].content, vec![ContentBlock::Text { text: crate::prompts::ANSWER_THE_PERSON_NOTE.to_string() }]);
    }

    #[test]
    fn a_chat_ending_on_the_person_is_left_alone() {
        let mut messages = vec![said(LlmRole::Assistant, "hi"), said(LlmRole::User, "add a dark mode")];
        end_on_the_person(&mut messages);
        assert_eq!(messages.len(), 2);
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply { text: String, memory_ids_used: Vec<Uuid>, input_tokens: u32, output_tokens: u32 },
    /// The agent answered with something already in the chat — a recording (`speak`) or a view
    /// (`show`) — and wrote nothing after it: that message, `message_id`, is the reply. `text` is
    /// its words (what was said, or the view as text).
    Posted { message_id: Uuid, text: String, memory_ids_used: Vec<Uuid>, input_tokens: u32, output_tokens: u32 },
    Completed { status: String, summary: String },
    AwaitingApproval { message_id: Uuid },
    /// The agent asked the person questions (`ask_user`) and waits for their answer, which comes
    /// as their next message to the same agent. The card is already posted as `message_id`;
    /// `text` is its plain-text version.
    AwaitingAnswer { message_id: Uuid, text: String },
    /// The agent handed the conversation to `target_agent`, which answers the user in this same
    /// turn. Nothing of this agent's is posted: the specialist speaks for itself.
    HandOff { target_agent: String, task: String },
    /// The user stopped this agent mid-turn (see `crate::stop`). Nothing more is posted; whoever
    /// asked for the stop has already told the user.
    Cancelled,
}

/// The personality agent's tools that change which personality a chat replies in.
const PERSONALITY_TOOL_NAMES: [&str; 3] = ["set_personality", "reset_chat_personality", "rollback_personality"];

/// After an agent answers in chat (a reply, or the summary it finished with), queues the exchange
/// so the memory worker can save anything lasting the person said. Never waits on a model call.
async fn learn_from_turn(
    conn: &mut PoolConnection<Postgres>,
    agent: &dyn SubAgent,
    user_id: Uuid,
    session_id: Uuid,
    messages: &[LlmMessage],
    answer: &str,
) {
    // Fallback only: the queue reads the person's latest chat message itself.
    let last_user_text = messages
        .iter()
        .rev()
        .find(|m| m.role == LlmRole::User)
        .and_then(|m| m.content.iter().find_map(|b| match b {
            ContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        }))
        .unwrap_or_default();
    memory::queue_learning(conn, user_id, session_id, agent.agent_type().as_ref(), &last_user_text, answer).await;
}

/// Runs one agent turn to completion: retrieves memory first if `agent.uses_memory()`,
/// then drives the tool-calling loop with every LLM call streamed (deltas published over
/// `mqtt` when provided — this now happens for every agent, not just a hardcoded chitchat
/// case), then extracts+stores memory after a `Reply` outcome if `agent.uses_memory()`.
/// The whole turn is one reply (`crate::reply`): what its tools change is grouped by it.
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
    messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
    crate::reply::within_reply(run_agent_turn_in_reply(
        conn,
        mqtt,
        s3,
        provider,
        embedding_provider,
        registry,
        agent,
        session_id,
        agent_session_id,
        user_id,
        messages,
        max_tokens,
    ))
    .await
}

#[allow(clippy::too_many_arguments)]
async fn run_agent_turn_in_reply(
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
    end_on_the_person(&mut messages);
    let mut tools = agent.tools();
    let mut own_tools = always_on_own_tools(agent);
    let chat_tools = agent.chat_tools(conn, session_id, user_id, &messages).await;
    if !agent.own_tools_switchable() {
        own_tools.extend(chat_tools.iter().map(|t| t.name.clone()));
    }
    tools.extend(chat_tools);
    // The members the person switched off take no work and answer no questions.
    let off = crate::personal_crew::disabled_agent_types(conn, user_id).await;
    // Whatever an admin switched off in Admin → Tools.
    let switches = crate::tools::ToolSwitches::load(conn).await;
    let inputs = OfferInputs {
        web_search_ready: switches.allows(crate::web::WEB_SEARCH_TOOL_NAME) && crate::web::search_setup(conn).await.is_some(),
        voice_ready: switches.allows(crate::voice::SPEAK_TOOL_NAME) && crate::voice::is_ready(conn).await,
        mentions_files: crate::attachments::mentions_files(&messages),
        off,
        switches,
    };
    tools.extend(engine_tools_for(agent, registry, &inputs));
    let switches = inputs.switches;
    tools.retain(|t| own_tools.contains(&t.name) || switches.allows(&t.name));
    // In a project's chat, its tools default to that project: no one asks the person for an id.
    let chat_project = crate::chat_project::of_chat(conn, session_id, user_id).await;
    if let Some((_, name)) = &chat_project {
        for tool in &mut tools {
            crate::chat_project::default_to_chat_project(tool, name);
        }
    }

    // What the person lets Nomi do with memory (Settings → Memory).
    let memory_settings = crate::memory_settings::load(&mut **conn, user_id).await;
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
        let found = memory::try_retrieve_memories(conn, embedding_provider, user_id, &last_user_text).await;
        memory::within_budget(found, memory_settings.memory_budget)
    } else {
        Vec::new()
    };

    // The agent's own prompt, then anything only this kind of request needs (domain_context).
    let base_prompt = match agent.domain_context(conn, s3, session_id, user_id, &messages).await {
        Some(context) => format!("{}\n\n{context}", agent.system_prompt()),
        None => agent.system_prompt().to_string(),
    };
    let system_prompt = if memories.is_empty() {
        base_prompt
    } else {
        let mut prompt = format!("{base_prompt}\n\nWhat you remember about this person that matters here (use it only where it helps):\n");
        for m in &memories {
            prompt.push_str(&format!("- ({}) {}\n", m.kind, m.content));
        }
        prompt
    };

    // The profile Nomi keeps of the person, unless they turned it off.
    let system_prompt = match agent.uses_memory() {
        true => match crate::memory_settings::profile_for_agents(conn, user_id, &memory_settings).await {
            Some(profile) => format!("{system_prompt}\n\n{}\n{profile}", crate::prompts::MEMORY_PROFILE_HEADING),
            None => system_prompt,
        },
        false => system_prompt,
    };

    // What the person did lately across their chats (recent_activity): every agent sees it, so
    // whoever picks up a new chat knows what they were up to, unless they turned off searching
    // past chats.
    let past_chats = match memory_settings.search_past_chats {
        true => crate::recent_activity::for_person(&mut **conn, user_id).await?,
        false => None,
    };
    let system_prompt = match past_chats {
        Some(note) => format!("{system_prompt}\n\n{}\n{note}", crate::prompts::RECENT_ACTIVITY_HEADING),
        None => system_prompt,
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
        match crate::personality::personality_for_chat(conn, user_id, session_id).await {
            Some(p) => format!("{system_prompt}\n\nAdopt this personality in your replies: {p}"),
            None => system_prompt,
        }
    } else {
        system_prompt
    };

    // The soul.md the person wrote for this member (Settings › Crew).
    let system_prompt = match crate::personal_crew::current_soul(conn, user_id, agent.agent_type().as_ref()).await {
        Ok(Some((soul, _))) if !soul.trim().is_empty() => format!("{system_prompt}\n\n{}", crate::personal_crew::soul_prompt(&soul)),
        _ => system_prompt,
    };

    // Every agent sees what the rest of the crew is doing in this chat, so two never work the
    // same task or talk over each other.
    let system_prompt = match crate::crew::crew_board(&mut **conn, session_id, agent.agent_type().as_ref()).await {
        Some(board) => format!("{system_prompt}\n\n{board}"),
        None => system_prompt,
    };
    // And the notes the crew pinned for each other here.
    let system_prompt = match crate::crew_talk::notes_board(&mut **conn, session_id).await {
        Some(board) => format!("{system_prompt}\n\n{board}"),
        None => system_prompt,
    };
    let system_prompt = match &chat_project {
        Some((project_id, name)) => format!("{system_prompt}\n\n{}", crate::chat_project::prompt_line(*project_id, name)),
        None => system_prompt,
    };
    // In an issue's own chat: which issue, who has it, and what else runs beside it.
    let system_prompt = match crate::issue_chat::prompt_for(&mut **conn, session_id).await {
        Some(issue) => format!("{system_prompt}\n\n{issue}"),
        None => system_prompt,
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
        "max" => nomi_llm::ReasoningEffort::Max,
        _ => nomi_llm::ReasoningEffort::Medium,
    };
    // Thinking is shown in chat, so keep it to the point rather than a running monologue.
    let system_prompt = format!(
        "{system_prompt}\n\n{}\n\n{REASONING_STYLE}\n\n{}\n\n{}",
        crate::prompts::BACKGROUND_EXECUTION_GUIDE,
        crate::prompts::FOLLOW_UP_SUGGESTIONS,
        crate::prompts::ABOUT_NOMI
    );
    let agent_type = agent.agent_type();

    // Whether this run already saved a plan draft, and already asked once for a missing answer.
    let mut wrote_plan = false;
    let mut asked_for_answer = false;
    let mut cut_off_retries = 0;
    // How often each tool call has failed which way, for repeated_failure_limit.
    let mut failures: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for turn_idx in 0..agent.max_tool_turns() {
        if crate::stop::is_stop_requested(conn, user_id, session_id, &agent_type, started_at).await {
            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
            return Ok(LoopOutcome::Cancelled);
        }

        update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_THINKING, None).await;
        tracing::info!(
            %session_id,
            %agent_session_id,
            agent = %agent_type,
            step = turn_idx + 1,
            max_steps = agent.max_tool_turns(),
            "agent requesting LLM completion"
        );

        // A long tool loop resends every earlier result: past the budget, older ones are trimmed.
        let trimmed = crate::context::compact_tool_results(&mut messages, crate::context::TURN_BUDGET_CHARS, crate::context::KEEP_RECENT_ROUNDS);
        if trimmed > 0 {
            tracing::debug!(%session_id, trimmed, "trimmed older tool results to keep the request small");
        }

        let request = LlmRequest {
            system: Some(system_prompt.clone()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
            enable_reasoning: true,
            reasoning_effort,
        };

        // The LLM call happens outside any DB transaction: holding a transaction open across
        // a slow network round trip would needlessly extend how long this connection's locks
        // are held.
        // A provider that can't be reached, is overloaded or rate limited is tried again (up to
        // LLM_MAX_ATTEMPTS), and the person sees the retry and its countdown instead of a silent
        // wait. A response that already started streaming isn't retried: they've seen part of it.
        let mut attempt: u32 = 1;
        let response = loop {
            let streamed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let result = match provider.complete_stream(request.clone()).await {
                Err(e) => Err(e),
                Ok(stream) => match mqtt {
                    Some((publisher, turn_job_id)) => {
                        use futures_util::StreamExt;
                        // LlmEventStream requires 'static (boxed as `dyn Stream + Send`, no
                        // lifetime), so this clones the publisher handle (cheap) rather than
                        // capturing the `&MqttPublisher` borrow.
                        let publisher = publisher.clone();
                        let streamed = streamed.clone();
                        let published = stream.then(move |event_result| {
                            let publisher = publisher.clone();
                            let streamed = streamed.clone();
                            async move {
                                if let Ok(event) = &event_result {
                                    streamed.store(true, std::sync::atomic::Ordering::Relaxed);
                                    let envelope = StreamEnvelope::Delta { turn_job_id, event: event.clone() };
                                    // Best-effort: an MQTT publish failure never fails the turn.
                                    let _ = publisher.publish(session_id, &envelope).await;
                                }
                                event_result
                            }
                        });
                        nomi_llm::collect_stream(Box::pin(published)).await
                    }
                    None => nomi_llm::collect_stream(stream).await,
                },
            };
            match result {
                Ok(response) => {
                    tracing::info!(
                        %session_id,
                        %agent_session_id,
                        agent = %agent_type,
                        stop_reason = ?response.stop_reason,
                        input_tokens = response.input_tokens,
                        output_tokens = response.output_tokens,
                        "agent received LLM completion"
                    );
                    break response;
                }
                Err(e) if e.is_retryable() && attempt < LLM_MAX_ATTEMPTS && !streamed.load(std::sync::atomic::Ordering::Relaxed) => {
                    let delay = retry_delay(attempt);
                    tracing::warn!(%session_id, attempt, error = %e, "model call failed; retrying in {delay:?}");
                    let retry_at = chrono::Utc::now() + chrono::Duration::from_std(delay).unwrap_or_default();
                    let detail = serde_json::json!({ "attempt": attempt + 1, "max": LLM_MAX_ATTEMPTS, "retry_at": retry_at }).to_string();
                    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_RETRYING, Some(&detail)).await;
                    // Waits in short steps so a stop still lands quickly.
                    let mut waited = std::time::Duration::ZERO;
                    while waited < delay {
                        let step = std::time::Duration::from_millis(500).min(delay - waited);
                        tokio::time::sleep(step).await;
                        waited += step;
                        if crate::stop::is_stop_requested(conn, user_id, session_id, &agent_type, started_at).await {
                            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
                            return Ok(LoopOutcome::Cancelled);
                        }
                    }
                    attempt += 1;
                    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_THINKING, None).await;
                }
                Err(e) => return Err(TurnError::LlmCallFailed(e)),
            }
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
            // Said it out loud, or showed a view, and wrote nothing more: that is the whole reply.
            if reply_text.is_empty() && !cut_off {
                if let Some((message_id, text)) = answer_posted_since(conn, session_id, agent.display_name().as_ref(), started_at).await {
                    if agent.uses_memory() {
                        learn_from_turn(conn, agent, user_id, session_id, &messages, &text).await;
                    }
                    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
                    return Ok(LoopOutcome::Posted {
                        message_id,
                        text,
                        memory_ids_used: memories.iter().map(|m| m.id).collect(),
                        input_tokens,
                        output_tokens,
                    });
                }
            }
            // Only thinking, no answer: ask once for the answer itself before giving up.
            if reply_text.is_empty() && !cut_off && !asked_for_answer {
                asked_for_answer = true;
                if response.content.is_empty() {
                    messages.pop();
                }
                messages.push(LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: ANSWER_NOW.to_string() }] });
                continue;
            }
            // Cut off before writing an answer, often mid tool call: drop it and ask for smaller steps.
            let was_calling_tools = response.content.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. }));
            if cut_off && (reply_text.is_empty() || was_calling_tools) && cut_off_retries < MAX_CUT_OFF_RETRIES {
                cut_off_retries += 1;
                messages.pop();
                messages.push(LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: agent.cut_off_hint().to_string() }] });
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
                learn_from_turn(conn, agent, user_id, session_id, &messages, &reply_text).await;
            }

            update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;

            tracing::info!(
                %session_id,
                %agent_session_id,
                agent = %agent_type,
                reply_len = reply_text.len(),
                "agent turn completed with text reply"
            );

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
        // Text written with a view ("Here's where your money went:") introduces it: it goes in the
        // chat as an ordinary message, before the view, for every agent.
        let shows = pending_tool_use_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { name, .. } if name.as_str() == crate::views::SHOW_TOOL_NAME));
        if shows && !finishing && !written.is_empty() {
            post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, agent.display_name().as_ref(), &written, None).await;
        } else if agent.surfaces_activity() && !finishing && !written.is_empty() {
            post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, agent.display_name().as_ref(), &format!("💭 {written}"), None).await;
        }

        wrote_plan |= pending_tool_use_blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolUse { name, .. } if name.as_str() == WRITE_PLAN_TOOL_NAME));

        // Questions for the person end the turn: the card goes up and the agent waits for their
        // answer, which reaches it as their next message.
        if agent.asks_questions() {
            let asked = pending_tool_use_blocks.iter().find_map(|b| match b {
                ContentBlock::ToolUse { name, input, .. } if name.as_str() == ASK_USER_TOOL_NAME => Some(input),
                _ => None,
            });
            if let Some(input) = asked {
                match parse_questions(input) {
                    Ok((block, text)) => {
                        let message_id = post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, agent.display_name().as_ref(), &text, Some(&block)).await;
                        update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
                        if let Some(message_id) = message_id {
                            return Ok(LoopOutcome::AwaitingAnswer { message_id, text });
                        }
                    }
                    Err(err) => {
                        // Let the agent fix its question instead of failing the turn.
                        let results = pending_tool_use_blocks
                            .iter()
                            .filter_map(|b| match b {
                                ContentBlock::ToolUse { id, name, .. } => Some(ContentBlock::ToolResult {
                                    tool_use_id: id.clone(),
                                    content: if name.as_str() == ASK_USER_TOOL_NAME { err.clone() } else { "Not run: call ask_user on its own.".to_string() },
                                    is_error: true,
                                }),
                                _ => None,
                            })
                            .collect();
                        messages.push(LlmMessage { role: LlmRole::User, content: results });
                        continue;
                    }
                }
            }
        }

        match resolve_tool_batch_in_reply(conn, mqtt, s3, Some(provider), Some(embedding_provider), registry, agent, session_id, agent_session_id, user_id, &pending_tool_use_blocks, &messages, &[]).await? {
            ToolBatchOutcome::AwaitingApproval { message_id } => {
                tracing::info!(%session_id, %agent_session_id, %message_id, "agent turn paused awaiting approval");
                return Ok(LoopOutcome::AwaitingApproval { message_id });
            }
            ToolBatchOutcome::Completed { status, summary } => {
                let summary = if written.is_empty() { summary } else { written };
                let summary = keep_plan_in_draft(conn, s3, mqtt, agent, session_id, agent_session_id, user_id, locale, wrote_plan, summary).await;
                // The summary is this agent's answer in chat, so it's a turn to learn from too.
                if agent.uses_memory() {
                    learn_from_turn(conn, agent, user_id, session_id, &messages, &summary).await;
                }
                tracing::info!(%session_id, %agent_session_id, %status, summary_len = summary.len(), "agent completed task");
                return Ok(LoopOutcome::Completed { status, summary });
            }
            ToolBatchOutcome::HandOff { target_agent, task } => {
                update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
                tracing::info!(%session_id, %agent_session_id, to_agent = %target_agent, %task, "agent handed off task");
                return Ok(LoopOutcome::HandOff { target_agent, task });
            }
            ToolBatchOutcome::Resolved(tool_results) => {
                if let Some(stuck) = repeated_failure(agent, &pending_tool_use_blocks, &tool_results, &mut failures) {
                    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
                    return Err(stuck);
                }
                messages.push(LlmMessage { role: LlmRole::User, content: tool_results });
            }
        }
    }

    Err(TurnError::ToolLoopExceeded)
}

/// How long to wait before trying the model again: 2s, 4s, 8s, then 15s, capped at 30s. No wait
/// at all in tests (this crate's own, or another crate's with the `instant-retries` feature).
fn retry_delay(attempt: u32) -> std::time::Duration {
    let seconds = match attempt {
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 15,
        _ => 30,
    };
    std::time::Duration::from_secs(if cfg!(any(test, feature = "instant-retries")) { 0 } else { seconds })
}

#[cfg(test)]
mod background_engine_tests {
    use super::*;
    use std::borrow::Cow;
    use async_trait::async_trait;
    use crate::background::{
        CANCEL_BACKGROUND_TASK_TOOL_NAME, LIST_BACKGROUND_TASKS_TOOL_NAME,
        RUN_BACKGROUND_TASK_TOOL_NAME,
    };

    struct DummyDefaultAgent;
    #[async_trait]
    impl SubAgent for DummyDefaultAgent {
        fn agent_type(&self) -> Cow<'static, str> { Cow::Borrowed("chitchat") }
        fn system_prompt(&self) -> Cow<'static, str> { Cow::Borrowed("test") }
        fn tools(&self) -> Vec<ToolDefinition> { vec![] }
        async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: serde_json::Value) -> Result<crate::content_block::ToolOutcome, String> {
            Err("none".to_string())
        }
        fn intent_label(&self) -> Cow<'static, str> { Cow::Borrowed("chitchat") }
        fn intent_description(&self) -> Cow<'static, str> { Cow::Borrowed("default") }
        fn is_default(&self) -> bool { true }
    }

    #[test]
    fn crew_tools_list_find_tools_ungated_and_create_agent_gated() {
        let registry = AgentRegistry::new(vec![Box::new(DummyDefaultAgent)]);
        let defs = crew_tool_definitions(&registry);
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&FIND_TOOLS_TOOL_NAME) && names.contains(&CREATE_AGENT_TOOL_NAME));
        assert!(!is_gateable(FIND_TOOLS_TOOL_NAME), "looking through the catalog changes nothing");
        assert!(is_gateable(CREATE_AGENT_TOOL_NAME), "making a member goes through the person's permission level");
        assert!(!crate::tools::REQUIRED_TOOLS.contains(&FIND_TOOLS_TOOL_NAME), "an admin can switch it off");
    }

    #[test]
    fn crew_tools_include_all_background_task_tools_and_never_gate_them() {
        let registry = AgentRegistry::new(vec![Box::new(DummyDefaultAgent)]);
        let defs = crew_tool_definitions(&registry);
        let names: Vec<&str> = defs.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&RUN_BACKGROUND_TASK_TOOL_NAME));
        assert!(names.contains(&LIST_BACKGROUND_TASKS_TOOL_NAME));
        assert!(names.contains(&CANCEL_BACKGROUND_TASK_TOOL_NAME));

        assert!(!is_gateable(RUN_BACKGROUND_TASK_TOOL_NAME));
        assert!(!is_gateable(LIST_BACKGROUND_TASKS_TOOL_NAME));
        assert!(!is_gateable(CANCEL_BACKGROUND_TASK_TOOL_NAME));
    }
}
