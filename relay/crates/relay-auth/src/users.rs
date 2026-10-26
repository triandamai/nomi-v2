//! An app's users and their sign-in methods, as the app and its admins see them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{normalize_email, AuthError};

/// Postgres' unique violation.
const UNIQUE_VIOLATION: &str = "23505";
/// The most users one page of `list` answers.
pub const MAX_PER_PAGE: i64 = 1000;
pub const EMAIL_PROVIDER: &str = "email";

#[derive(Debug, Clone, Serialize)]
pub struct Identity {
    pub id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_sign_in_at: Option<DateTime<Utc>>,
    pub identities: Vec<Identity>,
    pub user_metadata: Value,
}

/// What an admin may set; fields left out stay as they are. `user_metadata` merges.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Attributes {
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub user_metadata: Option<Value>,
}

type UserRow = (Uuid, Option<String>, bool, DateTime<Utc>, Option<DateTime<Utc>>, Value);
const USER_COLUMNS: &str = "id, email, email_verified, created_at, last_sign_in_at, user_metadata";

async fn identities(pool: &PgPool, project: Uuid, user: Uuid) -> Result<Vec<Identity>, AuthError> {
    let rows: Vec<(Uuid, String, String, Option<String>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, provider, provider_user_id, email, created_at FROM nomi.identities WHERE project_id = $1 AND user_id = $2 ORDER BY created_at",
    )
    .bind(project)
    .bind(user)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(id, provider, provider_user_id, email, created_at)| Identity { id, provider, provider_user_id, email, created_at }).collect())
}

async fn with_identities(pool: &PgPool, project: Uuid, row: UserRow) -> Result<User, AuthError> {
    let (id, email, email_verified, created_at, last_sign_in_at, user_metadata) = row;
    Ok(User { id, email, email_verified, created_at, last_sign_in_at, identities: identities(pool, project, id).await?, user_metadata })
}

pub async fn get(pool: &PgPool, project: Uuid, id: Uuid) -> Result<Option<User>, AuthError> {
    let row: Option<UserRow> = sqlx::query_as(&format!("SELECT {USER_COLUMNS} FROM nomi.users WHERE project_id = $1 AND id = $2"))
        .bind(project)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    match row {
        Some(row) => Ok(Some(with_identities(pool, project, row).await?)),
        None => Ok(None),
    }
}

/// One page of users, newest first, and how many there are.
pub async fn list(pool: &PgPool, project: Uuid, page: i64, per_page: i64) -> Result<(Vec<User>, i64), AuthError> {
    let per_page = per_page.clamp(1, MAX_PER_PAGE);
    let offset = (page.max(1) - 1) * per_page;
    let rows: Vec<UserRow> = sqlx::query_as(&format!("SELECT {USER_COLUMNS} FROM nomi.users WHERE project_id = $1 ORDER BY created_at DESC, id LIMIT $2 OFFSET $3"))
        .bind(project)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM nomi.users WHERE project_id = $1").bind(project).fetch_one(pool).await?;
    let mut users = Vec::with_capacity(rows.len());
    for row in rows {
        users.push(with_identities(pool, project, row).await?);
    }
    Ok((users, total))
}

fn taken(e: sqlx::Error) -> AuthError {
    if e.as_database_error().and_then(|d| d.code()).is_some_and(|c| c == UNIQUE_VIOLATION) {
        AuthError::bad("USER_EXISTS", "A user with that email already exists.")
    } else {
        e.into()
    }
}

pub async fn create(pool: &PgPool, project: Uuid, attributes: Attributes) -> Result<User, AuthError> {
    let email = attributes.email.as_deref().map(normalize_email).transpose()?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO nomi.users (project_id, email, email_verified, user_metadata) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(project)
        .bind(&email)
        .bind(attributes.email_verified.unwrap_or(false))
        .bind(attributes.user_metadata.unwrap_or_else(|| Value::Object(Default::default())))
        .fetch_one(pool)
        .await
        .map_err(taken)?;
    if let Some(email) = &email {
        add_identity(pool, project, id, EMAIL_PROVIDER, email, Some(email)).await?;
    }
    get(pool, project, id).await?.ok_or_else(|| AuthError::Internal("the new user vanished".into()))
}

pub async fn update(pool: &PgPool, project: Uuid, id: Uuid, attributes: Attributes) -> Result<User, AuthError> {
    let email = attributes.email.as_deref().map(normalize_email).transpose()?;
    let changed = sqlx::query(
        "UPDATE nomi.users SET email = COALESCE($3, email), email_verified = COALESCE($4, email_verified), \
         user_metadata = user_metadata || COALESCE($5, '{}'::jsonb) WHERE project_id = $1 AND id = $2",
    )
    .bind(project)
    .bind(id)
    .bind(&email)
    .bind(attributes.email_verified)
    .bind(attributes.user_metadata)
    .execute(pool)
    .await
    .map_err(taken)?
    .rows_affected();
    if changed == 0 {
        return Err(AuthError::NotFound("No such user.".into()));
    }
    get(pool, project, id).await?.ok_or_else(|| AuthError::NotFound("No such user.".into()))
}

