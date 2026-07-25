use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::{
    authorize::authorize_org_action,
    extractor::AuthClaims,
    login::login,
    refresh_token::{issue_refresh_token, refresh_access_token, revoke_refresh_token},
    registration::{register_user, OrgMode},
};

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum OrgModeRequest {
    Create { name: String },
    Join { invite_code: String },
}

impl From<OrgModeRequest> for OrgMode {
    fn from(value: OrgModeRequest) -> Self {
        match value {
            OrgModeRequest::Create { name } => OrgMode::Create { name },
            OrgModeRequest::Join { invite_code } => OrgMode::Join { invite_code },
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub org: OrgModeRequest,
}

#[derive(Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<TokenPairResponse>, (StatusCode, &'static str)> {
    register_user(&state.pool, &req.email, &req.password, req.org.into())
        .await
        .map_err(|e| match e {
            crate::auth::registration::RegistrationError::EmailTaken => {
                (StatusCode::CONFLICT, "email already registered")
            }
            crate::auth::registration::RegistrationError::InvalidInvite => {
                (StatusCode::BAD_REQUEST, "invite code not found, expired, or already used")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "registration failed"),
        })?;

    // Per the design doc: registration immediately performs the same claims
    // computation as login and returns the token pair — no separate login
    // call needed. Reuses `login` directly since the credentials just
    // written are already valid.
    let (access_token, user_id) = login(&state.pool, &req.email, &req.password, &state.jwt_secret)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "registered but failed to issue tokens"))?;
    let refresh_token = issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to issue refresh token"))?;

    Ok(Json(TokenPairResponse { access_token, refresh_token }))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TokenPairResponse>, (StatusCode, &'static str)> {
    let (access_token, user_id) = login(&state.pool, &req.email, &req.password, &state.jwt_secret)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid credentials"))?;

    let refresh_token = issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to issue refresh token"))?;

    Ok(Json(TokenPairResponse { access_token, refresh_token }))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AccessTokenResponse>, (StatusCode, &'static str)> {
    let access_token = refresh_access_token(&state.pool, &req.refresh_token, &state.jwt_secret)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "refresh token invalid, revoked, or expired"))?;

    Ok(Json(AccessTokenResponse { access_token }))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    revoke_refresh_token(&state.pool, &req.refresh_token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to revoke refresh token"))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_org_action(&state.pool, claims.sub, org_id, &["owner", "admin"])
        .await
        .map_err(|_| (StatusCode::FORBIDDEN, "not authorized for this organization"))?;

    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove member"))?;

    Ok(StatusCode::NO_CONTENT)
}
