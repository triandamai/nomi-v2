use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};

use super::claims::Claims;

/// Implemented by whatever axum state type wires up JWT auth (today, `AppState` in the
/// server crate) — lets `AuthClaims` work as an extractor for any state without nomi-auth
/// depending on the server crate that defines it.
pub trait HasJwtSecret {
    fn jwt_secret(&self) -> &str;
}

pub struct AuthClaims(pub Claims);

#[axum::async_trait]
impl<S: HasJwtSecret + Send + Sync> FromRequestParts<S> for AuthClaims {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "malformed authorization header"))?;

        let claims = Claims::decode(token, state.jwt_secret())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

        Ok(AuthClaims(claims))
    }
}
