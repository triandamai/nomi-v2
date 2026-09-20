use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::reminders::{cancel_reminder, create_reminder, get_user_timezone, list_reminders};

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

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_timezone_defaults_to_utc_when_no_preferences_row_exists(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let tz = get_user_timezone(&mut conn, user_id).await;

    assert_eq!(tz, "UTC");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_timezone_returns_the_stored_value(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, 'America/New_York')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let tz = get_user_timezone(&mut conn, user_id).await;

    assert_eq!(tz, "America/New_York");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_inserts_a_one_time_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let input = serde_json::json!({
        "run_at": "2026-09-21T11:00:00-04:00",
        "label": "take a bath",
        "prompt": "Remind the user to take a bath.",
        "target_agent": "chitchat"
    });

    let result = create_reminder(&mut conn, session_id, user_id, "chitchat", &input).await;
    assert!(result.is_ok(), "{result:?}");

    let (label, recurrence): (String, Option<String>) =
        sqlx::query_as("SELECT label, recurrence FROM scheduled_jobs WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(label, "take a bath");
    assert_eq!(recurrence, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_rejects_weekly_recurrence_without_a_weekday(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let input = serde_json::json!({
        "run_at": "2026-09-21T11:00:00-04:00",
        "label": "check spending",
        "prompt": "Check recent spending.",
        "target_agent": "chitchat",
        "recurrence": "weekly"
    });

    let result = create_reminder(&mut conn, session_id, user_id, "chitchat", &input).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("recurrence_weekday"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_reminders_returns_only_this_users_active_jobs(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let (_, other_user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    create_reminder(
        &mut conn,
        session_id,
        user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "mine", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();
    create_reminder(
        &mut conn,
        session_id,
        other_user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "not mine", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();

    let listing = list_reminders(&mut conn, user_id).await.unwrap();

    assert!(listing.contains("mine"));
    assert!(!listing.contains("not mine"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_reminder_marks_it_cancelled(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    create_reminder(
        &mut conn,
        session_id,
        user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "take a bath", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();
    let id: Uuid = sqlx::query_scalar("SELECT id FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = cancel_reminder(&mut conn, user_id, &serde_json::json!({"reminder_id": id.to_string()})).await;
    assert!(result.is_ok(), "{result:?}");

    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_reminder_rejects_another_users_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let (_, other_user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    create_reminder(
        &mut conn,
        session_id,
        user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "take a bath", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();
    let id: Uuid = sqlx::query_scalar("SELECT id FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = cancel_reminder(&mut conn, other_user_id, &serde_json::json!({"reminder_id": id.to_string()})).await;

    assert!(result.is_err());
    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "active", "another user's cancel attempt must not change this job's status");
}
