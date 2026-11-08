// The issue board's crew: picking up Ready to dev issues in their own chats, one at a time or more,
// following hand-overs and marking them done or failed.
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

/// Ana's "Studio" organization with one project in it, picked for her projects.
async fn studio_with_a_project(router: &axum::Router, ana: &str) -> (String, String, String) {
    let (status, org) = call(router, "POST", "/api/orgs", json!({"name": "  Studio  "}), ana).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(org["name"], "Studio");
    let org_id = org["id"].as_str().unwrap().to_string();
    let (status, created) = call(router, "POST", "/api/projects", json!({"org_id": org_id}), ana).await;
    assert_eq!(status, StatusCode::CREATED);
    (org_id, created["project_id"].as_str().unwrap().to_string(), created["session_id"].as_str().unwrap().to_string())
}



use nomi_server::issue_worker::{sweep, Swept};

/// A project with issues "One", "Two" and "Three" all moved into Ready to dev, in that order.
async fn three_ready_issues(router: &axum::Router, ana: &str) -> String {
    let (_, project_id, _) = studio_with_a_project(router, ana).await;
    let issues = format!("/api/projects/{project_id}/issues");
    let (_, board) = call(router, "GET", &format!("/api/projects/{project_id}/board"), Value::Null, ana).await;
    let ready = board["columns"][1]["id"].as_str().unwrap().to_string();
    for (at, title) in ["One", "Two", "Three"].into_iter().enumerate() {
        call(router, "POST", &issues, json!({"title": title, "body": "Make it so.", "kind": "task", "severity": "medium", "points": 2}), ana).await;
        call(router, "POST", &format!("{issues}/{}/move", at + 1), json!({"column_id": ready, "index": at}), ana).await;
    }
    project_id
}

async fn states(pool: &PgPool) -> Vec<(i32, Option<String>)> {
    sqlx::query_as("SELECT number, crew_state FROM project_issues ORDER BY number").fetch_all(pool).await.unwrap()
}

async fn finish_turn(pool: &PgPool, number: i32, status: &str) {
    sqlx::query("UPDATE turn_jobs SET status = $2 WHERE id = (SELECT turn_job_id FROM project_issues WHERE number = $1)")
        .bind(number)
        .bind(status)
        .execute(pool)
        .await
        .unwrap();
}

fn s(state: &str) -> Option<String> {
    Some(state.to_string())
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_crew_takes_ready_issues_one_at_a_time_in_their_own_chats(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let project_id = three_ready_issues(&router, &ana).await;

    assert_eq!(sweep(&pool, 1).await.unwrap(), Swept { picked_up: 1, ..Swept::default() });
    assert_eq!(states(&pool).await, [(1, s("working")), (2, s("queued")), (3, s("queued"))]);

    // Its chat opens with the issue, addressed to Koda, as the person who queued it; the chat
    // belongs to the project, so the crew works on the project's files there.
    let (session_id, first, chat_project): (uuid::Uuid, String, Option<uuid::Uuid>) = sqlx::query_as(
        "SELECT i.session_id, m.content, chat_project(i.session_id) FROM project_issues i \
         JOIN messages m ON m.session_id = i.session_id WHERE i.number = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(first.starts_with("@Koda Issue #1: One\n\nType: task · Severity: medium · Points: 2"), "{first}");
    assert_eq!(chat_project.map(|p| p.to_string()), Some(project_id.clone()));
    let title: Option<String> = sqlx::query_scalar("SELECT title FROM sessions WHERE id = $1").bind(session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(title.as_deref(), Some("#1 · One"));
    let (status, issue) = call(&router, "GET", &format!("/api/projects/{project_id}/issues/1"), Value::Null, &ana).await;
    assert_eq!((status, issue["session_id"].as_str()), (StatusCode::OK, Some(session_id.to_string().as_str())));

    // Still going: the next waits.
    assert_eq!(sweep(&pool, 1).await.unwrap(), Swept::default());
    // Done: it stays in Ready to dev, open and marked done, and the next one starts.
    finish_turn(&pool, 1, "completed").await;
    assert_eq!(sweep(&pool, 1).await.unwrap(), Swept { picked_up: 1, finished: 1, ..Swept::default() });
    assert_eq!(states(&pool).await, [(1, s("done")), (2, s("working")), (3, s("queued"))]);
    let (_, board) = call(&router, "GET", &format!("/api/projects/{project_id}/board"), Value::Null, &ana).await;
    let ready: Vec<i64> = board["columns"][1]["issues"].as_array().unwrap().iter().map(|i| i["number"].as_i64().unwrap()).collect();
    assert_eq!(ready, [1, 2, 3]);

    // A failed turn marks it failed, and it can be tried again.
    finish_turn(&pool, 2, "failed").await;
    sweep(&pool, 1).await.unwrap();
    assert_eq!(states(&pool).await, [(1, s("done")), (2, s("failed")), (3, s("working"))]);
    assert_eq!(call(&router, "POST", &format!("/api/projects/{project_id}/issues/2/retry"), Value::Null, &ana).await.0, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn with_room_for_two_two_run_and_the_third_waits(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    three_ready_issues(&router, &ana).await;

    assert_eq!(sweep(&pool, 2).await.unwrap().picked_up, 2);
    assert_eq!(sweep(&pool, 2).await.unwrap().picked_up, 0);
    assert_eq!(states(&pool).await, [(1, s("working")), (2, s("working")), (3, s("queued"))]);
    let chats: i64 = sqlx::query_scalar("SELECT count(DISTINCT session_id) FROM project_issues WHERE session_id IS NOT NULL").fetch_one(&pool).await.unwrap();
    assert_eq!(chats, 2);

    // Each knows what else runs beside it.
    let session: uuid::Uuid = sqlx::query_scalar("SELECT session_id FROM project_issues WHERE number = 1").fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let prompt = nomi_agent_core::issue_chat::prompt_for(&mut conn, session).await.unwrap();
    assert!(prompt.contains("issue #1 \"One\"") && prompt.contains("- #2 \"Two\" (Koda)"), "{prompt}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_handed_on_issue_follows_its_new_crew_member_until_they_finish(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    three_ready_issues(&router, &ana).await;
    sweep(&pool, 1).await.unwrap();

    // Koda finishes his turn by handing the issue to Dana, who is still on it.
    finish_turn(&pool, 1, "completed").await;
    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) \
         SELECT i.session_id, s.user_id, 'coding', 'money', 'Work out the totals', 'processing' \
         FROM project_issues i JOIN sessions s ON s.id = i.session_id WHERE i.number = 1",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(sweep(&pool, 1).await.unwrap(), Swept { handed_on: 1, ..Swept::default() });
    let (agent, by, state): (String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT crew_agent, assigned_by_agent, crew_state FROM project_issues WHERE number = 1").fetch_one(&pool).await.unwrap();
    assert_eq!((agent.as_str(), by.as_deref(), state.as_deref()), ("money", Some("coding"), Some("working")));

    sqlx::query("UPDATE agent_delegations SET status = 'completed'").execute(&pool).await.unwrap();
    assert_eq!(sweep(&pool, 1).await.unwrap(), Swept { picked_up: 1, finished: 1, ..Swept::default() });
    assert_eq!(states(&pool).await[0], (1, s("done")));
}
