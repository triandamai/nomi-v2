//! Settings › General's account rows: moving the account to a new email address (confirmed with a
//! code sent to the new address) and closing the account (with the password or a fresh code).
//!
//! A closed account is kept as a row, as an admin removing someone keeps their membership row:
//! chats, projects and usage other people share point at it. Its sign-in, sign-ins on every
//! device, memberships and personal data (profile, memories, connections, keys, reminders) are
//! removed. A workspace only they are in closes with it; one they're the only owner of, with
//! other people in it, has to be handed over (or closed) first.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::email_code::{self, CodeError, Purpose};
use nomi_auth::extractor::AuthClaims;

use super::auth::{
    code_error, code_mailer, email_the_code, json_error, JsonError, VerificationInfo,
};
use crate::app::AppState;

async fn locale_of(state: &AppState, user_id: Uuid) -> nomi_i18n::Locale {
    match state.pool.acquire().await {
        Ok(mut conn) => nomi_agent_core::locale::user_locale(&mut conn, user_id).await,
        Err(_) => nomi_i18n::Locale::default(),
    }
}

fn db_error(e: sqlx::Error) -> JsonError {
    code_error(CodeError::Db(e))
}

/// Sends a code for `purpose` (to `email`, or the account's own address) and says where it went.
async fn send_account_code(
    state: &AppState,
    user_id: Uuid,
    purpose: Purpose,
    email: Option<&str>,
) -> Result<Json<VerificationInfo>, JsonError> {
    let (challenge, code) = email_code::start_for_account(&state.pool, user_id, purpose, email)
        .await
        .map_err(code_error)?;
    let locale = locale_of(state, user_id).await;
    email_the_code(code_mailer(state).as_ref(), &challenge, &code, locale).await?;
    Ok(Json(VerificationInfo::from(&challenge)))
}

#[derive(Deserialize)]
pub struct ChangeEmailRequest {
    pub email: String,
}

/// `POST /api/account/email`: sends a code to the new address. Nothing changes until it's entered
/// (`POST /api/account/email/verify`).
pub async fn start_email_change(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<ChangeEmailRequest>,
) -> Result<Json<VerificationInfo>, JsonError> {
    let email = req.email.trim().to_lowercase();
    if !super::orgs::valid_email(&email) {
        return Err(json_error(StatusCode::BAD_REQUEST, "invalid_email"));
    }
    let current: Option<String> =
        sqlx::query_scalar("SELECT lower(email) FROM web_credentials WHERE user_id = $1")
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(db_error)?;
    let Some(current) = current else {
        return Err(json_error(StatusCode::NOT_FOUND, "no_email_account"));
    };
    if current == email {
        return Err(json_error(StatusCode::BAD_REQUEST, "same_email"));
    }
    if email_taken(&state, &email).await? {
        return Err(json_error(StatusCode::CONFLICT, "email_taken"));
    }
    send_account_code(&state, claims.sub, Purpose::ChangeEmail, Some(&email)).await
}

async fn email_taken(state: &AppState, email: &str) -> Result<bool, JsonError> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM web_credentials WHERE lower(email) = $1)")
        .bind(email)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)
}

#[derive(Deserialize)]
pub struct CodeRequest {
    pub challenge_id: Uuid,
    pub code: String,
}

#[derive(Serialize)]
pub struct EmailChanged {
    pub email: String,
}

/// `POST /api/account/email/verify`: the code from the new address moves the account to it (it
/// counts as confirmed). Sign-in with the old address stops working.
pub async fn verify_email_change(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<CodeRequest>,
) -> Result<Json<EmailChanged>, JsonError> {
    let email = email_code::verify_for_account(
        &state.pool,
        req.challenge_id,
        claims.sub,
        Purpose::ChangeEmail,
        &req.code,
    )
    .await
    .map_err(code_error)?
    .to_lowercase();
    // Someone may have taken it while the code was on its way.
    if email_taken(&state, &email).await? {
        return Err(json_error(StatusCode::CONFLICT, "email_taken"));
    }
    let updated = sqlx::query(
        "UPDATE web_credentials SET email = $2, email_verified_at = now() WHERE user_id = $1",
    )
    .bind(claims.sub)
    .bind(&email)
    .execute(&state.pool)
    .await;
    match updated {
        Ok(_) => Ok(Json(EmailChanged { email })),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            Err(json_error(StatusCode::CONFLICT, "email_taken"))
        }
        Err(e) => Err(db_error(e)),
    }
}

