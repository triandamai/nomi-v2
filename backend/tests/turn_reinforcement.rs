use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::memory::{reinforce, ReinforcementSignal};

fn zero_embedding_literal() -> String {
    format!("[{}]", vec!["0.0"; 1536].join(","))
}

async fn seed_memory_linked_to_a_message(pool: &PgPool, weight: f64) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id).fetch_one(pool).await.unwrap();
    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'reply') RETURNING id")
        .bind(session_id).fetch_one(pool).await.unwrap();
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding, weight) VALUES ($1, 'fact', $2::vector, $3) RETURNING id",
    )
    .bind(user_id)
    .bind(zero_embedding_literal())
    .bind(weight)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
        .bind(message_id)
        .bind(memory_id)
        .execute(pool)
        .await
        .unwrap();
    (message_id, memory_id)
}

#[sqlx::test]
async fn positive_signal_increases_weight(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 1.0).await;

    reinforce(&pool, message_id, ReinforcementSignal::Positive).await.unwrap();

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 1.2).abs() < 1e-9);
}

#[sqlx::test]
async fn negative_signal_decreases_weight(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 1.0).await;

    reinforce(&pool, message_id, ReinforcementSignal::Negative).await.unwrap();

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 0.8).abs() < 1e-9);
}

#[sqlx::test]
async fn weight_is_clamped_to_the_upper_bound_under_repeated_positive_reinforcement(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 4.9).await;

    for _ in 0..10 {
        reinforce(&pool, message_id, ReinforcementSignal::Positive).await.unwrap();
    }

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 5.0).abs() < 1e-9);
}

#[sqlx::test]
async fn weight_is_clamped_to_the_lower_bound_under_repeated_negative_reinforcement(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 0.15).await;

    for _ in 0..10 {
        reinforce(&pool, message_id, ReinforcementSignal::Negative).await.unwrap();
    }

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 0.1).abs() < 1e-9);
}

#[sqlx::test]
async fn a_message_with_no_linked_memories_is_a_no_op(pool: PgPool) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id).fetch_one(&pool).await.unwrap();
    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'reply') RETURNING id")
        .bind(session_id).fetch_one(&pool).await.unwrap();

    let result = reinforce(&pool, message_id, ReinforcementSignal::Positive).await;
    assert!(result.is_ok());
}
