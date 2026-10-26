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
    pub role: String,
    /// The project the key belongs to.
    pub r#ref: Uuid,
    pub env: String,
    pub iat: u64,
    pub exp: u64,
}

pub fn new_signing_secret() -> String {
    let mut bytes = [0u8; SECRET_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
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
    let claims = Claims { role: role.as_str().to_string(), r#ref: project, env: env.as_str().to_string(), iat: issued, exp: issued + API_KEY_LIFETIME_SECONDS };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes())).expect("HS256 signing can't fail")
}

pub fn verify(secret: &str, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256)).map(|data| data.claims)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::Env;
    use uuid::Uuid;

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
}
