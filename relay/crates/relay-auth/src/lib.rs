//! Sign-in for an app's users, in the app's own database (`nomi.users`, `nomi.identities`,
//! `nomi.sessions`, `nomi.codes`): email codes, sessions with rotating refresh tokens, the
//! signed-in user, and user management with the service key.

pub mod codes;
pub mod config;
pub mod mail;
pub mod sessions;
pub mod users;

use relay_core::Env;
use sqlx::PgPool;
use uuid::Uuid;

/// The environment of one project a request is for: its database, and the key its tokens are
/// signed with.
pub struct Scope<'a> {
    pub pool: &'a PgPool,
    pub project: Uuid,
    pub env: Env,
    pub secret: &'a str,
}

/// What went wrong, in the words the app gets back (`{ message, code }`).
#[derive(Debug)]
pub enum AuthError {
    BadRequest { code: &'static str, message: String },
    Unauthorized { code: &'static str, message: String },
    NotFound(String),
    TooMany { message: String },
    Unavailable { code: &'static str, message: String },
    Internal(String),
}

impl AuthError {
    pub fn bad(code: &'static str, message: impl Into<String>) -> Self {
        AuthError::BadRequest { code, message: message.into() }
    }

    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        AuthError::Unauthorized { code, message: message.into() }
    }

    pub fn status(&self) -> u16 {
        match self {
            AuthError::BadRequest { .. } => 400,
            AuthError::Unauthorized { .. } => 401,
            AuthError::NotFound(_) => 404,
            AuthError::TooMany { .. } => 429,
            AuthError::Unavailable { .. } => 503,
            AuthError::Internal(_) => 500,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            AuthError::BadRequest { code, .. } | AuthError::Unauthorized { code, .. } | AuthError::Unavailable { code, .. } => code,
            AuthError::NotFound(_) => "NOT_FOUND",
            AuthError::TooMany { .. } => "TOO_MANY_REQUESTS",
            AuthError::Internal(_) => "RELAY_INTERNAL",
        }
    }

    /// What the app is told; an internal failure's details stay in the log.
    pub fn message(&self) -> String {
        match self {
            AuthError::BadRequest { message, .. } | AuthError::Unauthorized { message, .. } | AuthError::Unavailable { message, .. } => message.clone(),
            AuthError::NotFound(message) | AuthError::TooMany { message } => message.clone(),
            AuthError::Internal(_) => "Relay couldn't do that right now.".to_string(),
        }
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        AuthError::Internal(e.to_string())
    }
}

/// `ana@example.test` from ` Ana@Example.test `, or a refusal when it isn't an address.
pub(crate) fn normalize_email(email: &str) -> Result<String, AuthError> {
    let email = email.trim().to_lowercase();
    let shaped = email.split_once('@').is_some_and(|(name, domain)| !name.is_empty() && domain.contains('.') && !domain.starts_with('.') && !email.contains(char::is_whitespace));
    if shaped {
        Ok(email)
    } else {
        Err(AuthError::bad("INVALID_EMAIL", "That isn't an email address."))
    }
}

/// Hex SHA-256 of `parts` joined by `:`.
pub(crate) fn digest(parts: &[&str]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(parts.join(":").as_bytes()))
}