/// Deletes the user; their identities and sessions go with them.
pub async fn delete(pool: &PgPool, project: Uuid, id: Uuid) -> Result<(), AuthError> {
    let deleted = sqlx::query("DELETE FROM nomi.users WHERE project_id = $1 AND id = $2").bind(project).bind(id).execute(pool).await?.rows_affected();
    if deleted == 0 {
        return Err(AuthError::NotFound("No such user.".into()));
    }
    Ok(())
}

pub(crate) async fn add_identity(pool: &PgPool, project: Uuid, user: Uuid, provider: &str, provider_user_id: &str, email: Option<&str>) -> Result<(), AuthError> {
    sqlx::query(
        "INSERT INTO nomi.identities (project_id, user_id, provider, provider_user_id, email) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (project_id, provider, provider_user_id) DO NOTHING",
    )
    .bind(project)
    .bind(user)
    .bind(provider)
    .bind(provider_user_id)
    .bind(email)
    .execute(pool)
    .await?;
    Ok(())
}

/// The user who owns `email`, made (verified, with an `email` identity) when there's none.
pub(crate) async fn signed_in_by_email(pool: &PgPool, project: Uuid, email: &str) -> Result<User, AuthError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO nomi.users (project_id, email, email_verified, last_sign_in_at) VALUES ($1, $2, true, now()) \
         ON CONFLICT (project_id, email) DO UPDATE SET email_verified = true, last_sign_in_at = now() RETURNING id",
    )
    .bind(project)
    .bind(email)
    .fetch_one(pool)
    .await?;
    add_identity(pool, project, id, EMAIL_PROVIDER, email, Some(email)).await?;
    get(pool, project, id).await?.ok_or_else(|| AuthError::Internal("the user vanished".into()))
}

pub(crate) async fn exists_by_email(pool: &PgPool, project: Uuid, email: &str) -> Result<bool, AuthError> {
    Ok(sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM nomi.users WHERE project_id = $1 AND email = $2)").bind(project).bind(email).fetch_one(pool).await?)
}

/// The user an outside sign-in belongs to: the identity's owner; else, when linking, the user
/// being linked; else the user with that verified email; else a new user (with the email only
/// when the provider verified it and nobody has it).
pub(crate) async fn signed_in_by_identity(pool: &PgPool, project: Uuid, provider: &str, profile: &crate::oauth::Profile, link_user: Option<Uuid>) -> Result<Uuid, AuthError> {
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM nomi.identities WHERE project_id = $1 AND provider = $2 AND provider_user_id = $3")
        .bind(project)
        .bind(provider)
        .bind(&profile.provider_user_id)
        .fetch_optional(pool)
        .await?;
    let user = match (owner, link_user) {
        (Some(owner), Some(linking)) if owner != linking => {
            return Err(AuthError::bad("IDENTITY_TAKEN", "That account already signs in to another user of this app."));
        }
        (Some(owner), _) => owner,
        (None, Some(linking)) => linking,
        (None, None) => {
            let verified_email = profile.email.as_deref().filter(|_| profile.email_verified);
            let joined: Option<Uuid> = match verified_email {
                Some(email) => sqlx::query_scalar("SELECT id FROM nomi.users WHERE project_id = $1 AND email = $2 AND email_verified")
                    .bind(project)
                    .bind(email)
                    .fetch_optional(pool)
                    .await?,
                None => None,
            };
            match joined {
                Some(id) => id,
                None => {
                    let made: Option<Uuid> = sqlx::query_scalar(
                        "INSERT INTO nomi.users (project_id, email, email_verified) VALUES ($1, $2, $3) ON CONFLICT (project_id, email) DO NOTHING RETURNING id",
                    )
                    .bind(project)
                    .bind(verified_email)
                    .bind(verified_email.is_some())
                    .fetch_optional(pool)
                    .await?;
                    match made {
                        Some(id) => id,
                        None => sqlx::query_scalar("INSERT INTO nomi.users (project_id) VALUES ($1) RETURNING id").bind(project).fetch_one(pool).await?,
                    }
                }
            }
        }
    };
    add_identity(pool, project, user, provider, &profile.provider_user_id, profile.email.as_deref()).await?;
    sqlx::query("UPDATE nomi.users SET last_sign_in_at = now() WHERE id = $1").bind(user).execute(pool).await?;
    Ok(user)
}

/// Removes one of a user's sign-in methods; never their last.
pub async fn unlink(pool: &PgPool, project: Uuid, user: Uuid, identity: Uuid) -> Result<(), AuthError> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM nomi.identities WHERE project_id = $1 AND user_id = $2").bind(project).bind(user).fetch_one(pool).await?;
    if count <= 1 {
        return Err(AuthError::bad("LAST_IDENTITY", "That's this user's only way to sign in, so it stays."));
    }
    let removed = sqlx::query("DELETE FROM nomi.identities WHERE project_id = $1 AND user_id = $2 AND id = $3").bind(project).bind(user).bind(identity).execute(pool).await?.rows_affected();
    if removed == 0 {
        return Err(AuthError::NotFound("No such sign-in method on this user.".into()));
    }
    Ok(())
}
