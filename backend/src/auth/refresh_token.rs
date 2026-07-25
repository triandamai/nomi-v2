use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::{claims::Claims, permissions::compute_permissions};

const REFRESH_TOKEN_TTL_DAYS: &str = "30 days";
const ACCESS_TOKEN_TTL_SECONDS: i64 = 30 * 60;

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("refresh token invalid, revoked, or expired")]
    Invalid,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Claims(#[from] super::claims::ClaimsError),
}

pub fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_raw_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub async fn issue_refresh_token(pool: &PgPool, user_id: Uuid) -> Result<String, sqlx::Error> {
    let raw = generate_raw_token();
    let token_hash = hash_token(&raw);
    sqlx::query(&format!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, now() + interval '{REFRESH_TOKEN_TTL_DAYS}')"
    ))
    .bind(user_id)
    .bind(&token_hash)
    .execute(pool)
    .await?;
    Ok(raw)
}

pub async fn refresh_access_token(
    pool: &PgPool,
    raw_refresh_token: &str,
    jwt_secret: &str,
) -> Result<String, RefreshError> {
    let token_hash = hash_token(raw_refresh_token);

    let user_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM refresh_tokens WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    let user_id = user_id.ok_or(RefreshError::Invalid)?;

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

pub async fn revoke_refresh_token(pool: &PgPool, raw_refresh_token: &str) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(raw_refresh_token);
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
