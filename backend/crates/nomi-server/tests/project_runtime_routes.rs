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
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
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
    let (_, body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    body["access_token"].as_str().unwrap().to_string()
}

// sessions has no user_id column (migration 0004_sessions.sql: org_id, channel, chat_id) — seed
// a fresh org per call so this works regardless of whether user_id came from registration (which
// creates its own org) or a raw `INSERT INTO users DEFAULT VALUES`.
async fn seed_project(pool: &PgPool, user_id: Uuid) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, description, plan) VALUES ($1, $2, 'Todo app', 'a simple list', '1. index.html') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn user_id_by_email(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}


#[sqlx::test(migrations = "../../migrations")]
async fn the_preview_page_loads_files_syncs_edits_and_runs_commands(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "maker@example.com").await;
    let user_id = user_id_by_email(&pool, "maker@example.com").await;
    let project_id = seed_project(&pool, user_id).await;
    let base = format!("/api/projects/{project_id}");

    let (status, _) = json_request(router.clone(), "PUT", &format!("{base}/files/src/app.ts"), json!({"content": "export const a = 1;"}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);

    let (_, detail) = json_request(router.clone(), "GET", &base, json!(null), Some(&token)).await;
    assert_eq!((detail["stack"].as_str(), detail["files_version"].as_i64()), (Some("sveltekit"), Some(1)));

    let (status, snapshot) = json_request(router.clone(), "GET", &format!("{base}/snapshot"), json!(null), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot["files_version"], 1);
    assert_eq!(snapshot["files"][0]["path"], "src/app.ts");
    assert_eq!(snapshot["files"][0]["content"], "export const a = 1;");

    // Up to date: the check-in waits, then answers with no commands. Behind: it answers at once.
    let started = std::time::Instant::now();
    let (_, check) = json_request(router.clone(), "GET", &format!("{base}/runtime?since=1&wait=1"), json!(null), Some(&token)).await;
    assert!(started.elapsed() >= std::time::Duration::from_millis(900));
    assert_eq!(check, json!({"files_version": 1, "runs": []}));
    let started = std::time::Instant::now();
    json_request(router.clone(), "GET", &format!("{base}/runtime?since=0&wait=20"), json!(null), Some(&token)).await;
    assert!(started.elapsed() < std::time::Duration::from_secs(2));

    // A queued command is handed over and its output recorded.
    let run_id: Uuid = sqlx::query_scalar("INSERT INTO project_runs (project_id, command, args) VALUES ($1, 'npm', '[\"run\", \"build\"]') RETURNING id")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (_, check) = json_request(router.clone(), "GET", &format!("{base}/runtime?since=1&wait=5"), json!(null), Some(&token)).await;
    assert_eq!(check["runs"][0], json!({"id": run_id, "command": "npm", "args": ["run", "build"]}));
    let (status, _) = json_request(router.clone(), "POST", &format!("{base}/runtime/runs/{run_id}"), json!({"exit_code": 0, "output": "built"}), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (run_status, output): (String, String) = sqlx::query_as("SELECT status, output FROM project_runs WHERE id = $1").bind(run_id).fetch_one(&pool).await.unwrap();
    assert_eq!((run_status.as_str(), output.as_str()), ("done", "built"));

    // A failed check on opening goes to Koda once.
    let (_, first) = json_request(router.clone(), "POST", &format!("{base}/runtime/check"), json!({"files_version": 1, "ok": false, "output": "Type error"}), Some(&token)).await;
    let (_, again) = json_request(router.clone(), "POST", &format!("{base}/runtime/check"), json!({"files_version": 1, "ok": false, "output": "Type error"}), Some(&token)).await;
    assert_eq!((first["handed_to_koda"].as_bool(), again["handed_to_koda"].as_bool()), (Some(true), Some(false)));
    let task: String = sqlx::query_scalar("SELECT task FROM agent_delegations WHERE target_agent_type = 'coding'").fetch_one(&pool).await.unwrap();
    assert!(task.starts_with(&format!("Project {project_id}:")) && task.contains("Type error"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_preview_is_out_of_reach(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "intruder@example.com").await;
    let owner: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let project_id = seed_project(&pool, owner).await;
    for (method, path) in [("GET", "snapshot"), ("GET", "manifest"), ("GET", "runtime"), ("POST", "runtime/check")] {
        let body = if method == "POST" { json!({"files_version": 0, "ok": true, "output": ""}) } else { json!(null) };
        let (status, _) = json_request(router.clone(), method, &format!("/api/projects/{project_id}/{path}"), body, Some(&token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
}
