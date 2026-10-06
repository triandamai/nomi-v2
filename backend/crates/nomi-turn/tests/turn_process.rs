use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_realtime::MqttPublisher;
use nomi_turn::ingest::ingest_inbound_message;
use nomi_turn::process_turn;

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
async fn process_turn_produces_a_reply_for_an_already_ingested_message(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();

    let provider = FakeLlmProvider::success(canned_response("hi there"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
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
        None,
        &provider,
        &embedder,
        &registry,
        &catalog,
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

#[sqlx::test(migrations = "../../migrations")]
async fn process_turn_records_a_turn_failed_event_on_llm_failure(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();

    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-process-{}", Uuid::new_v4()));

    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = process_turn(
        &pool,
        &mqtt,
        None,
        &provider,
        &embedder,
        &registry,
        &catalog,
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

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_turn_leaves_no_agent_looking_busy(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "plan my day", None).await.unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    // A specialist was mid-thought in this chat when the model failed (Gemini's 503).
    let planning: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, current_phase) \
         VALUES ($1, $2, 'planning', 'active', 'thinking') RETURNING id",
    )
    .bind(ingested.session_id)
    .bind(ingested.sender_channel_identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let provider = FakeLlmProvider::failure("gemini returned 503 Service Unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-settle-{}", Uuid::new_v4()));
    let result = process_turn(
        &pool, &mqtt, None, &provider, &embedder, &registry, &catalog, ingested.turn_job_id, ingested.session_id,
        ingested.sender_channel_identity_id, user_id, "plan my day",
    )
    .await;

    assert!(result.is_err());
    let phase: String = sqlx::query_scalar("SELECT current_phase FROM agent_sessions WHERE id = $1").bind(planning).fetch_one(&pool).await.unwrap();
    assert_eq!(phase, "waiting");
    // And the person is told what happened.
    let notices: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NULL")
        .bind(ingested.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(notices, 1);
}
