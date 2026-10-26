//! Sessions: a 15-minute access token and a refresh token that works once. Each refresh hands back
//! a new refresh token in the same family; using an old one means it was stolen, so the whole
//! family ends.

use chrono::{Duration, Utc};
use rand::RngCore;
use relay_core::keys::{issued_now, user_token, ACCESS_TOKEN_SECONDS};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::users::{self, User};
use crate::{digest, AuthError, Scope};

/// Bytes of randomness in a refresh token.
const REFRESH_TOKEN_BYTES: usize = 32;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    /// Unix time, in seconds, when the access token stops working.
    pub expires_at: u64,
    pub user: User,
}

fn new_refresh_token() -> String {
    let mut bytes = [0u8; REFRESH_TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hashed(token: &str) -> String {
    digest(&["refresh", token])
}

fn issue(scope: &Scope<'_>, user: User, session: Uuid, refresh_token: String) -> Session {
    let now = issued_now();
    let access_token = user_token(scope.secret, scope.project, scope.env, user.id, session, user.email.as_deref(), now, ACCESS_TOKEN_SECONDS);
    Session { access_token, refresh_token, token_type: "bearer", expires_at: now + ACCESS_TOKEN_SECONDS, user }
}

/// Signs `user` in: a new session family lasting `ttl_days`.
pub async fn start(scope: &Scope<'_>, user: User, ttl_days: u32) -> Result<Session, AuthError> {
    let refresh_token = new_refresh_token();
    let expires = Utc::now() + Duration::days(i64::from(ttl_days));
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO nomi.sessions (project_id, user_id, family, refresh_token_hash, expires_at) VALUES ($1, $2, gen_random_uuid(), $3, $4) RETURNING id",
    )
    .bind(scope.project)
    .bind(user.id)
    .bind(hashed(&refresh_token))
    .bind(expires)
    .fetch_one(scope.pool)
    .await?;
    Ok(issue(scope, user, id, refresh_token))
}

/// Trades a refresh token for a new session in the same family.
pub async fn refresh(scope: &Scope<'_>, refresh_token: &str) -> Result<Session, AuthError> {
    let mut tx = scope.pool.begin().await?;
    let row: Option<(Uuid, Uuid, Uuid, bool, bool)> = sqlx::query_as(
        "SELECT id, user_id, family, revoked_at IS NOT NULL, expires_at <= now() FROM nomi.sessions \
         WHERE project_id = $1 AND refresh_token_hash = $2 FOR UPDATE",
    )
    .bind(scope.project)
    .bind(hashed(refresh_token))
    .fetch_optional(&mut *tx)
    .await?;
    let Some((id, user_id, family, revoked, expired)) = row else {
        return Err(AuthError::unauthorized("INVALID_REFRESH_TOKEN", "That refresh token isn't valid here: sign in again."));
    };
    if revoked {
        sqlx::query("UPDATE nomi.sessions SET revoked_at = COALESCE(revoked_at, now()) WHERE project_id = $1 AND family = $2")
            .bind(scope.project)
            .bind(family)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Err(AuthError::unauthorized("REFRESH_TOKEN_REUSED", "That refresh token was already used, so this session has ended: sign in again."));
    }
    if expired {
        return Err(AuthError::unauthorized("SESSION_EXPIRED", "This session has expired: sign in again."));
    }
    sqlx::query("UPDATE nomi.sessions SET revoked_at = now() WHERE id = $1").bind(id).execute(&mut *tx).await?;
    let next = new_refresh_token();
    let next_id: Uuid = sqlx::query_scalar(
        "INSERT INTO nomi.sessions (project_id, user_id, family, refresh_token_hash, expires_at) \
         SELECT project_id, user_id, family, $2, expires_at FROM nomi.sessions WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(hashed(&next))
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    let user = users::get(scope.pool, scope.project, user_id).await?.ok_or_else(|| AuthError::unauthorized("USER_GONE", "This user no longer exists."))?;
    Ok(issue(scope, user, next_id, next))
}

/// Whether the sign-in `session` belongs to is still going (not signed out, not reused, not expired).
pub async fn live(pool: &PgPool, project: Uuid, session: Uuid) -> Result<bool, AuthError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM nomi.sessions s JOIN nomi.sessions mine ON mine.family = s.family \
         WHERE mine.id = $2 AND mine.project_id = $1 AND s.revoked_at IS NULL AND s.expires_at > now())",
    )
    .bind(project)
    .bind(session)
    .fetch_one(pool)
    .await?)
}

/// Signs out: ends every session in `session`'s family.
pub async fn end(pool: &PgPool, project: Uuid, session: Uuid) -> Result<(), AuthError> {
    sqlx::query(
        "UPDATE nomi.sessions SET revoked_at = COALESCE(revoked_at, now()) WHERE project_id = $1 \
         AND family = (SELECT family FROM nomi.sessions WHERE id = $2 AND project_id = $1)",
    )
    .bind(project)
    .bind(session)
    .execute(pool)
    .await?;
    Ok(())
}
