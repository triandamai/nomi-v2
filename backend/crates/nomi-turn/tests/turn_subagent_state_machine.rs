
use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_turn::handle_inbound_message;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
use nomi_agent_money::MoneyAgent;
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

async fn seed_speaker(pool: &PgPool, channel_user_id: &str) -> (Uuid, Uuid) {
    // Returns (user_id, channel_identity_id). Also creates the personal org + membership that
    // bootstrap_identity_and_session's existing-identity path requires when it resolves this
    // user's org — without this, handle_inbound_message's own bootstrap step (which always runs
    // first) fails with RowNotFound trying to find a personal org for this "pre-existing" identity.
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

#[sqlx::test(migrations = "../../migrations")]
async fn money_intent_with_no_active_agent_spawns_and_runs_the_money_agent(pool: PgPool) {
    let (user_id, _identity_id) = seed_speaker(&pool, "tg-1").await;
    // Permission-gated by default (Task 5) — allow `list_transactions` so this test still
    // exercises normal tool execution rather than the approval pause.
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'list_transactions', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("Here's your spending"),
    ]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Here's your spending");

    let (agent_type, status): (String, String) =
        sqlx::query_as("SELECT agent_type, status FROM agent_sessions WHERE session_id = $1")
            .bind(outcome.session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(agent_type, "money");
    assert_eq!(status, "active");

    let spawned_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE session_id = $1 AND event_type = 'AgentSpawned'",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(spawned_count, 1);

    let tool_called_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled'",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tool_called_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_active_agent_is_continued_without_reclassifying_intent(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id).bind(identity_id).execute(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![text_response("Sure, here's more info")]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "and rent?", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Sure, here's more info");

    let agent_session_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(agent_session_count, 1); // no new row — the existing one was reused

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(requests.len(), 1); // classification was skipped entirely
    // "financial assistant" only ever appears in the money agent's own system prompt — never in
    // the classify_intent prompt — so this proves the single call was the money-agent turn, not
    // a repeated intent classification.
    assert!(requests[0].system.as_ref().unwrap().contains("financial assistant"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn complete_task_marks_the_agent_session_completed(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active') RETURNING id",
    )
    .bind(session_id).bind(identity_id).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        "complete_task",
        "complete_task",
        serde_json::json!({"status": "completed", "summary": "All set!"}),
    )]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "thanks, that's all", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "All set!");

    let status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "completed");

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM agent_events WHERE agent_session_id = $1 AND event_type = 'AgentCompleted'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload, serde_json::json!({"summary": "All set!"}));
}

#[sqlx::test(migrations = "../../migrations")]
async fn complete_task_with_cancelled_status_marks_the_agent_session_cancelled(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active') RETURNING id",
    )
    .bind(session_id).bind(identity_id).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        "complete_task",
        "complete_task",
        serde_json::json!({"status": "cancelled", "summary": "Nevermind, no problem!"}),
    )]);

    handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "actually nevermind", None)
        .await
        .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE agent_session_id = $1 AND event_type = 'AgentCancelled'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stale_active_agent_is_expired_and_the_message_falls_through_to_chitchat(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let stale_time = chrono::Utc::now() - chrono::Duration::hours(25);
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, last_activity_at) VALUES ($1, $2, 'money', 'active', $3) RETURNING id",
    )
    .bind(session_id).bind(identity_id).bind(stale_time).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        text_response("chitchat"),
        text_response("Hi there!"),
        text_response("NONE"),
    ]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hello again", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Hi there!");

    let status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "expired");

    let expired_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE agent_session_id = $1 AND event_type = 'AgentExpired'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expired_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn after_completion_a_new_money_intent_message_spawns_a_fresh_second_agent_session(pool: PgPool) {
    let (user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    // Permission-gated by default (Task 5) — allow `list_transactions` so this test still
    // exercises normal tool execution rather than the approval pause.
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'list_transactions', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let old_agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, ended_at) VALUES ($1, $2, 'money', 'completed', now()) RETURNING id",
    )
    .bind(session_id).bind(identity_id).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("Here's your spending again"),
    ]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "what about now?", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Here's your spending again");

    let total_agent_sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total_agent_sessions, 2);

    let new_active_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM agent_sessions WHERE session_id = $1 AND status = 'active'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_ne!(new_active_id, old_agent_session_id);
}
