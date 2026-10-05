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
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, bearer: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, pool: &PgPool, email: &str) -> (String, uuid::Uuid) {
    json_request_post(
        router.clone(),
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
    )
    .await;
    let (_, login_body) = json_request_post(router, "/api/auth/login", json!({ "email": email, "password": "correct-password" })).await;
    let token = login_body["access_token"].as_str().unwrap().to_string();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap();
    (token, user_id)
}

async fn json_request_post(router: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}


async fn authed(router: axum::Router, method: &str, uri: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("authorization", format!("Bearer {token}"));
    let body = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let response = router.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn org_session(pool: &PgPool, user_id: uuid::Uuid, title: &str) -> uuid::Uuid {
    sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id, title) \
         VALUES ((SELECT org_id FROM memberships WHERE user_id = $1 LIMIT 1), 'web', gen_random_uuid()::text, $2) RETURNING id",
    )
    .bind(user_id)
    .bind(title)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_reports_what_happened_whats_due_today_and_open_plans(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "home@example.com").await;
    let chat = org_session(&pool, user_id, "Bali, 6 days").await;

    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result, completed_at) \
         VALUES ($1, $2, 'chitchat', 'money', 'Find idle subscriptions', 'completed', '3 unused since July', now() - interval '1 hour')",
    )
    .bind(chat)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'planning', 'planning', 'Call Mum back', 'remind', now() + interval '1 minute')",
    )
    .bind(chat)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (session_id, content, content_blocks, agent_display_name) VALUES ($1, 'To-do', \
         '[{\"kind\":\"todo_list\",\"items\":[{\"id\":\"1\",\"text\":\"a\",\"status\":\"done\"},{\"id\":\"2\",\"text\":\"b\",\"status\":\"pending\"}]}]', 'Planning')",
    )
    .bind(chat)
    .execute(&pool)
    .await
    .unwrap();

    let (status, home) = authed(router.clone(), "GET", "/api/home", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    let out = home["while_you_were_out"].as_array().unwrap();
    assert!(out.iter().any(|i| i["kind"] == "finished" && i["title"] == "Money finished Find idle subscriptions"), "{out:?}");
    // The reminder is due later today, unless the test runs in the last minute of the UTC day.
    let today = home["today"].as_array().unwrap();
    assert!(today.is_empty() || today[0]["label"] == "Call Mum back");
    let plans = home["plans"].as_array().unwrap();
    assert_eq!(plans[0]["title"], "Bali, 6 days");
    assert_eq!((plans[0]["done"].as_i64(), plans[0]["total"].as_i64()), (Some(1), Some(2)));

    // A second look within the same visit reports the same things.
    let (_, again) = authed(router, "GET", "/api/home", &token, None).await;
    assert_eq!(again["since"], home["since"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn money_totals_a_month_against_the_one_before(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "money@example.com").await;
    for (when, cents, category) in [("2026-03-03", 1200, "food"), ("2026-03-09", 800, "food"), ("2026-03-10", 5000, "travel"), ("2026-02-20", 1000, "food")] {
        sqlx::query("INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) VALUES ($1, $2::date + time '12:00', $3, $4, 'x')")
            .bind(user_id)
            .bind(when)
            .bind(cents as i64)
            .bind(category)
            .execute(&pool)
            .await
            .unwrap();
    }

    let (status, money) = authed(router, "GET", "/api/money?month=2026-03", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(money["month"], "2026-03");
    assert_eq!(money["total_cents"], 7000);
    assert_eq!(money["previous_total_cents"], 1000);
    assert_eq!(money["by_category"][0]["category"], "travel");
    assert_eq!(money["by_day"].as_array().unwrap().len(), 3);
    assert_eq!(money["transactions"].as_array().unwrap().len(), 3);
    assert_eq!(money["months"], json!(["2026-03", "2026-02"]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reminder_made_on_the_page_lands_in_a_reminders_chat_and_can_be_cancelled(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, _) = register_and_login(router.clone(), &pool, "remind@example.com").await;
    let run_at = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();

    let (status, created) =
        authed(router.clone(), "POST", "/api/reminders", &token, Some(json!({"label": "Stretch", "run_at": run_at, "recurrence": "daily"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["recurrence"], "daily");
    let (_, second) = authed(router.clone(), "POST", "/api/reminders", &token, Some(json!({"label": "Water", "run_at": run_at}))).await;
    assert_eq!(second["session_id"], created["session_id"], "both share one Reminders chat");

    let (status, _) = authed(router.clone(), "POST", "/api/reminders", &token, Some(json!({"label": "Past", "run_at": "2020-01-01T00:00:00Z"}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let id = created["id"].as_str().unwrap();
    let (status, _) = authed(router.clone(), "POST", &format!("/api/reminders/{id}/cancel"), &token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = authed(router.clone(), "GET", "/api/reminders", &token, None).await;
    assert_eq!(list["upcoming"].as_array().unwrap().len(), 1);
    assert_eq!(list["past"][0]["status"], "cancelled");

    let (other, _) = register_and_login(router.clone(), &pool, "other@example.com").await;
    let second_id = second["id"].as_str().unwrap();
    let (status, _) = authed(router, "POST", &format!("/api/reminders/{second_id}/cancel"), &other, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "another user can't cancel it");
}
