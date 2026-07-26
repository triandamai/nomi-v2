mod support;

use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::handle_inbound_message;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test]
async fn new_sender_gets_bootstrapped_and_receives_a_chitchat_reply(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("Hi! How can I help?"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hello")
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

#[sqlx::test]
async fn existing_sender_reuses_identity_and_session_across_two_calls(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let first = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "first")
        .await
        .unwrap();
    let second = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "second")
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

#[sqlx::test]
async fn inbound_message_is_durable_even_when_the_provider_call_fails(pool: PgPool) {
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let result = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hello")
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

#[sqlx::test]
async fn an_active_agent_session_does_not_block_the_chitchat_fallback_in_this_slice(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("still chatting"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let first = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hi")
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

    let second = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "still there?")
        .await
        .unwrap();

    assert_eq!(second.reply, "still chatting");
}

#[sqlx::test]
async fn concurrent_messages_for_the_same_session_are_serialized(pool: PgPool) {
    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "bootstrap")
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok")).with_delay(Duration::from_millis(200));

    let call1 = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "first");
    let call2 = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "second");
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
