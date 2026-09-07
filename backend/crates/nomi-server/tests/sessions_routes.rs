
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

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
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router,
        "POST",
        "/api/auth/login",
        json!({ "email": email, "password": "correct-password" }),
        None,
    )
    .await;
    login_body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_session_then_appears_in_list(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "alice@example.com").await;

    let (status, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, list_body) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let sessions = list_body["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["id"].as_str().unwrap(), session_id);
    assert_eq!(sessions[0]["channel"], "web");
    assert!(sessions[0]["last_message"].is_null());
    assert_eq!(sessions[0]["agent_active"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn creating_two_sessions_reuses_the_same_web_identity(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "bob@example.com").await;

    let (_, first) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let (_, second) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    assert_ne!(first["session_id"], second["session_id"]);

    let identity_count: i64 = sqlx::query_scalar("SELECT count(*) FROM channel_identities WHERE channel = 'web'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(identity_count, 1);

    let (_, list_body) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(list_body["sessions"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_sessions_only_returns_the_callers_active_org(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token_a = register_and_login(router.clone(), "carol@example.com").await;
    let token_b = register_and_login(router.clone(), "dave@example.com").await;

    json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_a)).await;
    json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_b)).await;

    let (_, list_a) = json_request(router.clone(), "GET", "/api/sessions", Value::Null, Some(&token_a)).await;
    assert_eq!(list_a["sessions"].as_array().unwrap().len(), 1);

    let (_, list_b) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token_b)).await;
    assert_eq!(list_b["sessions"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_session_requires_authentication(pool: PgPool) {
    let router = build_router(test_state(pool));
    let (status, _) = json_request(router, "POST", "/api/sessions", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn send_message_returns_202_with_only_the_user_message(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "erin@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();

    let (status, send_body) = json_request(
        router,
        "POST",
        &format!("/api/sessions/{session_id}/messages"),
        json!({ "text": "hi" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(send_body["user_message"]["content"], "hi");
    assert_eq!(send_body["user_message"]["sender"], "user");
    assert!(send_body["user_message"]["id"].is_string());
    assert!(send_body.get("assistant_message").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_messages_returns_history_oldest_first(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "frank@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "first"}), Some(&token)).await;
    json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "second"}), Some(&token)).await;

    let (status, list_body) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let messages = list_body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["content"], "first");
    assert_eq!(messages[1]["content"], "second");
}

#[sqlx::test(migrations = "../../migrations")]
async fn send_message_to_a_nonexistent_session_returns_not_found(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "grace2@example.com").await;

    let fake_id = Uuid::new_v4();
    let (status, _) = json_request(
        router,
        "POST",
        &format!("/api/sessions/{fake_id}/messages"),
        json!({"text": "hi"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn send_message_to_another_orgs_session_returns_not_found(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token_a = register_and_login(router.clone(), "henry@example.com").await;
    let token_b = register_and_login(router.clone(), "irene@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_a)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        router,
        "POST",
        &format!("/api/sessions/{session_id}/messages"),
        json!({"text": "hi"}),
        Some(&token_b),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn ingest_returns_accepted_even_with_failure_sentinel(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "jack@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, send_body) = json_request(
        router,
        "POST",
        &format!("/api/sessions/{session_id}/messages"),
        json!({"text": "hello"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(send_body.get("user_message").is_some());

    let session_uuid = Uuid::parse_str(&session_id).unwrap();
    let content: String = sqlx::query_scalar("SELECT content FROM messages WHERE session_id = $1")
        .bind(session_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content, "hello");
}

#[sqlx::test(migrations = "../../migrations")]
async fn send_message_rejects_empty_text(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "karen@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        router,
        "POST",
        &format!("/api/sessions/{session_id}/messages"),
        json!({"text": "   "}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_messages_respects_limit_and_before_cursor(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "leo@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    for i in 0..5 {
        json_request(
            router.clone(),
            "POST",
            &format!("/api/sessions/{session_id}/messages"),
            json!({"text": format!("msg-{i}")}),
            Some(&token),
        )
        .await;
    }

    let (status, page1) = json_request(
        router.clone(),
        "GET",
        &format!("/api/sessions/{session_id}/messages?limit=4"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let page1_messages = page1["messages"].as_array().unwrap();
    assert_eq!(page1_messages.len(), 4);

    let oldest_in_page1 = page1_messages[0]["id"].as_str().unwrap().to_string();
    let (status, page2) = json_request(
        router,
        "GET",
        &format!("/api/sessions/{session_id}/messages?limit=4&before={oldest_in_page1}"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let page2_messages = page2["messages"].as_array().unwrap();
    assert_eq!(page2_messages.len(), 1);

    let page1_ids: Vec<&str> = page1_messages.iter().map(|m| m["id"].as_str().unwrap()).collect();
    for m in page2_messages {
        assert!(!page1_ids.contains(&m["id"].as_str().unwrap()));
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_message_has_no_feedback_by_default(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "kate@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hi"}), Some(&token)).await;

    let (_, list_body) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, Some(&token)).await;
    assert!(list_body["messages"][0]["my_feedback"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_feedback_then_listing_messages_reflects_it(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "liam@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let (_, send_body) =
        json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hi"}), Some(&token)).await;
    let message_id = send_body["user_message"]["id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        json!({"rating": "up"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list_body) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, Some(&token)).await;
    assert_eq!(list_body["messages"][0]["my_feedback"], "up");
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_feedback_twice_updates_rather_than_duplicates(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "mona@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let (_, send_body) =
        json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hi"}), Some(&token)).await;
    let message_id = send_body["user_message"]["id"].as_str().unwrap().to_string();

    json_request(
        router.clone(),
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        json!({"rating": "up"}),
        Some(&token),
    )
    .await;
    let (status, _) = json_request(
        router.clone(),
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        json!({"rating": "down"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list_body) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, Some(&token)).await;
    assert_eq!(list_body["messages"][0]["my_feedback"], "down");
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_feedback_clears_it(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "noah@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let (_, send_body) =
        json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hi"}), Some(&token)).await;
    let message_id = send_body["user_message"]["id"].as_str().unwrap().to_string();

    json_request(
        router.clone(),
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        json!({"rating": "up"}),
        Some(&token),
    )
    .await;
    let (status, _) = json_request(
        router.clone(),
        "DELETE",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list_body) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, Some(&token)).await;
    assert!(list_body["messages"][0]["my_feedback"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_invalid_rating_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "olive@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let (_, send_body) =
        json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hi"}), Some(&token)).await;
    let message_id = send_body["user_message"]["id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        json!({"rating": "sideways"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn feedback_on_another_orgs_message_is_forbidden(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token_a = register_and_login(router.clone(), "peter@example.com").await;
    let token_b = register_and_login(router.clone(), "quinn@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_a)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let (_, send_body) =
        json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hi"}), Some(&token_a)).await;
    let message_id = send_body["user_message"]["id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/feedback"),
        json!({"rating": "up"}),
        Some(&token_b),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn feedback_on_a_nonexistent_message_returns_not_found(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "ray@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let fake_message_id = Uuid::new_v4();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{fake_message_id}/feedback"),
        json!({"rating": "up"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_session_removes_it_and_its_messages(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "erin@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "hello"}), Some(&token)).await;

    let (status, _) = json_request(router.clone(), "DELETE", &format!("/api/sessions/{session_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list_body) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(list_body["sessions"].as_array().unwrap().len(), 0);

    let remaining_messages: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(Uuid::parse_str(&session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining_messages, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_another_orgs_session_is_not_found(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token_a = register_and_login(router.clone(), "frank@example.com").await;
    let token_b = register_and_login(router.clone(), "grace@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_a)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, _) = json_request(router, "DELETE", &format!("/api/sessions/{session_id}"), Value::Null, Some(&token_b)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_session_cascades_its_project_and_disk_files(pool: PgPool) {
    let storage = nomi_test_support::test_project_storage();
    let router = build_router(AppState {
        pool: pool.clone(),
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: storage.clone(),
    });
    let token = register_and_login(router.clone(), "heidi@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'heidi@example.com'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Test app') RETURNING id",
    )
    .bind(user_id)
    .bind(Uuid::parse_str(&session_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    storage.put_object(&format!("{project_id}/index.html"), "<h1>hi</h1>", "text/html").await.unwrap();

    let (status, _) = json_request(router, "DELETE", &format!("/api/sessions/{session_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let remaining_projects: i64 = sqlx::query_scalar("SELECT count(*) FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining_projects, 0);
    assert_eq!(storage.get_object(&format!("{project_id}/index.html")).await.unwrap(), None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_message_returns_the_message_with_its_content_blocks(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "ivan@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let content_blocks = json!([{"type": "todo_list", "items": [{"text": "step 1", "status": "pending"}]}]);
    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, content, content_blocks) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(Uuid::parse_str(&session_id).unwrap())
    .bind("todo list updated")
    .bind(&content_blocks)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(
        router,
        "GET",
        &format!("/api/sessions/{session_id}/messages/{message_id}"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"].as_str().unwrap(), message_id.to_string());
    assert_eq!(body["content"], "todo list updated");
    assert_eq!(body["sender"], "assistant");
    assert_eq!(body["content_blocks"], content_blocks);
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_message_returns_not_found_for_an_unknown_message_id(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "judy@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, _) = json_request(
        router,
        "GET",
        &format!("/api/sessions/{session_id}/messages/{}", Uuid::new_v4()),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_message_returns_not_found_when_message_belongs_to_another_session(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "kevin@example.com").await;

    let (_, create_a) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_a = create_a["session_id"].as_str().unwrap().to_string();
    let (_, create_b) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_b = create_b["session_id"].as_str().unwrap().to_string();

    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(Uuid::parse_str(&session_a).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "GET",
        &format!("/api/sessions/{session_b}/messages/{message_id}"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
