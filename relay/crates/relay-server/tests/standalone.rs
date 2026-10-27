//! A standalone Relay on its one host: `/control/…` is the control plane, anything else is the
//! one project's backend, whatever the hostname (wiremock stands in for its PostgREST).
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use relay_control::backend::ensure_backend;
use relay_control::Relay;
use relay_core::config::Standalone;
use relay_core::{Env, Tier};
use relay_server::app::router;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CONTROL_KEY: &str = "control-key";

async fn own_url(pool: &PgPool) -> String {
    let base = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await.unwrap();
    let (server, _) = base.rsplit_once('/').unwrap();
    format!("{server}/{database}")
}

struct World {
    relay: Arc<Relay>,
    router: axum::Router,
    postgrest: MockServer,
    project: Uuid,
}

async fn world(pool: &PgPool) -> World {
    let postgrest = MockServer::start().await;
    let project = Uuid::new_v4();
    let config = Standalone { project, env: Env::Production, postgrest_url: postgrest.uri(), pgrst_password: "pgrst-pw".into(), functions: None };
    let relay = Arc::new(Relay::start_standalone(pool.clone(), [6u8; 32], own_url(pool).await, config).await.unwrap());
    World { router: router(relay.clone(), CONTROL_KEY.into()), relay, postgrest, project }
}

async fn send(router: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[sqlx::test(migrations = "../../migrations")]
async fn control_calls_still_need_the_control_key(pool: PgPool) {
    let w = world(&pool).await;
    let request = Request::get("/control/settings").header("host", "app-backend.apps.test").body(Body::empty()).unwrap();
    assert_eq!(send(&w.router, request).await.0, StatusCode::UNAUTHORIZED);
    let health = Request::get("/health").header("host", "app-backend.apps.test").body(Body::empty()).unwrap();
    assert_eq!(send(&w.router, health).await.0, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn before_the_backend_exists_the_gateway_says_not_configured(pool: PgPool) {
    let w = world(&pool).await;
    let request = Request::get("/rest/task").header("host", "app-backend.apps.test").body(Body::empty()).unwrap();
    let (status, body) = send(&w.router, request).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], relay_control::NOT_CONFIGURED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn any_host_reaches_the_one_project(pool: PgPool) {
    let w = world(&pool).await;
    let keys = ensure_backend(&w.relay, w.project, "Barbershop", Tier::Free, None).await.unwrap();
    Mock::given(method("GET")).and(path("/task")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).expect(1).mount(&w.postgrest).await;

    let anon = keys.environments["production"].anon_key.clone();
    let request = Request::get("/rest/task?select=*")
        .header("host", "whatever.example")
        .header("apikey", &anon)
        .header("authorization", format!("Bearer {anon}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(&w.router, request).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, serde_json::json!([]));
}

const REALTIME: &str = "/realtime";

#[sqlx::test(migrations = "../../migrations")]
async fn realtime_is_served_by_a_standalone_relay(pool: PgPool) {
    let w = world(&pool).await;
    let keys = ensure_backend(&w.relay, w.project, "Barbershop", Tier::Free, None).await.unwrap();
    let anon = keys.environments["production"].anon_key.clone();

    let request = Request::get(format!("{REALTIME}?apikey={anon}")).header("host", "whatever.example").body(Body::empty()).unwrap();
    let (status, _) = send(&w.router, request).await;

    // Not a WebSocket upgrade, so the socket itself says no; but never "dedicated isn't served".
    assert_ne!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(status.is_client_error(), "{status}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shared_relay_still_refuses_a_dedicated_projects_realtime_and_data(pool: PgPool) {
    use relay_control::backend::Databases;
    use std::collections::HashMap;
    let own = own_url(&pool).await;
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Arc::new(Relay::new(pool.clone(), shared, [6u8; 32], "api.relay.test".into()));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Barbershop", Tier::Dedicated, Some(Databases { dev: own.clone(), production: own })).await.unwrap();
    let router = router(relay, CONTROL_KEY.into());
    let host = format!("{}.api.relay.test", keys.slug);
    let anon = keys.environments["production"].anon_key.clone();

    let socket = Request::get(format!("{REALTIME}?apikey={anon}")).header("host", &host).body(Body::empty()).unwrap();
    assert_eq!(send(&router, socket).await.0, StatusCode::NOT_IMPLEMENTED);
    let data = Request::get("/rest/task").header("host", &host).header("apikey", &anon).header("authorization", format!("Bearer {anon}")).body(Body::empty()).unwrap();
    assert_eq!(send(&router, data).await.0, StatusCode::NOT_IMPLEMENTED);
}