/// `POST /api/account/delete/code`: sends a code to the account's address, to close it without
/// the password (an account made with Google or a code has none).
pub async fn start_delete(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<VerificationInfo>, JsonError> {
    let has_email: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM web_credentials WHERE user_id = $1)")
            .bind(claims.sub)
            .fetch_one(&state.pool)
            .await
            .map_err(db_error)?;
    if !has_email {
        return Err(json_error(StatusCode::NOT_FOUND, "no_email_account"));
    }
    send_account_code(&state, claims.sub, Purpose::DeleteAccount, None).await
}

/// The password, or a code from `POST /api/account/delete/code`.
#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub password: Option<String>,
    pub challenge_id: Option<Uuid>,
    pub code: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct OrgToHandOver {
    pub id: Uuid,
    pub name: String,
}

/// Workspaces the person is the only owner of while other people are in them.
async fn orgs_to_hand_over(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<OrgToHandOver>, sqlx::Error> {
    sqlx::query_as(
        "SELECT o.id, o.name FROM memberships m JOIN organizations o ON o.id = m.org_id AND o.deleted_at IS NULL \
         WHERE m.user_id = $1 AND m.status = 'active' AND m.role = 'owner' \
           AND NOT EXISTS (SELECT 1 FROM memberships x WHERE x.org_id = m.org_id AND x.user_id <> $1 AND x.status = 'active' AND x.role = 'owner') \
           AND EXISTS (SELECT 1 FROM memberships x WHERE x.org_id = m.org_id AND x.user_id <> $1 AND x.status = 'active') \
         ORDER BY o.name",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
}

async fn check_password(
    state: &AppState,
    user_id: Uuid,
    password: &str,
) -> Result<bool, JsonError> {
    let hash: Option<String> = sqlx::query_scalar(
        "SELECT password_hash FROM web_credentials WHERE user_id = $1 AND password_login",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?;
    Ok(hash
        .is_some_and(|hash| nomi_auth::password::verify_password(password, &hash).unwrap_or(false)))
}

/// Workspaces only the person is in: they close with the account.
async fn orgs_to_close(state: &AppState, user_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT m.org_id FROM memberships m JOIN organizations o ON o.id = m.org_id AND o.deleted_at IS NULL \
         WHERE m.user_id = $1 AND m.status = 'active' \
           AND NOT EXISTS (SELECT 1 FROM memberships x WHERE x.org_id = m.org_id AND x.user_id <> $1 AND x.status = 'active')",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
}

/// How a workspace that closes with the account is closed (as `DELETE /api/orgs/:id` does);
/// `$1` is the workspaces' ids.
const CLOSE_ORGS: &[&str] = &[
    "DELETE FROM org_invites WHERE used_at IS NULL AND org_id = ANY($1)",
    "UPDATE user_preferences SET project_org_id = NULL WHERE project_org_id = ANY($1)",
    "UPDATE user_preferences SET open_org_id = NULL WHERE open_org_id = ANY($1)",
    "UPDATE organizations SET deleted_at = now() WHERE id = ANY($1)",
];

/// What closing an account removes or ends, in order; `$1` is the account.
const CLOSE_ACCOUNT: &[&str] = &[
    "DELETE FROM org_invites WHERE used_at IS NULL AND invited_by = $1",
    "UPDATE memberships SET status = 'removed' WHERE user_id = $1",
    "DELETE FROM project_members WHERE user_id = $1",
    "DELETE FROM project_presence WHERE user_id = $1",
    "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
    "DELETE FROM devices WHERE user_id = $1",
    "DELETE FROM sign_in_challenges WHERE user_id = $1",
    "DELETE FROM password_resets WHERE user_id = $1",
    "DELETE FROM google_identities WHERE user_id = $1",
    "DELETE FROM web_credentials WHERE user_id = $1",
    "DELETE FROM chat_shares WHERE created_by = $1",
    "DELETE FROM user_profiles WHERE user_id = $1",
    "DELETE FROM memory_items WHERE user_id = $1",
    "DELETE FROM memory_profiles WHERE user_id = $1",
    "DELETE FROM memory_settings WHERE user_id = $1",
    "DELETE FROM reminders WHERE user_id = $1",
    "UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() WHERE user_id = $1 AND status = 'active'",
    "DELETE FROM notifications WHERE user_id = $1",
    "DELETE FROM workspace_connections WHERE user_id = $1",
    "DELETE FROM github_connections WHERE user_id = $1",
    "DELETE FROM health_connections WHERE user_id = $1",
    "DELETE FROM mcp_servers WHERE user_id = $1",
    "DELETE FROM user_llm_selections WHERE user_id = $1",
    "DELETE FROM llm_provider_connections c WHERE c.user_id = $1 \
       AND NOT EXISTS (SELECT 1 FROM admin_llm_models a WHERE a.connection_id = c.id)",
    "DELETE FROM user_preferences WHERE user_id = $1",
    "UPDATE users SET deleted_at = now(), active_org_id = NULL, recent_activity = NULL, recent_activity_at = NULL WHERE id = $1",
];

/// `DELETE /api/account`: closes the signed-in person's account, with their password or a fresh
/// code. Every device is signed out. `409 sole_owner` lists the workspaces to hand over first.
pub async fn delete_account(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<DeleteAccountRequest>,
) -> Result<StatusCode, JsonError> {
    let user_id = claims.sub;
    // Check what's wrong before using up a code, so a refused delete can be tried again.
    let blocking = orgs_to_hand_over(&state, user_id).await.map_err(db_error)?;
    if !blocking.is_empty() {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "sole_owner", "orgs": blocking })),
        ));
    }
    match (
        req.password.as_deref().filter(|p| !p.is_empty()),
        req.challenge_id,
        req.code.as_deref(),
    ) {
        (Some(password), _, _) => {
            if !check_password(&state, user_id, password).await? {
                return Err(json_error(StatusCode::FORBIDDEN, "wrong_password"));
            }
        }
        (None, Some(challenge_id), Some(code)) => {
            email_code::verify_for_account(
                &state.pool,
                challenge_id,
                user_id,
                Purpose::DeleteAccount,
                code,
            )
            .await
            .map_err(code_error)?;
        }
        _ => {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                "password_or_code_required",
            ))
        }
    }

    let devices: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM devices WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?;
    let closing = orgs_to_close(&state, user_id).await.map_err(db_error)?;
    let mut tx = state.pool.begin().await.map_err(db_error)?;
    if !closing.is_empty() {
        for statement in CLOSE_ORGS {
            sqlx::query(statement).bind(&closing).execute(&mut *tx).await.map_err(|e| {
                tracing::error!(error = %e, statement, "closing an account's workspaces failed");
                db_error(e)
            })?;
        }
    }
    for statement in CLOSE_ACCOUNT {
        sqlx::query(statement)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, statement, "closing an account failed");
                db_error(e)
            })?;
    }
    tx.commit().await.map_err(db_error)?;
    for device_id in devices {
        crate::presence::announce(
            user_id,
            &nomi_realtime::UserEnvelope::DeviceSignedOut { device_id },
        );
    }
    tracing::info!(%user_id, "account closed");
    Ok(StatusCode::NO_CONTENT)
}
