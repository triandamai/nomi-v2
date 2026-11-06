//! Smaller chat paths: the composer's limits, a long first message's title, and approvals that
//! aren't waiting any more.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn composer_limits_titles_and_stale_approvals(pool: PgPool) {
    std::env::set_var("LLM_PROVIDER", "fake");
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;

    let (status, limits) = app.get("/api/attachments/limits", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert!(limits["max_files"].as_u64().unwrap() > 0);
    assert!(limits["max_message_chars"].as_u64().unwrap() > 0);

    let (_, created) = app.call("POST", "/api/sessions", Some(&ana.token), None).await;
    let session: Uuid = created["session_id"].as_str().unwrap().parse().unwrap();
    let long = "I want to plan a long trip across Java and Bali next year with my family, visiting temples, beaches and markets along the way";
    assert_eq!(app.send("POST", &format!("/api/sessions/{session}/messages"), &ana, json!({"text": long})).await.0, StatusCode::ACCEPTED);
    let mut title: Option<String> = None;
    for _ in 0..50 {
        title = sqlx::query_scalar("SELECT title FROM sessions WHERE id = $1").bind(session).fetch_one(&pool).await.unwrap();
        if title.is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    }
    if let Some(title) = title {
        assert!(title.chars().count() <= 70, "a title stays short: {title}");
    }

    // An approval card that was already decided can't be decided again.
    let card: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, content, content_blocks) VALUES ($1, 'Send it?', $2) RETURNING id",
    )
    .bind(session)
    .bind(json!([{"kind": "approval_request", "id": "ap1", "tool_name": "gmail_send", "description": "Send", "input": {}, "status": "approved", "decided_at": "2026-11-05T00:00:00Z"}]))
    .fetch_one(&pool)
    .await
    .unwrap();
    let (status, _) = app.send("PUT", &format!("/api/sessions/{session}/messages/{card}/approval"), &ana, json!({"decision": "approve", "remember": false})).await;
    assert!(status.is_client_error(), "{status}");
    let (status, _) = app.send("PUT", &format!("/api/sessions/{session}/messages/{card}/approval"), &ana, json!({"decision": "maybe"})).await;
    assert!(status.is_client_error());
}
