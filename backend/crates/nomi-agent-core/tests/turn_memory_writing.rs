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

#[sqlx::test(migrations = "../../migrations")]
async fn the_same_fact_said_twice_strengthens_one_memory_instead_of_duplicating_it(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("User is vegetarian"));
    let embedder = FakeEmbeddingProvider::success(vec![0.2; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I don't eat meat", "Noted!").await;
    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "Remember, no meat", "Got it!").await;

    let rows: Vec<f64> = sqlx::query_scalar("SELECT weight FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0] > 1.0);
}

fn unit(index: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[index] = 1.0;
    v
}

async fn seed_known(pool: &PgPool, user_id: Uuid, content: &str, kind: &str, embedding: &[f32], weight: f64) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, kind, embedding, weight, embedding_provider, embedding_model) \
         VALUES ($1, $2, $3, $4::vector, $5, 'fake', 'fake-model') RETURNING id",
    )
    .bind(user_id)
    .bind(content)
    .bind(kind)
    .bind(format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")))
    .bind(weight)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_structured_fact_is_stored_short_with_its_kind_and_source(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Home', true) RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', 'c', $2) RETURNING id")
        .bind(org)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity: Uuid = sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', 'x') RETURNING id")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let source: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, 'I stopped eating meat') RETURNING id")
        .bind(session_id)
        .bind(identity)
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response(r#"{"action":"add","kind":"preference","text":"Vegetarian"}"#));
    let embedder = FakeEmbeddingProvider::success(unit(0));

    nomi_agent_core::memory::extract_and_store_memory_in(&mut conn, &llm, &embedder, user_id, Some(session_id), "I stopped eating meat", "Noted!").await;

    let (content, kind, from): (String, String, Option<Uuid>) =
        sqlx::query_as("SELECT content, kind, source_message_id FROM memory_items WHERE user_id = $1").bind(user_id).fetch_one(&pool).await.unwrap();
    assert_eq!((content.as_str(), kind.as_str(), from), ("Vegetarian", "preference", Some(source)));
    // The extractor saw no related memories, and was asked for JSON.
    let request = &llm.received_requests.lock().unwrap()[0];
    assert!(request.system.as_ref().unwrap().contains("JSON only"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_correction_replaces_the_old_memory_and_keeps_its_strength(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let old = seed_known(&pool, user_id, "Lives in Jakarta", "fact", &unit(0), 2.0).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response(r#"{"action":"update","target":1,"text":"Lives in Bandung"}"#));
    // The new wording embeds right next to the old one, as real corrections do.
    let embedder = FakeEmbeddingProvider::success(unit(0));

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I moved to Bandung last month", "Congrats on the move!").await;

    let prompt = match &llm.received_requests.lock().unwrap()[0].messages[0].content[0] {
        ContentBlock::Text { text } => text.clone(),
        _ => panic!("text"),
    };
    assert!(prompt.contains("1. (fact) Lives in Jakarta"), "{prompt}");

    let live: Vec<(String, String, f64)> =
        sqlx::query_as("SELECT content, kind, weight FROM memory_items WHERE user_id = $1 AND archived_at IS NULL").bind(user_id).fetch_all(&pool).await.unwrap();
    assert_eq!(live, vec![("Lives in Bandung".to_string(), "fact".to_string(), 2.0)]);
    let (archived, replaced_by): (bool, Option<Uuid>) =
        sqlx::query_as("SELECT archived_at IS NOT NULL, superseded_by FROM memory_items WHERE id = $1").bind(old).fetch_one(&pool).await.unwrap();
    assert!(archived);
    assert!(replaced_by.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn something_no_longer_true_is_retired(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let old = seed_known(&pool, user_id, "Training for a marathon", "goal", &unit(0), 1.0).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response(r#"{"action":"delete","target":1}"#));
    let embedder = FakeEmbeddingProvider::success(unit(0));

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I gave up on the marathon", "That's okay!").await;

    let archived: bool = sqlx::query_scalar("SELECT archived_at IS NOT NULL FROM memory_items WHERE id = $1").bind(old).fetch_one(&pool).await.unwrap();
    assert!(archived);
}
