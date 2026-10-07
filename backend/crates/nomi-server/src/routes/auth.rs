use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::sign_in_codes::{code_email, EmailCodes};
use nomi_auth::{
    authorize::authorize_org_action,
    email_code::{self, Challenge, CodeError, Purpose},
    extractor::AuthClaims,
    login::{check_password, issue_access_token, login},
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
    /// The language the sign-up page was shown in, for the confirmation email.
    pub language: Option<String>,
}

#[derive(Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
}

/// A code is on its way: sign-in finishes at `POST /api/auth/verify`.
#[derive(Serialize)]
pub struct VerificationInfo {
    pub challenge_id: Uuid,
    pub purpose: &'static str,
    /// The address the code went to, partly hidden.
    pub email: String,
    pub expires_in: i64,
    pub resend_in: i64,
    pub sends_left: i32,
}

impl From<&Challenge> for VerificationInfo {
    fn from(c: &Challenge) -> Self {
        let now = chrono::Utc::now();
        Self {
            challenge_id: c.id,
            purpose: c.purpose.as_str(),
            email: email_code::mask_email(&c.email),
            expires_in: (c.expires_at - now).num_seconds().max(0),
            resend_in: (c.resend_at - now).num_seconds().max(0),
            sends_left: c.sends_left,
        }
    }
}

/// What password sign-in (and sign-up) answers: the tokens, or — while emailed codes are on —
/// where the code went.
#[derive(Serialize)]
#[serde(untagged)]
pub enum SignInResponse {
    Tokens(TokenPairResponse),
    Verification { verification: VerificationInfo },
}

type JsonError = (StatusCode, Json<serde_json::Value>);

fn json_error(status: StatusCode, error: &str) -> JsonError {
    (status, Json(serde_json::json!({ "error": error })))
}

fn code_error(e: CodeError) -> JsonError {
    match e {
        CodeError::Wrong { attempts_left } => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "wrong_code", "attempts_left": attempts_left })))
        }
        CodeError::Expired => json_error(StatusCode::GONE, "expired"),
        CodeError::TooManyAttempts => json_error(StatusCode::GONE, "too_many_attempts"),
        CodeError::Wait { seconds } => {
            (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({ "error": "wait", "seconds": seconds })))
        }
        CodeError::TooManySends => json_error(StatusCode::TOO_MANY_REQUESTS, "too_many_codes"),
        CodeError::Db(e) => {
            tracing::error!(error = %e, "sign-in code: database error");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed")
        }
    }
}

async fn tokens_for(state: &AppState, user_id: Uuid) -> Result<TokenPairResponse, JsonError> {
    let access_token = issue_access_token(&state.pool, user_id, &state.jwt_secret)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to issue tokens"))?;
    let refresh_token = issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to issue refresh token"))?;
    Ok(TokenPairResponse { access_token, refresh_token })
}

async fn email_the_code(mailer: &dyn nomi_mail::Mailer, challenge: &Challenge, code: &str, locale: nomi_i18n::Locale) -> Result<(), JsonError> {
    mailer.send(code_email(challenge, code, locale)).await.map_err(|e| {
        tracing::error!(error = %e, "couldn't send a sign-in code");
        json_error(StatusCode::BAD_GATEWAY, "email_failed")
    })
}

