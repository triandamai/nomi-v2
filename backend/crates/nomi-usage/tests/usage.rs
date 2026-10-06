use std::sync::Arc;

use futures_util::StreamExt;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmProvider, LlmRequest, LlmResponse, StopReason};
use nomi_test_support::FakeLlmProvider;
use nomi_usage::{MeteredProvider, ModelTag};

async fn person(pool: &PgPool, timezone: &str) -> Uuid {
    let id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, $2)").bind(id).bind(timezone).execute(pool).await.unwrap();
    id
}

/// An admin model at $3 / $15 per million input / output tokens.
async fn priced_model(pool: &PgPool, admin: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by, input_usd_per_mtok, output_usd_per_mtok) \
         VALUES ('Nomi Standard', 'anthropic', 'claude-x', '\\x00', true, $1, 3, 15) RETURNING id",
    )
    .bind(admin)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn reply(input_tokens: u32, output_tokens: u32) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: "hi".into() }], stop_reason: StopReason::EndTurn, input_tokens, output_tokens }
}

fn request() -> LlmRequest {
    LlmRequest { system: None, messages: vec![], tools: vec![], max_tokens: 100, enable_reasoning: false, reasoning_effort: Default::default() }
}

/// Runs one call through the metered provider, reading the whole stream as the engine does.
async fn call(provider: &MeteredProvider) {
    let mut events = provider.complete_stream(request()).await.unwrap();
    while events.next().await.is_some() {}
}

fn tag(admin_model_id: Option<Uuid>, own_key: bool) -> ModelTag {
    ModelTag { admin_model_id, own_key, label: "Nomi Standard".into(), provider: "anthropic".into(), model_id: "claude-x".into() }
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_call_is_metered_and_priced_by_its_model(pool: PgPool) {
    let ana = person(&pool, "UTC").await;
    let model = priced_model(&pool, ana).await;
    let inner = Arc::new(FakeLlmProvider::sequence(vec![reply(1_000_000, 100_000), reply(200_000, 0)]));
    let provider = MeteredProvider::new(inner, pool.clone(), ana, tag(Some(model), false));
    call(&provider).await;
    call(&provider).await;

    let usage = nomi_usage::month(&pool, ana, None).await.unwrap();
    assert_eq!(usage.tokens_used, 1_300_000);
    assert_eq!((usage.input_tokens, usage.output_tokens, usage.calls), (1_200_000, 100_000, 2));
    // 1.2M in at $3 + 0.1M out at $15
    assert!((usage.spend_usd - 5.1).abs() < 1e-9, "{}", usage.spend_usd);
    assert_eq!(usage.by_model.len(), 1);
    assert_eq!(usage.by_model[0].label, "Nomi Standard");
    assert_eq!(usage.by_day.len(), 1);
    assert_eq!(nomi_usage::brief(&pool, ana).await.unwrap().tokens_used, 1_300_000);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_persons_own_key_counts_as_usage_but_not_spend_or_allowance(pool: PgPool) {
    let ana = person(&pool, "UTC").await;
    let inner = Arc::new(FakeLlmProvider::success(reply(5_000, 1_000)));
    call(&MeteredProvider::new(inner, pool.clone(), ana, tag(None, true))).await;

    let usage = nomi_usage::month(&pool, ana, None).await.unwrap();
    assert_eq!((usage.tokens_used, usage.own_key_tokens, usage.spend_usd), (0, 6_000, 0.0));
    assert_eq!(usage.by_model[0].source, "own_key");
}

#[sqlx::test(migrations = "../../migrations")]
async fn months_follow_the_persons_timezone_and_stay_private(pool: PgPool) {
    let ana = person(&pool, "Asia/Jakarta").await;
    let budi = person(&pool, "UTC").await;
    // 20:00 UTC on 31 January is already 1 February in Jakarta (UTC+7).
    sqlx::query(
        "INSERT INTO llm_usage (user_id, source, model_label, provider, model_id, input_tokens, output_tokens, created_at) \
         VALUES ($1, 'nomi', 'M', 'fake', 'm', 10, 5, '2026-01-31T20:00:00Z')",
    )
    .bind(ana)
    .execute(&pool)
    .await
    .unwrap();

    let february = nomi_usage::month(&pool, ana, Some("2026-02")).await.unwrap();
    assert_eq!((february.month.as_str(), february.tokens_used), ("2026-02", 15));
    assert_eq!(february.by_day[0].date, "2026-02-01");
    assert_eq!(nomi_usage::month(&pool, ana, Some("2026-01")).await.unwrap().tokens_used, 0);
    assert_eq!(nomi_usage::month(&pool, budi, Some("2026-02")).await.unwrap().tokens_used, 0, "only your own usage");
    // Anything that isn't YYYY-MM falls back to this month.
    let current = nomi_usage::month(&pool, ana, Some("Feb")).await.unwrap();
    assert_eq!(current.month, current.current_month);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_call_records_nothing(pool: PgPool) {
    let ana = person(&pool, "UTC").await;
    let provider = MeteredProvider::new(Arc::new(FakeLlmProvider::failure("down")), pool.clone(), ana, tag(None, false));
    assert!(provider.complete_stream(request()).await.is_err());
    assert_eq!(nomi_usage::month(&pool, ana, None).await.unwrap().calls, 0);
}
