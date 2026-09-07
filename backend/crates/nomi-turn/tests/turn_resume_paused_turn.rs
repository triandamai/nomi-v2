// End-to-end coverage for the resume subsystem's highest-risk function,
// `resume_paused_turn`/`resume_locked`: a full pause -> approve/deny -> resume cycle driven
// through the real `handle_inbound_message` -> `resume_paused_turn` path (no mocked pause
// state — the paused `agent_sessions.state` shape is produced by the real pre-scan in
// `resolve_tool_batch`, exactly as a live turn would leave it).

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_turn::{handle_inbound_message, resume_paused_turn};

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
use nomi_agent_money::MoneyAgent;
use nomi_realtime::MqttPublisher;
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

/// A single assistant turn declaring MULTIPLE tool_use blocks at once — used to regression-test
/// that a pause on a non-first block preserves every block, not just the one from the pause
/// index onward (see `a_pause_on_a_non_first_block_preserves_every_block_for_resume` below).
fn multi_tool_use_response(calls: &[(&str, &str, serde_json::Value)]) -> LlmResponse {
    LlmResponse {
        content: calls
            .iter()
            .map(|(id, name, input)| ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input: input.clone(), thought_signature: None })
            .collect(),
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

async fn seed_speaker(pool: &PgPool, channel_user_id: &str) -> (Uuid, Uuid) {
    // Returns (user_id, channel_identity_id) plus the personal org + membership that
    // bootstrap_identity_and_session's existing-identity path requires (mirrors the identical
    // helper in turn_subagent_state_machine.rs).
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (name, is_personal) VALUES ('Personal', true) RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(org_id)
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(user_id)
    .bind(channel_user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, identity_id)
}

async fn pending_approval_message_id(pool: &PgPool, session_id: Uuid) -> Uuid {
    sqlx::query_scalar("SELECT (state->>'pending_approval_message_id')::uuid FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn approving_a_paused_tool_call_resumes_the_turn_and_actually_executes_it(pool: PgPool) {
    let (user_id, _identity_id) = seed_speaker(&pool, "tg-1").await;

    // Seed a real transaction so the resumed tool call has something to return — this is what
    // proves the tool actually ran on resume, not just that some placeholder text came back.
    sqlx::query(
        "INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) \
         VALUES ($1, now(), 450, 'food', 'Coffee Shop Purchase')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    // No tool_permission_rules row for list_transactions => the pre-scan always finds "Ask" and
    // pauses the very first time it's about to run — deliberately, so this test exercises a
    // real pause via the real code path instead of hand-writing the paused state's JSON shape.
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),                                                       // intent classification
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})), // pauses for approval
        text_response("Here's your spending"),                                        // resumed turn's final reply
    ]);
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-resume-{}", Uuid::new_v4()));

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None)
        .await
        .unwrap();
    assert_eq!(first.reply, "Waiting for approval.");
    assert_eq!(first.message_id, None);

    let (paused, tool_use_id): (bool, String) = sqlx::query_as(
        "SELECT (state->>'paused_for_approval')::boolean, state->>'pending_tool_use_id' FROM agent_sessions WHERE session_id = $1",
    )
    .bind(first.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(paused);
    assert_eq!(tool_use_id, "t1");

    let message_id = pending_approval_message_id(&pool, first.session_id).await;

    let resumed = resume_paused_turn(&pool, &mqtt, &provider, &embedder, &registry, message_id, "approve", false)
        .await
        .unwrap();

    assert_eq!(resumed.session_id, first.session_id);
    assert_eq!(resumed.reply, "Here's your spending");
    assert!(resumed.message_id.is_some(), "a real message_id must be threaded through on resume, not Uuid::nil()");

    // The pause state must be fully cleared — nothing should look paused after a resume.
    let still_paused: Option<bool> = sqlx::query_scalar("SELECT (state->>'paused_for_approval')::boolean FROM agent_sessions WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(still_paused, None);

    // Proof the previously-pending tool actually executed: a ToolCalled event for
    // list_transactions exists, succeeded, and its result contains the seeded transaction.
    let tool_result: String = sqlx::query_scalar(
        "SELECT payload->>'result' FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled' AND payload->>'tool_name' = 'list_transactions'",
    )
    .bind(first.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(tool_result.contains("Coffee Shop Purchase"), "tool result was: {tool_result}");

    let is_error: bool = sqlx::query_scalar(
        "SELECT (payload->>'is_error')::boolean FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled' AND payload->>'tool_name' = 'list_transactions'",
    )
    .bind(first.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!is_error);

    // The final reply got persisted as a real message row too.
    let persisted_content: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1")
        .bind(resumed.message_id.unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(persisted_content, "Here's your spending");
}

#[sqlx::test(migrations = "../../migrations")]
async fn denying_a_paused_tool_call_skips_execution_but_still_resumes_the_turn(pool: PgPool) {
    let (user_id, _identity_id) = seed_speaker(&pool, "tg-1").await;

    // Even though a transaction exists, a denied tool call must never read it.
    sqlx::query(
        "INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) \
         VALUES ($1, now(), 450, 'food', 'Should Never Be Returned')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("No worries, I won't look that up."),
    ]);
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-resume-deny-{}", Uuid::new_v4()));

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None)
        .await
        .unwrap();
    let message_id = pending_approval_message_id(&pool, first.session_id).await;

    let resumed = resume_paused_turn(&pool, &mqtt, &provider, &embedder, &registry, message_id, "deny", false)
        .await
        .unwrap();

    assert_eq!(resumed.reply, "No worries, I won't look that up.");

    // A ToolCalled event is still logged (log_tool_call runs for every tool_use block
    // regardless of outcome) but it must record a denial, never the tool's real result — proof
    // `MoneyAgent::execute_tool` (the thing that would read `mock_transactions`) was never
    // actually invoked for the denied block.
    let (result, is_error): (String, bool) = sqlx::query_as(
        "SELECT payload->>'result', (payload->>'is_error')::boolean FROM agent_events \
         WHERE session_id = $1 AND event_type = 'ToolCalled' AND payload->>'tool_name' = 'list_transactions'",
    )
    .bind(first.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result, "Denied by your permission rules.");
    assert!(is_error);
    assert!(!result.contains("Should Never Be Returned"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn resuming_a_message_that_is_no_longer_pending_returns_approval_no_longer_pending(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let _ = identity_id;
    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![]);
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-resume-gone-{}", Uuid::new_v4()));

    let result = resume_paused_turn(&pool, &mqtt, &provider, &embedder, &registry, message_id, "approve", false).await;

    assert!(matches!(result, Err(nomi_agent_core::TurnError::ApprovalNoLongerPending)));
}

/// Regression test for a bug where the pause state persisted `tool_use_blocks[index..]` — a
/// slice starting at the block that triggered the pause — instead of the full batch. The
/// pre-scan loop only checks permissions; it never executes anything itself (execution only
/// happens in the second pass, once the pre-scan clears with zero remaining "Ask" blocks), so
/// every block before the paused one has been *scanned* but NOT yet *executed* when the pause
/// happens. Slicing silently dropped those earlier blocks from `agent_sessions.state` forever.
///
/// This constructs a single assistant response with TWO tool_use blocks: `summarize_budget`
/// (pre-approved via a permission rule, so the pre-scan resolves it to Allow and moves on
/// without executing it) followed by `list_transactions` (no rule, resolves to Ask and pauses).
/// With the bug, only the `list_transactions` block would survive into `agent_sessions.state`;
/// `summarize_budget` would be silently lost and would never run, on this resume or any other.
#[sqlx::test(migrations = "../../migrations")]
async fn a_pause_on_a_non_first_block_preserves_every_block_for_resume(pool: PgPool) {
    let (user_id, _identity_id) = seed_speaker(&pool, "tg-1").await;

    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'summarize_budget', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) \
         VALUES ($1, now(), 450, 'marker_category', 'Marker Description')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        multi_tool_use_response(&[
            ("t1", "summarize_budget", serde_json::json!({})),
            ("t2", "list_transactions", serde_json::json!({"limit": 10})),
        ]),
        text_response("Here's everything"),
    ]);
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-resume-multi-{}", Uuid::new_v4()));

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None)
        .await
        .unwrap();
    assert_eq!(first.reply, "Waiting for approval.");

    // The persisted state must contain BOTH blocks, not just the one that paused (t2).
    let state: serde_json::Value = sqlx::query_scalar("SELECT state FROM agent_sessions WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let persisted_blocks = state["tool_use_blocks"].as_array().expect("tool_use_blocks must be a JSON array");
    assert_eq!(persisted_blocks.len(), 2, "both tool_use blocks must survive the pause: {persisted_blocks:?}");
    assert_eq!(persisted_blocks[0]["ToolUse"]["id"], "t1");
    assert_eq!(persisted_blocks[0]["ToolUse"]["name"], "summarize_budget");
    assert_eq!(persisted_blocks[1]["ToolUse"]["id"], "t2");
    assert_eq!(persisted_blocks[1]["ToolUse"]["name"], "list_transactions");

    let message_id = pending_approval_message_id(&pool, first.session_id).await;

    let resumed = resume_paused_turn(&pool, &mqtt, &provider, &embedder, &registry, message_id, "approve", false)
        .await
        .unwrap();
    assert_eq!(resumed.reply, "Here's everything");

    // Proof block 0 (summarize_budget) wasn't silently dropped: it actually ran on resume too,
    // even though only block 1's id ("t2") was the one explicitly decided by the approval.
    let summarize_result: String = sqlx::query_scalar(
        "SELECT payload->>'result' FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled' AND payload->>'tool_name' = 'summarize_budget'",
    )
    .bind(first.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(summarize_result.contains("marker_category"), "summarize_budget result was: {summarize_result}");

    let list_result: String = sqlx::query_scalar(
        "SELECT payload->>'result' FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled' AND payload->>'tool_name' = 'list_transactions'",
    )
    .bind(first.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(list_result.contains("Marker Description"), "list_transactions result was: {list_result}");
}
