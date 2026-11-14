use sqlx::{PgConnection, PgPool};
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

    if !place_in_org(&mut tx, user_id, org_mode).await? {
        return Err(RegistrationError::InvalidInvite);
    }

    tx.commit().await?;
    Ok(user_id)
}

/// A new account from an emailed code (`POST /api/auth/start`): no password yet (the stored hash
/// is of a random secret nobody knows, so password sign-in stays off until they set one), the
/// email already confirmed by the code, and the invite's workspace, or a personal one of their own
/// when there's no invite or it no longer works.
pub async fn create_from_code(conn: &mut PgConnection, email: &str, invite_code: Option<&str>) -> Result<Uuid, RegistrationError> {
    let unusable_password = {
        use rand::RngCore;
        let mut bytes = [0u8; UNUSABLE_SECRET_BYTES];
        rand::thread_rng().fill_bytes(&mut bytes);
        hash_password(&hex::encode(bytes))?
    };
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&mut *conn).await?;
    sqlx::query(
        "INSERT INTO web_credentials (user_id, email, password_hash, password_login, email_verified_at) VALUES ($1, $2, $3, false, now())",
    )
    .bind(user_id)
    .bind(email)
    .bind(&unusable_password)
    .execute(&mut *conn)
    .await?;

    let joined = match invite_code {
        Some(code) => place_in_org(conn, user_id, OrgMode::Join { invite_code: code.to_string() }).await?,
        None => false,
    };
    if !joined {
        let local = email.split('@').next().filter(|l| !l.is_empty()).unwrap_or("My");
        let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ($1, true) RETURNING id")
            .bind(format!("{local}'s space"))
            .fetch_one(&mut *conn)
            .await?;
        sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
            .bind(org_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await?;
    }
    Ok(user_id)
}

/// Bytes of the random secret behind an account's unusable starting password.
const UNUSABLE_SECRET_BYTES: usize = 32;

/// Puts a new user in an organization: one of their own (as owner), or the invite's (with its
/// role, using the invite up). `false` when the invite is unknown, expired or already used.
pub(crate) async fn place_in_org(conn: &mut PgConnection, user_id: Uuid, org_mode: OrgMode) -> Result<bool, sqlx::Error> {
    match org_mode {
        OrgMode::Create { name } => {
            let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ($1, false) RETURNING id")
                .bind(&name)
                .fetch_one(&mut *conn)
                .await?;
            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(org_id)
                .bind(user_id)
                .execute(&mut *conn)
                .await?;
        }
        OrgMode::Join { invite_code } => {
            let invite: Option<(Uuid, String)> =
                sqlx::query_as("SELECT org_id, role FROM org_invites WHERE code = $1 AND used_at IS NULL AND expires_at > now()")
                    .bind(&invite_code)
                    .fetch_optional(&mut *conn)
                    .await?;
            let Some((org_id, role)) = invite else { return Ok(false) };
            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
                .bind(org_id)
                .bind(user_id)
                .bind(&role)
                .execute(&mut *conn)
                .await?;
            sqlx::query("UPDATE org_invites SET used_at = now() WHERE code = $1").bind(&invite_code).execute(&mut *conn).await?;
        }
    }
    Ok(true)
}
