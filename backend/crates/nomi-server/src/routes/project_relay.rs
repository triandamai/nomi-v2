//! A project's Nomi Relay backend: as its preview reaches it, and as its admins see it in the
//! data sheet (tables, files, live tables, functions, sign-in, and SQL as the project's owner).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use nomi_relay_client::{Relay, RelayConfig};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::relay_ops::{on_relay, overview as relay_overview, preview_access, relay_env, relay_error, PreviewAccess};
use crate::routes::deploy::{require_admin, ApiError};
use crate::routes::project_database::{Source, SourceQuery};

/// The Relay for this project's `env`: its own, or the shared one. Says why when it isn't ready.
async fn connection(state: &AppState, project_id: Uuid, env: &str) -> Result<RelayConfig, ApiError> {
    crate::project_relays::for_request(&state.pool, Some(&state.settings_key), project_id, env)
        .await?
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "this project is on Relay, and Relay isn't set up on this Nomi".to_string()))
}

/// `GET /api/projects/:id/relay?source=`: the backend for the data sheet, or `{ backend:
/// "drizzle" }` for a project that keeps its data in Shipyard's Postgres.
pub async fn overview(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SourceQuery>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if !on_relay(&state.pool, project_id).await? {
        return Ok(Json(json!({ "backend": "drizzle" })));
    }
    let env = relay_env(query.source == Source::Production);
    let config = connection(&state, project_id, env).await?;
    relay_overview(&state.http_client, &config, project_id, env).await.map(Json)
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub sql: String,
    #[serde(default)]
    pub source: Source,
}

/// `POST /api/projects/:id/relay/query`: one statement as the project's owner, from its admins.
/// Answers `{ columns, rows, truncated, rows_affected, took_ms }`.
pub async fn run_query(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if !on_relay(&state.pool, project_id).await? {
        return Err((StatusCode::NOT_FOUND, "this project isn't on Relay".to_string()));
    }
    let env = relay_env(req.source == Source::Production);
    let config = connection(&state, project_id, env).await?;
    tracing::info!(%project_id, user = %claims.sub, env, "SQL run on a Relay backend");
    Relay::new(&state.http_client, &config).query(project_id, env, &req.sql, false).await.map(Json).map_err(relay_error)
}

/// `GET /api/projects/:id/relay/preview`: the dev backend's address and anon key (and, for the
/// project's admins, its service key), which the preview gets as `PUBLIC_NOMI_URL` and friends.
pub async fn preview(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<PreviewAccess>, ApiError> {
    let config = crate::project_relays::for_request(&state.pool, Some(&state.settings_key), project_id, "dev").await?;
    preview_access(&state.pool, &state.http_client, config.as_ref(), project_id, claims.sub).await.map(Json)
}
