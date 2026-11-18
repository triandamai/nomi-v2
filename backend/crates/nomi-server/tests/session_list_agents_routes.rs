//! The chat list says who last answered in each chat and which crew have.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool, shipyard: Option<&MockServer>) -> AppState {
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
        shipyard: shipyard.map(|server| nomi_shipyard::ShipyardConfig {
            base_url: server.uri(),
            key: "ship_int_manage".into(),
            reveal_key: Some("ship_int_reveal".into()),
            webhook_secret: None,
        }),
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

async fn register(router: axum::Router, email: &str) -> String {
    request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}



async fn login_and_project(pool: &PgPool) -> (axum::Router, String, Uuid) {
    let router = build_router(test_state(pool.clone(), None));
    let ana = register(router.clone(), "ana@acme.test").await;
    // A project goes in the organization chosen for projects, so choose Ana's first.
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(&ana), None).await;
    let org_id: Uuid = orgs["orgs"][0]["id"].as_str().unwrap().parse().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(&ana), Some(json!({ "org_id": org_id }))).await;
    let (_, created) = request(router.clone(), "POST", "/api/projects", Some(&ana), None).await;
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();
    let session: Uuid = sqlx::query_scalar("SELECT session_id FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap();
    (router, ana, session)
}

async fn say(pool: &PgPool, session: Uuid, agent: Option<&str>, text: &str, seconds_ago: i32) {
    let sender: Option<Uuid> = if agent.is_none() {
        Some(
            sqlx::query_scalar(
                "INSERT INTO channel_identities (user_id, channel, channel_user_id) SELECT user_id, 'web', 'list-test' FROM web_credentials WHERE email = 'ana@acme.test' \
                 ON CONFLICT (channel, channel_user_id) DO UPDATE SET user_id = EXCLUDED.user_id RETURNING id",
            )
            .fetch_one(pool)
            .await
            .unwrap(),
        )
    } else {
        None
    };
    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name, created_at) VALUES ($1, $2, $3, $4, now() - make_interval(secs => $5))")
        .bind(session)
        .bind(sender)
        .bind(text)
        .bind(agent)
        .bind(seconds_ago)
        .execute(pool)
        .await
        .unwrap();
}

fn mine(list: &Value, session: Uuid) -> &Value {
    list["sessions"].as_array().unwrap().iter().find(|s| s["id"] == session.to_string()).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_list_names_who_answered_last_and_the_crew_who_answered(pool: PgPool) {
    let (router, ana, session) = login_and_project(&pool).await;
    say(&pool, session, None, "Add a menu page", 40).await;
    say(&pool, session, Some("Koda"), "Added it.", 30).await;
    say(&pool, session, Some("Dana"), "Rp 2,1 jt on food.", 20).await;
    say(&pool, session, Some("Koda"), "Tidied up.", 10).await;
    let (_, list) = request(router.clone(), "GET", "/api/sessions", Some(&ana), None).await;
    let row = mine(&list, session);
    assert_eq!(row["last_message"]["agent_type"], "coding");
    assert_eq!(row["agents"], json!(["coding", "money"]));

    // A person's own message last: no agent on it, the crew list stays.
    say(&pool, session, None, "Thanks", 0).await;
    let (_, list) = request(router, "GET", "/api/sessions", Some(&ana), None).await;
    let row = mine(&list, session);
    assert!(row["last_message"]["agent_type"].is_null());
    assert_eq!(row["agents"], json!(["coding", "money"]));
}
