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
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(router.clone(), "POST", "/api/auth/register", json!({"email": email, "password": "correct-password", "org": {"mode": "create", "name": "Acme"}}), None).await;
    let (_, login_body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    login_body["access_token"].as_str().unwrap().to_string()
}

/// The routes sit next to `:plan_id/items/:index`; building the router proves they don't clash.
#[tokio::test]
async fn the_run_route_sits_beside_the_plan_item_routes_without_clashing() {
    let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://nobody@localhost:1/none").unwrap();
    let _ = build_router(test_state(pool));
}

/// A chat with a two-step plan written by `agent_session_id`.
async fn chat_with_plan(pool: &PgPool, router: &axum::Router, token: &str, email: &str) -> (String, uuid::Uuid) {
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let agent_session_id = uuid::Uuid::new_v4();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version) VALUES ($1, $2, $3, 'Bali trip', '- [x] Pick dates\n- [ ] Book flights\n- [ ] Book hotel', 1)")
        .bind(uuid::Uuid::parse_str(&session_id).unwrap())
        .bind(agent_session_id)
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    (session_id, agent_session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_plan_version_lists_its_steps_and_an_estimate(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "steps@example.com").await;
    let (session_id, agent_session_id) = chat_with_plan(&pool, &router, &token, "steps@example.com").await;

    let (status, body) = json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-plans/{agent_session_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let steps = body["plans"][0]["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0]["text"], "Pick dates");
    assert_eq!(steps[0]["status"], "done");
    assert_eq!(steps[1]["status"], "waiting");
    for key in ["owner_agent_type", "status_detail", "started_at", "finished_at"] {
        assert!(steps[1].get(key).is_some(), "{key} is part of a step");
    }
    let estimate = 2 * nomi_agent_core::plans::PLAN_STEP_ESTIMATE_SECONDS;
    assert_eq!(body["plans"][0]["estimate_seconds"], estimate);
    assert_eq!(body["estimate_seconds"], estimate);
}

#[sqlx::test(migrations = "../../migrations")]
async fn running_a_plan_queues_a_turn_like_a_message_and_a_bad_mode_is_a_400(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "run@example.com").await;
    let (session_id, agent_session_id) = chat_with_plan(&pool, &router, &token, "run@example.com").await;
    let uri = format!("/api/sessions/{session_id}/agent-plans/{agent_session_id}/run");

    let (status, _) = json_request(router.clone(), "POST", &uri, json!({"mode": "explore"}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(router.clone(), "POST", &uri, json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let queued: i64 = sqlx::query_scalar("SELECT count(*) FROM turn_jobs WHERE session_id = $1").bind(uuid::Uuid::parse_str(&session_id).unwrap()).fetch_one(&pool).await.unwrap();
    assert_eq!(queued, 0, "a refused mode starts nothing");

    let (status, body) = json_request(router.clone(), "POST", &uri, json!({"mode": "research"}), Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(body["turn_job_id"].is_string());
    let text: String = sqlx::query_scalar("SELECT text FROM turn_jobs WHERE session_id = $1").bind(uuid::Uuid::parse_str(&session_id).unwrap()).fetch_one(&pool).await.unwrap();
    assert!(text.contains("Bali trip") && text.contains("research"), "{text}");
    let said: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1").bind(uuid::Uuid::parse_str(body["user_message_id"].as_str().unwrap()).unwrap()).fetch_one(&pool).await.unwrap();
    assert_eq!(said, text);

    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/agent-plans/{}/run", uuid::Uuid::new_v4()), json!({"mode": "run"}), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_chat_cannot_run_the_plan(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let outsider = register_and_login(router.clone(), "outsider@example.com").await;
    let (session_id, agent_session_id) = chat_with_plan(&pool, &router, &token, "owner@example.com").await;

    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/agent-plans/{agent_session_id}/run"), json!({"mode": "run"}), Some(&outsider)).await;
    assert!(matches!(status, StatusCode::NOT_FOUND | StatusCode::FORBIDDEN), "{status}");
}
