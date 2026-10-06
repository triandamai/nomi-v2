//! Two people in the same organization never see each other's chats: not in the chat list, not by
//! opening or posting into one directly, not on Home. Chats carry what Nomi remembers about their
//! owner, so any of these would leak one person's memories to a colleague.
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
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
    }
}

async fn request(router: axum::Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
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

async fn login(router: axum::Router, email: &str) -> String {
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

/// Ana creates the org; Budi joins it with an invite, so it's his only (and active) org too.
async fn two_colleagues(router: axum::Router, pool: &PgPool) -> (String, Uuid, String, Uuid) {
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": "ana@acme.test", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    let ana = user_id(pool, "ana@acme.test").await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1").bind(ana).fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('ACME', $1, 'member', $2, now() + interval '1 day')")
        .bind(org)
        .bind(ana)
        .execute(pool)
        .await
        .unwrap();
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": "budi@acme.test", "password": "correct-password", "org": { "mode": "join", "invite_code": "ACME" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    let budi = user_id(pool, "budi@acme.test").await;

    (login(router.clone(), "ana@acme.test").await, ana, login(router, "budi@acme.test").await, budi)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_colleague_cannot_list_open_or_post_into_someone_elses_chat(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, _, budi_token, _) = two_colleagues(router.clone(), &pool).await;

    let (status, created) = request(router.clone(), "POST", "/api/sessions", Some(&ana_token), None).await;
    assert_eq!(status, StatusCode::CREATED);
    let chat = created["session_id"].as_str().unwrap().to_string();
    sqlx::query(
        "INSERT INTO messages (session_id, content, agent_display_name) VALUES ($1::uuid, 'Noted: you are allergic to peanuts.', 'Nomi')",
    )
    .bind(&chat)
    .execute(&pool)
    .await
    .unwrap();

    // Ana sees her own chat.
    let (_, ana_list) = request(router.clone(), "GET", "/api/sessions", Some(&ana_token), None).await;
    assert!(ana_list["sessions"].as_array().unwrap().iter().any(|s| s["id"] == chat.as_str()));
    let (status, _) = request(router.clone(), "GET", &format!("/api/sessions/{chat}/messages"), Some(&ana_token), None).await;
    assert_eq!(status, StatusCode::OK);

    // Budi, in the same org, can't find it, read it, post into it or delete it.
    let (_, budi_list) = request(router.clone(), "GET", "/api/sessions", Some(&budi_token), None).await;
    assert!(budi_list["sessions"].as_array().unwrap().is_empty(), "{budi_list}");
    for (method, path, body) in [
        ("GET", format!("/api/sessions/{chat}/messages"), None),
        ("POST", format!("/api/sessions/{chat}/messages"), Some(json!({ "text": "what do you know about me?" }))),
        ("GET", format!("/api/sessions/{chat}/agent-activity"), None),
        ("DELETE", format!("/api/sessions/{chat}"), None),
    ] {
        let (status, _) = request(router.clone(), method, &path, Some(&budi_token), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}");
    }
    let budi_messages: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1::uuid AND sender_channel_identity_id IS NOT NULL")
        .bind(&chat)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(budi_messages, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_only_shows_your_own_chats(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, _, budi_token, _) = two_colleagues(router.clone(), &pool).await;

    let (_, created) = request(router.clone(), "POST", "/api/sessions", Some(&ana_token), None).await;
    let chat = created["session_id"].as_str().unwrap().to_string();
    sqlx::query("UPDATE sessions SET title = 'Ana private' WHERE id = $1::uuid").bind(&chat).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO messages (session_id, content, agent_display_name) VALUES ($1::uuid, 'Your therapist moved to Thursday.', 'Nomi')")
        .bind(&chat)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO messages (session_id, content, content_blocks, agent_display_name) VALUES ($1::uuid, 'To-do', \
         '[{\"kind\":\"todo_list\",\"items\":[{\"id\":\"1\",\"text\":\"a\",\"status\":\"pending\"}]}]', 'Planning')",
    )
    .bind(&chat)
    .execute(&pool)
    .await
    .unwrap();

    let (_, ana_home) = request(router.clone(), "GET", "/api/home", Some(&ana_token), None).await;
    assert!(ana_home.to_string().contains("Ana private"), "{ana_home}");

    let (status, budi_home) = request(router.clone(), "GET", "/api/home", Some(&budi_token), None).await;
    assert_eq!(status, StatusCode::OK);
    let text = budi_home.to_string();
    assert!(!text.contains("Ana private") && !text.contains("therapist") && !text.contains(&chat), "{budi_home}");
    assert!(budi_home["plans"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn memories_stay_with_the_person_they_are_about(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, ana, budi_token, _) = two_colleagues(router.clone(), &pool).await;

    let embedding = format!("[{}]", vec!["0.1"; 1536].join(","));
    let memory: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding, embedding_provider, embedding_model) \
         VALUES ($1, 'Ana is allergic to peanuts', $2::vector, 'fake', 'fake') RETURNING id",
    )
    .bind(ana)
    .bind(&embedding)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (_, ana_memories) = request(router.clone(), "GET", "/api/memory", Some(&ana_token), None).await;
    assert_eq!(ana_memories["memories"].as_array().unwrap().len(), 1);
    let (_, budi_memories) = request(router.clone(), "GET", "/api/memory", Some(&budi_token), None).await;
    assert!(budi_memories["memories"].as_array().unwrap().is_empty());
    let (status, _) = request(router, "DELETE", &format!("/api/memory/{memory}"), Some(&budi_token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
