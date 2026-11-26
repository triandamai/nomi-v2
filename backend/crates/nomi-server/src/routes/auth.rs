use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::sign_in_codes::{code_email, reset_email, EmailCodes};
use nomi_auth::{
    email_code::{self, Challenge, CodeError, Purpose},
    extractor::AuthClaims,
    password_reset::{self, ChangeError, ResetError},
    login::{check_password, issue_access_token, issue_sign_in_token, login, switch_active_org},
    refresh_token::{issue_refresh_token, refresh_access_token, revoke_refresh_token, tag_device},
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

pub(crate) type JsonError = (StatusCode, Json<serde_json::Value>);

pub(crate) fn json_error(status: StatusCode, error: &str) -> JsonError {
    (status, Json(serde_json::json!({ "error": error })))
}

pub(crate) fn code_error(e: CodeError) -> JsonError {
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
        CodeError::Account(e) => {
            tracing::error!(error = %e, "sign-in code: couldn't create the account");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed")
        }
        CodeError::Db(e) => {
            tracing::error!(error = %e, "sign-in code: database error");
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed")
        }
    }
}

async fn tokens_for(state: &AppState, headers: &HeaderMap, user_id: Uuid) -> Result<TokenPairResponse, JsonError> {
    let access_token = issue_sign_in_token(&state.pool, user_id, &state.jwt_secret)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to issue tokens"))?;
    let refresh_token = issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to issue refresh token"))?;
    note_device(state, headers, &refresh_token).await;
    Ok(TokenPairResponse { access_token, refresh_token })
}

/// Ties a sign-in to the device that made it (`X-Device-Id`), when it said. Best effort: without
/// it the sign-in still works, it just can't be ended from the device list.
pub(crate) async fn note_device(state: &AppState, headers: &HeaderMap, refresh_token: &str) {
    let Some(device_id) = super::devices::device_id(headers) else { return };
    if let Err(e) = tag_device(&state.pool, refresh_token, device_id).await {
        tracing::warn!(error = %e, %device_id, "couldn't tie a sign-in to its device");
    }
}

pub(crate) async fn email_the_code(mailer: &dyn nomi_mail::Mailer, challenge: &Challenge, code: &str, locale: nomi_i18n::Locale) -> Result<(), JsonError> {
    mailer.send(code_email(challenge, code, locale)).await.map_err(|e| {
        tracing::error!(error = %e, "couldn't send a sign-in code");
        json_error(StatusCode::BAD_GATEWAY, "email_failed")
    })
}

/// Where emailed codes go out: the sign-in mailer, or the configured one when password sign-in
/// doesn't ask for codes (`AUTH_EMAIL_CODES=off`) but someone signs in with their email alone.
pub(crate) fn code_mailer(state: &AppState) -> std::sync::Arc<dyn nomi_mail::Mailer> {
    match &state.email_codes {
        EmailCodes::Required(mailer) => mailer.clone(),
        EmailCodes::Off => nomi_mail::configured(),
    }
}

