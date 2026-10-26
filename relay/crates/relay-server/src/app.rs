//! The control plane's routes (only the Nomi backend calls them, with `RELAY_CONTROL_KEY`), and
//! the hostname dispatch that sends an app's requests to the gateway instead.

use std::sync::Arc;

use axum::extract::{Path, Request, State};
use axum::http::header::HOST;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use relay_control::backend::{backend_keys, delete_backend, ensure_backend, Databases};
use relay_control::{migrations, schema, Relay};
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use tower::ServiceExt;

use crate::envelope::{ok, Answer, Failure};
use crate::gateway::Gateway;
pub use crate::gateway::{GatewaySettings, MAX_BODY_BYTES};

#[derive(Clone)]
struct AppState {
    relay: Arc<Relay>,
    control_key: Arc<String>,
}

#[derive(Deserialize)]
struct EnsureRequest {
    name: String,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    databases: Option<Databases>,
}

#[derive(Deserialize)]
struct MigrationsRequest {
    files: Vec<MigrationFile>,
}

type Result<T> = std::result::Result<T, Failure>;

/// Constant-time enough for a single shared key: compares every byte.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn require_control_key(State(state): State<AppState>, request: Request, next: Next) -> std::result::Result<Response, Failure> {
    let given = request.headers().get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).unwrap_or_default();
    if !same(given, &state.control_key) {
        return Err(Failure::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "this needs Relay's control key"));
    }
    Ok(next.run(request).await)
}

fn env(name: &str) -> Result<Env> {
    Env::parse(name).ok_or_else(|| Failure::new(StatusCode::NOT_FOUND, "NOT_FOUND", format!("no environment called {name}: use dev or production")))
}

async fn put_project(State(state): State<AppState>, Path(project): Path<Uuid>, Json(body): Json<EnsureRequest>) -> Result<Answer> {
    let tier = match body.tier.as_deref() {
        None => Tier::Free,
        Some(text) => Tier::parse(text).ok_or_else(|| Failure::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", "tier is free or dedicated"))?,
    };
    Ok(ok(ensure_backend(&state.relay, project, &body.name, tier, body.databases).await?))
}

async fn get_project(State(state): State<AppState>, Path(project): Path<Uuid>) -> Result<Answer> {
    Ok(ok(backend_keys(&state.relay, project).await?))
}

async fn delete_project(State(state): State<AppState>, Path(project): Path<Uuid>) -> Result<Answer> {
    delete_backend(&state.relay, project).await?;
    Ok(ok(json!({ "deleted": project })))
}

async fn post_migrations(State(state): State<AppState>, Path((project, env_name)): Path<(Uuid, String)>, Json(body): Json<MigrationsRequest>) -> Result<Answer> {
    let applied = migrations::apply(&state.relay, project, env(&env_name)?, &body.files).await?;
    Ok(ok(json!({ "applied": applied })))
}

async fn get_schema(State(state): State<AppState>, Path((project, env_name)): Path<(Uuid, String)>) -> Result<Answer> {
    Ok(ok(schema::describe(&state.relay, project, env(&env_name)?).await?))
}

async fn get_types(State(state): State<AppState>, Path((project, env_name)): Path<(Uuid, String)>) -> Result<Answer> {
    Ok(ok(json!({ "types": schema::types(&state.relay, project, env(&env_name)?).await? })))
}

async fn health() -> Answer {
    ok(json!({ "ok": true }))
}

/// Relay's whole surface: a project host (`<slug>[-dev].<domain>`) goes to the gateway when one
/// is configured, every other host to the control plane.
pub fn router(relay: Arc<Relay>, control_key: String, gateway: Option<GatewaySettings>) -> Router {
    let control = control_router(relay.clone(), control_key);
    let Some(settings) = gateway else { return control };
    let domain = relay.public_domain.clone();
    let gateway = Arc::new(Gateway { relay, settings, http: reqwest::Client::new() });
    Router::new().fallback(move |request: Request| {
        let control = control.clone();
        let gateway = gateway.clone();
        let domain = domain.clone();
        async move {
            let host = request.headers().get(HOST).and_then(|v| v.to_str().ok()).unwrap_or_default();
            match relay_gateway::host::route(host, &domain) {
                Some((slug, env)) => gateway.handle(slug, env, request).await,
                None => control.oneshot(request).await.unwrap_or_else(|never| match never {}),
            }
        }
    })
}

fn control_router(relay: Arc<Relay>, control_key: String) -> Router {
    let state = AppState { relay, control_key: Arc::new(control_key) };
    let control = Router::new()
        .route("/control/projects/:id", get(get_project).put(put_project).delete(delete_project))
        .route("/control/projects/:id/environments/:env/migrations", post(post_migrations))
        .route("/control/projects/:id/environments/:env/schema", get(get_schema))
        .route("/control/projects/:id/environments/:env/types", get(get_types))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_control_key));
    Router::new().route("/health", get(health)).merge(control).with_state(state)
}
