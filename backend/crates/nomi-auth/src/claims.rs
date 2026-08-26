use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    pub sub: Uuid,
    pub active_org_id: Uuid,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ClaimsError {
    #[error("token encoding failed: {0}")]
    Encode(#[from] jsonwebtoken::errors::Error),
}

impl Claims {
    pub fn new(sub: Uuid, active_org_id: Uuid, permissions: Vec<String>, ttl_seconds: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Claims {
            sub,
            active_org_id,
            permissions,
            exp: now + ttl_seconds,
            iat: now,
        }
    }

    pub fn encode(&self, secret: &str) -> Result<String, ClaimsError> {
        Ok(encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?)
    }

    pub fn decode(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }

    pub fn has_permission(&self, scope: &str, resource: &str, action: &str) -> bool {
        let prefix = format!("nomi:{scope}:{resource}:[");
        self.permissions.iter().any(|p| {
            p.strip_prefix(prefix.as_str())
                .and_then(|rest| rest.strip_suffix(']'))
                .map(|actions| actions.split(',').any(|a| a == action))
                .unwrap_or(false)
        })
    }
}

pub fn permission_string(scope: &str, resource: &str, actions: &[&str]) -> String {
    format!("nomi:{scope}:{resource}:[{}]", actions.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    const SECRET: &str = "test-secret-do-not-use-in-prod";

    fn sample_claims(permissions: Vec<String>) -> Claims {
        Claims::new(Uuid::new_v4(), Uuid::new_v4(), permissions, 1800)
    }

    #[test]
    fn encode_decode_roundtrip_preserves_claims() {
        let claims = sample_claims(vec![permission_string("admin", "user", &["view", "manage"])]);
        let token = claims.encode(SECRET).unwrap();
        let decoded = Claims::decode(&token, SECRET).unwrap();
        assert_eq!(decoded, claims);
    }

    #[test]
    fn decode_rejects_wrong_secret() {
        let claims = sample_claims(vec![]);
        let token = claims.encode(SECRET).unwrap();
        let result = Claims::decode(&token, "a-different-secret");
        assert!(result.is_err());
    }

    #[test]
    fn decode_rejects_expired_token() {
        let mut claims = sample_claims(vec![]);
        claims.exp = chrono::Utc::now().timestamp() - 3600; // expired one hour ago (well past jsonwebtoken's default 60s leeway)
        let token = claims.encode(SECRET).unwrap();
        let result = Claims::decode(&token, SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn has_permission_matches_admin_scope() {
        let claims = sample_claims(vec![permission_string("admin", "user", &["view", "manage"])]);
        assert!(claims.has_permission("admin", "user", "view"));
        assert!(claims.has_permission("admin", "user", "manage"));
    }

    #[test]
    fn has_permission_matches_org_scope() {
        let org_id = "11111111-1111-1111-1111-111111111111";
        let claims = sample_claims(vec![permission_string(org_id, "member", &["view", "manage"])]);
        assert!(claims.has_permission(org_id, "member", "manage"));
    }

    #[test]
    fn has_permission_rejects_wrong_org() {
        let org_a = "11111111-1111-1111-1111-111111111111";
        let org_b = "22222222-2222-2222-2222-222222222222";
        let claims = sample_claims(vec![permission_string(org_a, "member", &["view", "manage"])]);
        assert!(!claims.has_permission(org_b, "member", "manage"));
    }

    #[test]
    fn has_permission_rejects_wrong_action() {
        let org_id = "11111111-1111-1111-1111-111111111111";
        let claims = sample_claims(vec![permission_string(org_id, "member", &["view"])]);
        assert!(!claims.has_permission(org_id, "member", "manage"));
    }

    #[test]
    fn permission_string_formats_as_expected() {
        assert_eq!(
            permission_string("admin", "user", &["view", "manage"]),
            "nomi:admin:user:[view,manage]"
        );
    }
}
