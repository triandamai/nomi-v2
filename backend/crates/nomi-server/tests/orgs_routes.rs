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

#[sqlx::test(migrations = "../../migrations")]
async fn an_invited_teammate_joins_and_works_on_the_projects(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let (org_id, project_id, session_id) = studio_with_a_project(&router, &ana).await;

    // Before joining, Bob can't see the organization, its project or its chat.
    assert_eq!(call(&router, "GET", &format!("/api/orgs/{org_id}"), Value::Null, &bob).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&router, "GET", &format!("/api/projects/{project_id}"), Value::Null, &bob).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, &bob).await.0, StatusCode::NOT_FOUND);

    let (status, invite) = call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": " Bob@Example.com "}), &ana).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(invite["email"], "bob@example.com");
    assert_eq!(invite["role"], "member");
    let code = invite["code"].as_str().unwrap().to_string();

    // Bob already has an account, so it's in his inbox.
    let (_, inbox) = call(&router, "GET", "/api/notifications", Value::Null, &bob).await;
    assert_eq!(inbox["items"][0]["link"], format!("/invite/{code}"));

    // Only the invited address can use it.
    let carol = sign_up(&router, "carol@example.com").await;
    let (_, info) = call(&router, "GET", &format!("/api/invites/{code}"), Value::Null, &carol).await;
    assert_eq!(info["state"], "other_address");
    assert_eq!(call(&router, "POST", &format!("/api/invites/{code}/accept"), Value::Null, &carol).await.0, StatusCode::FORBIDDEN);

    let (_, info) = call(&router, "GET", &format!("/api/invites/{code}"), Value::Null, &bob).await;
    assert_eq!(info["state"], "open");
    assert_eq!(info["org_name"], "Studio");
    assert_eq!(info["inviter"], "ana");
    let (status, accepted) = call(&router, "POST", &format!("/api/invites/{code}/accept"), Value::Null, &bob).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(accepted["org_id"], org_id.as_str());

    // Joining makes Studio his projects' organization, so its project is listed.
    let (_, listed) = call(&router, "GET", "/api/projects", Value::Null, &bob).await;
    assert_eq!(listed["org_id"], org_id.as_str());
    assert_eq!(listed["projects"].as_array().unwrap().len(), 1);
    assert_eq!(call(&router, "GET", &format!("/api/projects/{project_id}"), Value::Null, &bob).await.0, StatusCode::OK);
    assert_eq!(call(&router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, &bob).await.0, StatusCode::OK);
    // He works in Ana's project chat, and she sees who wrote what; it's still hers to delete.
    assert_eq!(call(&router, "POST", &format!("/api/sessions/{session_id}/messages"), json!({"text": "Can we add a menu page?"}), &bob).await.0, StatusCode::ACCEPTED);
    let (_, thread) = call(&router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, &ana).await;
    assert_eq!(thread["messages"][0]["author_name"], "bob");
    let (_, thread) = call(&router, "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, &bob).await;
    assert_eq!(thread["messages"][0]["author_name"], Value::Null);
    assert_eq!(call(&router, "DELETE", &format!("/api/sessions/{session_id}"), Value::Null, &bob).await.0, StatusCode::FORBIDDEN);

    let (_, org) = call(&router, "GET", &format!("/api/orgs/{org_id}"), Value::Null, &ana).await;
    let members: Vec<&str> = org["members"].as_array().unwrap().iter().map(|m| m["name"].as_str().unwrap()).collect();
    assert_eq!(members, ["ana", "bob"]);
    assert!(org["invites"].as_array().unwrap().is_empty());

    // Ana hears that he joined, and the invite can't be used again.
    let (_, inbox) = call(&router, "GET", "/api/notifications", Value::Null, &ana).await;
    assert_eq!(inbox["items"][0]["link"], format!("/organizations/{org_id}"));
    let (_, info) = call(&router, "GET", &format!("/api/invites/{code}"), Value::Null, &bob).await;
    assert_eq!(info["state"], "member");

    // Once removed, he's out again.
    assert_eq!(call(&router, "DELETE", &format!("/api/orgs/{org_id}/members/{}", org["members"][1]["user_id"].as_str().unwrap()), Value::Null, &ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(call(&router, "GET", &format!("/api/projects/{project_id}"), Value::Null, &bob).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn roles_decide_who_manages_the_team(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let (org_id, _, _) = studio_with_a_project(&router, &ana).await;
    let (_, invite) = call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "bob@example.com"}), &ana).await;
    call(&router, "POST", &format!("/api/invites/{}/accept", invite["code"].as_str().unwrap()), Value::Null, &bob).await;
    let (_, org) = call(&router, "GET", &format!("/api/orgs/{org_id}"), Value::Null, &ana).await;
    let ana_id = org["members"][0]["user_id"].as_str().unwrap().to_string();
    let bob_id = org["members"][1]["user_id"].as_str().unwrap().to_string();

    // A member can't invite, rename or see the open invites.
    assert_eq!(call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "carol@example.com"}), &bob).await.0, StatusCode::FORBIDDEN);
    assert_eq!(call(&router, "PATCH", &format!("/api/orgs/{org_id}"), json!({"name": "Mine"}), &bob).await.0, StatusCode::FORBIDDEN);
    // Inviting someone already in it, or a malformed address, is refused.
    assert_eq!(call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "bob@example.com"}), &ana).await.0, StatusCode::CONFLICT);
    assert_eq!(call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "not-an-email"}), &ana).await.0, StatusCode::BAD_REQUEST);

    // The last owner can't leave or step down; with a second owner she can.
    assert_eq!(call(&router, "DELETE", &format!("/api/orgs/{org_id}/members/{ana_id}"), Value::Null, &ana).await.0, StatusCode::CONFLICT);
    assert_eq!(call(&router, "PATCH", &format!("/api/orgs/{org_id}/members/{ana_id}"), json!({"role": "member"}), &ana).await.0, StatusCode::CONFLICT);
    assert_eq!(call(&router, "PATCH", &format!("/api/orgs/{org_id}/members/{bob_id}"), json!({"role": "owner"}), &ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(call(&router, "DELETE", &format!("/api/orgs/{org_id}/members/{ana_id}"), Value::Null, &ana).await.0, StatusCode::NO_CONTENT);
    let (_, orgs) = call(&router, "GET", "/api/orgs", Value::Null, &ana).await;
    assert!(orgs["orgs"].as_array().unwrap().iter().all(|o| o["id"] != org_id.as_str()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_open_invite_can_be_taken_back(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let (org_id, _, _) = studio_with_a_project(&router, &ana).await;
    let (_, invite) = call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "new@example.com", "role": "admin"}), &ana).await;
    let code = invite["code"].as_str().unwrap();
    let (_, org) = call(&router, "GET", &format!("/api/orgs/{org_id}"), Value::Null, &ana).await;
    assert_eq!(org["invites"][0]["email"], "new@example.com");

    assert_eq!(call(&router, "DELETE", &format!("/api/orgs/{org_id}/invites/{code}"), Value::Null, &ana).await.0, StatusCode::NO_CONTENT);
    let newcomer = sign_up(&router, "new@example.com").await;
    assert_eq!(call(&router, "GET", &format!("/api/invites/{code}"), Value::Null, &newcomer).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn issues_are_filed_numbered_filtered_and_closed(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let (org_id, project_id, _) = studio_with_a_project(&router, &ana).await;
    let issues = format!("/api/projects/{project_id}/issues");

    // Not in the organization: no issues for him.
    assert_eq!(call(&router, "GET", &issues, Value::Null, &bob).await.0, StatusCode::NOT_FOUND);
    let (_, invite) = call(&router, "POST", &format!("/api/orgs/{org_id}/invites"), json!({"email": "bob@example.com"}), &ana).await;
    call(&router, "POST", &format!("/api/invites/{}/accept", invite["code"].as_str().unwrap()), Value::Null, &bob).await;

    let (status, first) = call(&router, "POST", &issues, json!({"title": "Menu prices are cut off", "body": "On phones.", "kind": "bug", "severity": "high"}), &ana).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(first["number"], 1);
    assert_eq!(first["status"], "open");
    assert_eq!(first["created_by"], "ana");
    let (_, second) = call(&router, "POST", &issues, json!({"title": "Online ordering", "kind": "feature", "severity": "medium"}), &bob).await;
    assert_eq!(second["number"], 2);
    assert_eq!(second["mine"], true);
    assert_eq!(call(&router, "POST", &issues, json!({"title": "x", "kind": "bug", "severity": "urgent"}), &ana).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(call(&router, "POST", &issues, json!({"title": "  ", "kind": "bug", "severity": "low"}), &ana).await.0, StatusCode::BAD_REQUEST);

    let (_, listed) = call(&router, "GET", &issues, Value::Null, &ana).await;
    let numbers: Vec<i64> = listed["issues"].as_array().unwrap().iter().map(|i| i["number"].as_i64().unwrap()).collect();
    assert_eq!(numbers, [2, 1]);
    let (_, bugs) = call(&router, "GET", &format!("{issues}?kind=bug"), Value::Null, &ana).await;
    assert_eq!(bugs["issues"].as_array().unwrap().len(), 1);

    let (status, closed) = call(&router, "PATCH", &format!("{issues}/1"), json!({"status": "closed", "severity": "critical"}), &bob).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(closed["status"], "closed");
    assert_eq!(closed["severity"], "critical");
    assert!(closed["closed_at"].is_string());
    let (_, open) = call(&router, "GET", &issues, Value::Null, &ana).await;
    assert_eq!(open["counts"], json!({"open": 1, "closed": 1}));
    let (_, done) = call(&router, "GET", &format!("{issues}?status=closed"), Value::Null, &ana).await;
    assert_eq!(done["issues"][0]["number"], 1);
    let (_, reopened) = call(&router, "PATCH", &format!("{issues}/1"), json!({"status": "open"}), &ana).await;
    assert_eq!(reopened["closed_at"], Value::Null);

    // Bob filed neither #1 nor owns the project, so he can't delete it; Ana can delete his.
    assert_eq!(call(&router, "DELETE", &format!("{issues}/1"), Value::Null, &bob).await.0, StatusCode::FORBIDDEN);
    assert_eq!(call(&router, "DELETE", &format!("{issues}/2"), Value::Null, &ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(call(&router, "GET", &format!("{issues}/2"), Value::Null, &ana).await.0, StatusCode::NOT_FOUND);

    let (_, project) = call(&router, "GET", &format!("/api/projects/{project_id}"), Value::Null, &ana).await;
    assert_eq!(project["open_issues"], 1);
    assert_eq!(project["org_name"], "Studio");
}
