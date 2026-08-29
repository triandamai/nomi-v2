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
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
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

async fn register_via_api(router: axum::Router, email: &str) {
    json_request(
        router,
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
}

async fn login_via_api(router: axum::Router, email: &str) -> String {
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

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

async fn user_id_by_email(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_listing_users(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/users", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn admin_can_list_users_and_search_by_email(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "alice@example.com").await;
    register_via_api(router.clone(), "bob@example.com").await;

    let (status, body) = json_request(router.clone(), "GET", "/api/admin/users?query=alice", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["email"], "alice@example.com");
    assert_eq!(users[0]["is_staff"], false);

    let (status, body) = json_request(router, "GET", "/api/admin/users", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    // admin + alice + bob
    assert_eq!(body["total"], 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn promoting_a_user_makes_them_staff_and_gets_them_into_the_admin_panel(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "staff@example.com").await;
    let staff_user_id = user_id_by_email(&pool, "staff@example.com").await;

    let (status, _) = json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{staff_user_id}/permissions"),
        json!({ "scope_type": "admin", "org_id": null, "resource": "user", "actions": ["view"] }),
        Some(&admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Confirm it lands in the roster's is_staff flag.
    let (_, body) = json_request(router.clone(), "GET", "/api/admin/users", Value::Null, Some(&admin_token)).await;
    let staff_row = body["users"].as_array().unwrap().iter().find(|u| u["id"] == staff_user_id.to_string()).unwrap();
    assert_eq!(staff_row["is_staff"], true);

    // Confirm it lands in the staff member's own token after a fresh login (claims-freshness
    // rule from the auth-claims design: only guaranteed on refresh/re-login, not instantly).
    let staff_token = login_via_api(router.clone(), "staff@example.com").await;
    let (status, whoami_body) = json_request(router, "GET", "/api/whoami", Value::Null, Some(&staff_token)).await;
    assert_eq!(status, StatusCode::OK);
    let permissions = whoami_body["permissions"].as_array().unwrap();
    assert!(permissions.iter().any(|p| p == "nomi:admin:user:[view]"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_detail_returns_permissions_and_memberships(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Beta Org') RETURNING id").fetch_one(&pool).await.unwrap();
    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "member" }),
        Some(&admin_token),
    )
    .await;
    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/permissions"),
        json!({ "scope_type": "admin", "org_id": null, "resource": "billing", "actions": ["view"] }),
        Some(&admin_token),
    )
    .await;

    let (status, body) = json_request(router, "GET", &format!("/api/admin/users/{target_id}"), Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "target@example.com");
    assert_eq!(body["permissions"].as_array().unwrap().len(), 1);
    assert_eq!(body["permissions"][0]["resource"], "billing");
    // Registration itself creates and assigns an "Acme" org (owner), so the assigned "Beta Org"
    // membership shows up alongside it.
    let memberships = body["memberships"].as_array().unwrap();
    assert_eq!(memberships.len(), 2);
    let beta = memberships.iter().find(|m| m["org_name"] == "Beta Org").unwrap();
    assert_eq!(beta["role"], "member");
}

#[sqlx::test(migrations = "../../migrations")]
async fn assigning_to_org_twice_updates_the_role_instead_of_erroring(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Beta Org') RETURNING id").fetch_one(&pool).await.unwrap();

    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "member" }),
        Some(&admin_token),
    )
    .await;
    let (status, body) = json_request(
        router,
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "admin" }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let rows = body.as_array().unwrap();
    // Registration itself creates and assigns an "Acme" org (owner); confirm the assigned org
    // wasn't duplicated by the second assignment, and its role was updated in place.
    assert_eq!(rows.len(), 2);
    let beta_rows: Vec<_> = rows.iter().filter(|m| m["org_id"] == org_id.to_string()).collect();
    assert_eq!(beta_rows.len(), 1);
    assert_eq!(beta_rows[0]["role"], "admin");
}

#[sqlx::test(migrations = "../../migrations")]
async fn removing_from_org_sets_status_removed_and_excludes_from_the_list(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Beta Org') RETURNING id").fetch_one(&pool).await.unwrap();
    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "member" }),
        Some(&admin_token),
    )
    .await;

    let (status, body) = json_request(
        router,
        "DELETE",
        &format!("/api/admin/users/{target_id}/memberships/{org_id}"),
        Value::Null,
        Some(&admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // Registration itself creates and assigns an "Acme" org (owner), which remains; only the
    // removed "Beta Org" membership should be gone.
    let rows = body.as_array().unwrap();
    assert!(rows.iter().all(|m| m["org_id"] != org_id.to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn granting_an_invalid_resource_returns_400(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;

    let (status, _) = json_request(
        router,
        "POST",
        &format!("/api/admin/users/{target_id}/permissions"),
        json!({ "scope_type": "admin", "org_id": null, "resource": "Not Valid", "actions": ["view"] }),
        Some(&admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_orgs_excludes_personal_orgs(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    sqlx::query("INSERT INTO organizations (name, is_personal) VALUES ('Real Org', false)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO organizations (name, is_personal) VALUES ('Someones Personal Org', true)").execute(&pool).await.unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/orgs", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    let orgs = body.as_array().unwrap();
    // The admin's own registration-created "Acme" org is non-personal too, so it's present
    // alongside "Real Org"; only the personal org must be excluded.
    let names: Vec<&str> = orgs.iter().map(|o| o["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Real Org"));
    assert!(!names.contains(&"Someones Personal Org"));
}
