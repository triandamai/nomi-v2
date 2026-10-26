//! The control plane over HTTP, as Nomi's `nomi-relay-client` calls it.
use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use relay_control::Relay;
use relay_core::Env;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const KEY: &str = "control-key";

fn router(pool: &PgPool) -> axum::Router {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    relay_server::app::router(Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".into())), KEY.into(), None)
}

async fn call(router: &axum::Router, method: &str, path: &str, key: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(key) = key {
        request = request.header("authorization", format!("Bearer {key}"));
    }
    let request = match body {
        Some(body) => request.header("content-type", "application/json").body(Body::from(body.to_string())).unwrap(),
        None => request.body(Body::empty()).unwrap(),
    };
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_the_control_key_nothing_answers(pool: PgPool) {
    let router = router(&pool);
    let path = format!("/control/projects/{}", Uuid::new_v4());
    let (status, body) = call(&router, "PUT", &path, None, Some(json!({ "name": "x" }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
    assert_eq!(call(&router, "PUT", &path, Some("wrong"), Some(json!({ "name": "x" }))).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(call(&router, "GET", "/health", None, None).await.0, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_goes_from_nothing_to_migrated_to_typed(pool: PgPool) {
    let router = router(&pool);
    let project = Uuid::new_v4();
    let base = format!("/control/projects/{project}");

    let (status, made) = call(&router, "PUT", &base, Some(KEY), Some(json!({ "name": "Todo" }))).await;
    assert_eq!(status, StatusCode::OK, "{made}");
    assert_eq!(made["data"]["id"], json!(project));
    assert!(made["data"]["environments"]["production"]["anon_key"].is_string());
    assert!(made["error"].is_null());

    let init = "create table task (id int primary key, title text not null);\nalter table task enable row level security;\n";
    let files = json!({ "files": [{ "path": "nomi/migrations/0001_init.sql", "content": init }] });
    let (status, applied) = call(&router, "POST", &format!("{base}/environments/dev/migrations"), Some(KEY), Some(files)).await;
    assert_eq!(status, StatusCode::OK, "{applied}");
    assert_eq!(applied["data"]["applied"], json!(["0001_init.sql"]));

    let (_, schema) = call(&router, "GET", &format!("{base}/environments/dev/schema"), Some(KEY), None).await;
    assert_eq!(schema["data"]["tables"][0]["name"], "task");
    let (_, types) = call(&router, "GET", &format!("{base}/environments/dev/types"), Some(KEY), None).await;
    assert!(types["data"]["types"].as_str().unwrap().contains("title: string"));

    let (status, _) = call(&router, "DELETE", &base, Some(KEY), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(&router, "GET", &format!("{base}/environments/dev/schema"), Some(KEY), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_migration_answers_422_with_where(pool: PgPool) {
    let router = router(&pool);
    let base = format!("/control/projects/{}", Uuid::new_v4());
    call(&router, "PUT", &base, Some(KEY), Some(json!({ "name": "Todo" }))).await;
    let broken = "create table t (id int);\nalter table t enable row level security;\nselect * from nope;\n";
    let files = json!({ "files": [{ "path": "nomi/migrations/0001_t.sql", "content": broken }] });

    let (status, body) = call(&router, "POST", &format!("{base}/environments/dev/migrations"), Some(KEY), Some(files)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "MIGRATION_FAILED");
    assert_eq!(body["error"]["details"]["file"], "0001_t.sql");
    assert_eq!(body["error"]["details"]["line"], 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_environment_or_project_is_not_found(pool: PgPool) {
    let router = router(&pool);
    let base = format!("/control/projects/{}", Uuid::new_v4());
    assert_eq!(call(&router, "GET", &format!("{base}/environments/staging/schema"), Some(KEY), None).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&router, "GET", &format!("{base}/environments/dev/schema"), Some(KEY), None).await.0, StatusCode::NOT_FOUND);
}
