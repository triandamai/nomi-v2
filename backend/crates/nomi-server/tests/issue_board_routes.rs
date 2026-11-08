// The issue board: fixed and added columns, moving issues, and Ready to dev queuing them for the crew.
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


/// The board's columns as (name, fixed, issue numbers).
fn columns(board: &Value) -> Vec<(String, Option<String>, Vec<i64>)> {
    board["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["name"].as_str().unwrap().to_string(),
                c["fixed"].as_str().map(str::to_string),
                c["issues"].as_array().unwrap().iter().map(|i| i["number"].as_i64().unwrap()).collect(),
            )
        })
        .collect()
}

fn column_id(board: &Value, name: &str) -> String {
    board["columns"].as_array().unwrap().iter().find(|c| c["name"] == name).unwrap()["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn new_issues_land_in_backlog_and_fixed_columns_stay_put(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let (_, project_id, _) = studio_with_a_project(&router, &ana).await;
    let issues = format!("/api/projects/{project_id}/issues");
    let board_url = format!("/api/projects/{project_id}/board");

    let (status, first) = call(
        &router,
        "POST",
        &issues,
        json!({"title": "Login button does nothing", "kind": "bug", "severity": "high", "points": 3, "start_on": "2026-11-09", "due_on": "2026-11-12"}),
        &ana,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!((first["points"].as_i64(), first["due_on"].as_str(), first["crew_agent"].as_str()), (Some(3), Some("2026-11-12"), Some("coding")));
    call(&router, "POST", &issues, json!({"title": "Dark mode", "kind": "enhancement", "severity": "low", "crew_agent": "planning"}), &ana).await;
    // A feature is a task now; the deadline can't come before the start; only the crew take issues.
    assert_eq!(call(&router, "POST", &issues, json!({"title": "x", "kind": "feature", "severity": "low"}), &ana).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(call(&router, "POST", &issues, json!({"title": "x", "kind": "task", "severity": "low", "start_on": "2026-11-12", "due_on": "2026-11-09"}), &ana).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(call(&router, "POST", &issues, json!({"title": "x", "kind": "task", "severity": "low", "crew_agent": "someone"}), &ana).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(call(&router, "PATCH", &format!("{issues}/1"), json!({"start_on": "2026-11-20"}), &ana).await.0, StatusCode::BAD_REQUEST);
    let (_, cleared) = call(&router, "PATCH", &format!("{issues}/1"), json!({"due_on": null, "points": 5}), &ana).await;
    assert_eq!((cleared["due_on"].clone(), cleared["points"].as_i64(), cleared["start_on"].as_str()), (Value::Null, Some(5), Some("2026-11-09")));

    let (status, board) = call(&router, "GET", &board_url, Value::Null, &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        columns(&board),
        [("Backlog".to_string(), Some("backlog".to_string()), vec![1, 2]), ("Ready to dev".to_string(), Some("ready".to_string()), vec![])]
    );

    let (status, _) = call(&router, "POST", &format!("{board_url}/columns"), json!({"name": "In review"}), &ana).await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, board) = call(&router, "POST", &format!("{board_url}/columns"), json!({"name": "Shipped"}), &ana).await;
    let names: Vec<String> = columns(&board).into_iter().map(|c| c.0).collect();
    assert_eq!(names, ["Backlog", "Ready to dev", "In review", "Shipped"]);

    // The fixed columns can't be renamed, moved or deleted.
    let backlog = column_id(&board, "Backlog");
    assert_eq!(call(&router, "PATCH", &format!("{board_url}/columns/{backlog}"), json!({"name": "Inbox"}), &ana).await.0, StatusCode::CONFLICT);
    assert_eq!(call(&router, "DELETE", &format!("{board_url}/columns/{backlog}"), Value::Null, &ana).await.0, StatusCode::CONFLICT);

    // Added columns can: renamed, and moved, but never ahead of the fixed ones.
    let shipped = column_id(&board, "Shipped");
    let (_, board) = call(&router, "PATCH", &format!("{board_url}/columns/{shipped}"), json!({"name": "Live", "index": 0}), &ana).await;
    let names: Vec<String> = columns(&board).into_iter().map(|c| c.0).collect();
    assert_eq!(names, ["Backlog", "Ready to dev", "Live", "In review"]);

    // Deleting a column sends its issues back to Backlog.
    let review = column_id(&board, "In review");
    let (status, _) = call(&router, "POST", &format!("{issues}/2/move"), json!({"column_id": review, "index": 0}), &ana).await;
    assert_eq!(status, StatusCode::OK);
    let (_, board) = call(&router, "DELETE", &format!("{board_url}/columns/{review}"), Value::Null, &ana).await;
    assert_eq!(columns(&board)[0].2, vec![1, 2]);
    assert_eq!(columns(&board).len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn moving_into_ready_to_dev_queues_it_for_the_crew(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let (_, project_id, _) = studio_with_a_project(&router, &ana).await;
    let issues = format!("/api/projects/{project_id}/issues");
    for title in ["One", "Two", "Three"] {
        call(&router, "POST", &issues, json!({"title": title, "kind": "task", "severity": "medium"}), &ana).await;
    }
    let (_, board) = call(&router, "GET", &format!("/api/projects/{project_id}/board"), Value::Null, &ana).await;
    let ready = column_id(&board, "Ready to dev");
    let backlog = column_id(&board, "Backlog");

    let (_, moved) = call(&router, "POST", &format!("{issues}/1/move"), json!({"column_id": ready, "index": 0}), &ana).await;
    assert_eq!(moved["crew_state"], "queued");
    // A move puts it between its new neighbours.
    call(&router, "POST", &format!("{issues}/3/move"), json!({"column_id": ready, "index": 0}), &ana).await;
    call(&router, "POST", &format!("{issues}/2/move"), json!({"column_id": ready, "index": 1}), &ana).await;
    let (_, board) = call(&router, "GET", &format!("/api/projects/{project_id}/board"), Value::Null, &ana).await;
    assert_eq!(columns(&board)[1].2, vec![3, 2, 1]);

    // Out again before the crew took it: no longer queued.
    let (_, back) = call(&router, "POST", &format!("{issues}/2/move"), json!({"column_id": backlog, "index": 0}), &ana).await;
    assert_eq!(back["crew_state"], Value::Null);

    // Only a failed one can be tried again.
    assert_eq!(call(&router, "POST", &format!("{issues}/1/retry"), Value::Null, &ana).await.0, StatusCode::CONFLICT);
    sqlx::query("UPDATE project_issues SET crew_state = 'failed' WHERE number = 1").execute(&pool).await.unwrap();
    let (status, retried) = call(&router, "POST", &format!("{issues}/1/retry"), Value::Null, &ana).await;
    assert_eq!((status, retried["crew_state"].as_str()), (StatusCode::OK, Some("queued")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn work_immediately_files_an_issue_straight_into_ready_to_dev(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let (_, project_id, _) = studio_with_a_project(&router, &ana).await;
    let base = format!("/api/projects/{project_id}");

    let (status, later) = call(&router, "POST", &format!("{base}/issues"), json!({"title": "Later", "kind": "task", "severity": "low"}), &ana).await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(later["crew_state"].is_null());

    let (status, now) = call(&router, "POST", &format!("{base}/issues"), json!({"title": "Now", "kind": "bug", "severity": "high", "start_now": true, "crew_agent": "money"}), &ana).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(now["crew_state"], "queued");
    assert_eq!(now["crew_agent"], "money");
    assert_eq!(now["start_on"].as_str(), Some(chrono::Utc::now().date_naive().to_string().as_str()));

    // A start date given with it is kept, and still checked against the deadline.
    let (status, _) = call(&router, "POST", &format!("{base}/issues"), json!({"title": "Dated", "kind": "task", "severity": "low", "start_now": true, "start_on": "2026-12-01", "due_on": "2026-11-01"}), &ana).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, board) = call(&router, "GET", &format!("{base}/board"), Value::Null, &ana).await;
    let shown = columns(&board);
    assert_eq!(shown[0], ("Backlog".to_string(), Some("backlog".to_string()), vec![1]));
    assert_eq!(shown[1], ("Ready to dev".to_string(), Some("ready".to_string()), vec![2]));
    let queued_by: Option<uuid::Uuid> = sqlx::query_scalar("SELECT queued_by FROM project_issues WHERE number = 2").fetch_one(&pool).await.unwrap();
    assert!(queued_by.is_some());
}
