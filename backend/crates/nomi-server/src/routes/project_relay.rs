//! A project's Nomi Relay backend, as its preview reaches it.

use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_relay_client::RelayConfig;

use crate::app::AppState;
use crate::relay_ops::{preview_access, PreviewAccess};
use crate::routes::deploy::ApiError;

/// `GET /api/projects/:id/relay/preview`: the dev backend's address and anon key (and, for the
/// project's admins, its service key), which the preview gets as `PUBLIC_NOMI_URL` and friends.
pub async fn preview(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<PreviewAccess>, ApiError> {
    let config = RelayConfig::from_env();
    preview_access(&state.pool, &state.http_client, config.as_ref(), project_id, claims.sub).await.map(Json)
}
