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

    let active_org_id: Uuid = sqlx::query_scalar(
        "SELECT org_id FROM memberships WHERE user_id = $1 AND status = 'active' ORDER BY created_at ASC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let claims = Claims::new(user_id, active_org_id, permissions, ACCESS_TOKEN_TTL_SECONDS);
    Ok(claims.encode(jwt_secret)?)
}
