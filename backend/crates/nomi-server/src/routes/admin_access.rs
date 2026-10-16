//! Who may use which part of the admin console: every admin route asks for an area and a level
//! (view, write or manage), which comes from the person's grants (see `nomi_auth::catalog`).

use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use nomi_auth::catalog::{ADMIN_RESOURCES, LEVELS};
use nomi_auth::claims::Claims;
use nomi_auth::extractor::AuthClaims;

/// Ok when they have `level` (or higher) on the admin area.
pub(crate) fn require_admin(claims: &Claims, resource: &str, level: &str) -> Result<(), (StatusCode, &'static str)> {
    if claims.has_permission("admin", resource, level) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "you don't have access to this part of the admin console"))
    }
}

/// [`require_admin`] for routes whose errors carry an owned message.
pub(crate) fn require_admin_owned(claims: &Claims, resource: &str, level: &str) -> Result<(), (StatusCode, String)> {
    require_admin(claims, resource, level).map_err(|(s, m)| (s, m.to_string()))
}

#[derive(Serialize)]
pub struct Catalog {
    /// Every admin area, in console order.
    pub resources: &'static [&'static str],
    /// Lowest to highest; a level includes the ones below it.
    pub levels: &'static [&'static str],
}

/// `GET /api/admin/permissions/catalog`: the areas and levels a role can be built from.
pub async fn catalog(AuthClaims(claims): AuthClaims) -> Result<Json<Catalog>, (StatusCode, &'static str)> {
    require_admin(&claims, "user", "view")?;
    Ok(Json(Catalog { resources: &ADMIN_RESOURCES, levels: &LEVELS }))
}
