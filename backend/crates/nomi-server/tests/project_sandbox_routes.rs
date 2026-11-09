//! The sandbox API the project page uses: status, start/stop, logs, and where new projects run.

mod support;

use std::collections::BTreeMap;

use axum::http::StatusCode;
use nomi_server::sandbox_store::{self as store, Desired, State};
use serde_json::json;
use sqlx::PgPool;
use support::{App, Person};
use uuid::Uuid;

/// `sandbox_config()` reads through the process-wide settings cache and pool, so the tests that
/// set it take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const PREVIEW: &str = "https://preview.example.test";

/// Turns the admin switch on (with the rollout given) or off, as the admin page would.
async fn switch(pool: &PgPool, owner: Uuid, on: bool, rollout: &str) {
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", hex::encode(nomi_test_support::TEST_SETTINGS_KEY));
    nomi_settings::config::install(pool.clone());
    let values: BTreeMap<String, String> = [
        ("enabled", if on { "on" } else { "off" }),
        ("api_key", "boat-key-123"),
        ("rollout", rollout),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    nomi_settings::config::save(pool, &nomi_test_support::TEST_SETTINGS_KEY, &nomi_settings::config::SANDBOX, &values, owner).await.unwrap();
    nomi_settings::config::forget();
}

async fn sandbox_project(app: &App, owner: &Person, runtime: &str) -> Uuid {
    let (project, _) = app.project(owner, "Todo").await;
    sqlx::query("UPDATE projects SET runtime = $2 WHERE id = $1").bind(project).bind(runtime).execute(&app.pool).await.unwrap();
    project
}

async fn set_state(pool: &PgPool, project: Uuid, state: State) {
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, "boat").await.unwrap();
    store::set_state(&mut conn, project, state, None).await.unwrap();
    sqlx::query("UPDATE project_sandboxes SET preview_url = $2 WHERE project_id = $1").bind(project).bind(PREVIEW).execute(&mut *conn).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_member_gets_404(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let stranger = app.person("stranger@other.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    switch(&pool, owner.id, true, "all").await;
    for (method, path) in [("GET", ""), ("POST", "/start"), ("POST", "/stop"), ("GET", "/logs")] {
        let (status, _) = app.call(method, &format!("/api/projects/{project}/sandbox{path}"), Some(&stranger.token), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn preview_url_absent_unless_running(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    switch(&pool, owner.id, true, "all").await;
    let uri = format!("/api/projects/{project}/sandbox");

    set_state(&pool, project, State::Starting).await;
    let (_, body) = app.get(&uri, &owner).await;
    assert_eq!(body["phase"], "waking");
    assert_eq!(body["preview_url"], json!(null));

    set_state(&pool, project, State::Running).await;
    let (_, body) = app.get(&uri, &owner).await;
    assert_eq!((body["state"].as_str(), body["phase"].as_str()), (Some("running"), Some("running")));
    assert_eq!(body["preview_url"], PREVIEW);

    set_state(&pool, project, State::Stopped).await;
    let (_, body) = app.get(&uri, &owner).await;
    assert_eq!(body["phase"], "stopped");
    assert_eq!(body["preview_url"], json!(null));
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_sets_desired_running_and_returns_phase(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    switch(&pool, owner.id, true, "all").await;

    let (status, body) = app.call("POST", &format!("/api/projects/{project}/sandbox/start"), Some(&owner.token), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!((body["phase"].as_str(), body["available"].as_bool()), (Some("idle"), Some(true)));
    let row = store::get(&mut pool.acquire().await.unwrap(), project).await.unwrap().unwrap();
    assert_eq!(row.desired, Desired::Running);

    let (status, _) = app.call("POST", &format!("/api/projects/{project}/sandbox/stop"), Some(&owner.token), None).await;
    assert_eq!(status, StatusCode::OK);
    let row = store::get(&mut pool.acquire().await.unwrap(), project).await.unwrap().unwrap();
    assert_eq!(row.desired, Desired::Stopped);
}

#[sqlx::test(migrations = "../../migrations")]
async fn available_false_when_kill_switch_off(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "browser").await;
    switch(&pool, owner.id, false, "all").await;
    let (_, body) = app.get(&format!("/api/projects/{project}/sandbox"), &owner).await;
    assert_eq!(body["available"], false);

    // On, but the rollout leaves the person out.
    switch(&pool, owner.id, true, "off").await;
    let (_, body) = app.get(&format!("/api/projects/{project}/sandbox"), &owner).await;
    assert_eq!(body["available"], false);

    switch(&pool, owner.id, true, "all").await;
    let (_, body) = app.get(&format!("/api/projects/{project}/sandbox"), &owner).await;
    assert_eq!(body["available"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn sandbox_project_reports_webcontainer_when_switch_off(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    switch(&pool, owner.id, false, "all").await;
    let (status, body) = app.get(&format!("/api/projects/{project}/sandbox"), &owner).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["runtime"].as_str(), body["available"].as_bool()), (Some("sandbox"), Some(false)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_is_refused_when_switch_off(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    switch(&pool, owner.id, false, "all").await;
    let (status, body) = app.call("POST", &format!("/api/projects/{project}/sandbox/start"), Some(&owner.token), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(store::get(&mut pool.acquire().await.unwrap(), project).await.unwrap().is_none(), "nothing was started");
}

#[sqlx::test(migrations = "../../migrations")]
async fn logs_endpoint_pages_by_offset(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    let uri = format!("/api/projects/{project}/sandbox/logs");

    // No dev server output yet.
    let (status, body) = app.get(&uri, &owner).await;
    assert_eq!((status, body["text"].as_str(), body["next_offset"].as_u64()), (StatusCode::OK, Some(""), Some(0)));

    // The worker (maybe in another process) saves the dev server's output on the row.
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, "boat").await.unwrap();
    store::append_log(&mut conn, project, "hello ", nomi_server::sandbox_devserver::LOG_KEPT).await.unwrap();
    let (_, first) = app.get(&uri, &owner).await;
    assert_eq!((first["text"].as_str(), first["next_offset"].as_u64()), (Some("hello "), Some(6)));

    store::append_log(&mut conn, project, "world", nomi_server::sandbox_devserver::LOG_KEPT).await.unwrap();
    let (_, rest) = app.get(&format!("{uri}?offset=6"), &owner).await;
    assert_eq!((rest["text"].as_str(), rest["next_offset"].as_u64()), (Some("world"), Some(11)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn new_project_gets_sandbox_runtime_only_when_switch_on_and_rollout_includes_user(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1 LIMIT 1").bind(owner.id).fetch_one(&pool).await.unwrap();
    let create = || async {
        let (status, body) = app.call("POST", "/api/projects", Some(&owner.token), Some(json!({ "org_id": org }))).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        let id: Uuid = body["project_id"].as_str().unwrap().parse().unwrap();
        sqlx::query_scalar::<_, String>("SELECT runtime FROM projects WHERE id = $1").bind(id).fetch_one(&pool).await.unwrap()
    };

    switch(&pool, owner.id, false, "all").await;
    assert_eq!(create().await, "browser");
    switch(&pool, owner.id, true, "off").await;
    assert_eq!(create().await, "browser");
    switch(&pool, owner.id, true, "all").await;
    assert_eq!(create().await, "sandbox");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_running_sandbox_whose_preview_failed_shows_why_and_can_retry(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let project = sandbox_project(&app, &owner, "sandbox").await;
    switch(&pool, owner.id, true, "all").await;
    let uri = format!("/api/projects/{project}/sandbox");
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, "boat").await.unwrap();
    store::set_state(&mut conn, project, State::Running, Some("installing dependencies failed: ERESOLVE")).await.unwrap();
    let (_, body) = app.get(&uri, &owner).await;
    assert_eq!(body["phase"], "running");
    assert_eq!(body["preview_url"], json!(null));
    assert!(body["error"].as_str().is_some_and(|e| e.contains("ERESOLVE")), "{body}");

    let (status, _) = app.send("POST", &format!("{uri}/start"), &owner, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    let error: Option<String> = sqlx::query_scalar("SELECT last_error FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(error, None, "retrying clears the failure, so the worker tries the preview again");
}
