//! Connections: each user links their own Google account for the Workspace agent. The OAuth
//! client is the server's (GOOGLE_* env); the account and its tokens are the user's alone.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_agent_workspace::connection::{self, Activity, ConnectionStatus, GoogleConfig};
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct GoogleConnectionResponse {
    /// Whether whoever runs Nomi has set up Google sign-in at all.
    pub configured: bool,
    pub connection: Option<ConnectionStatus>,
    pub activity: Vec<Activity>,
    pub services: Vec<&'static str>,
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!(error = %e, "connections request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "connections request failed".to_string())
}

fn not_configured() -> (StatusCode, String) {
    (StatusCode::SERVICE_UNAVAILABLE, "Google sign-in isn't set up on this server yet.".to_string())
}

pub async fn get_google(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<GoogleConnectionResponse>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let status = connection::status(&mut conn, claims.sub).await.map_err(internal)?;
    let activity = if status.is_some() { connection::recent_activity(&mut conn, claims.sub, 10).await.map_err(internal)? } else { Vec::new() };
    Ok(Json(GoogleConnectionResponse {
        configured: GoogleConfig::current().await.is_some(),
        connection: status,
        activity,
        services: connection::SERVICES.to_vec(),
    }))
}

#[derive(Deserialize)]
pub struct StartRequest {
    pub services: Vec<String>,
    /// The chat whose request needed Google: it carries on once connected.
    pub resume_session_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct StartResponse {
    pub url: String,
}

pub async fn start_google(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<StartRequest>,
) -> Result<Json<StartResponse>, (StatusCode, String)> {
    let config = GoogleConfig::current().await.ok_or_else(not_configured)?;
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    // Only a chat this user wrote in can be resumed.
    let resume = match req.resume_session_id {
        Some(session_id) => sqlx::query_scalar::<_, Uuid>(
            "SELECT m.session_id FROM messages m JOIN channel_identities ci ON ci.id = m.sender_channel_identity_id \
             WHERE m.session_id = $1 AND ci.user_id = $2 LIMIT 1",
        )
        .bind(session_id)
        .bind(claims.sub)
        .fetch_optional(&mut *conn)
        .await
        .map_err(internal)?,
        None => None,
    };
    let google_email: Option<String> = sqlx::query_scalar("SELECT email FROM google_identities WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_optional(&mut *conn)
        .await
        .map_err(internal)?;
    let url = connection::start_authorization(&mut conn, &config, claims.sub, &req.services, resume, google_email.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(StartResponse { url }))
}

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub state: String,
}

#[derive(Serialize)]
pub struct CallbackResponse {
    pub email: String,
    pub services: Vec<String>,
    /// Set when a chat's request is being picked up again.
    pub resume_session_id: Option<Uuid>,
}

pub async fn complete_google(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<CallbackRequest>,
) -> Result<Json<CallbackResponse>, (StatusCode, String)> {
    let config = GoogleConfig::current().await.ok_or_else(not_configured)?;
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let connected = connection::complete_authorization(&mut conn, &state.http_client, &config, &state.settings_key, claims.sub, &req.code, &req.state)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Carry on with the request that asked for Google: the user's last message in that chat runs
    // again, now that Workspace can reach their account.
    if let Some(session_id) = connected.resume_session_id {
        let last: Option<(Uuid, String)> = sqlx::query_as(
            "SELECT m.sender_channel_identity_id, m.content FROM messages m JOIN channel_identities ci ON ci.id = m.sender_channel_identity_id \
             WHERE m.session_id = $1 AND ci.user_id = $2 ORDER BY m.created_at DESC LIMIT 1",
        )
        .bind(session_id)
        .bind(claims.sub)
        .fetch_optional(&mut *conn)
        .await
        .map_err(internal)?;
        if let Some((identity_id, text)) = last {
            let resume_notice = nomi_agent_core::user_locale(&mut conn, claims.sub).await.t("turn.workspace_resumed");
            let mut tx = state.pool.begin().await.map_err(internal)?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, NULL)")
                .bind(session_id)
                .bind(&resume_notice)
                .execute(&mut *tx)
                .await
                .map_err(internal)?;
            nomi_turn::queue::enqueue(&mut tx, session_id, identity_id, &text, None).await.map_err(internal)?;
            tx.commit().await.map_err(internal)?;
        }
    }

    Ok(Json(CallbackResponse { email: connected.email, services: connected.services, resume_session_id: connected.resume_session_id }))
}

pub async fn disconnect_google(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let config = GoogleConfig::current().await;
    connection::disconnect(&mut conn, &state.http_client, config.as_ref(), &state.settings_key, claims.sub).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Every connector's state for this person (the Connections list).
#[derive(Serialize)]
pub struct ConnectionsOverview {
    pub google: GoogleOverview,
    pub hevy: Option<nomi_agent_health::connection::HealthConnection>,
}

#[derive(Serialize)]
pub struct GoogleOverview {
    pub configured: bool,
    pub email: Option<String>,
    pub services: Vec<String>,
}

/// `GET /api/connections`
pub async fn overview(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ConnectionsOverview>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let google = connection::status(&mut conn, claims.sub).await.map_err(internal)?;
    let hevy = nomi_agent_health::connection::status(&mut conn, claims.sub, nomi_agent_health::connection::HEVY).await.map_err(internal)?;
    Ok(Json(ConnectionsOverview {
        google: GoogleOverview {
            configured: GoogleConfig::current().await.is_some(),
            email: google.as_ref().map(|g| g.email.clone()),
            services: google.map(|g| g.services).unwrap_or_default(),
        },
        hevy,
    }))
}

#[derive(Serialize)]
pub struct HevyResponse {
    pub connection: Option<nomi_agent_health::connection::HealthConnection>,
}

/// `GET /api/connections/hevy`
pub async fn get_hevy(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<HevyResponse>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let connection = nomi_agent_health::connection::status(&mut conn, claims.sub, nomi_agent_health::connection::HEVY).await.map_err(internal)?;
    Ok(Json(HevyResponse { connection }))
}

#[derive(Deserialize)]
pub struct HevyKeyRequest {
    pub api_key: String,
}

/// A Hevy route's error: `{"error": <code>, "detail"?}`, worded on the page in the person's language.
type HevyRouteError = (StatusCode, Json<serde_json::Value>);

fn hevy_error(status: StatusCode, code: &str, detail: Option<String>) -> HevyRouteError {
    (status, Json(serde_json::json!({ "error": code, "detail": detail })))
}

/// `PUT /api/connections/hevy`: checks the key with Hevy, then saves it.
pub async fn connect_hevy(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<HevyKeyRequest>,
) -> Result<Json<HevyResponse>, HevyRouteError> {
    use nomi_agent_health::connection::ConnectError;
    use nomi_agent_health::hevy::HevyError;
    let key = req.api_key.trim();
    if key.is_empty() || key.len() > 200 {
        return Err(hevy_error(StatusCode::BAD_REQUEST, "missing_key", None));
    }
    let mut conn = state.pool.acquire().await.map_err(|e| {
        tracing::error!(error = %e, "hevy: no connection");
        hevy_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None)
    })?;
    match nomi_agent_health::connection::connect_hevy(&mut conn, &state.http_client, &nomi_agent_health::hevy::base_url(), &state.settings_key, claims.sub, key).await {
        Ok(connection) => Ok(Json(HevyResponse { connection: Some(connection) })),
        Err(ConnectError::Rejected(HevyError::KeyRejected)) => Err(hevy_error(StatusCode::BAD_REQUEST, "key_rejected", None)),
        Err(ConnectError::Rejected(HevyError::Unreachable(why))) => {
            tracing::warn!(error = %why, "hevy: couldn't check a key");
            Err(hevy_error(StatusCode::BAD_GATEWAY, "unreachable", None))
        }
        Err(ConnectError::Rejected(other)) => Err(hevy_error(StatusCode::BAD_REQUEST, "rejected", Some(other.to_string()))),
        Err(ConnectError::Db(e)) => {
            tracing::error!(error = %e, "hevy: couldn't save the key");
            Err(hevy_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None))
        }
    }
}

/// `DELETE /api/connections/hevy`
pub async fn disconnect_hevy(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    nomi_agent_health::connection::disconnect(&mut conn, claims.sub, nomi_agent_health::connection::HEVY).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}
