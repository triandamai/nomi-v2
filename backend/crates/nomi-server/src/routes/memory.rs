use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct MemoryItemResponse {
    pub id: Uuid,
    pub content: String,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// The raw embedding vector — the frontend reduces this to 2D itself (PCA) to plot a
    /// similarity map; the backend has no reason to do that projection since every consumer
    /// so far is a single per-user view.
    pub embedding: Vec<f32>,
}

#[derive(Serialize)]
pub struct MemoryListResponse {
    pub memories: Vec<MemoryItemResponse>,
}

/// pgvector has no direct sqlx decoder in this workspace (no `pgvector` crate dependency) — the
/// query casts the column to `::text` (Postgres's own `[0.1,0.2,...]` rendering) and this parses
/// that back into floats, avoiding a new dependency for a single read-only listing endpoint.
fn parse_vector_literal(text: &str) -> Vec<f32> {
    text.trim_start_matches('[').trim_end_matches(']').split(',').filter_map(|s| s.trim().parse::<f32>().ok()).collect()
}

pub async fn list_my_memories(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<MemoryListResponse>, (StatusCode, &'static str)> {
    let rows: Vec<(Uuid, String, f64, DateTime<Utc>, DateTime<Utc>, String)> = sqlx::query_as(
        "SELECT id, content, weight, created_at, updated_at, embedding::text FROM memory_items \
         WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load memories"))?;

    let memories = rows
        .into_iter()
        .map(|(id, content, weight, created_at, updated_at, embedding_text)| MemoryItemResponse {
            id,
            content,
            weight,
            created_at,
            updated_at,
            embedding: parse_vector_literal(&embedding_text),
        })
        .collect();

    Ok(Json(MemoryListResponse { memories }))
}
