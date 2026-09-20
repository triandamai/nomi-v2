use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::AgentRegistry;
use nomi_agent_core::{LogOnlyDelivery, NotificationDelivery};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

async fn insert_job(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
    run_at_offset: &str,
    recurrence: Option<&str>,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, recurrence) \
         VALUES ($1, $2, 'chitchat', 'chitchat', 'take a bath', 'Remind the user to take a bath.', now() + $3::interval, $4) \
         RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(run_at_offset)
    .bind(recurrence)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_does_not_claim_a_future_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    insert_job(&pool, session_id, user_id, "1 hour", None).await;

    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();

    assert!(claimed.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_claims_a_due_job_and_sets_claimed_at(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let id = insert_job(&pool, session_id, user_id, "-1 hour", None).await;

    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();

    assert!(claimed.is_some());
    assert_eq!(claimed.unwrap().id, id);
    let claimed_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar("SELECT claimed_at FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(claimed_at.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_does_not_reclaim_an_already_claimed_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    insert_job(&pool, session_id, user_id, "-1 hour", None).await;

    let first = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();
    assert!(first.is_some());
    let second = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();

    assert!(second.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_one_time_job_is_marked_completed_after_a_successful_fire(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let id = insert_job(&pool, session_id, user_id, "-1 hour", None).await;
    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap().unwrap();

    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![ContentBlock::Text { text: "Time for a bath!".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(nomi_agent_chitchat::ChitchatAgent)]);
    let notification: std::sync::Arc<dyn NotificationDelivery> = std::sync::Arc::new(LogOnlyDelivery);

    nomi_server::scheduler_worker::process_claimed_job(&pool, None, None, &provider, &embedding_provider, &registry, notification.as_ref(), claimed)
        .await;

    let (status, last_fired_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, last_fired_at FROM scheduled_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "completed");
    assert!(last_fired_at.is_some());

    let message_content: String = sqlx::query_scalar("SELECT content FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_content, "Time for a bath!");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_recurring_job_has_its_run_at_advanced_and_claim_cleared_after_a_fire(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let id = insert_job(&pool, session_id, user_id, "-1 hour", Some("daily")).await;
    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap().unwrap();
    let run_at_before_fire = claimed.run_at;

    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![ContentBlock::Text { text: "Time for a bath!".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(nomi_agent_chitchat::ChitchatAgent)]);
    let notification: std::sync::Arc<dyn NotificationDelivery> = std::sync::Arc::new(LogOnlyDelivery);

    nomi_server::scheduler_worker::process_claimed_job(&pool, None, None, &provider, &embedding_provider, &registry, notification.as_ref(), claimed)
        .await;

    let (status, claimed_at, run_at): (String, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as("SELECT status, claimed_at, run_at FROM scheduled_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "active", "a recurring job stays active after firing");
    assert!(claimed_at.is_none(), "claimed_at must be cleared so the next poll can claim it again");
    assert_eq!(run_at, run_at_before_fire + chrono::Duration::days(1));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_target_agent_type_cancels_the_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'chitchat', 'not_a_real_agent', 'x', 'y', now() - interval '1 hour')",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap().unwrap();
    let id = claimed.id;

    let provider = FakeLlmProvider::sequence(vec![]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(nomi_agent_chitchat::ChitchatAgent)]);
    let notification: std::sync::Arc<dyn NotificationDelivery> = std::sync::Arc::new(LogOnlyDelivery);

    nomi_server::scheduler_worker::process_claimed_job(&pool, None, None, &provider, &embedding_provider, &registry, notification.as_ref(), claimed)
        .await;

    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
}
