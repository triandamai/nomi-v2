//! A project's API keys: JWTs signed with its environment's own secret, saying which project,
//! which environment and which role. The anon key is public (it's in the app's browser code);
//! the service key is for the app's server only. A token from one project means nothing to
//! another, because each environment signs with its own secret.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::names::Env;

/// Bytes of randomness in a signing secret (hex-encoded, so twice as many characters).
const SECRET_BYTES: usize = 32;
pub const MIN_SECRET_HEX_LEN: usize = SECRET_BYTES * 2;
/// API keys are long-lived: they're rotated by the control plane, not by expiry. Ten years.
const API_KEY_LIFETIME_SECONDS: u64 = 10 * 365 * 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiRole {
    Anon,
    Service,
}

impl ApiRole {
    fn as_str(self) -> &'static str {
        match self {
            ApiRole::Anon => "anon",
            ApiRole::Service => "service",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// `anon` or `service` for a key, `authenticated` for a signed-in user's token.
    pub role: String,
    /// The project the key belongs to.
    pub r#ref: Uuid,
    pub env: String,
    pub iat: u64,
    pub exp: u64,
    /// The signed-in user, on a user's token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The session a user's token belongs to, so signing out can end it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sid: Option<Uuid>,
}

/// How long a signed-in user's access token works; the refresh token gets the next one.
pub const ACCESS_TOKEN_SECONDS: u64 = 15 * 60;

/// The role on a signed-in user's access token.
pub const AUTHENTICATED: &str = "authenticated";

pub fn new_signing_secret() -> String {
    let mut bytes = [0u8; SECRET_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// The password of a project role's own database login, derived from Relay's key so every Relay
/// instance gives the same one and nothing needs storing.
pub fn role_password(key: &[u8; 32], role: &str) -> String {
    use hmac::{Hmac, Mac};
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key).expect("any key length works");
    mac.update(b"relay-role-login:");
    mac.update(role.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

/// The current time, for a new environment's `issued_at`.
pub fn issued_now() -> u64 {
    now()
}

/// A key for `role`, issued at `issued_at`: the same inputs always give the same key.
pub fn api_key(secret: &str, project: Uuid, env: Env, role: ApiRole, issued_at: u64) -> String {
    let issued = issued_at;
    let claims = Claims {
        role: role.as_str().to_string(),
        r#ref: project,
        env: env.as_str().to_string(),
        iat: issued,
        exp: issued + API_KEY_LIFETIME_SECONDS,
        sub: None,
        email: None,
        sid: None,
    };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes())).expect("HS256 signing can't fail")
}

/// A signed-in user's access token for one of their sessions, valid for `lifetime_seconds` from
/// `issued_at`.
#[allow(clippy::too_many_arguments)]
pub fn user_token(secret: &str, project: Uuid, env: Env, user: Uuid, session: Uuid, email: Option<&str>, issued_at: u64, lifetime_seconds: u64) -> String {
    let claims = Claims {
        role: AUTHENTICATED.to_string(),
        r#ref: project,
        env: env.as_str().to_string(),
        iat: issued_at,
        exp: issued_at + lifetime_seconds,
        sub: Some(user),
        email: email.map(str::to_string),
        sid: Some(session),
    };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes())).expect("HS256 signing can't fail")
}

/// Checks the signature only; the caller decides what time it is (see `verify_at`).
pub fn verify(secret: &str, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256)).map(|data| data.claims)
}

/// Checks the signature and that the token hasn't expired at `now`.
pub fn verify_at(secret: &str, token: &str, now: u64) -> Option<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    let claims = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation).ok()?.claims;
    (claims.exp > now).then_some(claims)
}

/// What a signed file URL's token names: one file of one bucket, in one project environment.
/// It has no `role`, so it never passes as an API key or a user's token.
#[derive(Debug, Serialize, Deserialize)]
struct FileClaims {
    r#use: String,
    r#ref: Uuid,
    env: String,
    bucket: String,
    path: String,
    exp: u64,
}

const FILE_USE: &str = "file";

