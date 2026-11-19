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


async fn get_raw(router: axum::Router, uri: &str, token: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder().method("GET").uri(uri).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    (status, response.into_body().collect().await.unwrap().to_bytes().to_vec())
}

/// A chat with a CSV the crew made in it, stored the way `make_file` stores one.
async fn chat_with_made_file(pool: &PgPool, router: &axum::Router, token: &str, email: &str, sources: &[uuid::Uuid]) -> (String, uuid::Uuid) {
    std::env::set_var("ATTACHMENTS_DIR", std::env::temp_dir().join(format!("nomi-test-made-files-{}", std::process::id())));
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap();
    let file_id = uuid::Uuid::new_v4();
    let key = nomi_attachments::made::storage_key(user_id, file_id);
    let csv = b"item,amount\r\nrice,12500\r\ntea,3000\r\n".to_vec();
    nomi_storage::blob::attachment_store().put_bytes(&key, csv.clone(), "text/csv").await.unwrap();
    sqlx::query(
        "INSERT INTO attachments (id, user_id, session_id, name, mime, kind, size_bytes, storage_key, status, made_by, made_summary, made_from) \
         VALUES ($1, $2, $3, 'spend.csv', 'text/csv', 'text', $4, $5, 'ready', 'files', 'May spending', $6)",
    )
    .bind(file_id)
    .bind(user_id)
    .bind(uuid::Uuid::parse_str(&session_id).unwrap())
    .bind(csv.len() as i64)
    .bind(&key)
    .bind(sources)
    .execute(pool)
    .await
    .unwrap();
    (session_id, file_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_made_file_opens_and_previews_for_its_owner_only(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "maker@example.com").await;
    let other = register_and_login(router.clone(), "other@example.com").await;
    let (session_id, file_id) = chat_with_made_file(&pool, &router, &token, "maker@example.com", &[]).await;

    let (status, bytes) = get_raw(router.clone(), &format!("/api/sessions/{session_id}/files/{file_id}/content"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8(bytes).unwrap().starts_with("item,amount"));

    let (status, preview) = json_request(router.clone(), "GET", &format!("/api/sessions/{session_id}/files/{file_id}/preview?rows=2"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["rows"].as_array().unwrap().len(), 2);
    assert_eq!(preview["total_rows"], 3);
    assert_eq!(preview["truncated"], true);

    let (status, _) = get_raw(router.clone(), &format!("/api/sessions/{session_id}/files/{file_id}/content"), &other).await;
    assert!(status == StatusCode::NOT_FOUND || status == StatusCode::FORBIDDEN, "{status}");
    let (status, _) = json_request(router, "DELETE", &format!("/api/sessions/{session_id}/files/{file_id}"), Value::Null, Some(&other)).await;
    assert!(status == StatusCode::NOT_FOUND || status == StatusCode::FORBIDDEN, "{status}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_made_file_keeps_its_row_and_says_if_its_sources_remain(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "keeper@example.com").await;
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'keeper@example.com'").fetch_one(&pool).await.unwrap();
    let source = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO attachments (id, user_id, name, mime, kind, size_bytes, storage_key, status) VALUES ($1, $2, 'receipt.txt', 'text/plain', 'text', 1, 'k', 'ready')")
        .bind(source)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let (session_id, file_id) = chat_with_made_file(&pool, &router, &token, "keeper@example.com", &[source]).await;

    let uri = format!("/api/sessions/{session_id}/files/{file_id}");
    let (status, first) = json_request(router.clone(), "DELETE", &uri, Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["sources_exist"], true);
    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar("SELECT deleted_at FROM attachments WHERE id = $1").bind(file_id).fetch_one(&pool).await.unwrap();
    assert!(deleted_at.is_some());

    let (_, second) = json_request(router.clone(), "DELETE", &uri, Value::Null, Some(&token)).await;
    assert_eq!(second["deleted_at"], first["deleted_at"]);
    let (status, _) = get_raw(router.clone(), &format!("{uri}/content"), &token).await;
    assert_eq!(status, StatusCode::GONE);

    sqlx::query("UPDATE attachments SET deleted_at = now() WHERE id = $1").bind(source).execute(&pool).await.unwrap();
    let (_, third) = json_request(router, "DELETE", &uri, Value::Null, Some(&token)).await;
    assert_eq!(third["sources_exist"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn making_a_file_again_asks_the_crew_in_the_chat(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "again@example.com").await;
    let (session_id, file_id) = chat_with_made_file(&pool, &router, &token, "again@example.com", &[]).await;
    let (status, body) = json_request(router, "POST", &format!("/api/sessions/{session_id}/files/{file_id}/remake"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert!(body["turn_job_id"].is_string());
    let text: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1").bind(uuid::Uuid::parse_str(body["user_message_id"].as_str().unwrap()).unwrap()).fetch_one(&pool).await.unwrap();
    assert!(text.contains("spend.csv") && text.contains(&file_id.to_string()));
}
