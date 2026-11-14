//! Memory settings, the profile, and who learned each memory.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
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

async fn request(
    router: axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
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
    (
        status,
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        },
    )
}

async fn login(router: axum::Router, email: &str) -> String {
    let (_, body) = request(
        router,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "email": email, "password": "correct-password" })),
    )
    .await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn register(router: axum::Router, email: &str) -> String {
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    login(router, email).await
}

async fn user_id(pool: &PgPool, email: &str) -> uuid::Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn seed_memory(
    pool: &PgPool,
    user: uuid::Uuid,
    content: &str,
    weight: f64,
    learned_by: Option<&str>,
) -> uuid::Uuid {
    let embedding = format!("[{}]", vec!["0.01"; 1536].join(","));
    sqlx::query_scalar("INSERT INTO memory_items (user_id, content, embedding, weight, learned_by) VALUES ($1, $2, $3::vector, $4, $5) RETURNING id")
        .bind(user)
        .bind(content)
        .bind(&embedding)
        .bind(weight)
        .bind(learned_by)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn settings_start_on_with_default_budgets_and_change_only_what_is_sent(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mem-settings@example.com").await;
    let me = user_id(&pool, "mem-settings@example.com").await;
    seed_memory(&pool, me, "Vegetarian", 1.0, Some("chitchat")).await;

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/memory/settings",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["learn_from_chats"], true);
    assert_eq!(body["keep_profile"], true);
    assert_eq!(body["search_past_chats"], true);
    assert_eq!(body["memory_budget"], 8000);
    assert_eq!(body["profile_budget"], 1000);
    assert_eq!(body["memory_used"], 3, "\"Vegetarian\" is about 3 tokens");
    assert_eq!(body["profile_used"], 0);

    let (status, body) = request(
        router.clone(),
        "PUT",
        "/api/memory/settings",
        Some(&token),
        Some(json!({ "search_past_chats": false, "profile_budget": 500 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["search_past_chats"], false);
    assert_eq!(body["profile_budget"], 500);
    assert_eq!(body["learn_from_chats"], true, "unsent fields stay");

    for bad in [
        json!({ "memory_budget": 10 }),
        json!({ "profile_budget": 1_000_000 }),
    ] {
        let (status, body) = request(
            router.clone(),
            "PUT",
            "/api/memory/settings",
            Some(&token),
            Some(bad),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "invalid_budget");
    }
    let (status, _) = request(router, "GET", "/api/memory/settings", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lower_memory_budget_retires_the_weakest_memories(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mem-budget@example.com").await;
    let me = user_id(&pool, "mem-budget@example.com").await;
    // 400 characters each: 100 tokens.
    let strong = seed_memory(&pool, me, &"s".repeat(400), 3.0, None).await;
    let weak = seed_memory(&pool, me, &"w".repeat(400), 0.5, None).await;
    let lots = "m".repeat(400 * 5);
    seed_memory(&pool, me, &lots, 2.0, None).await;

    let (status, body) = request(
        router.clone(),
        "PUT",
        "/api/memory/settings",
        Some(&token),
        Some(json!({ "memory_budget": 600 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["memory_used"], 600);
    let archived: Vec<uuid::Uuid> = sqlx::query_scalar(
        "SELECT id FROM memory_items WHERE user_id = $1 AND archived_at IS NOT NULL",
    )
    .bind(me)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(archived, vec![weak]);
    let (_, list) = request(router, "GET", "/api/memory", Some(&token), None).await;
    assert!(list["memories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["id"] == strong.to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn turning_learning_off_drops_what_was_queued_and_queues_nothing(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mem-learn@example.com").await;
    let me = user_id(&pool, "mem-learn@example.com").await;
    let org: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let session: uuid::Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', $2, $3) RETURNING id")
        .bind(org)
        .bind(me.to_string())
        .bind(me)
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::memory::queue_learning(
        &mut conn,
        me,
        session,
        "chitchat",
        "I take the MRT",
        "Noted!",
    )
    .await;
    let queued = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM memory_jobs WHERE user_id = $1")
            .bind(me)
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    assert_eq!(queued().await, 1);
    let agent: Option<String> =
        sqlx::query_scalar("SELECT agent_type FROM memory_jobs WHERE user_id = $1")
            .bind(me)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(agent.as_deref(), Some("chitchat"));

    let (status, _) = request(
        router,
        "PUT",
        "/api/memory/settings",
        Some(&token),
        Some(json!({ "learn_from_chats": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(queued().await, 0);
    nomi_agent_core::memory::queue_learning(
        &mut conn,
        me,
        session,
        "chitchat",
        "I take the bus now",
        "Noted!",
    )
    .await;
    assert_eq!(queued().await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_profile_is_read_and_edited_within_its_budget(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mem-profile@example.com").await;

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/memory/profile",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "text": "", "tokens": 0, "updated_at": null }));

    let (status, body) = request(
        router.clone(),
        "PUT",
        "/api/memory/profile",
        Some(&token),
        Some(json!({ "text": "  Designer in Jakarta. Vegetarian.  " })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["text"], "Designer in Jakarta. Vegetarian.");
    assert_eq!(body["tokens"], 8);
    assert!(body["updated_at"].is_string());

    request(
        router.clone(),
        "PUT",
        "/api/memory/settings",
        Some(&token),
        Some(json!({ "profile_budget": 100 })),
    )
    .await;
    let (status, body) = request(
        router.clone(),
        "PUT",
        "/api/memory/profile",
        Some(&token),
        Some(json!({ "text": "x".repeat(401) })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "too_long");

    let (_, settings) = request(router, "GET", "/api/memory/settings", Some(&token), None).await;
    assert_eq!(settings["profile_used"], 8);
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_memory_says_who_learned_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mem-learned-by@example.com").await;
    let me = user_id(&pool, "mem-learned-by@example.com").await;
    seed_memory(&pool, me, "Runs on Sundays", 1.0, Some("health")).await;
    seed_memory(&pool, me, "Old memory", 1.0, None).await;

    let (_, body) = request(router, "GET", "/api/memory", Some(&token), None).await;
    let by: Vec<Value> = body["memories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["learned_by"].clone())
        .collect();
    assert!(by.contains(&json!("health")));
    assert!(by.contains(&Value::Null));
}
