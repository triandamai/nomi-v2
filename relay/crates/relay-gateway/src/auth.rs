//! Who's calling: the `apikey` header must be this project's key for this environment (anon or
//! service); `Authorization: Bearer` may carry a signed-in user's token or repeat the key. The
//! answer becomes the database role PostgREST switches to, in a short token only PostgREST
//! trusts.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use relay_core::keys::{verify_at, AUTHENTICATED};
use relay_core::names::{role, RoleKind};
use relay_core::Env;
use serde::Serialize;
use uuid::Uuid;

/// The internal token PostgREST sees lives this long: one request.
pub const INTERNAL_TOKEN_SECONDS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Anon,
    Authenticated,
    Service,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Anon => "anon",
            Kind::Authenticated => AUTHENTICATED,
            Kind::Service => "service",
        }
    }

    /// The service key runs as the project's table owner: it bypasses the project's own
    /// row-level security and nothing else (other projects' `nomi` rows stay closed).
    fn database_role(self) -> RoleKind {
        match self {
            Kind::Anon => RoleKind::Anon,
            Kind::Authenticated => RoleKind::Authenticated,
            Kind::Service => RoleKind::Owner,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Caller {
    pub kind: Kind,
    pub sub: Option<Uuid>,
    pub email: Option<String>,
    /// The session a signed-in user's token belongs to.
    pub sid: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    MissingKey,
    BadKey,
    BadToken,
}

impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Refusal::MissingKey => "No apikey: send the project's anon key in the apikey header.",
            Refusal::BadKey => "That apikey isn't this project's, or it's for the other environment.",
            Refusal::BadToken => "The access token isn't valid here, or it has expired: sign in again.",
        }
    }
}

fn belongs(claims: &relay_core::keys::Claims, project: Uuid, env: Env) -> bool {
    claims.r#ref == project && claims.env == env.as_str()
}

/// Resolves the caller from the `apikey` header and the bearer token, at `now`.
pub fn resolve(secret: &str, project: Uuid, env: Env, apikey: Option<&str>, bearer: Option<&str>, now: u64) -> Result<Caller, Refusal> {
    let apikey = apikey.filter(|k| !k.is_empty()).ok_or(Refusal::MissingKey)?;
    let key = verify_at(secret, apikey, now).filter(|c| belongs(c, project, env) && c.sub.is_none()).ok_or(Refusal::BadKey)?;
    let key_kind = if key.role == "service" { Kind::Service } else { Kind::Anon };
    let Some(bearer) = bearer.filter(|b| !b.is_empty() && *b != apikey) else {
        return Ok(Caller { kind: key_kind, sub: None, email: None, sid: None });
    };
    let token = verify_at(secret, bearer, now).filter(|c| belongs(c, project, env)).ok_or(Refusal::BadToken)?;
    match (token.role.as_str(), token.sub) {
        (AUTHENTICATED, Some(sub)) => Ok(Caller { kind: Kind::Authenticated, sub: Some(sub), email: token.email, sid: token.sid }),
        ("service", None) => Ok(Caller { kind: Kind::Service, sub: None, email: None, sid: None }),
        ("anon", None) => Ok(Caller { kind: key_kind, sub: None, email: None, sid: None }),
        _ => Err(Refusal::BadToken),
    }
}

#[derive(Serialize)]
struct InternalClaims {
    /// The database role PostgREST switches to (its `jwt-role-claim-key` is `.db_role`).
    db_role: String,
    /// What `nomi.role()` answers: anon, authenticated or service.
    role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    r#ref: Uuid,
    iat: u64,
    exp: u64,
}

/// The database role a request from `caller` runs as.
pub fn database_role(project: Uuid, caller: &Caller) -> String {
    role(project, caller.kind.database_role())
}

/// What `nomi.claims()` answers for `caller` (`request.jwt.claims`), as PostgREST would set it.
pub fn request_claims(project: Uuid, caller: &Caller) -> serde_json::Value {
    serde_json::json!({ "role": caller.kind.as_str(), "sub": caller.sub, "email": caller.email, "ref": project })
}

