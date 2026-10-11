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

#[sqlx::test(migrations = "../../migrations")]
async fn a_renamed_model_keeps_its_usage_together_under_its_new_name(pool: PgPool) {
    let ana = person(&pool, "UTC").await;
    let model = priced_model(&pool, ana).await;
    let inner = Arc::new(FakeLlmProvider::sequence(vec![reply(100, 10), reply(200, 20)]));
    let provider = MeteredProvider::new(inner, pool.clone(), ana, tag(Some(model), false));
    call(&provider).await;
    sqlx::query("UPDATE admin_llm_models SET label = 'Nomi Fast' WHERE id = $1").bind(model).execute(&pool).await.unwrap();
    // The tag a later turn builds carries the new name.
    let inner = Arc::new(FakeLlmProvider::success(reply(200, 20)));
    let renamed = ModelTag { label: "Nomi Fast".into(), ..tag(Some(model), false) };
    call(&MeteredProvider::new(inner, pool.clone(), ana, renamed)).await;

    let usage = nomi_usage::month(&pool, ana, None).await.unwrap();
    assert_eq!(usage.by_model.len(), 1, "{:?}", usage.by_model);
    assert_eq!(usage.by_model[0].admin_model_id, Some(model));
    assert_eq!(usage.by_model[0].label, "Nomi Fast");
    assert_eq!((usage.by_model[0].tokens, usage.by_model[0].calls), (330, 2));
}

#[sqlx::test(migrations = "../../migrations")]
async fn switching_models_shows_each_one_and_a_deleted_model_keeps_its_rows(pool: PgPool) {
    let ana = person(&pool, "UTC").await;
    let gemini = priced_model(&pool, ana).await;
    let deepseek: Uuid = sqlx::query_scalar(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by) \
         VALUES ('DeepSeek', 'deepseek', 'deepseek-chat', '\\x00', false, $1) RETURNING id",
    )
    .bind(ana)
    .fetch_one(&pool)
    .await
    .unwrap();
    call(&MeteredProvider::new(Arc::new(FakeLlmProvider::success(reply(100, 10))), pool.clone(), ana, tag(Some(gemini), false))).await;
    let deepseek_tag = ModelTag { admin_model_id: Some(deepseek), own_key: false, label: "DeepSeek".into(), provider: "deepseek".into(), model_id: "deepseek-chat".into() };
    call(&MeteredProvider::new(Arc::new(FakeLlmProvider::success(reply(500, 50))), pool.clone(), ana, deepseek_tag)).await;

    let usage = nomi_usage::month(&pool, ana, None).await.unwrap();
    let labels: Vec<&str> = usage.by_model.iter().map(|m| m.label.as_str()).collect();
    assert_eq!(labels, ["DeepSeek", "Nomi Standard"]);

    sqlx::query("DELETE FROM admin_llm_models WHERE id = $1").bind(deepseek).execute(&pool).await.unwrap();
    let report = nomi_usage::admin::report(&pool, None, None).await.unwrap();
    let gone = report.models.iter().find(|m| m.admin_model_id == Some(deepseek)).expect("a deleted model's usage stays");
    assert!(gone.deleted);
    assert_eq!((gone.label.as_str(), gone.tokens, gone.people), ("DeepSeek", 550, 1));
    assert_eq!(report.chat.tokens, 660);
    assert_eq!(report.chat.people, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn embeddings_are_metered_with_the_providers_count_or_an_estimate(pool: PgPool) {
    use nomi_embedding::EmbeddingProvider;
    let ana = person(&pool, "UTC").await;
    let fake: Arc<dyn EmbeddingProvider> = Arc::new(nomi_embedding::fake::FakeEmbeddingProvider);
    let metered = nomi_usage::MeteredEmbeddingProvider::new(fake, pool.clone(), Some(ana));
    metered.embed("twelve chars").await.unwrap();
    metered.embed_for_query("hi").await.unwrap();
    nomi_usage::embedding::record(&pool, None, "openai", "text-embedding-3-small", 42, false).await.unwrap();

    let report = nomi_usage::admin::report(&pool, None, Some(("openai", "text-embedding-3-small"))).await.unwrap();
    // The fake provider reports no count: 12 chars ≈ 3 tokens, 2 chars ≈ 1.
    assert_eq!((report.embedding.tokens, report.embedding.calls, report.embedding.estimated_calls), (46, 3, 2));
    let openai = report.embedding_models.iter().find(|m| m.provider == "openai").unwrap();
    assert!(openai.current);
    assert_eq!((openai.tokens, openai.estimated_calls), (42, 0));
    assert!(!report.embedding_models.iter().find(|m| m.provider == "fake").unwrap().current);
    assert_eq!(report.by_day.iter().map(|d| d.embedding_tokens).sum::<i64>(), 46);
    // Embeddings never count toward a person's chat allowance.
    assert_eq!(nomi_usage::brief(&pool, ana).await.unwrap().tokens_used, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn cached_input_is_priced_at_the_cached_price(pool: PgPool) {
    use nomi_llm::{LlmEventStream, StreamEvent};
    let ana = person(&pool, "UTC").await;
    let model = priced_model(&pool, ana).await;
    sqlx::query("UPDATE admin_llm_models SET cached_input_usd_per_mtok = 0.3 WHERE id = $1").bind(model).execute(&pool).await.unwrap();

    /// Streams a reply whose input was mostly served from the prompt cache.
    struct Cached;
    #[async_trait::async_trait]
    impl LlmProvider for Cached {
        async fn complete_stream(&self, _request: LlmRequest) -> Result<LlmEventStream, nomi_llm::LlmError> {
            Ok(Box::pin(futures_util::stream::iter(vec![
                Ok(StreamEvent::CacheUsage { cached_input_tokens: 800_000 }),
                Ok(StreamEvent::Done { stop_reason: StopReason::EndTurn, input_tokens: 1_000_000, output_tokens: 0 }),
            ])))
        }
    }
    call(&MeteredProvider::new(Arc::new(Cached), pool.clone(), ana, tag(Some(model), false))).await;

    let (cached, cost): (i32, f64) = sqlx::query_as("SELECT cached_input_tokens, cost_usd::float8 FROM llm_usage").fetch_one(&pool).await.unwrap();
    assert_eq!(cached, 800_000);
    // 0.2M uncached at $3 + 0.8M cached at $0.30
    assert!((cost - 0.84).abs() < 1e-9, "{cost}");
    // Every input token still counts toward the allowance.
    assert_eq!(nomi_usage::brief(&pool, ana).await.unwrap().tokens_used, 1_000_000);
}
