//! Apps calling their backend: `https://<slug>[-dev].<domain>/rest/…` through the gateway to
//! PostgREST (wiremock stands in for it).
use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use relay_control::backend::{ensure_backend, BackendKeys};
use relay_control::Relay;
use relay_core::{Env, Tier};
use relay_server::app::{router, MAX_BODY_BYTES};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DOMAIN: &str = "api.relay.test";
const PGRST_SECRET: &str = "postgrest-secret-postgrest-secret-postgrest";

struct World {
    router: axum::Router,
    postgrest: MockServer,
    keys: BackendKeys,
}

async fn world(pool: &PgPool) -> World {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let postgrest = MockServer::start().await;
    let uri = postgrest.uri();
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()).with_settings(|s| {
        s.postgrest_dev_url = uri.clone();
        s.postgrest_production_url = uri.clone();
        s.postgrest_jwt_secret = PGRST_SECRET.into();
        s.preview_origins = vec![".preview.nomi.test".into()];
    }));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Bakery", Tier::Free, None).await.unwrap();
    World { router: router(relay, "control-key".into()), postgrest, keys }
}

impl World {
    fn host(&self, env: Env) -> String {
        match env {
            Env::Production => format!("{}.{DOMAIN}", self.keys.slug),
            Env::Dev => format!("{}-dev.{DOMAIN}", self.keys.slug),
        }
    }

    fn anon(&self, env: Env) -> String {
        self.keys.environments[env.as_str()].anon_key.clone()
    }

    async fn send(&self, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Value) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, headers, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }
}

fn claims(token: &str) -> Value {
    let key = jsonwebtoken::DecodingKey::from_secret(PGRST_SECRET.as_bytes());
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = false;
    jsonwebtoken::decode::<Value>(token, &key, &validation).expect("signed with PostgREST's secret").claims
}

