//! Running a batch of tool calls from one model response: permissions and approvals, the
//! engine's own tools, delegation, and the agent's own tools.

use super::*;

/// Tool result for a call the user refused. Says plainly not to retry: models otherwise tend to
/// try the same call again, which only puts the same approval card in front of the user again.
pub(super) const USER_DENIED_RESULT: &str =
    "The user denied this action. Do not try it again; tell them it wasn't done and ask how they'd like to proceed.";

/// Tools that are never permission-gated: engine-level bookkeeping (complete_task,
/// delegate_to_agent, show, update_todos) — none of these touch anything a user would
/// want to approve/deny.
pub fn is_gateable(tool_name: &str) -> bool {
    tool_name != COMPLETE_TASK_TOOL_NAME
        && tool_name != DELEGATE_TOOL_NAME
        && tool_name != crate::views::SHOW_TOOL_NAME
        && tool_name != ASK_USER_TOOL_NAME
        && tool_name != READ_PLAN_TOOL_NAME
        && tool_name != RENAME_CHAT_TOOL_NAME
        && tool_name != UPDATE_TODOS_TOOL_NAME
        && tool_name != WRITE_PLAN_TOOL_NAME
        && tool_name != CREATE_REMINDER_TOOL_NAME
        && tool_name != LIST_REMINDERS_TOOL_NAME
        && tool_name != CANCEL_REMINDER_TOOL_NAME
        // Looking through the tool catalog changes nothing.
        && tool_name != FIND_TOOLS_TOOL_NAME
        // Talking with the crew stays in the chat.
        && tool_name != crate::crew_talk::ASK_AGENT_TOOL_NAME
        && tool_name != crate::crew_talk::NOTE_TOOL_NAME
        // Reading the public web, or the user's own files, changes nothing.
        && !crate::web::is_web_tool(tool_name)
        && tool_name != crate::attachments::READ_ATTACHMENT_TOOL_NAME
        // Speaking only posts a recording in the chat the person asked for.
        && tool_name != crate::voice::SPEAK_TOOL_NAME
        // Record tools only touch the calling agent's own private records.
        && !crate::records::RECORD_TOOL_NAMES.contains(&tool_name)
        // Background task orchestration tools queue/inspect/cancel background tasks without blocking.
        && !crate::background::BACKGROUND_TOOL_NAMES.contains(&tool_name)
}

pub(super) fn describe_pending_action(tool_name: &str, input: &serde_json::Value, locale: Locale) -> String {
    let path = input.get("path").and_then(|v| v.as_str());
    match (tool_name, path) {
        ("delete_file", Some(path)) => locale.tf("engine.approve_delete", &[("path", path)]),
        ("write_file", Some(path)) => locale.tf("engine.approve_overwrite", &[("path", path)]),
        (other, Some(path)) => locale.tf("engine.approve_run_on", &[("tool", other), ("path", path)]),
        (other, None) => locale.tf("engine.approve_run", &[("tool", other)]),
    }
}

/// Counts this round's failures (an error result, or output the agent says is a failure) by the
/// call and what it returned. Once the same call has failed the same way
/// `agent.repeated_failure_limit()` times, the run isn't getting anywhere: that's the error.
pub(super) fn repeated_failure(
    agent: &dyn SubAgent,
    tool_uses: &[ContentBlock],
    results: &[ContentBlock],
    failures: &mut std::collections::HashMap<String, u32>,
) -> Option<TurnError> {
    let limit = agent.repeated_failure_limit()?;
    for result in results {
        let ContentBlock::ToolResult { tool_use_id, content, is_error } = result else { continue };
        let Some((name, input)) = tool_uses.iter().find_map(|b| match b {
            ContentBlock::ToolUse { id, name, input, .. } if id == tool_use_id => Some((name, input)),
            _ => None,
        }) else {
            continue;
        };
        if !*is_error && !agent.tool_output_is_failure(name, content) {
            continue;
        }
        let seen = failures.entry(format!("{name}\u{0}{input}\u{0}{content}")).or_insert(0);
        *seen += 1;
        if *seen >= limit {
            return Some(TurnError::RepeatedFailure { tool: name.clone(), attempts: *seen, output: content.clone() });
        }
    }
    None
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
/// `blocks` with this chat's project filled in where a call to a tool that takes a `project_id`
/// left it out (or made one up); `None` when nothing needed it.
pub(super) async fn with_chat_project(
    conn: &mut PoolConnection<Postgres>,
    agent: &dyn SubAgent,
    session_id: Uuid,
    user_id: Uuid,
    blocks: &[ContentBlock],
    conversation: &[LlmMessage],
) -> Option<Vec<ContentBlock>> {
    let unnamed = |input: &serde_json::Value| crate::chat_project::fill(input, Uuid::nil()).is_some();
    if !blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { input, .. } if unnamed(input))) {
        return None;
    }
    let (project_id, _) = crate::chat_project::of_chat(conn, session_id, user_id).await?;
    let mut definitions = agent.tools();
    definitions.extend(agent.chat_tools(conn, session_id, user_id, conversation).await);
    let takes_project = |name: &str| definitions.iter().any(|t| t.name == name && crate::chat_project::takes_project(t));
    let mut changed = false;
    let filled = blocks
        .iter()
        .map(|block| match block {
            ContentBlock::ToolUse { id, name, input, thought_signature } if takes_project(name) => match crate::chat_project::fill(input, project_id) {
                Some(input) => {
                    changed = true;
                    ContentBlock::ToolUse { id: id.clone(), name: name.clone(), input, thought_signature: thought_signature.clone() }
                }
                None => block.clone(),
            },
            _ => block.clone(),
        })
        .collect();
    changed.then_some(filled)
}