/// The token of a signed URL for `bucket`/`path`, good until `now + seconds`.
pub fn file_token(secret: &str, project: Uuid, env: Env, bucket: &str, path: &str, now: u64, seconds: u64) -> String {
    let claims = FileClaims { r#use: FILE_USE.into(), r#ref: project, env: env.as_str().into(), bucket: bucket.into(), path: path.into(), exp: now + seconds };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes())).expect("HS256 signing can't fail")
}

/// Whether `token` opens exactly this file now.
pub fn file_token_opens(secret: &str, token: &str, project: Uuid, env: Env, bucket: &str, path: &str, now: u64) -> bool {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    let Ok(data) = decode::<FileClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation) else { return false };
    let c = data.claims;
    c.r#use == FILE_USE && c.r#ref == project && c.env == env.as_str() && c.bucket == bucket && c.path == path && c.exp > now
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::Env;
    use uuid::Uuid;

    #[test]
    fn a_file_token_opens_only_its_own_file_until_it_expires() {
        let secret = new_signing_secret();
        let project = Uuid::new_v4();
        let token = file_token(&secret, project, Env::Dev, "docs", "a/b.pdf", 1000, 60);
        assert!(file_token_opens(&secret, &token, project, Env::Dev, "docs", "a/b.pdf", 1059));
        assert!(!file_token_opens(&secret, &token, project, Env::Dev, "docs", "a/b.pdf", 1060), "expired");
        assert!(!file_token_opens(&secret, &token, project, Env::Dev, "docs", "a/c.pdf", 1001));
        assert!(!file_token_opens(&secret, &token, project, Env::Dev, "other", "a/b.pdf", 1001));
        assert!(!file_token_opens(&secret, &token, project, Env::Production, "docs", "a/b.pdf", 1001));
        assert!(!file_token_opens(&secret, &token, Uuid::new_v4(), Env::Dev, "docs", "a/b.pdf", 1001));
        assert!(!file_token_opens(&new_signing_secret(), &token, project, Env::Dev, "docs", "a/b.pdf", 1001));
        assert!(verify_at(&secret, &token, 1001).is_none(), "never an API key");
        let key = api_key(&secret, project, Env::Dev, ApiRole::Anon, 1000);
        assert!(!file_token_opens(&secret, &key, project, Env::Dev, "docs", "a/b.pdf", 1001), "an API key never opens a file");
    }

    #[test]
    fn an_api_key_says_whose_it_is_and_only_its_secret_opens_it() {
        let secret = new_signing_secret();
        let other = new_signing_secret();
        assert_ne!(secret, other);
        assert!(secret.len() >= MIN_SECRET_HEX_LEN);
        let project = Uuid::new_v4();
        let key = api_key(&secret, project, Env::Dev, ApiRole::Anon, issued_now());
        assert_eq!(api_key(&secret, project, Env::Dev, ApiRole::Anon, 100), api_key(&secret, project, Env::Dev, ApiRole::Anon, 100));
        let claims = verify(&secret, &key).unwrap();
        assert_eq!(claims.role, "anon");
        assert_eq!(claims.r#ref, project);
        assert_eq!(claims.env, "dev");
        assert!(verify(&other, &key).is_err());
        assert_eq!(verify(&secret, &api_key(&secret, project, Env::Production, ApiRole::Service, issued_now())).unwrap().role, "service");
    }

    #[test]
    fn a_users_token_names_them_and_their_session_until_it_expires() {
        let secret = new_signing_secret();
        let (project, user, session) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let token = user_token(&secret, project, Env::Dev, user, session, Some("a@b.test"), 1_000, ACCESS_TOKEN_SECONDS);
        let claims = verify_at(&secret, &token, 1_000 + ACCESS_TOKEN_SECONDS - 1).unwrap();
        assert_eq!((claims.sub, claims.sid, claims.role.as_str()), (Some(user), Some(session), AUTHENTICATED));
        assert!(verify_at(&secret, &token, 1_000 + ACCESS_TOKEN_SECONDS).is_none());
    }
}
