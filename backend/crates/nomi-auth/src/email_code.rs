//! Six-digit codes sent by email to finish a password sign-in, or to confirm a new account's
//! email. A code lasts 10 minutes and allows 5 tries; a new one can be sent once a minute.
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
}

impl Purpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Purpose::Register => "register",
            Purpose::Login => "login",
        }
    }

    fn parse(value: &str) -> Self {
        if value == "register" {
            Purpose::Register
        } else {
            Purpose::Login
        }
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
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// A code waiting to be entered.
#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub purpose: Purpose,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub resend_at: DateTime<Utc>,
    pub sends_left: i32,
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

/// user_id, purpose, email, expires_at, resend_at, sends_left.
type ChallengeRow = (Uuid, String, String, DateTime<Utc>, DateTime<Utc>, i32);

/// An open challenge (not used, not expired, not locked by wrong tries), or `None`.
pub async fn find(pool: &PgPool, id: Uuid) -> Result<Option<Challenge>, sqlx::Error> {
    let row: Option<ChallengeRow> = sqlx::query_as(
        "SELECT c.user_id, c.purpose, w.email, c.expires_at, \
                c.last_sent_at + make_interval(secs => $2), $3 - c.sends \
         FROM sign_in_challenges c JOIN web_credentials w ON w.user_id = c.user_id \
         WHERE c.id = $1 AND c.used_at IS NULL AND c.expires_at > now() AND c.attempts < $4",
    )
    .bind(id)
    .bind(RESEND_COOLDOWN_SECONDS as f64)
    .bind(MAX_SENDS)
    .bind(MAX_ATTEMPTS)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(user_id, purpose, email, expires_at, resend_at, sends_left)| Challenge {
        id,
        user_id,
        purpose: Purpose::parse(&purpose),
        email,
        expires_at,
        resend_at,
        sends_left,
    }))
}

/// Checks `code`. On success the challenge is used up, the account's email counts as confirmed,
/// and the signed-in user is returned.
pub async fn verify(pool: &PgPool, id: Uuid, code: &str) -> Result<(Uuid, Purpose), CodeError> {
    let mut tx = pool.begin().await?;
    let row: Option<(Uuid, String, String, bool, i32)> = sqlx::query_as(
        "SELECT user_id, purpose, code_hash, (used_at IS NULL AND expires_at > now()), attempts \
         FROM sign_in_challenges WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((user_id, purpose, code_hash, open, attempts)) = row else { return Err(CodeError::Expired) };
    if !open {
        return Err(CodeError::Expired);
    }
    if attempts >= MAX_ATTEMPTS {
        return Err(CodeError::TooManyAttempts);
    }
    let digits: String = code.chars().filter(|c| c.is_ascii_digit()).collect();
    if !same(&hash_code(id, &digits), &code_hash) {
        sqlx::query("UPDATE sign_in_challenges SET attempts = attempts + 1 WHERE id = $1").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        let attempts_left = MAX_ATTEMPTS - attempts - 1;
        return Err(if attempts_left <= 0 { CodeError::TooManyAttempts } else { CodeError::Wrong { attempts_left } });
    }
    sqlx::query("UPDATE sign_in_challenges SET used_at = now() WHERE id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("UPDATE web_credentials SET email_verified_at = now() WHERE user_id = $1 AND email_verified_at IS NULL")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((user_id, Purpose::parse(&purpose)))
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
