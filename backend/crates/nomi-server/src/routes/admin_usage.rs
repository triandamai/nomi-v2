//! Admin → Usage: tokens per model (chat and embedding) across everyone, one month at a time.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use nomi_auth::extractor::AuthClaims;
use nomi_usage::admin::AdminUsage;

use super::settings::require_system_config_permission;
use crate::app::AppState;

#[derive(Deserialize)]
pub struct UsageQuery {
    /// `YYYY-MM` (UTC); this month when absent.
    pub month: Option<String>,
}

/// `GET /api/admin/usage?month=YYYY-MM`
pub async fn get_usage(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<UsageQuery>,
) -> Result<Json<AdminUsage>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    let current = crate::bootstrap::current_embedding_model(&state.pool).await;
    let report = nomi_usage::admin::report(&state.pool, query.month.as_deref(), current.as_ref().map(|(p, m)| (p.as_str(), m.as_str())))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to read usage");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to read usage")
        })?;
    Ok(Json(report))
}
