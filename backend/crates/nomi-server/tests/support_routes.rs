//! The support agent and its knowledge base, admins switching models off, maintenance mode,
//! and importing code into a project.
use std::io::Write;

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
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

async fn send(router: axum::Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
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

async fn register(router: axum::Router, pool: &PgPool, email: &str, admin: bool) -> String {
    send(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    if admin {
        sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }
    let (_, body) = send(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn support_answers_from_articles_and_drafts_the_gaps(pool: PgPool) {
    std::env::set_var("LLM_PROVIDER", "fake");
    let router = build_router(test_state(pool.clone()));
    let admin = register(router.clone(), &pool, "admin@acme.test", true).await;
    let member = register(router.clone(), &pool, "ana@acme.test", false).await;

    // Only admins manage the knowledge base.
    let (status, _) = send(router.clone(), "GET", "/api/admin/support/articles", Some(&member), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, article) = send(
        router.clone(),
        "POST",
        "/api/admin/support/articles",
        Some(&admin),
        Some(json!({ "title": "Getting a refund", "body": "Refunds are issued within 14 days from Settings > Billing.", "tags": ["billing"] })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{article}");

    let (status, key) = send(router.clone(), "POST", "/api/admin/support/keys", Some(&admin), Some(json!({ "name": "Acme web", "product_name": "Acme" }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let secret = key["secret"].as_str().unwrap().to_string();
    assert!(secret.starts_with("nsk_"));

    // Without a key, or with a wrong one, nothing is answered.
    let (status, _) = send(router.clone(), "POST", "/api/support/v1/ask", None, Some(json!({ "question": "refund?" }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(router.clone(), "POST", "/api/support/v1/ask", Some("nsk_wrong"), Some(json!({ "question": "refund?" }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // A question the knowledge base covers is answered (the fake model gives no JSON, so a
    // reply grounded in found articles counts).
    let (status, answer) = send(router.clone(), "POST", "/api/support/v1/ask", Some(&secret), Some(json!({ "question": "How do I get a refund?", "customer": "c-1" }))).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["answered"], true);
    assert_eq!(answer["sources"][0]["title"], "Getting a refund");

    // One it doesn't cover is drafted for the admins, once however often it's asked.
    for _ in 0..2 {
        let (status, answer) = send(router.clone(), "POST", "/api/support/v1/ask", Some(&secret), Some(json!({ "question": "Can I export my data to CSV?" }))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(answer["answered"], false);
    }
    let (_, drafts) = send(router.clone(), "GET", "/api/admin/support/drafts", Some(&admin), None).await;
    assert_eq!(drafts.as_array().unwrap().len(), 1, "{drafts}");
    assert_eq!(drafts[0]["times_asked"], 2);
    let draft_id = drafts[0]["id"].as_str().unwrap();

    // Publishing the draft makes an article and closes the draft.
    let (status, published) = send(
        router.clone(),
        "POST",
        &format!("/api/admin/support/drafts/{draft_id}/publish"),
        Some(&admin),
        Some(json!({ "title": "Exporting your data", "body": "Use Settings > Export to download a CSV." })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{published}");
    let (_, open) = send(router.clone(), "GET", "/api/admin/support/drafts", Some(&admin), None).await;
    assert!(open.as_array().unwrap().is_empty());
    let (_, answer) = send(router.clone(), "POST", "/api/support/v1/ask", Some(&secret), Some(json!({ "question": "How do I export data?" }))).await;
    assert_eq!(answer["answered"], true);

    let (_, questions) = send(router.clone(), "GET", "/api/admin/support/questions", Some(&admin), None).await;
    assert_eq!(questions.as_array().unwrap().len(), 4);

    // A revoked key stops working.
    let key_id = key["id"].as_str().unwrap();
    let (status, _) = send(router.clone(), "DELETE", &format!("/api/admin/support/keys/{key_id}"), Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(router, "POST", "/api/support/v1/ask", Some(&secret), Some(json!({ "question": "refund?" }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn switched_off_models_are_not_offered_and_the_default_stays_on(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin = register(router.clone(), &pool, "admin@acme.test", true).await;
    let member = register(router.clone(), &pool, "ana@acme.test", false).await;
    let mut ids = Vec::new();
    for label in ["Main", "Spare"] {
        let (status, model) = send(
            router.clone(),
            "POST",
            "/api/admin/settings/llm/models",
            Some(&admin),
            Some(json!({ "label": label, "provider": "fake", "model_id": "", "api_key": "" })),
        )
        .await;
        assert!(status.is_success(), "{model}");
        ids.push(model["id"].as_str().unwrap().to_string());
    }
    // The first one is the default: it can't be switched off.
    let (status, _) = send(router.clone(), "PUT", &format!("/api/admin/settings/llm/models/{}/enabled", ids[0]), Some(&admin), Some(json!({ "enabled": false }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = send(router.clone(), "PUT", &format!("/api/admin/settings/llm/models/{}/enabled", ids[1]), Some(&admin), Some(json!({ "enabled": false }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, models) = send(router.clone(), "GET", "/api/llm/models", Some(&member), None).await;
    let offered: Vec<_> = models["admin_models"].as_array().unwrap().iter().map(|m| m["id"].as_str().unwrap().to_string()).collect();
    assert_eq!(offered, vec![ids[0].clone()]);
    let (status, _) = send(router.clone(), "PUT", "/api/llm/selection", Some(&member), Some(json!({ "kind": "admin", "admin_model_id": ids[1] }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, listed) = send(router, "GET", "/api/admin/settings/llm/models", Some(&admin), None).await;
    assert_eq!(listed[1]["enabled"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn maintenance_mode_is_public_to_read_and_admin_to_change(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin = register(router.clone(), &pool, "admin@acme.test", true).await;
    let member = register(router.clone(), &pool, "ana@acme.test", false).await;
    let (status, state) = send(router.clone(), "GET", "/api/maintenance", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["enabled"], false);
    let (status, _) = send(router.clone(), "PUT", "/api/admin/maintenance", Some(&member), Some(json!({ "enabled": true }))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, state) = send(router.clone(), "PUT", "/api/admin/maintenance", Some(&admin), Some(json!({ "enabled": true, "message": "Upgrading the database" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["message"], "Upgrading the database");
    let (_, state) = send(router.clone(), "GET", "/api/maintenance", None, None).await;
    assert_eq!(state["enabled"], true);
    // The admin page reads it back from the admin route; members can't.
    let (status, state) = send(router.clone(), "GET", "/api/admin/maintenance", Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(state["message"], "Upgrading the database");
    let (status, _) = send(router, "GET", "/api/admin/maintenance", Some(&member), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_zip_of_existing_code_is_imported_into_the_project(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), &pool, "ana@acme.test", false).await;
    let (_, orgs) = send(router.clone(), "GET", "/api/orgs", Some(&token), None).await;
    let org_id = orgs["orgs"][0]["id"].as_str().unwrap().to_string();
    send(router.clone(), "PUT", "/api/preferences/project-org", Some(&token), Some(json!({ "org_id": org_id }))).await;
    let (_, created) = send(router.clone(), "POST", "/api/projects", Some(&token), None).await;
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();

    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (path, content) in [
        ("shop-main/package.json", r#"{"dependencies":{"react":"^18"}}"#),
        ("shop-main/src/App.tsx", "export default () => null"),
        ("shop-main/node_modules/react/index.js", "x"),
    ] {
        writer.start_file(path, zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }
    let bytes = writer.finish().unwrap().into_inner();
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/projects/{project_id}/import"))
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/zip")
        .body(Body::from(bytes))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let result: Value = serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(result["imported"], 2);
    assert_eq!(result["skipped"], 1);
    assert_eq!(result["stack"], "react");

    let (_, project) = send(router, "GET", &format!("/api/projects/{project_id}"), Some(&token), None).await;
    let paths: Vec<_> = project["files"].as_array().unwrap().iter().map(|f| f["path"].as_str().unwrap().to_string()).collect();
    assert_eq!(paths, ["package.json", "src/App.tsx"]);
    assert_eq!(project["stack"], "react");
}
