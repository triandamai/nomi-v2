use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::memory::retrieve_relevant_memories;

fn make_embedding(first: f32, second: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v[1] = second;
    v
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn seed_memory(pool: &PgPool, user_id: Uuid, content: &str, embedding: &[f32], weight: f64) {
    sqlx::query("INSERT INTO memory_items (user_id, content, embedding, weight) VALUES ($1, $2, $3::vector, $4)")
        .bind(user_id)
        .bind(content)
        .bind(to_vector_literal(embedding))
        .bind(weight)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn retrieves_memories_ordered_by_weighted_similarity(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory(&pool, user_id, "close match", &make_embedding(1.0, 0.0), 1.0).await;
    seed_memory(&pool, user_id, "far match", &make_embedding(0.0, 1.0), 1.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(0.9, 0.1);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 5).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].content, "close match");
    assert_eq!(results[1].content, "far match");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_high_weight_can_outrank_a_higher_raw_similarity(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory(&pool, user_id, "closer but low weight", &make_embedding(1.0, 0.0), 0.5).await;
    seed_memory(&pool, user_id, "farther but high weight", &make_embedding(0.7, 0.3), 5.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 5).await.unwrap();

    assert_eq!(results[0].content, "farther but high weight");
}

#[sqlx::test(migrations = "../../migrations")]
async fn respects_the_limit_parameter(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    for i in 0..3 {
        seed_memory(&pool, user_id, &format!("memory-{i}"), &make_embedding(1.0 - (i as f32 * 0.1), 0.0), 1.0).await;
    }

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 2).await.unwrap();

    assert_eq!(results.len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_returns_memories_for_the_queried_user(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    seed_memory(&pool, user_a, "user a's memory", &make_embedding(1.0, 0.0), 1.0).await;
    seed_memory(&pool, user_b, "user b's memory", &make_embedding(1.0, 0.0), 1.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_a, &query, 5).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "user a's memory");
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_empty_when_the_user_has_no_memories(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 5).await.unwrap();

    assert!(results.is_empty());
}
