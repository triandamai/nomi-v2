use std::sync::OnceLock;

use sqlx::PgPool;
use uuid::Uuid;

use super::{
    claims::Claims,
    password::{hash_password, verify_password},
    permissions::compute_permissions,
};

pub const ACCESS_TOKEN_TTL_SECONDS: i64 = 30 * 60;

static DUMMY_PASSWORD_HASH: OnceLock<String> = OnceLock::new();

fn dummy_password_hash() -> &'static str {
    DUMMY_PASSWORD_HASH
        .get_or_init(|| hash_password("dummy-password-for-timing-safety").expect("dummy hash must succeed"))
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Claims(#[from] super::claims::ClaimsError),
}

pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<(String, Uuid), LoginError> {
    let user_id = check_password(pool, email, password).await?;
    let token = issue_access_token(pool, user_id, jwt_secret).await?;
    Ok((token, user_id))
}

/// The account `email` belongs to, when `password` is its password. Issues nothing: the caller
/// decides what else sign-in needs (an emailed code, see `email_code`).
pub async fn check_password(pool: &PgPool, email: &str, password: &str) -> Result<Uuid, LoginError> {
    let row: Option<(Uuid, String)> =
        sqlx::query_as("SELECT user_id, password_hash FROM web_credentials WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

    let (user_id, password_hash) = match row {
        Some(row) => row,
        None => {
            // Burn equivalent argon2 time so unknown-email and wrong-password paths
            // are indistinguishable by latency (closes an email-enumeration timing oracle).
            let _ = verify_password(password, dummy_password_hash());
            return Err(LoginError::InvalidCredentials);
        }
    };

    let valid = verify_password(password, &password_hash).unwrap_or(false);
    if !valid {
        return Err(LoginError::InvalidCredentials);
    }
    Ok(user_id)
}

/// An access token for a user whose identity was already proven some other way (a password
/// above, or a Google sign-in). Same claims as a password login.
pub async fn issue_access_token(pool: &PgPool, user_id: Uuid, jwt_secret: &str) -> Result<String, LoginError> {
    let permissions = compute_permissions(pool, user_id).await?;

    let active_org_id = active_org(pool, user_id).await?;

    let claims = Claims::new(user_id, active_org_id, permissions, ACCESS_TOKEN_TTL_SECONDS);
    Ok(claims.encode(jwt_secret)?)
}

/// The workspace a token carries: the one the person last switched to, while they're still an
/// active member of it, or else their first.
pub async fn active_org(pool: &PgPool, user_id: Uuid) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT m.org_id FROM memberships m JOIN users u ON u.id = m.user_id \
         WHERE m.user_id = $1 AND m.status = 'active' \
         ORDER BY (m.org_id = u.active_org_id) IS TRUE DESC, m.created_at ASC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Makes `org_id` the person's active workspace. `false` (and nothing changes) when they aren't
/// an active member of it.
pub async fn switch_active_org(pool: &PgPool, user_id: Uuid, org_id: Uuid) -> Result<bool, sqlx::Error> {
    let switched = sqlx::query(
        "UPDATE users SET active_org_id = $2 WHERE id = $1 \
         AND EXISTS (SELECT 1 FROM memberships WHERE user_id = $1 AND org_id = $2 AND status = 'active')",
    )
    .bind(user_id)
    .bind(org_id)
    .execute(pool)
    .await?;
    Ok(switched.rows_affected() > 0)
}
