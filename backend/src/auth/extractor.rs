use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};

use super::claims::Claims;
use crate::app::AppState;

pub struct AuthClaims(pub Claims);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthClaims {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "malformed authorization header"))?;

        let claims = Claims::decode(token, &state.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

        Ok(AuthClaims(claims))
    }
}
