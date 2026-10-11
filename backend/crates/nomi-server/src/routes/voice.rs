//! Recordings the crew made with `speak`, served to anyone signed in (a shared project chat's
//! teammates play them too). File names are random, and Nomi makes them.

use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

/// `GET /api/voice/:user_id/:file`.
pub async fn get_recording(AuthClaims(_claims): AuthClaims, Path((user_id, file)): Path<(Uuid, String)>) -> Result<Response, (StatusCode, String)> {
    let stem = file.strip_suffix(".mp3").ok_or((StatusCode::NOT_FOUND, "not found".to_string()))?;
    stem.parse::<Uuid>().map_err(|_| (StatusCode::NOT_FOUND, "not found".to_string()))?;
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&nomi_agent_core::voice::voice_key(user_id, &file))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to read a recording");
            (StatusCode::BAD_GATEWAY, "couldn't read the recording".to_string())
        })?
        .ok_or((StatusCode::NOT_FOUND, "not found".to_string()))?;
    Ok((
        [
            (header::CONTENT_TYPE, "audio/mpeg"),
            (header::CACHE_CONTROL, "private, max-age=31536000, immutable"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}
