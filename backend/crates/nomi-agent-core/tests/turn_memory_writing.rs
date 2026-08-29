use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_agent_core::memory::extract_and_store_memory;

use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

fn extraction_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 5,
        output_tokens: 3,
    }
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn none_extraction_stores_nothing(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("NONE"));
    let embedder = FakeEmbeddingProvider::success(vec![0.1; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "hi", "hello").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_real_extracted_fact_is_embedded_and_stored(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("User is vegetarian"));
    let embedder = FakeEmbeddingProvider::success(vec![0.2; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I don't eat meat", "Noted!").await;

    let (content, provider, model): (String, String, String) = sqlx::query_as(
        "SELECT content, embedding_provider, embedding_model FROM memory_items WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "User is vegetarian");
    assert_eq!(provider, "fake");
    assert_eq!(model, "fake-model");
}

#[sqlx::test(migrations = "../../migrations")]
async fn extraction_llm_failure_stores_nothing(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::failure("provider down");
    let embedder = FakeEmbeddingProvider::success(vec![0.1; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "hi", "hello").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn embedding_failure_after_a_good_extraction_stores_nothing(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("User is vegetarian"));
    let embedder = FakeEmbeddingProvider::failure("embeddings unavailable");

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I don't eat meat", "Noted!").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
