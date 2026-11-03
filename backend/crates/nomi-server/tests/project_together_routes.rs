//! Building together: roles (a viewer changes nothing), the joined note, and the asks in line.
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

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}


/// Ana's project in her organization, with Ben as an active member of that organization (so he
/// can open it), and Cy in an organization of his own (so he can't).
struct Team {
    router: axum::Router,
    project_id: Uuid,
    ana: String,
    ben: String,
    cy: String,
}

async fn team(pool: &PgPool) -> Team {
    let router = build_router(test_state(pool.clone(), None));
    let ana = register(router.clone(), "ana@acme.test").await;
    let ben = register(router.clone(), "ben@acme.test").await;
    let cy = register(router.clone(), "cy@other.test").await;
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(&ana), None).await;
    let org_id: Uuid = orgs["orgs"][0]["id"].as_str().unwrap().parse().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(&ana), Some(json!({ "org_id": org_id }))).await;
    let (_, created) = request(router.clone(), "POST", "/api/projects", Some(&ana), None).await;
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();
    let ben_id = user_id(pool, "ben@acme.test").await;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')").bind(org_id).bind(ben_id).execute(pool).await.unwrap();
    Team { router, project_id, ana, ben, cy }
}


async fn project_session(pool: &PgPool, project_id: Uuid) -> Uuid {
    sqlx::query_scalar("SELECT session_id FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_viewer_watches_but_changes_nothing_and_joining_is_said_in_the_chat(pool: PgPool) {
    let t = team(&pool).await;
    let ben_id = user_id(&pool, "ben@acme.test").await;
    let (status, members) = request(t.router.clone(), "PUT", &format!("/api/projects/{}/members/{ben_id}", t.project_id), Some(&t.ana), Some(json!({ "role": "viewer" }))).await;
    assert_eq!(status, StatusCode::OK, "{members}");

    // The project's chat says Ben joined, and what he can do.
    let session = project_session(&pool, t.project_id).await;
    let blocks: serde_json::Value = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE session_id = $1 AND content_blocks->0->>'kind' = 'member_joined'")
        .bind(session)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((blocks[0]["name"].as_str(), blocks[0]["role"].as_str(), blocks[0]["publisher"].as_str()), (Some("ben"), Some("view"), Some("ana")));

    // A viewer sees the deploy overview as a viewer, and can't write to the chat, the files, or ask to publish.
    let (_, overview) = request(t.router.clone(), "GET", &format!("/api/projects/{}/deploy", t.project_id), Some(&t.ben), None).await;
    assert_eq!(overview["role"], "viewer");
    let (status, _) = request(t.router.clone(), "POST", &format!("/api/sessions/{session}/messages"), Some(&t.ben), Some(json!({ "text": "make it blue" }))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = request(t.router.clone(), "PUT", &format!("/api/projects/{}/files/index.html", t.project_id), Some(&t.ben), Some(json!({ "content": "<h1>hi</h1>" }))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = request(t.router.clone(), "POST", &format!("/api/projects/{}/deploy/requests", t.project_id), Some(&t.ben), Some(json!({}))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Changing his role later doesn't announce him again.
    request(t.router.clone(), "PUT", &format!("/api/projects/{}/members/{ben_id}", t.project_id), Some(&t.ana), Some(json!({ "role": "member" }))).await;
    let joined: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1 AND content_blocks->0->>'kind' = 'member_joined'")
        .bind(session)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(joined, 1);
    let (status, _) = request(t.router.clone(), "PUT", &format!("/api/projects/{}/members/{ben_id}", t.project_id), Some(&t.ana), Some(json!({ "role": "owner" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn asks_wait_in_line_and_one_can_be_moved_up(pool: PgPool) {
    let t = team(&pool).await;
    let session = project_session(&pool, t.project_id).await;
    let ana_id = user_id(&pool, "ana@acme.test").await;
    let ben_id = user_id(&pool, "ben@acme.test").await;
    let mut ids = Vec::new();
    for (who, text, status, minutes) in [(ana_id, "Add a menu page", "processing", 3), (ana_id, "Make it blue", "pending", 2), (ben_id, "Add opening hours", "pending", 1)] {
        let identity: Uuid = sqlx::query_scalar(
            "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', $2) \
             ON CONFLICT (channel, channel_user_id) DO UPDATE SET user_id = EXCLUDED.user_id RETURNING id",
        )
        .bind(who)
        .bind(format!("q-{who}"))
        .fetch_one(&pool)
        .await
        .unwrap();
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO turn_jobs (session_id, sender_channel_identity_id, text, status, created_at) VALUES ($1, $2, $3, $4, now() - make_interval(mins => $5)) RETURNING id",
        )
        .bind(session)
        .bind(identity)
        .bind(text)
        .bind(status)
        .bind(minutes)
        .fetch_one(&pool)
        .await
        .unwrap();
        ids.push(id);
    }
    let order = |body: &Value| body["asks"].as_array().unwrap().iter().map(|a| (a["text"].as_str().unwrap().to_string(), a["state"].as_str().unwrap().to_string())).collect::<Vec<_>>();
    let (_, queue) = request(t.router.clone(), "GET", &format!("/api/sessions/{session}/queue"), Some(&t.ben), None).await;
    assert_eq!(
        order(&queue),
        vec![("Add a menu page".into(), "running".into()), ("Make it blue".into(), "next".into()), ("Add opening hours".into(), "queued".into())]
    );
    assert_eq!(queue["asks"][2]["who"], "ben");
    assert_eq!(queue["asks"][2]["mine"], true);

    let (status, moved) = request(t.router.clone(), "POST", &format!("/api/sessions/{session}/queue/{}/first", ids[2]), Some(&t.ben), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(order(&moved)[1], ("Add opening hours".into(), "next".into()));
    // The worker answers it next.
    let next = nomi_turn::queue::claim_next(&pool).await.unwrap();
    assert!(next.is_none(), "the chat is still busy with its running ask");
    sqlx::query("UPDATE turn_jobs SET status = 'completed' WHERE id = $1").bind(ids[0]).execute(&pool).await.unwrap();
    assert_eq!(nomi_turn::queue::claim_next(&pool).await.unwrap().unwrap().text, "Add opening hours");

    // The running one can't move.
    let (status, _) = request(t.router.clone(), "POST", &format!("/api/sessions/{session}/queue/{}/first", ids[0]), Some(&t.ben), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn inviting_someone_to_build_puts_them_on_the_project_when_they_accept(pool: PgPool) {
    let t = team(&pool).await;
    let invites = format!("/api/projects/{}/invites", t.project_id);

    // Someone already in the organization is added straight away.
    let (status, added) = request(t.router.clone(), "POST", &invites, Some(&t.ana), Some(json!({ "email": "ben@acme.test", "role": "prompt" }))).await;
    assert_eq!(status, StatusCode::OK, "{added}");
    assert_eq!(added["added"], true);

    // Anyone else gets an invite to the organization that's also for this project.
    let (status, sent) = request(t.router.clone(), "POST", &invites, Some(&t.ana), Some(json!({ "email": "dee@new.test", "role": "view" }))).await;
    assert_eq!(status, StatusCode::CREATED, "{sent}");
    let path = sent["path"].as_str().unwrap().to_string();
    let code = path.trim_start_matches("/invite/");
    // Only the project's admins invite.
    let (status, _) = request(t.router.clone(), "POST", &invites, Some(&t.ben), Some(json!({ "email": "eve@new.test", "role": "view" }))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let dee = register(t.router.clone(), "dee@new.test").await;
    let (_, info) = request(t.router.clone(), "GET", &format!("/api/invites/{code}"), Some(&dee), None).await;
    assert_eq!((info["project"]["role"].as_str(), info["state"].as_str()), (Some("view"), Some("open")), "{info}");
    let (status, accepted) = request(t.router.clone(), "POST", &format!("/api/invites/{code}/accept"), Some(&dee), None).await;
    assert_eq!(status, StatusCode::OK, "{accepted}");
    let session = project_session(&pool, t.project_id).await;
    assert_eq!(accepted["project_session_id"], json!(session));

    let dee_id = user_id(&pool, "dee@new.test").await;
    let role: String = sqlx::query_scalar("SELECT role FROM project_members WHERE project_id = $1 AND user_id = $2")
        .bind(t.project_id)
        .bind(dee_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(role, "viewer");
    let joined: Vec<String> = sqlx::query_scalar("SELECT content_blocks->0->>'name' FROM messages WHERE session_id = $1 AND content_blocks->0->>'kind' = 'member_joined' ORDER BY created_at")
        .bind(session)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(joined, vec!["ben".to_string(), "dee".to_string()]);
}