/// The language a waiting code's emails are in: the account's, or the one it was asked in.
async fn challenge_locale(state: &AppState, challenge: &Challenge) -> nomi_i18n::Locale {
    match challenge.user_id {
        Some(user_id) => match state.pool.acquire().await {
            Ok(mut conn) => nomi_agent_core::locale::user_locale(&mut conn, user_id).await,
            Err(_) => nomi_i18n::Locale::default(),
        },
        None => nomi_i18n::Locale::from_code_or_default(challenge.language.as_deref()),
    }
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
    headers: HeaderMap,
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
    super::orgs::announce_signup_joins(&state.pool, user_id).await;
    crate::shipyard::spawn_sync_user_orgs_quota(state.pool.clone(), state.http_client.clone(), state.shipyard.clone(), user_id);

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
    note_device(&state, &headers, &refresh_token).await;

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
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<SignInResponse>, JsonError> {
    let user_id = check_password(&state.pool, &req.email, &req.password).await.map_err(|e| match e {
        nomi_auth::login::LoginError::InvalidCredentials => json_error(StatusCode::UNAUTHORIZED, "invalid credentials"),
        _ => json_error(StatusCode::INTERNAL_SERVER_ERROR, "login failed"),
    })?;
    match &state.email_codes {
        EmailCodes::Required(mailer) => Ok(Json(send_code(&state, mailer.as_ref(), user_id, Purpose::Login, None).await?)),
        EmailCodes::Off => Ok(Json(SignInResponse::Tokens(tokens_for(&state, &headers, user_id).await?))),
    }
}

#[derive(Deserialize)]
pub struct StartRequest {
    pub email: String,
    /// The language the page was shown in, for the email (and a new account).
    pub language: Option<String>,
    /// A workspace invite to join when the address has no account yet.
    pub invite_code: Option<String>,
}

/// `POST /api/auth/start`: sign in or create an account with the email alone. A code goes to the
/// address either way; `POST /api/auth/verify` then signs in, or makes the account.
pub async fn start_handler(State(state): State<AppState>, Json(req): Json<StartRequest>) -> Result<Json<SignInResponse>, JsonError> {
    let email = req.email.trim().to_lowercase();
    if !super::orgs::valid_email(&email) {
        return Err(json_error(StatusCode::BAD_REQUEST, "invalid_email"));
    }
    let mailer = code_mailer(&state);
    let existing: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE lower(email) = $1")
        .bind(&email)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| code_error(CodeError::Db(e)))?;
    if let Some(user_id) = existing {
        let locale = req.language.as_deref().and_then(nomi_i18n::Locale::from_code);
        return Ok(Json(send_code(&state, mailer.as_ref(), user_id, Purpose::Login, locale).await?));
    }

    let invite_code = req.invite_code.as_deref().map(str::trim).filter(|c| !c.is_empty());
    if let Some(code) = invite_code {
        let open: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM org_invites WHERE code = $1 AND used_at IS NULL AND expires_at > now())")
            .bind(code)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| code_error(CodeError::Db(e)))?;
        if !open {
            return Err(json_error(StatusCode::BAD_REQUEST, "invalid_invite"));
        }
    }
    let language = req.language.as_deref().and_then(nomi_i18n::Locale::from_code).map(|l| l.code().to_string());
    let (challenge, code) =
        email_code::start_for_email(&state.pool, &email, invite_code, language.as_deref()).await.map_err(code_error)?;
    let locale = challenge_locale(&state, &challenge).await;
    email_the_code(mailer.as_ref(), &challenge, &code, locale).await?;
    Ok(Json(SignInResponse::Verification { verification: VerificationInfo::from(&challenge) }))
}

/// The tokens, and whether the code just made the account (so first run can ask for a name).
#[derive(Serialize)]
pub struct VerifyResponse {
    #[serde(flatten)]
    pub tokens: TokenPairResponse,
    pub is_new: bool,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: Uuid,
    pub code: String,
}

/// Finishes a sign-in (or a new account) with the emailed code.
pub async fn verify_handler(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<VerifyRequest>) -> Result<Json<VerifyResponse>, JsonError> {
    let verified = email_code::verify(&state.pool, req.challenge_id, &req.code).await.map_err(code_error)?;
    if verified.is_new {
        super::orgs::announce_signup_joins(&state.pool, verified.user_id).await;
    }
    Ok(Json(VerifyResponse { tokens: tokens_for(&state, &headers, verified.user_id).await?, is_new: verified.is_new }))
}

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub challenge_id: Uuid,
}

