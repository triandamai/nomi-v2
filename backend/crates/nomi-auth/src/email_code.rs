//! Six-digit codes sent by email to finish a password sign-in, to confirm a new account's
//! email, or to sign in (or create an account) with the email alone (`start_for_email`). A code lasts 10 minutes and allows 5 tries; a new one can be sent once a minute.
//! Codes are stored hashed.

use chrono::{DateTime, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub const CODE_TTL_SECONDS: i64 = 10 * 60;
pub const MAX_ATTEMPTS: i32 = 5;
pub const RESEND_COOLDOWN_SECONDS: i64 = 60;
pub const MAX_SENDS: i32 = 5;
/// Most codes started for one account in an hour, so a known password can't flood its inbox.
pub const MAX_CHALLENGES_PER_HOUR: i64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    Register,
    Login,
    /// Confirms a new address for a signed-in account: the code goes to the new address.
    ChangeEmail,
    /// Confirms closing a signed-in account.
    DeleteAccount,
}

impl Purpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Purpose::Register => "register",
            Purpose::Login => "login",
            Purpose::ChangeEmail => "change_email",
            Purpose::DeleteAccount => "delete_account",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "register" => Purpose::Register,
            "change_email" => Purpose::ChangeEmail,
            "delete_account" => Purpose::DeleteAccount,
            _ => Purpose::Login,
        }
    }

    /// Whether a right code signs someone in (the others confirm something a signed-in person asked).
    pub fn signs_in(self) -> bool {
        matches!(self, Purpose::Register | Purpose::Login)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CodeError {
    #[error("wrong code")]
    Wrong { attempts_left: i32 },
    /// Expired, already used, or unknown: sign in again for a new one.
    #[error("this code has expired")]
    Expired,
    #[error("too many wrong codes")]
    TooManyAttempts,
    #[error("wait before asking for another code")]
    Wait { seconds: i64 },
    #[error("too many codes sent")]
    TooManySends,
    /// Making the account for a new address failed.
    #[error("couldn't create the account: {0}")]
    Account(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// A code waiting to be entered.
#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: Uuid,
    /// `None` for an address without an account yet: entering the code makes it.
    pub user_id: Option<Uuid>,
    pub purpose: Purpose,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub resend_at: DateTime<Utc>,
    pub sends_left: i32,
    /// The language the code was asked for in, when it was said (new addresses only).
    pub language: Option<String>,
}

/// Who a right code signed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verified {
    pub user_id: Uuid,
    pub purpose: Purpose,
    /// The code made the account just now.
    pub is_new: bool,
}

fn new_code() -> String {
    format!("{:06}", rand::thread_rng().gen_range(0..1_000_000))
}

fn hash_code(challenge_id: Uuid, code: &str) -> String {
    hex::encode(Sha256::digest(format!("{challenge_id}:{}", code.trim()).as_bytes()))
}

fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Starts a sign-in (or confirmation) for `user_id`: any earlier open code for it stops working.
/// Returns the challenge and the code to email.
pub async fn start(pool: &PgPool, user_id: Uuid, purpose: Purpose) -> Result<(Challenge, String), CodeError> {
    let recent: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sign_in_challenges WHERE user_id = $1 AND created_at > now() - interval '1 hour'")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if recent >= MAX_CHALLENGES_PER_HOUR {
        return Err(CodeError::TooManySends);
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE sign_in_challenges SET expires_at = now() WHERE user_id = $1 AND used_at IS NULL AND expires_at > now()")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    let id = Uuid::new_v4();
    let code = new_code();
    sqlx::query(
        "INSERT INTO sign_in_challenges (id, user_id, purpose, code_hash, expires_at) \
         VALUES ($1, $2, $3, $4, now() + make_interval(secs => $5))",
    )
    .bind(id)
    .bind(user_id)
    .bind(purpose.as_str())
    .bind(hash_code(id, &code))
    .bind(CODE_TTL_SECONDS as f64)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let challenge = find(pool, id).await?.ok_or(CodeError::Expired)?;
    Ok((challenge, code))
}

/// Starts a code for an address with no account yet: entering it makes the account (see
/// `verify`). Any earlier open code for the same address stops working.
pub async fn start_for_email(pool: &PgPool, email: &str, invite_code: Option<&str>, language: Option<&str>) -> Result<(Challenge, String), CodeError> {
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sign_in_challenges WHERE user_id IS NULL AND email = $1 AND created_at > now() - interval '1 hour'",
    )
    .bind(email)
    .fetch_one(pool)
    .await?;
    if recent >= MAX_CHALLENGES_PER_HOUR {
        return Err(CodeError::TooManySends);
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE sign_in_challenges SET expires_at = now() WHERE user_id IS NULL AND email = $1 AND used_at IS NULL AND expires_at > now()")
        .bind(email)
        .execute(&mut *tx)
        .await?;
    let id = Uuid::new_v4();
    let code = new_code();
    sqlx::query(
        "INSERT INTO sign_in_challenges (id, email, invite_code, language, purpose, code_hash, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, now() + make_interval(secs => $7))",
    )
    .bind(id)
    .bind(email)
    .bind(invite_code)
    .bind(language)
    .bind(Purpose::Register.as_str())
    .bind(hash_code(id, &code))
    .bind(CODE_TTL_SECONDS as f64)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let challenge = find(pool, id).await?.ok_or(CodeError::Expired)?;
    Ok((challenge, code))
}

/// Starts a code that confirms something a signed-in person asked (`ChangeEmail`, sent to
/// `email`, the new address; `DeleteAccount`, sent to the account's own). An earlier open code
/// for the same purpose stops working; sign-in codes are left alone.
pub async fn start_for_account(pool: &PgPool, user_id: Uuid, purpose: Purpose, email: Option<&str>) -> Result<(Challenge, String), CodeError> {
    let recent: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sign_in_challenges WHERE user_id = $1 AND created_at > now() - interval '1 hour'")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if recent >= MAX_CHALLENGES_PER_HOUR {
        return Err(CodeError::TooManySends);
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE sign_in_challenges SET expires_at = now() WHERE user_id = $1 AND purpose = $2 AND used_at IS NULL AND expires_at > now()")
        .bind(user_id)
        .bind(purpose.as_str())
        .execute(&mut *tx)
        .await?;
    let id = Uuid::new_v4();
    let code = new_code();
    sqlx::query(
        "INSERT INTO sign_in_challenges (id, user_id, email, purpose, code_hash, expires_at) \
         VALUES ($1, $2, $3, $4, $5, now() + make_interval(secs => $6))",
    )
    .bind(id)
    .bind(user_id)
    .bind(email)
    .bind(purpose.as_str())
    .bind(hash_code(id, &code))
    .bind(CODE_TTL_SECONDS as f64)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let challenge = find(pool, id).await?.ok_or(CodeError::Expired)?;
    Ok((challenge, code))
}

/// Checks `code` for one of `user_id`'s `purpose` codes (see `start_for_account`) and uses it up.
/// Returns the address it went to. Anyone else's code, or one for another purpose, is `Expired`.
pub async fn verify_for_account(pool: &PgPool, id: Uuid, user_id: Uuid, purpose: Purpose, code: &str) -> Result<String, CodeError> {
    let mut tx = pool.begin().await?;
    type Row = (Option<Uuid>, Option<String>, String, String, bool, i32);
    let row: Option<Row> = sqlx::query_as(
        "SELECT c.user_id, COALESCE(c.email, w.email), c.purpose, c.code_hash, (c.used_at IS NULL AND c.expires_at > now()), c.attempts \
         FROM sign_in_challenges c LEFT JOIN web_credentials w ON w.user_id = c.user_id WHERE c.id = $1 FOR UPDATE OF c",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((owner, email, stored_purpose, code_hash, open, attempts)) = row else { return Err(CodeError::Expired) };
    if owner != Some(user_id) || stored_purpose != purpose.as_str() || !open {
        return Err(CodeError::Expired);
    }
    if let Err(e) = check_code(&mut tx, id, code, &code_hash, attempts).await {
        tx.commit().await?;
        return Err(e);
    }
    sqlx::query("UPDATE sign_in_challenges SET used_at = now() WHERE id = $1").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    email.ok_or(CodeError::Expired)
}

/// Whether `code` is right; a wrong one counts a try (the caller commits it).
async fn check_code(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: Uuid, code: &str, code_hash: &str, attempts: i32) -> Result<(), CodeError> {
    if attempts >= MAX_ATTEMPTS {
        return Err(CodeError::TooManyAttempts);
    }
    let digits: String = code.chars().filter(|c| c.is_ascii_digit()).collect();
    if same(&hash_code(id, &digits), code_hash) {
        return Ok(());
    }
    sqlx::query("UPDATE sign_in_challenges SET attempts = attempts + 1 WHERE id = $1").bind(id).execute(&mut **tx).await?;
    let attempts_left = MAX_ATTEMPTS - attempts - 1;
    Err(if attempts_left <= 0 { CodeError::TooManyAttempts } else { CodeError::Wrong { attempts_left } })
}

/// user_id, purpose, email, expires_at, resend_at, sends_left, language.
type ChallengeRow = (Option<Uuid>, String, String, DateTime<Utc>, DateTime<Utc>, i32, Option<String>);

/// An open challenge (not used, not expired, not locked by wrong tries), or `None`.
pub async fn find(pool: &PgPool, id: Uuid) -> Result<Option<Challenge>, sqlx::Error> {
    let row: Option<ChallengeRow> = sqlx::query_as(
        "SELECT c.user_id, c.purpose, COALESCE(c.email, w.email), c.expires_at, \
                c.last_sent_at + make_interval(secs => $2), $3 - c.sends, c.language \
         FROM sign_in_challenges c LEFT JOIN web_credentials w ON w.user_id = c.user_id \
         WHERE c.id = $1 AND c.used_at IS NULL AND c.expires_at > now() AND c.attempts < $4",
    )
    .bind(id)
    .bind(RESEND_COOLDOWN_SECONDS as f64)
    .bind(MAX_SENDS)
    .bind(MAX_ATTEMPTS)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(user_id, purpose, email, expires_at, resend_at, sends_left, language)| Challenge {
        id,
        user_id,
        purpose: Purpose::parse(&purpose),
        email,
        expires_at,
        resend_at,
        sends_left,
        language,
    }))
}

