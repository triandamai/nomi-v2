mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::realtime::MqttPublisher;
use nomi_orchestrator::turn::ingest::ingest_inbound_message;
use nomi_orchestrator::turn::process_turn;

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
async fn process_turn_produces_a_reply_for_an_already_ingested_message(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();

    let provider = FakeLlmProvider::success(canned_response("hi there"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-process-{}", Uuid::new_v4()));

    let user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM channel_identities WHERE id = $1",
    )
    .bind(ingested.sender_channel_identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let outcome = process_turn(
        &pool,
        &mqtt,
        &provider,
        &embedder,
        ingested.turn_job_id,
        ingested.session_id,
        ingested.sender_channel_identity_id,
        user_id,
        "hello",
    )
    .await
    .unwrap();

    assert_eq!(outcome.reply, "hi there");

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(ingested.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 2); // ingest's inbound insert + process_turn's reply insert
}

#[sqlx::test]
async fn process_turn_records_a_turn_failed_event_on_llm_failure(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();

    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-process-{}", Uuid::new_v4()));

    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = process_turn(
        &pool,
        &mqtt,
        &provider,
        &embedder,
        ingested.turn_job_id,
        ingested.session_id,
        ingested.sender_channel_identity_id,
        user_id,
        "hello",
    )
    .await;

    assert!(result.is_err());

    let event_type: String = sqlx::query_scalar(
        "SELECT event_type FROM agent_events WHERE session_id = $1",
    )
    .bind(ingested.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "TurnFailed");
}
