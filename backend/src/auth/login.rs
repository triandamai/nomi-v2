use sqlx::PgPool;
use uuid::Uuid;

use super::{claims::Claims, permissions::compute_permissions, password::verify_password};

pub const ACCESS_TOKEN_TTL_SECONDS: i64 = 30 * 60;

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
    let row: Option<(Uuid, String)> =
        sqlx::query_as("SELECT user_id, password_hash FROM web_credentials WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

    let (user_id, password_hash) = row.ok_or(LoginError::InvalidCredentials)?;

    let valid = verify_password(password, &password_hash).unwrap_or(false);
    if !valid {
        return Err(LoginError::InvalidCredentials);
    }

    let permissions = compute_permissions(pool, user_id).await?;

    let active_org_id: Uuid = sqlx::query_scalar(
        "SELECT org_id FROM memberships WHERE user_id = $1 AND status = 'active' ORDER BY created_at ASC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let claims = Claims::new(user_id, active_org_id, permissions, ACCESS_TOKEN_TTL_SECONDS);
    let token = claims.encode(jwt_secret)?;

    Ok((token, user_id))
}