/// Runs inside the turn's reply (`crate::reply`), or a new one when called on its own.
#[allow(clippy::too_many_arguments)]
pub async fn resolve_tool_batch(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    // For asking another crew member (ask_agent).
    provider: Option<&dyn LlmProvider>,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    tool_use_blocks: &[ContentBlock],
    conversation_so_far: &[LlmMessage],
    already_decided: &[(String, bool)],
) -> Result<ToolBatchOutcome, TurnError> {
    crate::reply::within_reply(resolve_tool_batch_in_reply(
        conn,
        mqtt,
        s3,
        provider,
        // A resumed batch matches find_tools by words alone.
        None,
        registry,
        agent,
        session_id,
        agent_session_id,
        user_id,
        tool_use_blocks,
        conversation_so_far,
        already_decided,
    ))
    .await
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn resolve_tool_batch_in_reply(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    // For asking another crew member (ask_agent).
    provider: Option<&dyn LlmProvider>,
    // For find_tools' matches by meaning; without it, by words alone.
    embedding: Option<&dyn EmbeddingProvider>,
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
    let filled = with_chat_project(conn, agent, session_id, user_id, tool_use_blocks, conversation_so_far).await;
    let tool_use_blocks = filled.as_deref().unwrap_or(tool_use_blocks);
    // What the person lets this member do (Settings › Crew › Permissions), by tool.
    let levels = crate::personal_crew::member_levels(conn, user_id, agent.agent_type().as_ref()).await;
    for block in tool_use_blocks {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            if already_decided.iter().any(|(decided_id, _)| decided_id == id) {
                continue;
            }
            if !is_gateable(name) {
                continue;
            }
            let decision = match levels.get(name.as_str()) {
                // Refused below, or run without a card.
                Some(Level::Never) | Some(Level::Allow) => continue,
                // Always the card, unless this very call was decided in this chat just now.
                Some(Level::Ask) => permissions::session_decision(conn, user_id, session_id, name, input).await.unwrap_or(permissions::PermissionDecision::Ask),
                None if !agent.tool_needs_approval(name) => continue,
                None => permissions::check_tool_permission_in_session(conn, user_id, session_id, name, input).await,
            };
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
                tool_use_id: Some(id.clone()),
                edited: false,
                result_url: None,
                remembered: None,
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
                // The resumed turn carries on this reply (crate::reply).
                "reply_id": crate::reply::current_reply(),
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

    let switches = crate::tools::ToolSwitches::load(conn).await;
    let own_tools = always_on_own_tools(agent);
    let mut tool_results = Vec::new();
    for block in tool_use_blocks {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            // A command shows itself and how long it has run, so a long install never looks
            // like a silent wait.
            match input.get("command").and_then(|c| c.as_str()).filter(|_| name.as_str() == "run_command") {
                Some(command) => {
                    let detail = serde_json::json!({ "command": command, "started_at": chrono::Utc::now() }).to_string();
                    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_RUNNING_COMMAND, Some(&detail)).await;
                }
                None => {
                    let subject = tool_subject(input);
                    update_agent_phase_about(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_CALLING_TOOL, Some(name), subject).await
                }
            }
            tracing::info!(%session_id, %agent_session_id, agent = %agent.agent_type(), tool = %name, "agent executing tool");

            // What the posted message says, when it isn't the tool result itself (`show`).
            let mut posted_text: Option<String> = None;
            let (result_text, is_error, rich_block) = if !own_tools.contains(name) && !switches.allows(name) {
                (format!("The {name} tool is switched off. Don't call it again; do without it."), true, None)
            } else if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false, None)
            } else if name.as_str() == DELEGATE_TOOL_NAME {
                let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
                let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
                let target = registry.find(target_agent);
                let switched_off = match &target {
                    Some(t) if !crate::personal_crew::is_enabled(conn, user_id, target_agent).await => Some(crate::personal_crew::switched_off_result(&t.display_name())),
                    _ => None,
                };
                // The tool only lists the members this agent may reach; a name outside them is refused.
                let out_of_reach = (!agent.may_delegate_to(target_agent)).then(|| crate::prompts::DELEGATE_OUT_OF_REACH.replace("{agent}", target_agent));
                let mut rejection =
                    out_of_reach.or(switched_off).or_else(|| target.as_ref().and_then(|t| t.validate_delegation_task(task).err()));
                // A build moves through the crew in a fixed order: a hand-over that comes too
                // early goes back to the agent with what is still missing.
                if rejection.is_none() {
                    rejection = crate::build_flow::handover_refusal(conn, session_id, agent.agent_type().as_ref(), target_agent).await;
                }
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
            } else if name.as_str() == crate::views::SHOW_TOOL_NAME {
                match crate::views::parse_show_input(input) {
                    Ok(view) => {
                        let view = crate::web::fill_link_preview(view).await;
                        let shown = crate::content_block::ToolOutcome::view(view);
                        // The model gets a short receipt; the message keeps the view's own text.
                        posted_text = Some(shown.display_text);
                        (VIEW_SHOWN.to_string(), false, shown.block)
                    }
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == crate::attachments::READ_ATTACHMENT_TOOL_NAME {
                match crate::attachments::read_attachment(conn, user_id, input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == crate::crew_talk::ASK_AGENT_TOOL_NAME {
                match crate::crew_talk::ask(conn, mqtt.map(|(p, _)| p), s3, provider, registry, agent, session_id, user_id, input, conversation_so_far).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == crate::crew_talk::NOTE_TOOL_NAME {
                match crate::crew_talk::write_note(conn, session_id, agent.agent_type().as_ref(), input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == RENAME_CHAT_TOOL_NAME {
                match crate::chat_title::rename_from_tool(conn, session_id, input).await {
                    Ok(title) => {
                        if let Some((publisher, _)) = mqtt {
                            let _ = publisher.publish(session_id, &StreamEnvelope::SessionRenamed { title: title.clone() }).await;
                        }
                        (format!("Renamed the chat to \"{title}\"."), false, None)
                    }
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == UPDATE_TODOS_TOOL_NAME {
                match parse_todo_items(input) {
                    Ok(items) => match upsert_todo_list(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, agent.display_name().as_ref(), conversation_so_far, items).await {
                        Ok(text) => (text, false, None),
                        Err(err) => (err, true, None),
                    },
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == READ_PLAN_TOOL_NAME && agent.reads_plans() {
                (read_latest_plan(conn, s3, session_id).await, false, None)
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
            } else if name.as_str() == crate::background::RUN_BACKGROUND_TASK_TOOL_NAME {
                match crate::background::start_background_task(conn, mqtt, session_id, user_id, agent.agent_type().as_ref(), input).await {
                    Ok((text, card)) => {
                        posted_text = Some(card_text(&card));
                        (text, false, Some(card))
                    }
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == crate::background::LIST_BACKGROUND_TASKS_TOOL_NAME {
                match crate::background::list_background_tasks(conn, session_id, user_id).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == crate::background::CANCEL_BACKGROUND_TASK_TOOL_NAME {
                match crate::background::cancel_background_task(conn, mqtt.map(|(p, _)| p), session_id, user_id, input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == crate::voice::SPEAK_TOOL_NAME {
                match crate::voice::speak(conn, user_id, input).await {
                    Ok((text, block)) => (text, false, Some(block)),
                    Err(err) => (err, true, None),
                }
            } else if let (FIND_TOOLS_TOOL_NAME, Some(catalog)) = (name.as_str(), registry.tool_catalog()) {
                match crate::tool_finder::find_tools(conn, registry, catalog, embedding, &switches, agent, user_id, input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if crate::web::is_web_tool(name) {
                match crate::web::execute(conn, name, input).await {
                    Some(Ok(text)) => (text, false, None),
                    Some(Err(err)) => (err, true, None),
                    None => (format!("unknown tool: {name}"), true, None),
                }
            } else if levels.get(name.as_str()) == Some(&Level::Never) {
                (crate::prompts::MEMBER_TOOL_NEVER_RESULT.to_string(), true, None)
            } else if let Some((_, approved)) = already_decided.iter().find(|(decided_id, _)| decided_id == id) {
                if *approved {
                    match execute_agent_tool(conn, registry, agent, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(outcome) => (outcome.display_text, false, outcome.block),
                        Err(err) => (describe_tool_error(name, input, &err, locale), true, None),
                    }
                } else {
                    (USER_DENIED_RESULT.to_string(), true, None)
                }
            } else if levels.get(name.as_str()) != Some(&Level::Allow)
                && matches!(
                    permissions::check_tool_permission_in_session(conn, user_id, session_id, name, input).await,
                    permissions::PermissionDecision::Deny
                )
            {
                (USER_DENIED_RESULT.to_string(), true, None)
            } else {
                match execute_agent_tool(conn, registry, agent, session_id, agent_session_id, user_id, name, input.clone()).await {
                    Ok(outcome) => (outcome.display_text, false, outcome.block),
                    Err(err) => (describe_tool_error(name, input, &err, locale), true, None),
                }
            };

            // Tool output (command output, provider errors) never carries a secret onward.
            let result_text = nomi_llm::redact::redacted(&result_text);
            log_tool_call(conn, session_id, agent_session_id, agent.agent_type().as_ref(), name, input, &result_text, is_error).await;

            // A personality change shows in the chat's personality menu straight away.
            if !is_error && PERSONALITY_TOOL_NAMES.contains(&name.as_str()) {
                if let Some((publisher, _)) = mqtt {
                    let _ = publisher.publish(session_id, &StreamEnvelope::PersonalityChanged).await;
                }
            }

            // A tool that returns a block (a table, a file change, a Connect card) means it to be
            // seen, whether or not the agent narrates the rest of its tool calls.
            let should_post = name.as_str() == crate::views::SHOW_TOOL_NAME
                || rich_block.is_some()
                || (agent.surfaces_activity()
                    && name.as_str() != COMPLETE_TASK_TOOL_NAME
                    && name.as_str() != DELEGATE_TOOL_NAME
                    && name.as_str() != UPDATE_TODOS_TOOL_NAME
                    && name.as_str() != WRITE_PLAN_TOOL_NAME
                    && name.as_str() != READ_PLAN_TOOL_NAME
                    && name.as_str() != CREATE_REMINDER_TOOL_NAME
                    && name.as_str() != LIST_REMINDERS_TOOL_NAME
                    && name.as_str() != CANCEL_REMINDER_TOOL_NAME
                    && name.as_str() != FIND_TOOLS_TOOL_NAME
                    && !crate::background::BACKGROUND_TOOL_NAMES.contains(&name.as_str())
                    && !crate::web::is_web_tool(name)
                    && !crate::records::RECORD_TOOL_NAMES.contains(&name.as_str()));
            if should_post {
                post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, agent.display_name().as_ref(), posted_text.as_deref().unwrap_or(&result_text), rich_block.as_ref()).await;
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

/// Runs one of the agent's tools that the person may have to approve: its own, or `create_agent`
/// (which the engine runs for Nomi and the supervisor, through the same approval and levels).
#[allow(clippy::too_many_arguments)]
pub(super) async fn execute_agent_tool(
    conn: &mut PoolConnection<Postgres>,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    name: &str,
    input: serde_json::Value,
) -> Result<crate::content_block::ToolOutcome, String> {
    if name == CREATE_AGENT_TOOL_NAME && agent.creates_crew_members() {
        let Some(catalog) = registry.tool_catalog() else { return Err("the tool catalog isn't available".to_string()) };
        return crate::tool_finder::create_agent(conn, registry, catalog, user_id, &input).await.map(crate::content_block::ToolOutcome::text);
    }
    agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input).await
}

/// Templated, non-LLM description of a failed tool call — kept generic in the engine (unlike a
/// success's display_text, which each tool builds itself) since failures never carry a block and
/// the "⚠️ couldn't X — {reason}" shape is the same regardless of which crate the tool lives in.
pub(super) fn describe_tool_error(tool_name: &str, input: &serde_json::Value, result: &str, locale: Locale) -> String {
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