/// Checks `code`. On success the challenge is used up, the account's email counts as confirmed,
/// and the signed-in user is returned. A code for an address without an account makes the account
/// (or signs in to the one made for that address meanwhile).
pub async fn verify(pool: &PgPool, id: Uuid, code: &str) -> Result<Verified, CodeError> {
    let mut tx = pool.begin().await?;
    type Row = (Option<Uuid>, Option<String>, Option<String>, String, String, bool, i32);
    let row: Option<Row> = sqlx::query_as(
        "SELECT user_id, email, invite_code, purpose, code_hash, (used_at IS NULL AND expires_at > now()), attempts \
         FROM sign_in_challenges WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((user_id, email, invite_code, purpose, code_hash, open, attempts)) = row else { return Err(CodeError::Expired) };
    // A code that confirms an account change never signs anyone in.
    if !open || !Purpose::parse(&purpose).signs_in() {
        return Err(CodeError::Expired);
    }
    if let Err(e) = check_code(&mut tx, id, code, &code_hash, attempts).await {
        tx.commit().await?;
        return Err(e);
    }
    sqlx::query("UPDATE sign_in_challenges SET used_at = now() WHERE id = $1").bind(id).execute(&mut *tx).await?;
    let (user_id, is_new) = match user_id {
        Some(user_id) => (user_id, false),
        None => {
            let email = email.ok_or(CodeError::Expired)?;
            let existing: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE lower(email) = lower($1)")
                .bind(&email)
                .fetch_optional(&mut *tx)
                .await?;
            match existing {
                Some(user_id) => (user_id, false),
                None => {
                    let user_id = crate::registration::create_from_code(&mut tx, &email, invite_code.as_deref()).await.map_err(|e| match e {
                        crate::registration::RegistrationError::Db(e) => CodeError::Db(e),
                        other => CodeError::Account(other.to_string()),
                    })?;
                    (user_id, true)
                }
            }
        }
    };
    sqlx::query("UPDATE web_credentials SET email_verified_at = now() WHERE user_id = $1 AND email_verified_at IS NULL")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Verified { user_id, purpose: Purpose::parse(&purpose), is_new })
}

