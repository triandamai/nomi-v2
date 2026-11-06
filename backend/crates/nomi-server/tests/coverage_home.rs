//! Home's "while you were out": hand-offs that finished, failed or were stopped, and reminders
//! and follow-ups that went off since the person last looked.

mod support;

use axum::http::StatusCode;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn home_lists_what_happened_while_the_person_was_away(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let (_, session) = app.project(&ana, "Bakery").await;
    // The first look sets when they were last here.
    assert_eq!(app.get("/api/home", &ana).await.0, StatusCode::OK);
    sqlx::query("UPDATE sessions SET user_id = $2 WHERE id = $1").bind(session).bind(ana.id).execute(&pool).await.unwrap();

    for (status, result, error) in [("completed", Some("Spent Rp 1.2 juta"), None), ("failed", None, Some("model unavailable")), ("cancelled", None, None)] {
        sqlx::query(
            "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result, error, completed_at) \
             VALUES ($1, $2, 'supervisor', 'money', 'Sum up November', $3, $4, $5, now() + interval '1 minute')",
        )
        .bind(session)
        .bind(ana.id)
        .bind(status)
        .bind(result)
        .bind(error)
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query("INSERT INTO reminders (user_id, session_id, title, due_at, last_fired_at, status, created_by) VALUES ($1, $2, 'Pay rent', now(), now() + interval '1 minute', 'fired', 'user')")
        .bind(ana.id)
        .bind(session)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, last_fired_at) \
         VALUES ($1, $2, 'chitchat', 'money', 'Weekly summary', 'Sum up', now() + interval '7 days', now() + interval '1 minute')",
    )
    .bind(session)
    .bind(ana.id)
    .execute(&pool)
    .await
    .unwrap();

    let (status, updates) = app.get("/api/home/updates", &ana).await;
    assert_eq!(status, StatusCode::OK, "{updates}");
    let text = updates.to_string();
    for kind in ["finished", "failed", "reminder"] {
        assert!(text.contains(&format!("\"kind\":\"{kind}\"")), "{kind}: {text}");
    }
    assert!(text.contains("Pay rent") && text.contains("Weekly summary"), "{text}");
    let (status, home) = app.get("/api/home", &ana).await;
    assert_eq!(status, StatusCode::OK, "{home}");
    for page in ["/api/home/today", "/api/home/plans", "/api/home/updates?page=2"] {
        assert_eq!(app.get(page, &ana).await.0, StatusCode::OK, "{page}");
    }
    let _ = Uuid::nil();
}
