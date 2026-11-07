//! Forgot password: a single-use link, emailed to the account, lets the person choose a new
//! password. The link's token is random and only its hash is stored. It lasts an hour, and
//! using it signs the account out everywhere.

use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::password::{hash_password, PasswordError};

pub const RESET_TTL_SECONDS: i64 = 60 * 60;
/// Most links started for one account in an hour, so the form can't flood someone's inbox.
pub const MAX_RESETS_PER_HOUR: i64 = 5;
pub const MIN_PASSWORD_LENGTH: usize = 8;
const TOKEN_BYTES: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum ResetError {
    /// Unknown, expired, or already used: ask for a new link.
    #[error("this link has expired")]
    Expired,
    #[error("password too short")]
    TooShort,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Password(#[from] PasswordError),
}

/// Who a fresh link was made for, and the token to put in it.
#[derive(Debug, Clone)]
pub struct Started {
    pub user_id: Uuid,
    pub email: String,
    pub token: String,
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.trim().as_bytes()))
}

fn new_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Starts a reset for the account with this email. `None` when there is no such account, or too
/// many links went out lately; the caller answers the same either way, so nobody can probe which
/// emails have accounts. Any earlier open link for the account stops working.
pub async fn start(pool: &PgPool, email: &str) -> Result<Option<Started>, sqlx::Error> {
    let account: Option<(Uuid, String)> =
        sqlx::query_as("SELECT user_id, email FROM web_credentials WHERE lower(email) = lower($1)")
            .bind(email.trim())
            .fetch_optional(pool)
            .await?;
    let Some((user_id, email)) = account else { return Ok(None) };

    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM password_resets WHERE user_id = $1 AND created_at > now() - interval '1 hour'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if recent >= MAX_RESETS_PER_HOUR {
        return Ok(None);
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE password_resets SET expires_at = now() WHERE user_id = $1 AND used_at IS NULL AND expires_at > now()")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    let token = new_token();
    sqlx::query("INSERT INTO password_resets (user_id, token_hash, expires_at) VALUES ($1, $2, now() + make_interval(secs => $3))")
        .bind(user_id)
        .bind(hash_token(&token))
        .bind(RESET_TTL_SECONDS as f64)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Some(Started { user_id, email, token }))
}

/// Whether `token` can still be used (so the page can say so before asking for a password).
pub async fn is_open(pool: &PgPool, token: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM password_resets WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now())")
        .bind(hash_token(token))
        .fetch_one(pool)
        .await
}

/// Sets the new password, uses the link up, and signs the account out everywhere (refresh tokens
/// revoked, waiting sign-in codes cancelled). Choosing a password through the emailed link also
/// proves the email is theirs.
pub async fn finish(pool: &PgPool, token: &str, new_password: &str) -> Result<Uuid, ResetError> {
    if new_password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(ResetError::TooShort);
    }
    let password_hash = hash_password(new_password)?;

    let mut tx = pool.begin().await?;
    let user_id: Option<Uuid> = sqlx::query_scalar(
        "UPDATE password_resets SET used_at = now() \
         WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now() RETURNING user_id",
    )
    .bind(hash_token(token))
    .fetch_optional(&mut *tx)
    .await?;
    let Some(user_id) = user_id else { return Err(ResetError::Expired) };

    sqlx::query("UPDATE web_credentials SET password_hash = $2, email_verified_at = COALESCE(email_verified_at, now()) WHERE user_id = $1")
        .bind(user_id)
        .bind(&password_hash)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE sign_in_challenges SET expires_at = now() WHERE user_id = $1 AND used_at IS NULL AND expires_at > now()")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE password_resets SET expires_at = now() WHERE user_id = $1 AND used_at IS NULL AND expires_at > now()")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_random_and_hash_stably() {
        let (a, b) = (new_token(), new_token());
        assert_eq!(a.len(), TOKEN_BYTES * 2);
        assert_ne!(a, b);
        assert_eq!(hash_token(&a), hash_token(&format!(" {a} ")));
        assert_ne!(hash_token(&a), hash_token(&b));
    }
}