/// The token the gateway sends PostgREST, signed with the secret only Relay and PostgREST share.
pub fn internal_token(postgrest_secret: &str, project: Uuid, caller: &Caller, now: u64) -> String {
    let claims = InternalClaims {
        db_role: role(project, caller.kind.database_role()),
        role: caller.kind.as_str(),
        sub: caller.sub,
        email: caller.email.clone(),
        r#ref: project,
        iat: now,
        exp: now + INTERNAL_TOKEN_SECONDS,
    };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(postgrest_secret.as_bytes())).expect("HS256 signing can't fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use relay_core::keys::{api_key, user_token, ApiRole};
    use relay_core::Env;
    use uuid::Uuid;

    const SECRET: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const NOW: u64 = 1_800_000_000;

    #[test]
    fn the_anon_and_service_keys_resolve_to_their_roles() {
        let project = Uuid::new_v4();
        let anon = api_key(SECRET, project, Env::Dev, ApiRole::Anon, NOW);
        let caller = resolve(SECRET, project, Env::Dev, Some(&anon), None, NOW).unwrap();
        assert_eq!(caller.kind, Kind::Anon);
        assert_eq!(caller.sub, None);
        let service = api_key(SECRET, project, Env::Dev, ApiRole::Service, NOW);
        assert_eq!(resolve(SECRET, project, Env::Dev, Some(&anon), Some(&service), NOW).unwrap().kind, Kind::Service);
    }

    #[test]
    fn a_users_token_resolves_to_them() {
        let project = Uuid::new_v4();
        let user = Uuid::new_v4();
        let anon = api_key(SECRET, project, Env::Production, ApiRole::Anon, NOW);
        let token = user_token(SECRET, project, Env::Production, user, Uuid::new_v4(), Some("a@b.test"), NOW, 900);
        let caller = resolve(SECRET, project, Env::Production, Some(&anon), Some(&token), NOW + 10).unwrap();
        assert_eq!(caller.kind, Kind::Authenticated);
        assert_eq!(caller.sub, Some(user));
        assert_eq!(caller.email.as_deref(), Some("a@b.test"));
        assert!(resolve(SECRET, project, Env::Production, Some(&anon), Some(&token), NOW + 901).is_err(), "expired");
    }

    #[test]
    fn anything_not_signed_for_this_project_and_environment_is_refused() {
        let project = Uuid::new_v4();
        let anon = api_key(SECRET, project, Env::Dev, ApiRole::Anon, NOW);
        assert_eq!(resolve(SECRET, project, Env::Dev, None, None, NOW).unwrap_err(), Refusal::MissingKey);
        assert_eq!(resolve("bbbb", project, Env::Dev, Some(&anon), None, NOW).unwrap_err(), Refusal::BadKey);
        assert_eq!(resolve(SECRET, project, Env::Production, Some(&anon), None, NOW).unwrap_err(), Refusal::BadKey);
        assert_eq!(resolve(SECRET, Uuid::new_v4(), Env::Dev, Some(&anon), None, NOW).unwrap_err(), Refusal::BadKey);
        assert_eq!(resolve(SECRET, project, Env::Dev, Some(&anon), Some("not-a-token"), NOW).unwrap_err(), Refusal::BadToken);
    }

    #[test]
    fn postgrest_gets_a_short_token_naming_the_database_role() {
        let project = Uuid::new_v4();
        let user = Uuid::new_v4();
        let caller = Caller { kind: Kind::Authenticated, sub: Some(user), email: None, sid: None };
        let token = internal_token("pgrst-secret", project, &caller, NOW);
        let claims: serde_json::Value = jsonwebtoken::decode::<serde_json::Value>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(b"pgrst-secret"),
            &{
                let mut v = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
                v.validate_exp = false;
                v
            },
        )
        .unwrap()
        .claims;
        assert_eq!(claims["db_role"], relay_core::names::role(project, relay_core::names::RoleKind::Authenticated));
        assert_eq!(claims["role"], "authenticated");
        assert_eq!(claims["sub"], user.to_string());
        assert_eq!(claims["ref"], project.to_string());
        assert!(claims["exp"].as_u64().unwrap() <= NOW + INTERNAL_TOKEN_SECONDS);
        let service = Caller { kind: Kind::Service, sub: None, email: None, sid: None };
        let token = internal_token("pgrst-secret", project, &service, NOW);
        let mut insecure = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        insecure.insecure_disable_signature_validation();
        insecure.validate_exp = false;
        let claims = jsonwebtoken::decode::<serde_json::Value>(&token, &jsonwebtoken::DecodingKey::from_secret(b"x"), &insecure).unwrap().claims;
        assert_eq!(claims["db_role"], relay_core::names::role(project, relay_core::names::RoleKind::Owner));
    }
}
