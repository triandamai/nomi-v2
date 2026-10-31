//! The person's GitHub, and pushing a project's code to a repository with it. GitHub connects
//! through Nomi's OAuth app when Admin → Configuration has one, or with a token the person made
//! (either way it's theirs alone, encrypted with the settings key).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::github::{parse_repo, Github, GithubError, OAuthApp, Pushed};
use crate::routes::deploy::{internal, project_files, project_role, ApiError};

/// Characters in an OAuth state: unguessable for the ten minutes it lives.
const STATE_LENGTH: usize = 32;
/// The longest token accepted (GitHub's are well under it).
const MAX_TOKEN_CHARS: usize = 255;
/// The commit message when the person doesn't give one.
const DEFAULT_MESSAGE: &str = "Update from Nomi";
/// The longest commit message accepted, in characters.
const MAX_MESSAGE_CHARS: usize = 500;

#[derive(Serialize, sqlx::FromRow)]
pub struct GithubConnection {
    pub login: String,
    /// `oauth` (through Nomi's app) or `token` (one the person made).
    pub kind: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct GithubStatus {
    /// Whether "Connect GitHub" can sign in on GitHub (Nomi has an OAuth app); a token works always.
    pub oauth: bool,
    pub connection: Option<GithubConnection>,
}

fn github_error(e: GithubError) -> ApiError {
    match e {
        GithubError::TokenRejected => (StatusCode::BAD_REQUEST, "GitHub didn't accept the token; connect GitHub again".to_string()),
        GithubError::NotFound(message) => (StatusCode::NOT_FOUND, message),
        GithubError::Refused(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        GithubError::Unreachable(message) => {
            tracing::warn!(%message, "GitHub unavailable");
            (StatusCode::BAD_GATEWAY, "GitHub couldn't be reached. Try again in a moment.".to_string())
        }
    }
}

async fn connection(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<GithubConnection>, ApiError> {
    sqlx::query_as("SELECT login, kind, created_at, last_used_at FROM github_connections WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal("failed to load the GitHub connection"))
}

async fn status(state: &AppState, user_id: Uuid) -> Result<GithubStatus, ApiError> {
    Ok(GithubStatus { oauth: OAuthApp::current().await.is_some(), connection: connection(&state.pool, user_id).await? })
}

/// Checks `token` with GitHub and keeps it as the person's connection.
async fn save(state: &AppState, user_id: Uuid, token: &str, kind: &str) -> Result<GithubStatus, ApiError> {
    let login = Github::new(&state.http_client, token).login().await.map_err(github_error)?;
    sqlx::query(
        "INSERT INTO github_connections (user_id, login, kind, token_encrypted) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id) DO UPDATE SET login = EXCLUDED.login, kind = EXCLUDED.kind, token_encrypted = EXCLUDED.token_encrypted, \
         created_at = now(), last_used_at = NULL",
    )
    .bind(user_id)
    .bind(&login)
    .bind(kind)
    .bind(nomi_settings::crypto::encrypt(&state.settings_key, token))
    .execute(&state.pool)
    .await
    .map_err(internal("failed to save the GitHub connection"))?;
    status(state, user_id).await
}

/// `GET /api/connections/github`
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<GithubStatus>, ApiError> {
    Ok(Json(status(&state, claims.sub).await?))
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

/// `PUT /api/connections/github` `{ token }`: connects with a token the person made (repository
/// access). Checked with GitHub first.
pub async fn connect_token(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<TokenRequest>) -> Result<Json<GithubStatus>, ApiError> {
    let token = req.token.trim();
    if token.is_empty() || token.chars().count() > MAX_TOKEN_CHARS {
        return Err((StatusCode::BAD_REQUEST, "paste a GitHub token".to_string()));
    }
    Ok(Json(save(&state, claims.sub, token, "token").await?))
}

#[derive(Serialize)]
pub struct StartResponse {
    pub url: String,
}

/// `POST /api/connections/github/start`: the GitHub page to send the person to (503 when Nomi has
/// no OAuth app; a token still works).
pub async fn start(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<StartResponse>, ApiError> {
    let app = OAuthApp::current().await.ok_or((StatusCode::SERVICE_UNAVAILABLE, "GitHub sign-in isn't set up on this server; use a token".to_string()))?;
    let oauth_state = Alphanumeric.sample_string(&mut rand::thread_rng(), STATE_LENGTH);
    sqlx::query("DELETE FROM github_oauth_states WHERE created_at < now() - interval '10 minutes'")
        .execute(&state.pool)
        .await
        .map_err(internal("failed to start connecting GitHub"))?;
    sqlx::query("INSERT INTO github_oauth_states (state, user_id) VALUES ($1, $2)")
        .bind(&oauth_state)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to start connecting GitHub"))?;
    Ok(Json(StartResponse { url: app.authorize_url(&oauth_state) }))
}

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub state: String,
}

/// `POST /api/connections/github/callback` `{ code, state }`: what GitHub sent back to the
/// callback page. The state must be this person's, from the last ten minutes, and is used once.
pub async fn callback(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<CallbackRequest>) -> Result<Json<GithubStatus>, ApiError> {
    let app = OAuthApp::current().await.ok_or((StatusCode::SERVICE_UNAVAILABLE, "GitHub sign-in isn't set up on this server".to_string()))?;
    let ours: bool = sqlx::query_scalar(
        "DELETE FROM github_oauth_states WHERE state = $1 AND user_id = $2 AND created_at > now() - interval '10 minutes' RETURNING true",
    )
    .bind(&req.state)
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to finish connecting GitHub"))?
    .unwrap_or(false);
    if !ours {
        return Err((StatusCode::BAD_REQUEST, "that sign-in expired; connect GitHub again".to_string()));
    }
    let token = app.exchange(&state.http_client, &req.code).await.map_err(github_error)?;
    Ok(Json(save(&state, claims.sub, &token, "oauth").await?))
}

/// `DELETE /api/connections/github`: Nomi forgets the token. (A token made on GitHub stays
/// valid there until the person deletes it; an OAuth one is revoked from GitHub's settings.)
pub async fn disconnect(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<StatusCode, ApiError> {
    sqlx::query("DELETE FROM github_connections WHERE user_id = $1")
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to disconnect GitHub"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ExportRequest {
    /// `owner/name`, or a github.com link.
    pub repo: String,
    /// Make the repository when it doesn't exist.
    #[serde(default)]
    pub create: bool,
    /// For a repository made here; GitHub keeps an existing one as it is.
    #[serde(default = "yes")]
    pub private: bool,
    pub message: Option<String>,
}

fn yes() -> bool {
    true
}

/// `POST /api/projects/:id/export/github` `{ repo, create?, private?, message? }`: one commit of
/// the project's files on the repository's default branch, with the person's GitHub. Anyone who
/// can open the project may (as anyone may download it); what's never published (`.env` files,
/// build output, local databases) stays out.
pub async fn export(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<Pushed>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let (owner, name) = parse_repo(&req.repo).ok_or((StatusCode::BAD_REQUEST, "use owner/repo or a github.com link".to_string()))?;
    let message = req.message.as_deref().map(str::trim).filter(|m| !m.is_empty()).unwrap_or(DEFAULT_MESSAGE).to_string();
    if message.chars().count() > MAX_MESSAGE_CHARS {
        return Err((StatusCode::BAD_REQUEST, "the commit message is too long".to_string()));
    }
    let encrypted: Option<Vec<u8>> = sqlx::query_scalar("SELECT token_encrypted FROM github_connections WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to load the GitHub connection"))?;
    let encrypted = encrypted.ok_or((StatusCode::CONFLICT, "connect GitHub first".to_string()))?;
    let token = nomi_settings::crypto::decrypt(&state.settings_key, &encrypted).map_err(internal("failed to read the GitHub connection"))?;

    let files = project_files(&state, project_id).await?;
    if files.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "there's nothing to push yet".to_string()));
    }
    let pushed = Github::new(&state.http_client, &token).push(&owner, &name, req.create, req.private, &files, &message).await.map_err(github_error)?;
    sqlx::query("UPDATE github_connections SET last_used_at = now() WHERE user_id = $1")
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to record the push"))?;
    Ok(Json(pushed))
}
