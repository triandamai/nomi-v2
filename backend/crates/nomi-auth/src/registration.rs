use sqlx::PgPool;
use uuid::Uuid;

use super::password::{hash_password, PasswordError};

pub enum OrgMode {
    Create { name: String },
    Join { invite_code: String },
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("email already registered")]
    EmailTaken,
    #[error("invite code not found, expired, or already used")]
    InvalidInvite,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Password(#[from] PasswordError),
}

pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    org_mode: OrgMode,
) -> Result<Uuid, RegistrationError> {
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;
    if existing.is_some() {
        return Err(RegistrationError::EmailTaken);
    }

    let password_hash = hash_password(password)?;

    let mut tx = pool.begin().await?;

    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(email)
        .bind(&password_hash)
        .execute(&mut *tx)
        .await?;

    match org_mode {
        OrgMode::Create { name } => {
            let org_id: Uuid = sqlx::query_scalar(
                "INSERT INTO organizations (name, is_personal) VALUES ($1, false) RETURNING id",
            )
            .bind(&name)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(org_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
        OrgMode::Join { invite_code } => {
            let invite: Option<(Uuid, String)> = sqlx::query_as(
                "SELECT org_id, role FROM org_invites WHERE code = $1 AND used_at IS NULL AND expires_at > now()",
            )
            .bind(&invite_code)
            .fetch_optional(&mut *tx)
            .await?;

            let (org_id, role) = invite.ok_or(RegistrationError::InvalidInvite)?;

            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
                .bind(org_id)
                .bind(user_id)
                .bind(&role)
                .execute(&mut *tx)
                .await?;

            sqlx::query("UPDATE org_invites SET used_at = now() WHERE code = $1")
                .bind(&invite_code)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(user_id)
}
