use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
    }
}

async fn call(router: &axum::Router, method: &str, uri: &str, body: Value, bearer: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {bearer}"))
        .body(Body::from(if body.is_null() { String::new() } else { body.to_string() }))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn sign_up(router: &axum::Router, email: &str) -> String {
    let register = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({"email": email, "password": "correct-password", "org": {"mode": "create", "name": "Home"}}).to_string()))
        .unwrap();
    router.clone().oneshot(register).await.unwrap();
    let login = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({"email": email, "password": "correct-password"}).to_string()))
        .unwrap();
    let response = router.clone().oneshot(login).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

/// Ana's chat and her web identity.
async fn chat(router: &axum::Router, pool: &PgPool, token: &str) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let (_, body) = call(router, "POST", "/api/sessions", Value::Null, token).await;
    let session: uuid::Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let (user, identity): (uuid::Uuid, uuid::Uuid) = sqlx::query_as(
        "SELECT s.user_id, ci.id FROM sessions s JOIN channel_identities ci ON ci.user_id = s.user_id AND ci.channel = 'web' WHERE s.id = $1",
    )
    .bind(session)
    .fetch_one(pool)
    .await
    .unwrap();
    (session, user, identity)
}

async fn say(pool: &PgPool, session: uuid::Uuid, sender: Option<uuid::Uuid>, text: &str, blocks: Option<Value>) -> uuid::Uuid {
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    sqlx::query_scalar("INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(session)
        .bind(sender)
        .bind(text)
        .bind(blocks)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_request_takes_its_replies_and_what_the_crew_made_for_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let (session, user, me) = chat(&router, &pool, &ana).await;

    let request = say(&pool, session, Some(me), "Lunch was 50k, and remind me to pay rent", None).await;
    // What the crew made while answering it: the triggers point these at the request.
    sqlx::query("INSERT INTO money_transactions (user_id, occurred_at, amount_cents, category, description, source, kind, source_message_id) VALUES ($1, now(), 5000000, 'food', 'Lunch', 'agent', 'expense', $2)")
        .bind(user)
        .bind(request)
        .execute(&pool)
        .await
        .unwrap();
    let reminder: uuid::Uuid = sqlx::query_scalar("INSERT INTO reminders (user_id, session_id, title, due_at, created_by) VALUES ($1, $2, 'Pay rent', now() + interval '1 day', 'agent') RETURNING id")
        .bind(user)
        .bind(session)
        .fetch_one(&pool)
        .await
        .unwrap();
    let plan: uuid::Uuid = sqlx::query_scalar("INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version) VALUES ($1, gen_random_uuid(), $2, 'Rent plan', '1. pay', 1) RETURNING id")
        .bind(session)
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    let reply = say(&pool, session, None, "Logged it and set a reminder.", None).await;
    // A reminder going off before the next message isn't a reply to the request.
    let fired = say(&pool, session, None, "Pay rent", Some(json!([{"kind": "reminder", "reminder_id": reminder, "title": "Pay rent", "notes": null, "due_at": "2026-10-12T09:00:00Z", "recurrence": null}]))).await;
    let next = say(&pool, session, Some(me), "Thanks", None).await;
    let next_reply = say(&pool, session, None, "Anytime.", Some(json!([{"kind": "plan", "plan_id": plan, "agent_session_id": plan, "title": "Rent plan", "version": 1}]))).await;
    // Something from the Reminders page, not from this chat's turn.
    sqlx::query("INSERT INTO reminders (user_id, session_id, title, due_at, created_by) VALUES ($1, $2, 'Water plants', now() + interval '1 day', 'user')")
        .bind(user)
        .bind(session)
        .execute(&pool)
        .await
        .unwrap();

    let uri = format!("/api/sessions/{session}/messages/{request}");
    let (status, preview) = call(&router, "GET", &format!("{uri}/delete"), Value::Null, &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["message_ids"], json!([request, reply]));
    assert_eq!(preview["transactions"][0]["label"], "Lunch");
    assert_eq!(preview["transactions"][0]["amount_cents"], 5000000);
    assert_eq!(preview["reminders"].as_array().unwrap().len(), 1);
    assert_eq!(preview["reminders"][0]["label"], "Pay rent");
    assert_eq!(preview["plans"][0]["label"], "Rent plan (v1)");

    assert_eq!(call(&router, "DELETE", &uri, Value::Null, &ana).await.0, StatusCode::OK);
    assert_eq!(count(&pool, "SELECT count(*) FROM money_transactions").await, 0);
    assert_eq!(count(&pool, "SELECT count(*) FROM reminders").await, 1);
    assert_eq!(count(&pool, "SELECT count(*) FROM agent_plans").await, 0);
    let (_, left) = call(&router, "GET", &format!("/api/sessions/{session}/messages"), Value::Null, &ana).await;
    let ids: Vec<&str> = left["messages"].as_array().unwrap().iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert_eq!(ids, [fired.to_string(), next.to_string(), next_reply.to_string()]);

    // A reply goes on its own.
    let (_, preview) = call(&router, "GET", &format!("/api/sessions/{session}/messages/{next_reply}/delete"), Value::Null, &ana).await;
    assert_eq!(preview["message_ids"], json!([next_reply]));
    assert!(preview["plans"].as_array().unwrap().is_empty());
    assert_eq!(call(&router, "DELETE", &format!("/api/sessions/{session}/messages/{next_reply}"), Value::Null, &ana).await.0, StatusCode::OK);
    assert_eq!(count(&pool, "SELECT count(*) FROM messages WHERE content = 'Thanks'").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_the_author_deletes_a_message_in_a_shared_chat(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let (_, org) = call(&router, "POST", "/api/orgs", json!({"name": "Studio"}), &ana).await;
    let org_id = org["id"].as_str().unwrap();
    let (_, created) = call(&router, "POST", "/api/projects", json!({"org_id": org_id}), &ana).await;
    let session: uuid::Uuid = created["session_id"].as_str().unwrap().parse().unwrap();
    let ana_identity: uuid::Uuid = sqlx::query_scalar("SELECT ci.id FROM channel_identities ci JOIN sessions s ON s.user_id = ci.user_id WHERE s.id = $1")
        .bind(session)
        .fetch_one(&pool)
        .await
        .unwrap();
    let hers = say(&pool, session, Some(ana_identity), "Make the header blue", None).await;

    // Outside the organization the chat doesn't exist for him.
    let uri = format!("/api/sessions/{session}/messages/{hers}");
    assert_eq!(call(&router, "DELETE", &uri, Value::Null, &bob).await.0, StatusCode::NOT_FOUND);
    let (_, invite) = call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "bob@example.com"}), &ana).await;
    call(&router, "POST", &format!("/api/invites/{}/accept", invite["code"].as_str().unwrap()), Value::Null, &bob).await;
    // Inside it, it's still Ana's message.
    assert_eq!(call(&router, "DELETE", &uri, Value::Null, &bob).await.0, StatusCode::FORBIDDEN);
    assert_eq!(call(&router, "DELETE", &uri, Value::Null, &ana).await.0, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reminder_card_shows_where_its_reminder_stands_now(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let (session, user, _) = chat(&router, &pool, &ana).await;
    let card = |id: uuid::Uuid| json!([{"kind": "reminder", "reminder_id": id, "title": "Stretch", "notes": null, "due_at": "2026-10-11T09:00:00Z", "recurrence": null}]);
    let mut reminders = Vec::new();
    for status in ["fired", "done", "active"] {
        let id: uuid::Uuid = sqlx::query_scalar("INSERT INTO reminders (user_id, session_id, title, due_at, status) VALUES ($1, $2, 'Stretch', '2026-10-11T09:00:00Z', $3) RETURNING id")
            .bind(user)
            .bind(session)
            .bind(status)
            .fetch_one(&pool)
            .await
            .unwrap();
        say(&pool, session, None, "Stretch", Some(card(id))).await;
        reminders.push(id);
    }
    // The third was snoozed: it's due again later than its card says.
    sqlx::query("UPDATE reminders SET due_at = '2026-10-11T09:10:00Z' WHERE id = $1").bind(reminders[2]).execute(&pool).await.unwrap();

    let (_, list) = call(&router, "GET", &format!("/api/sessions/{session}/messages"), Value::Null, &ana).await;
    let states: Vec<&str> = list["messages"].as_array().unwrap().iter().map(|m| m["content_blocks"][0]["state"].as_str().unwrap()).collect();
    assert_eq!(states, ["open", "done", "snoozed"]);
    assert_eq!(list["messages"][2]["content_blocks"][0]["snoozed_until"], "2026-10-11T09:10:00+00:00");
}
