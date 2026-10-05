use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_reminders::{fire_due, RemindersAgent};

async fn seed(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'c') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, 'Asia/Jakarta')").bind(user_id).execute(pool).await.unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_agent_adds_a_reminder_in_the_users_timezone_and_lists_it(pool: PgPool) {
    let (user_id, session_id) = seed(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let tomorrow = (chrono::Utc::now() + chrono::Duration::days(1)).with_timezone(&chrono_tz::Asia::Jakarta).date_naive();

    let added = RemindersAgent
        .execute_tool(
            &mut conn,
            session_id,
            session_id,
            user_id,
            "add_reminder",
            serde_json::json!({"title": "Call Mum", "due_at": format!("{tomorrow}T16:00")}),
        )
        .await
        .unwrap();
    assert!(added.display_text.starts_with("Reminder set:"), "{}", added.display_text);

    let due: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT due_at FROM reminders WHERE user_id = $1").bind(user_id).fetch_one(&pool).await.unwrap();
    assert_eq!(due.with_timezone(&chrono_tz::Asia::Jakarta).format("%H:%M").to_string(), "16:00", "a bare time is the user's local time");

    let listed = RemindersAgent.execute_tool(&mut conn, session_id, session_id, user_id, "show_reminders", serde_json::json!({})).await.unwrap();
    assert!(listed.display_text.contains("Call Mum"));
    // Its own tools never wait on an approval card.
    assert!(!RemindersAgent.tool_needs_approval("add_reminder"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_due_reminder_is_posted_once_and_a_repeating_one_moves_on(pool: PgPool) {
    let (user_id, session_id) = seed(&pool).await;
    for (title, recurrence) in [("Take vitamins", None), ("Stand up", Some("daily"))] {
        sqlx::query("INSERT INTO reminders (user_id, session_id, title, due_at, recurrence) VALUES ($1, $2, $3, now() - interval '1 minute', $4)")
            .bind(user_id)
            .bind(session_id)
            .bind(title)
            .bind(recurrence)
            .execute(&pool)
            .await
            .unwrap();
    }

    let posted = fire_due(&pool, 10).await.unwrap();
    assert_eq!(posted.len(), 2);
    assert!(fire_due(&pool, 10).await.unwrap().is_empty(), "nothing fires twice");

    let blocks: Vec<serde_json::Value> =
        sqlx::query_scalar("SELECT content_blocks->0 FROM messages WHERE session_id = $1 AND agent_display_name = 'Reminders'")
            .bind(session_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(blocks.iter().all(|b| b["kind"] == "reminder"));

    let states: Vec<(String, String, bool)> =
        sqlx::query_as("SELECT title, status, due_at > now() FROM reminders ORDER BY title").fetch_all(&pool).await.unwrap();
    assert_eq!(states, vec![("Stand up".into(), "active".into(), true), ("Take vitamins".into(), "fired".into(), false)]);
}