/// A fresh code for an open challenge (the old one stops working, and the tries start over).
pub async fn resend(pool: &PgPool, id: Uuid) -> Result<(Challenge, String), CodeError> {
    let challenge = find(pool, id).await?.ok_or(CodeError::Expired)?;
    let wait = (challenge.resend_at - Utc::now()).num_seconds();
    if wait > 0 {
        return Err(CodeError::Wait { seconds: wait });
    }
    if challenge.sends_left <= 0 {
        return Err(CodeError::TooManySends);
    }
    let code = new_code();
    sqlx::query(
        "UPDATE sign_in_challenges SET code_hash = $2, attempts = 0, sends = sends + 1, last_sent_at = now(), \
             expires_at = now() + make_interval(secs => $3) WHERE id = $1",
    )
    .bind(id)
    .bind(hash_code(id, &code))
    .bind(CODE_TTL_SECONDS as f64)
    .execute(pool)
    .await?;
    let challenge = find(pool, id).await?.ok_or(CodeError::Expired)?;
    Ok((challenge, code))
}

/// `ana@example.com` → `a**@example.com`: enough to recognise, not to harvest.
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => {
            let first: String = local.chars().take(1).collect();
            let hidden = local.chars().count().saturating_sub(1).clamp(1, 6);
            format!("{first}{}@{domain}", "*".repeat(hidden))
        }
        None => "***".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_six_digits() {
        for _ in 0..200 {
            let code = new_code();
            assert_eq!(code.len(), 6);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn a_code_only_matches_its_own_challenge() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        assert!(same(&hash_code(a, "123456"), &hash_code(a, " 123456 ")));
        assert!(!same(&hash_code(a, "123456"), &hash_code(b, "123456")));
        assert!(!same(&hash_code(a, "123456"), &hash_code(a, "123457")));
    }

    #[test]
    fn emails_are_masked() {
        assert_eq!(mask_email("ana@example.com"), "a**@example.com");
        assert_eq!(mask_email("b@x.io"), "b*@x.io");
        assert_eq!(mask_email("averyveryverylongname@x.io"), "a******@x.io");
    }
}