/// Starts an emailed-code sign-in for `user_id` and sends the code.
async fn send_code(state: &AppState, mailer: &dyn nomi_mail::Mailer, user_id: Uuid, purpose: Purpose, locale: Option<nomi_i18n::Locale>) -> Result<SignInResponse, JsonError> {
    let (challenge, code) = email_code::start(&state.pool, user_id, purpose).await.map_err(code_error)?;
    let locale = match locale {
        Some(locale) => locale,
        None => match state.pool.acquire().await {
            Ok(mut conn) => nomi_agent_core::locale::user_locale(&mut conn, user_id).await,
            Err(_) => nomi_i18n::Locale::default(),
        },
    };
    email_the_code(mailer, &challenge, &code, locale).await?;
    Ok(SignInResponse::Verification { verification: VerificationInfo::from(&challenge) })
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<SignInResponse>, (StatusCode, &'static str)> {
    let user_id = register_user(&state.pool, &req.email, &req.password, req.org.into())
        .await
        .map_err(|e| match e {
            nomi_auth::registration::RegistrationError::EmailTaken => {
                (StatusCode::CONFLICT, "email already registered")
            }
            nomi_auth::registration::RegistrationError::InvalidInvite => {
                (StatusCode::BAD_REQUEST, "invite code not found, expired, or already used")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "registration failed"),
        })?;

    // A new account confirms its email with a code before it's signed in. If sending fails, the
    // account still exists: signing in sends a new code.
    if let EmailCodes::Required(mailer) = &state.email_codes {
        let locale = nomi_i18n::Locale::from_code(req.language.as_deref().unwrap_or("en")).unwrap_or_default();
        return match send_code(&state, mailer.as_ref(), user_id, Purpose::Register, Some(locale)).await {
            Ok(response) => Ok(Json(response)),
            Err((status, _)) if status == StatusCode::BAD_GATEWAY => Err((StatusCode::BAD_GATEWAY, "couldn't send the confirmation code")),
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "couldn't start email confirmation")),
        };
    }

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

    Ok(Json(SignInResponse::Tokens(TokenPairResponse { access_token, refresh_token })))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Password sign-in. With emailed codes on (the default), a right password only sends a code;
/// the tokens come from `POST /api/auth/verify`.
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<SignInResponse>, JsonError> {
    let user_id = check_password(&state.pool, &req.email, &req.password).await.map_err(|e| match e {
        nomi_auth::login::LoginError::InvalidCredentials => json_error(StatusCode::UNAUTHORIZED, "invalid credentials"),
        _ => json_error(StatusCode::INTERNAL_SERVER_ERROR, "login failed"),
    })?;
    match &state.email_codes {
        EmailCodes::Required(mailer) => Ok(Json(send_code(&state, mailer.as_ref(), user_id, Purpose::Login, None).await?)),
        EmailCodes::Off => Ok(Json(SignInResponse::Tokens(tokens_for(&state, user_id).await?))),
    }
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: Uuid,
    pub code: String,
}

/// Finishes a sign-in (or a new account) with the emailed code.
pub async fn verify_handler(State(state): State<AppState>, Json(req): Json<VerifyRequest>) -> Result<Json<TokenPairResponse>, JsonError> {
    let (user_id, _) = email_code::verify(&state.pool, req.challenge_id, &req.code).await.map_err(code_error)?;
    Ok(Json(tokens_for(&state, user_id).await?))
}

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub challenge_id: Uuid,
}

/// Sends a fresh code for a sign-in that's still waiting.
pub async fn resend_handler(State(state): State<AppState>, Json(req): Json<ChallengeRequest>) -> Result<Json<VerificationInfo>, JsonError> {
    let EmailCodes::Required(mailer) = &state.email_codes else { return Err(json_error(StatusCode::NOT_FOUND, "expired")) };
    let (challenge, code) = email_code::resend(&state.pool, req.challenge_id).await.map_err(code_error)?;
    let locale = match state.pool.acquire().await {
        Ok(mut conn) => nomi_agent_core::locale::user_locale(&mut conn, challenge.user_id).await,
        Err(_) => nomi_i18n::Locale::default(),
    };
    email_the_code(mailer.as_ref(), &challenge, &code, locale).await?;
    Ok(Json(VerificationInfo::from(&challenge)))
}

/// Where a waiting sign-in stands (for the code page after a reload).
pub async fn challenge_handler(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<VerificationInfo>, JsonError> {
    match email_code::find(&state.pool, id).await {
        Ok(Some(challenge)) => Ok(Json(VerificationInfo::from(&challenge))),
        Ok(None) => Err(json_error(StatusCode::GONE, "expired")),
        Err(e) => Err(code_error(CodeError::Db(e))),
    }
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
        .map_err(|e| match e {
            nomi_auth::refresh_token::RefreshError::Invalid => {
                (StatusCode::UNAUTHORIZED, "refresh token invalid, revoked, or expired")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "refresh failed"),
        })?;

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
