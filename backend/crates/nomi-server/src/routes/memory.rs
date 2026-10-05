use axum::extract::{Path, State};
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
    /// How many of Nomi's replies drew on this memory.
    pub uses: i64,
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
    let rows: Vec<(Uuid, String, f64, DateTime<Utc>, DateTime<Utc>, i64, String)> = sqlx::query_as(
        "SELECT m.id, m.content, m.weight, m.created_at, m.updated_at, \
                (SELECT count(*) FROM message_memory_usage u WHERE u.memory_id = m.id), m.embedding::text \
         FROM memory_items m WHERE m.user_id = $1 ORDER BY m.updated_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load memories"))?;

    let memories = rows
        .into_iter()
        .map(|(id, content, weight, created_at, updated_at, uses, embedding_text)| MemoryItemResponse {
            id,
            content,
            weight,
            created_at,
            updated_at,
            uses,
            embedding: parse_vector_literal(&embedding_text),
        })
        .collect();

    Ok(Json(MemoryListResponse { memories }))
}

/// Forgets one of the caller's memories. Another user's memory id reads as not found.
pub async fn delete_my_memory(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(memory_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let mut tx = state.pool.begin().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete memory"))?;
    let owned: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_items WHERE id = $1 AND user_id = $2)")
        .bind(memory_id)
        .bind(claims.sub)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete memory"))?;
    if !owned {
        return Err((StatusCode::NOT_FOUND, "memory not found"));
    }
    sqlx::query("DELETE FROM message_memory_usage WHERE memory_id = $1")
        .bind(memory_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete memory"))?;
    sqlx::query("DELETE FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete memory"))?;
    tx.commit().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete memory"))?;
    Ok(StatusCode::NO_CONTENT)
}