fn schema(w: &World) -> String {
    format!("app_{}", w.keys.id.simple())
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_read_reaches_postgrest_as_the_projects_anon_role_in_its_schema(pool: PgPool) {
    let w = world(&pool).await;
    Mock::given(method("GET"))
        .and(path("/task"))
        .and(query_param("select", "*"))
        .and(header("accept-profile", schema(&w).as_str()))
        .respond_with(ResponseTemplate::new(200).insert_header("content-range", "0-0/1").set_body_json(serde_json::json!([{ "id": 1 }])))
        .expect(1)
        .mount(&w.postgrest)
        .await;
    let request = Request::get("/rest/task?select=*")
        .header("host", w.host(Env::Production))
        .header("apikey", w.anon(Env::Production))
        .header("accept-profile", "app_someone_else")
        .body(Body::empty())
        .unwrap();
    let (status, headers, body) = w.send(request).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, serde_json::json!([{ "id": 1 }]));
    assert_eq!(headers["content-range"], "0-0/1");

    let received = &w.postgrest.received_requests().await.unwrap()[0];
    let bearer = received.headers["authorization"].to_str().unwrap().strip_prefix("Bearer ").unwrap().to_string();
    let claims = claims(&bearer);
    assert_eq!(claims["db_role"], format!("app_{}_anon", w.keys.id.simple()));
    assert_eq!(claims["role"], "anon");
    assert_eq!(received.headers.get_all("accept-profile").iter().count(), 1);
    assert!(received.headers.get("apikey").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_write_names_its_schema_and_rpc_paths_keep_their_shape(pool: PgPool) {
    let w = world(&pool).await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(header("content-profile", schema(&w).as_str()))
        .and(header("prefer", "return=representation"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([{ "id": 2 }])))
        .mount(&w.postgrest)
        .await;
    Mock::given(method("POST"))
        .and(path("/rpc/total"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(3)))
        .mount(&w.postgrest)
        .await;
    let write = Request::post("/rest/task")
        .header("host", w.host(Env::Dev))
        .header("apikey", w.anon(Env::Dev))
        .header("content-type", "application/json")
        .header("prefer", "return=representation")
        .body(Body::from(r#"{"title":"bread"}"#))
        .unwrap();
    let (status, _, body) = w.send(write).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let rpc = Request::post("/rest/rpc/total").header("host", w.host(Env::Dev)).header("apikey", w.anon(Env::Dev)).body(Body::from("{}")).unwrap();
    assert_eq!(w.send(rpc).await.2, serde_json::json!(3));
}

#[sqlx::test(migrations = "../../migrations")]
async fn postgrests_own_errors_pass_through(pool: PgPool) {
    let w = world(&pool).await;
    let error = serde_json::json!({ "message": "permission denied for table task", "code": "42501", "details": null, "hint": null });
    Mock::given(method("GET")).respond_with(ResponseTemplate::new(401).set_body_json(error.clone())).mount(&w.postgrest).await;
    let request = Request::get("/rest/task").header("host", w.host(Env::Production)).header("apikey", w.anon(Env::Production)).body(Body::empty()).unwrap();
    let (status, _, body) = w.send(request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body, error);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wrong_keys_unknown_projects_and_big_bodies_never_reach_postgrest(pool: PgPool) {
    let w = world(&pool).await;
    Mock::given(method("GET")).respond_with(ResponseTemplate::new(200)).expect(0).mount(&w.postgrest).await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200)).expect(0).mount(&w.postgrest).await;

    let no_key = Request::get("/rest/task").header("host", w.host(Env::Production)).body(Body::empty()).unwrap();
    let (status, _, body) = w.send(no_key).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["message"].is_string() && body["code"].is_string(), "{body}");

    let other_env = Request::get("/rest/task").header("host", w.host(Env::Production)).header("apikey", w.anon(Env::Dev)).body(Body::empty()).unwrap();
    assert_eq!(w.send(other_env).await.0, StatusCode::UNAUTHORIZED);

    let unknown = Request::get("/rest/task").header("host", format!("nobody-000000.{DOMAIN}")).header("apikey", w.anon(Env::Production)).body(Body::empty()).unwrap();
    assert_eq!(w.send(unknown).await.0, StatusCode::NOT_FOUND);

    let big = Request::post("/rest/task")
        .header("host", w.host(Env::Production))
        .header("apikey", w.anon(Env::Production))
        .body(Body::from(vec![b'x'; MAX_BODY_BYTES + 1]))
        .unwrap();
    assert_eq!(w.send(big).await.0, StatusCode::PAYLOAD_TOO_LARGE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_bare_host_is_still_the_control_plane(pool: PgPool) {
    let w = world(&pool).await;
    let health = Request::get("/health").header("host", DOMAIN).body(Body::empty()).unwrap();
    assert_eq!(w.send(health).await.0, StatusCode::OK);
    let project_health = Request::get("/control/projects/x").header("host", w.host(Env::Production)).body(Body::empty()).unwrap();
    assert_eq!(w.send(project_health).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn preflight_answers_previews_on_dev_and_anyone_on_production(pool: PgPool) {
    let w = world(&pool).await;
    let preflight = |env: Env, origin: &str| {
        Request::builder()
            .method("OPTIONS")
            .uri("/rest/task")
            .header("host", w.host(env))
            .header("origin", origin)
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap()
    };
    let (status, headers, _) = w.send(preflight(Env::Dev, "https://abc.preview.nomi.test")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(headers["access-control-allow-origin"], "https://abc.preview.nomi.test");
    assert!(headers["access-control-allow-headers"].to_str().unwrap().contains("apikey"));

    let (_, headers, _) = w.send(preflight(Env::Dev, "https://evil.test")).await;
    assert!(headers.get("access-control-allow-origin").is_none());

    let (_, headers, _) = w.send(preflight(Env::Production, "https://bakery.example")).await;
    assert_eq!(headers["access-control-allow-origin"], "https://bakery.example");
}
