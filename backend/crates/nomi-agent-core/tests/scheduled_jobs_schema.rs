use sqlx::PgPool;
use uuid::Uuid;

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_scheduled_job_defaults_to_active_status(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;

    let status: String = sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'chitchat', 'chitchat', 'take a bath', 'Remind the user to take a bath.', now() + interval '1 day') \
         RETURNING status",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(status, "active");
}

#[sqlx::test(migrations = "../../migrations")]
async fn recurrence_must_be_one_of_the_three_known_cadences(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;

    let result = sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, recurrence) \
         VALUES ($1, $2, 'chitchat', 'chitchat', 'x', 'y', now(), 'yearly')",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await;

    assert!(result.is_err(), "'yearly' is not one of the allowed recurrence values");
}

#[sqlx::test(migrations = "../../migrations")]
async fn user_preferences_timezone_defaults_to_utc(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let timezone: String = sqlx::query_scalar(
        "INSERT INTO user_preferences (user_id) VALUES ($1) RETURNING timezone",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(timezone, "UTC");
}
