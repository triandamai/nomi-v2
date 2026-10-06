// Its own test binary: it sets TURN_TIME_LIMIT_SECS for the whole process.
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog, TurnError};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_realtime::MqttPublisher;
use nomi_turn::ingest::ingest_inbound_message;
use nomi_turn::process_turn;

use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn reply(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.to_string() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stalled_model_fails_the_turn_at_the_time_limit_and_frees_the_chat(pool: PgPool) {
    std::env::set_var("TURN_TIME_LIMIT_SECS", "1");
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-limit-{}", Uuid::new_v4()));

    // The model never answers in time.
    let stalled = FakeLlmProvider::success(reply("too late")).with_delay(Duration::from_secs(30));
    let started = Instant::now();
    let result = process_turn(
        &pool, &mqtt, None, &stalled, &embedder, &registry, &catalog, ingested.turn_job_id, ingested.session_id,
        ingested.sender_channel_identity_id, user_id, "hello",
    )
    .await;

    assert!(matches!(result, Err(TurnError::TimedOut(_))), "{:?}", result.err());
    assert!(started.elapsed() < Duration::from_secs(10), "stopped at the limit, not when the model gave up");
    let failed: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_events WHERE session_id = $1 AND event_type = 'TurnFailed'")
        .bind(ingested.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(failed, 1);

    // The chat isn't left locked: the next message is answered.
    let next = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "still there?", None).await.unwrap();
    let quick = FakeLlmProvider::success(reply("yes"));
    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        process_turn(&pool, &mqtt, None, &quick, &embedder, &registry, &catalog, next.turn_job_id, next.session_id, next.sender_channel_identity_id, user_id, "still there?"),
    )
    .await
    .expect("the session lock was released")
    .unwrap();
    assert_eq!(outcome.reply, "yes");
}