/// Sends a fresh code for a sign-in that's still waiting.
pub async fn resend_handler(State(state): State<AppState>, Json(req): Json<ChallengeRequest>) -> Result<Json<VerificationInfo>, JsonError> {
    let (challenge, code) = email_code::resend(&state.pool, req.challenge_id).await.map_err(code_error)?;
    let locale = challenge_locale(&state, &challenge).await;
    email_the_code(code_mailer(&state).as_ref(), &challenge, &code, locale).await?;
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
    headers: HeaderMap,
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
    note_device(&state, &headers, &req.refresh_token).await;

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

#[derive(Deserialize)]
pub struct ForgotRequest {
    pub email: String,
    /// The language the page was shown in, for the email.
    pub language: Option<String>,
}

/// Starts "forgot password": emails a reset link when the address has an account. It answers the
/// same either way (and when sending fails), so it can't be used to find out who has an account.
pub async fn forgot_handler(State(state): State<AppState>, Json(req): Json<ForgotRequest>) -> StatusCode {
    let started = match password_reset::start(&state.pool, &req.email).await {
        Ok(started) => started,
        Err(e) => {
            tracing::error!(error = %e, "forgot password: database error");
            return StatusCode::NO_CONTENT;
        }
    };
    let Some(started) = started else { return StatusCode::NO_CONTENT };
    let Some(url) = crate::public_url::link(&format!("/reset-password?token={}", started.token)) else {
        tracing::error!("forgot password: APP_URL isn't set, so the reset link can't be built");
        return StatusCode::NO_CONTENT;
    };
    let locale = match req.language.as_deref() {
        Some(code) => nomi_i18n::Locale::from_code_or_default(Some(code)),
        None => match state.pool.acquire().await {
            Ok(mut conn) => nomi_agent_core::locale::user_locale(&mut conn, started.user_id).await,
            Err(_) => nomi_i18n::Locale::default(),
        },
    };
    let mailer = match &state.email_codes {
        EmailCodes::Required(mailer) => mailer.clone(),
        EmailCodes::Off => nomi_mail::configured(),
    };
    if let Err(e) = mailer.send(reset_email(&started.email, url, locale)).await {
        tracing::error!(error = %e, "couldn't send a password reset link");
    }
    StatusCode::NO_CONTENT
}

/// Whether a reset link can still be used (for the page, before it asks for a password).
pub async fn reset_check_handler(State(state): State<AppState>, Path(token): Path<String>) -> Result<StatusCode, JsonError> {
    match password_reset::is_open(&state.pool, &token).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(json_error(StatusCode::GONE, "expired")),
        Err(e) => {
            tracing::error!(error = %e, "password reset: database error");
            Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed"))
        }
    }
}

#[derive(Deserialize)]
pub struct ResetRequest {
    pub token: String,
    pub password: String,
}

/// Chooses the new password with an emailed link; the account is signed out everywhere.
pub async fn reset_handler(State(state): State<AppState>, Json(req): Json<ResetRequest>) -> Result<StatusCode, JsonError> {
    match password_reset::finish(&state.pool, &req.token, &req.password).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(ResetError::Expired) => Err(json_error(StatusCode::GONE, "expired")),
        Err(ResetError::TooShort) => Err(json_error(StatusCode::BAD_REQUEST, "too_short")),
        Err(e) => {
            tracing::error!(error = %e, "password reset failed");
            Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed"))
        }
    }
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    /// Required when the account has a password already.
    pub current_password: Option<String>,
    pub new_password: String,
}

/// `PUT /api/auth/password`: sets or changes the signed-in person's password. Other devices stay
/// signed in.
pub async fn change_password_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, JsonError> {
    match password_reset::change(&state.pool, claims.sub, req.current_password.as_deref(), &req.new_password).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(ChangeError::TooShort) => Err(json_error(StatusCode::BAD_REQUEST, "too_short")),
        Err(ChangeError::WrongPassword) => Err(json_error(StatusCode::FORBIDDEN, "wrong_password")),
        Err(ChangeError::NoAccount) => Err(json_error(StatusCode::NOT_FOUND, "no_password_account")),
        Err(e) => {
            tracing::error!(error = %e, "change password failed");
            Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed"))
        }
    }
}

#[derive(Deserialize)]
pub struct ActiveOrgRequest {
    pub org_id: Uuid,
}

/// `PUT /api/auth/active-org`: switches the workspace the person works in. Answers a new access
/// token carrying it; later sign-ins and refreshes keep it while they're a member.
pub async fn active_org_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<ActiveOrgRequest>,
) -> Result<Json<AccessTokenResponse>, JsonError> {
    let switched = switch_active_org(&state.pool, claims.sub, req.org_id).await.map_err(|e| code_error(CodeError::Db(e)))?;
    if !switched {
        return Err(json_error(StatusCode::NOT_FOUND, "not_a_member"));
    }
    let access_token = issue_access_token(&state.pool, claims.sub, &state.jwt_secret)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to issue tokens"))?;
    Ok(Json(AccessTokenResponse { access_token }))
}
