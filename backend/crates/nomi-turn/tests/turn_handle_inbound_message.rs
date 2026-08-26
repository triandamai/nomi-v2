
use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, LlmRole, StopReason};
use nomi_turn::handle_inbound_message;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
use nomi_agent_money::MoneyAgent;
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn new_sender_gets_bootstrapped_and_receives_a_chitchat_reply(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("Hi! How can I help?"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Hi! How can I help?");

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(outcome.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 2); // inbound + reply

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
    let org_count: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations").fetch_one(&pool).await.unwrap();
    assert_eq!(org_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn existing_sender_reuses_identity_and_session_across_two_calls(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "first", None)
        .await
        .unwrap();
    let second = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "second", None)
        .await
        .unwrap();

    assert_eq!(first.session_id, second.session_id);

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 4); // 2 inbound + 2 replies

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn inbound_message_is_durable_even_when_the_provider_call_fails(pool: PgPool) {
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let result = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await;
    assert!(result.is_err());

    let (content,): (String,) = sqlx::query_as(
        "SELECT content FROM messages m JOIN sessions s ON m.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "hello");

    let event_type: String = sqlx::query_scalar(
        "SELECT event_type FROM agent_events e JOIN sessions s ON e.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "TurnFailed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_active_agent_session_does_not_block_the_chitchat_fallback_in_this_slice(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("still chatting"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hi", None)
        .await
        .unwrap();

    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT ci.id FROM channel_identities ci WHERE ci.channel = 'telegram' AND ci.channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(first.session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let second = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "still there?", None)
        .await
        .unwrap();

    assert_eq!(second.reply, "still chatting");
}

#[sqlx::test(migrations = "../../migrations")]
async fn concurrent_messages_for_the_same_session_are_serialized(pool: PgPool) {
    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "bootstrap", None)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok")).with_delay(Duration::from_millis(200));

    let call1 = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "first", None);
    let call2 = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "second", None);
    let (result1, result2) = tokio::join!(call1, call2);
    result1.unwrap();
    result2.unwrap();

    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT m.sender_channel_identity_id, m.content FROM messages m \
         JOIN sessions s ON m.session_id = s.id \
         WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1' \
         ORDER BY m.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // bootstrap turn (2 rows) + two serialized turns (2 rows each) = 6, never interleaved.
    assert_eq!(rows.len(), 6);
    let (first_inbound, first_reply, second_inbound, second_reply) =
        (&rows[2], &rows[3], &rows[4], &rows[5]);
    assert!(first_inbound.0.is_some());
    assert!(first_inbound.1 == "first" || first_inbound.1 == "second");
    assert!(first_reply.0.is_none());
    assert_ne!(first_inbound.1, second_inbound.1);
    assert!(second_inbound.0.is_some());
    assert!(second_reply.0.is_none());
}

// The chitchat path's history fetch (`fetch_recent_messages` in `backend/src/turn/mod.rs`) is
// shared with the money-agent path and is no longer chitchat's own code (it used to be
// duplicated inline in the now-deleted `turn/chitchat.rs`, ported to `nomi-agent-chitchat` in
// Task 8). Since `nomi_agent_core::run_agent_turn` takes an already-built message list rather
// than fetching it itself, this behavior can no longer be exercised by calling into
// `ChitchatAgent` directly — it only shows up by driving the real `handle_inbound_message`
// entrypoint, which is what these two tests (ported from the old `turn_chitchat.rs`'s
// `keeps_only_the_last_20_messages_ordered_oldest_first` and
// `maps_sender_presence_to_role_correctly`) do.
#[sqlx::test(migrations = "../../migrations")]
async fn chitchat_turn_keeps_only_the_last_20_messages_ordered_oldest_first(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    let bootstrap = handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "bootstrap", None)
        .await
        .unwrap();

    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM channel_identities WHERE channel = 'telegram' AND channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    for i in 0..25 {
        sqlx::query(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(bootstrap.session_id)
        .bind(identity_id)
        .bind(format!("seq-{i}"))
        .bind(base + chrono::Duration::milliseconds(i))
        .execute(&pool)
        .await
        .unwrap();
    }

    let provider = FakeLlmProvider::success(canned_response("ok"));
    handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "latest", None)
        .await
        .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    // requests[0] is the intent classifier's own single-message request (every inbound message
    // is reclassified since chitchat never spawns an agent_session to "continue"); requests[1]
    // is the actual chitchat turn, built from `fetch_recent_messages`.
    let sent = &requests[1];
    assert_eq!(sent.messages.len(), 20);
    // The 25 seeded messages (seq-0..seq-24) plus this call's own "latest" inbound message (which
    // is persisted before this LLM call happens) make the most-recent-20 window seq-6..seq-24
    // followed by "latest".
    for (offset, message) in sent.messages.iter().take(19).enumerate() {
        let expected_seq = 6 + offset;
        match &message.content[0] {
            ContentBlock::Text { text } => assert_eq!(text, &format!("seq-{expected_seq}")),
            _ => panic!("expected a text block"),
        }
    }
    match &sent.messages[19].content[0] {
        ContentBlock::Text { text } => assert_eq!(text, "latest"),
        _ => panic!("expected a text block"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn chitchat_turn_maps_sender_presence_to_role_correctly(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let bootstrap_provider = FakeLlmProvider::success(canned_response("hello back"));
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hi", None)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok"));
    handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "latest", None)
        .await
        .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    // requests[0] is the intent classifier's own single-message request; requests[1] is the
    // actual chitchat turn, built from `fetch_recent_messages`.
    let sent = &requests[1];
    // sender present (the user's own messages) maps to User; sender absent (the assistant's own
    // replies, sender_channel_identity_id = NULL) maps to Assistant.
    assert_eq!(sent.messages[0].role, LlmRole::User); // "hi"
    assert_eq!(sent.messages[1].role, LlmRole::Assistant); // "hello back"
    assert_eq!(sent.messages[2].role, LlmRole::User); // "latest"
}
