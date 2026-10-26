//! Email sign-in codes: six digits, sent through the project's sender, good for ten minutes and
//! five tries.

use chrono::{Duration, Utc};
use rand::Rng;
use relay_core::settings::Smtp;

use crate::mail::{Email, Outbox};
use crate::users::{self, User};
use crate::{digest, normalize_email, AuthError, Scope};

pub const CODE_DIGITS: usize = 6;
const CODE_RANGE: u32 = 1_000_000;
pub const CODE_MINUTES: i64 = 10;
pub const MAX_ATTEMPTS: i32 = 5;
const LIMIT_WINDOW_MINUTES: i64 = 60;

/// How many codes a sender may send in an hour.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub per_email: i64,
    pub per_project: i64,
}

impl Limits {
    /// Nomi's shared sender: strict, to keep its reputation clean for every project.
    pub const SHARED: Limits = Limits { per_email: 5, per_project: 100 };
    /// A project's own SMTP server.
    pub const OWN: Limits = Limits { per_email: 5, per_project: 1000 };
}

fn hashed(scope: &Scope<'_>, email: &str, code: &str) -> String {
    digest(&["code", &scope.project.to_string(), email, code])
}

fn new_code() -> String {
    format!("{:0width$}", rand::thread_rng().gen_range(0..CODE_RANGE), width = CODE_DIGITS)
}

/// Sends a code to `email` through `smtp`. An unknown email with `create_user` off answers the
/// same as a sent code, and sends nothing, so nobody learns who has an account.
pub async fn send(scope: &Scope<'_>, outbox: &dyn Outbox, smtp: &Smtp, limits: Limits, app_name: &str, email: &str, create_user: bool) -> Result<(), AuthError> {
    let email = normalize_email(email)?;
    if !create_user && !users::exists_by_email(scope.pool, scope.project, &email).await? {
        return Ok(());
    }
    let since = Utc::now() - Duration::minutes(LIMIT_WINDOW_MINUTES);
    let (for_email, for_project): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE email = $2), count(*) FROM nomi.codes WHERE project_id = $1 AND created_at > $3",
    )
    .bind(scope.project)
    .bind(&email)
    .bind(since)
    .fetch_one(scope.pool)
    .await?;
    if for_email >= limits.per_email || for_project >= limits.per_project {
        return Err(AuthError::TooMany { message: "Too many codes for now: try again in a while.".into() });
    }
    let code = new_code();
    sqlx::query("INSERT INTO nomi.codes (project_id, email, code_hash, create_user, expires_at) VALUES ($1, $2, $3, $4, $5)")
        .bind(scope.project)
        .bind(&email)
        .bind(hashed(scope, &email, &code))
        .bind(create_user)
        .bind(Utc::now() + Duration::minutes(CODE_MINUTES))
        .execute(scope.pool)
        .await?;
    let message = Email {
        to: email.clone(),
        subject: format!("Your {app_name} sign-in code: {code}"),
        text: format!(
            "Here's your code to sign in to {app_name}:\n\n{code}\n\nIt works for {CODE_MINUTES} minutes. If you didn't ask for it, you can ignore this email."
        ),
    };
    outbox.send(smtp, message).await.map_err(|e| AuthError::Unavailable { code: "EMAIL_NOT_SENT", message: format!("The code couldn't be sent: {e}") })
}

/// Checks the latest code sent to `email`; on a match, signs the person in as their user (made
/// now when the code allowed it).
pub async fn verify(scope: &Scope<'_>, email: &str, code: &str) -> Result<User, AuthError> {
    let email = normalize_email(email)?;
    let wrong = || AuthError::bad("INVALID_CODE", "That code isn't right, or it has expired: ask for a new one.");
    let latest: Option<(uuid::Uuid, String, bool, i32)> = sqlx::query_as(
        "SELECT id, code_hash, create_user, attempts FROM nomi.codes WHERE project_id = $1 AND email = $2 \
         AND used_at IS NULL AND expires_at > now() ORDER BY created_at DESC LIMIT 1",
    )
    .bind(scope.project)
    .bind(&email)
    .fetch_optional(scope.pool)
    .await?;
    let Some((id, code_hash, create_user, attempts)) = latest else { return Err(wrong()) };
    if attempts >= MAX_ATTEMPTS {
        return Err(wrong());
    }
    if hashed(scope, &email, code.trim()) != code_hash {
        sqlx::query("UPDATE nomi.codes SET attempts = attempts + 1 WHERE id = $1").bind(id).execute(scope.pool).await?;
        return Err(wrong());
    }
    let used = sqlx::query("UPDATE nomi.codes SET used_at = now() WHERE id = $1 AND used_at IS NULL").bind(id).execute(scope.pool).await?.rows_affected();
    if used == 0 {
        return Err(wrong());
    }
    if !create_user && !users::exists_by_email(scope.pool, scope.project, &email).await? {
        return Err(wrong());
    }
    users::signed_in_by_email(scope.pool, scope.project, &email).await
}
