//! Admin → Integrations → Relay: how Nomi reaches Relay (its address and control key, kept
//! encrypted in Nomi's database) and Relay's own settings (kept by Relay; Nomi passes them on
//! and never sees a database password back).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use nomi_auth::extractor::AuthClaims;
use nomi_relay_client::{Relay, RelayConfig, RelayError};

use crate::app::AppState;
use crate::relay_connection::{self, ConnectionView};
use crate::routes::deploy::ApiError;

/// Where previews run (WebContainers), suggested as the origins a dev backend answers.
const DEFAULT_PREVIEW_ORIGINS: [&str; 1] = [".webcontainer-api.io"];

fn admin(claims: &nomi_auth::claims::Claims, level: &str) -> Result<(), ApiError> {
    crate::routes::admin_access::require_admin_owned(claims, "integrations", level)
}

#[derive(Serialize)]
pub struct RelayOverview {
    pub connection: ConnectionView,
    /// Relay's settings and checks; null when Nomi can't reach Relay.
    pub relay: Option<Value>,
    /// Why Relay couldn't be asked, when it couldn't.
    pub relay_error: Option<String>,
    pub default_preview_origins: Vec<&'static str>,
}

fn describe(e: &RelayError) -> String {
    match e {
        RelayError::Unauthorized(_) => "Relay refused the control key".to_string(),
        RelayError::Unavailable(m) => format!("Relay couldn't be reached: {m}"),
        RelayError::NotFound(m) | RelayError::Rejected { message: m, .. } => m.clone(),
    }
}

async fn connection(state: &AppState) -> Option<RelayConfig> {
    relay_connection::load(&state.pool, Some(&state.settings_key)).await
}

/// `GET /api/admin/relay`
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<RelayOverview>, ApiError> {
    admin(&claims, "view")?;
    let config = connection(&state).await;
    let view = relay_connection::view(&state.pool, Some(&state.settings_key), RelayConfig::from_env().as_ref()).await;
    let (relay, relay_error) = match &config {
        None => (None, None),
        Some(config) => match Relay::new(&state.http_client, config).settings().await {
            Ok(settings) => (Some(settings), None),
            Err(e) => (None, Some(describe(&e))),
        },
    };
    Ok(Json(RelayOverview { connection: view, relay, relay_error, default_preview_origins: DEFAULT_PREVIEW_ORIGINS.to_vec() }))
}

#[derive(Deserialize)]
pub struct ConnectionInput {
    pub control_url: String,
    #[serde(default)]
    pub control_key: Option<String>,
}

/// `PUT /api/admin/relay/connection`: a missing or empty key keeps the stored one.
pub async fn put_connection(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(body): Json<ConnectionInput>) -> Result<Json<ConnectionView>, ApiError> {
    admin(&claims, "manage")?;
    relay_connection::save(&state.pool, &state.settings_key, &body.control_url, body.control_key.as_deref(), claims.sub)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    tracing::info!(user = %claims.sub, "Relay connection saved");
    Ok(Json(relay_connection::view(&state.pool, Some(&state.settings_key), None).await))
}

/// `PUT /api/admin/relay/settings`: passed to Relay, which checks every database and PostgREST
/// before keeping them. A refusal answers 422 `{ message, checks }`.
pub async fn put_settings(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(body): Json<Value>) -> Response {
    if let Err(e) = admin(&claims, "manage") {
        return e.into_response();
    }
    let Some(config) = connection(&state).await else {
        return (StatusCode::CONFLICT, "save how Nomi reaches Relay first".to_string()).into_response();
    };
    match Relay::new(&state.http_client, &config).save_settings(&body).await {
        Ok(saved) => {
            tracing::info!(user = %claims.sub, "Relay settings saved");
            Json(saved).into_response()
        }
        Err(RelayError::Rejected { message, details, .. }) => {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({ "message": message, "checks": details.get("checks").cloned().unwrap_or(Value::Null) }))).into_response()
        }
        Err(e) => (StatusCode::BAD_GATEWAY, describe(&e)).into_response(),
    }
}
