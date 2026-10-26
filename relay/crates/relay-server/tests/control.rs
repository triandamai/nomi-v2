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
    relay_server::app::router(Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".into())), KEY.into())
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

#[sqlx::test(migrations = "../../migrations")]
async fn reading_a_project_answers_its_keys_without_making_one(pool: PgPool) {
    let router = router(&pool);
    let path = format!("/control/projects/{}", Uuid::new_v4());
    assert_eq!(call(&router, "GET", &path, Some(KEY), None).await.0, StatusCode::NOT_FOUND);
    let (_, made) = call(&router, "PUT", &path, Some(KEY), Some(json!({ "name": "Todo" }))).await;
    let (status, read) = call(&router, "GET", &path, Some(KEY), None).await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(read["data"], made["data"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn until_its_settings_are_saved_relay_answers_not_configured(pool: PgPool) {
    let router = relay_server::app::router(Arc::new(Relay::unconfigured(pool.clone(), [9u8; 32])), KEY.into());
    let (status, settings) = call(&router, "GET", "/control/settings", Some(KEY), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["data"]["configured"], false);
    let (status, body) = call(&router, "PUT", &format!("/control/projects/{}", Uuid::new_v4()), Some(KEY), Some(json!({ "name": "x" }))).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "NOT_CONFIGURED");
    assert_eq!(call(&router, "GET", "/control/settings", None, None).await.0, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn settings_that_dont_work_answer_422_with_each_check(pool: PgPool) {
    let router = relay_server::app::router(Arc::new(Relay::unconfigured(pool.clone(), [9u8; 32])), KEY.into());
    let body = json!({
        "public_domain": "api.relay.test",
        "shared_dev_url": "postgres://nobody:x@127.0.0.1:9/none",
        "shared_production_url": "postgres://nobody:x@127.0.0.1:9/none",
        "postgrest_dev_url": "http://127.0.0.1:9",
        "postgrest_production_url": "http://127.0.0.1:9",
        "postgrest_role": "authenticator"
    });
    let (status, answer) = call(&router, "PUT", "/control/settings", Some(KEY), Some(body)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    assert_eq!(answer["error"]["code"], "SETTINGS_INVALID");
    assert_eq!(answer["error"]["details"]["checks"].as_array().unwrap().len(), 4);
}

#[sqlx::test(migrations = "../../migrations")]
async fn buckets_and_realtime_tables_are_saved_and_described(pool: PgPool) {
    let router = router(&pool);
    let project = Uuid::new_v4();
    let base = format!("/control/projects/{project}");
    call(&router, "PUT", &base, Some(KEY), Some(json!({ "name": "Chat" }))).await;
    let init = "create table message (id int primary key, body text);\nalter table message enable row level security;\n";
    call(&router, "POST", &format!("{base}/environments/dev/migrations"), Some(KEY), Some(json!({ "files": [{ "path": "nomi/migrations/0001_init.sql", "content": init }] }))).await;

    let (status, saved) = call(&router, "PUT", &format!("{base}/environments/dev/storage"), Some(KEY), Some(json!({ "buckets": [{ "name": "avatars", "public": true, "write": "owner" }] }))).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (status, saved) = call(&router, "PUT", &format!("{base}/environments/dev/realtime"), Some(KEY), Some(json!({ "tables": ["message"] }))).await;
    assert_eq!(status, StatusCode::OK, "{saved}");

    let (_, described) = call(&router, "GET", &format!("{base}/environments/dev/schema"), Some(KEY), None).await;
    assert_eq!(described["data"]["buckets"][0]["name"], "avatars");
    assert_eq!(described["data"]["buckets"][0]["write"], "owner");
    assert_eq!(described["data"]["realtime"], json!(["message"]));
    assert_eq!(described["data"]["tables"][0]["name"], "message");

    let (status, refused) = call(&router, "PUT", &format!("{base}/environments/dev/realtime"), Some(KEY), Some(json!({ "tables": ["order"] }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["error"]["code"], "BACKEND_INVALID");
}

#[sqlx::test(migrations = "../../migrations")]
async fn functions_are_deployed_rolled_back_and_described(pool: PgPool) {
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let runtime = MockServer::start().await;
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true }))).mount(&runtime).await;
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let uri = runtime.uri();
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".into())
        .with_settings(|s| s.functions = Some(relay_core::settings::FunctionsRuntime { url: uri, secret: "s".into() }));
    let router = relay_server::app::router(Arc::new(relay), KEY.into());
    let project = Uuid::new_v4();
    let base = format!("/control/projects/{project}");
    call(&router, "PUT", &base, Some(KEY), Some(json!({ "name": "Shop" }))).await;
    let deploy = |text: &str| json!({ "files": [{ "path": "nomi/functions/hello.ts", "content": text }], "functions": { "hello": { "auth": "public" } } });

    let (status, first) = call(&router, "POST", &format!("{base}/environments/dev/functions"), Some(KEY), Some(deploy("export default () => new Response('1')"))).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["data"]["version"], 1);
    call(&router, "POST", &format!("{base}/environments/dev/functions"), Some(KEY), Some(deploy("export default () => new Response('2')"))).await;

    let (status, back) = call(&router, "POST", &format!("{base}/environments/dev/functions/rollback"), Some(KEY), Some(json!({}))).await;
    assert_eq!(status, StatusCode::OK, "{back}");
    assert_eq!(back["data"]["version"], 1);
    let (_, described) = call(&router, "GET", &format!("{base}/environments/dev/schema"), Some(KEY), None).await;
    assert_eq!(described["data"]["functions"]["version"], 1);
    assert_eq!(described["data"]["functions"]["functions"]["hello"]["auth"], "public");

    let (status, saved) = call(&router, "PUT", &format!("{base}/secrets/STRIPE_SECRET"), Some(KEY), Some(json!({ "value": "sk_test" }))).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (status, _) = call(&router, "PUT", &format!("{base}/secrets/not-a-name"), Some(KEY), Some(json!({ "value": "x" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(call(&router, "DELETE", &format!("{base}/secrets/STRIPE_SECRET"), Some(KEY), None).await.0, StatusCode::OK);
}
