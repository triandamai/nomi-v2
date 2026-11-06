//! Running a due scheduled job: the agent's reply is posted and the person told, a recurring one
//! moves to its next time, and a job for an agent that's gone is cancelled.

use std::sync::Mutex;

use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_server::scheduler_worker::{claim_next, process_claimed_job};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Default)]
struct Told(Mutex<Vec<String>>);

#[async_trait::async_trait]
impl nomi_agent_core::notification::NotificationDelivery for Told {
    async fn deliver(&self, _user_id: Uuid, message: &str) -> Result<(), String> {
        self.0.lock().unwrap().push(message.to_string());
        Ok(())
    }
}

async fn job(pool: &PgPool, target: &str, recurrence: Option<&str>) -> (Uuid, Uuid) {
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', $2, $3) RETURNING id")
        .bind(org)
        .bind(Uuid::new_v4().to_string())
        .bind(user)
        .fetch_one(pool)
        .await
        .unwrap();
    let id = sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, recurrence) \
         VALUES ($1, $2, 'chitchat', $3, 'drink water', 'Remind the user to drink water.', now() - interval '1 minute', $4) RETURNING id",
    )
    .bind(session)
    .bind(user)
    .bind(target)
    .bind(recurrence)
    .fetch_one(pool)
    .await
    .unwrap();
    (id, session)
}

fn reply(text: &str) -> FakeLlmProvider {
    FakeLlmProvider::success(LlmResponse { content: vec![ContentBlock::Text { text: text.into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 })
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_due_job_posts_the_agent_s_reply_and_recurring_ones_move_on(pool: PgPool) {
    let registry = nomi_server::build_agent_registry(&pool, nomi_test_support::test_project_storage());
    let embedding = FakeEmbeddingProvider::success(nomi_test_support::dummy_embedding());
    let told = Told::default();

    let (once, session) = job(&pool, "chitchat", None).await;
    let claimed = claim_next(&pool).await.unwrap().expect("a due job");
    assert_eq!(claimed.id, once);
    process_claimed_job(&pool, None, None, &reply("Time for some water!"), &embedding, &registry, &told, claimed).await;
    let posted: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1 AND content LIKE '%water%'").bind(session).fetch_one(&pool).await.unwrap();
    assert_eq!(posted, 1);
    assert_eq!(told.0.lock().unwrap().as_slice(), ["Time for some water!"]);
    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1").bind(once).fetch_one(&pool).await.unwrap();
    assert_ne!(status, "pending");

    let (daily, _) = job(&pool, "chitchat", Some("daily")).await;
    let claimed = claim_next(&pool).await.unwrap().unwrap();
    process_claimed_job(&pool, None, None, &reply("Water again"), &embedding, &registry, &told, claimed).await;
    let next: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT run_at FROM scheduled_jobs WHERE id = $1").bind(daily).fetch_one(&pool).await.unwrap();
    assert!(next > chrono::Utc::now(), "a daily job moves to tomorrow");

    // The model failing leaves a recurring job for its next time too, without posting.
    let (failing, failing_session) = job(&pool, "chitchat", Some("weekly")).await;
    let claimed = claim_next(&pool).await.unwrap().unwrap();
    process_claimed_job(&pool, None, None, &FakeLlmProvider::failure("down"), &embedding, &registry, &told, claimed).await;
    let _: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1").bind(failing).fetch_one(&pool).await.unwrap();
    let posted: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1 AND content LIKE '%water%'").bind(failing_session).fetch_one(&pool).await.unwrap();
    assert_eq!(posted, 0);

    let (orphan, _) = job(&pool, "an-agent-that-was-removed", None).await;
    let claimed = claim_next(&pool).await.unwrap().unwrap();
    process_claimed_job(&pool, None, None, &reply("x"), &embedding, &registry, &told, claimed).await;
    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1").bind(orphan).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "cancelled");
    assert!(claim_next(&pool).await.unwrap().is_none());
}
