use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct MemoryItemResponse {
    pub id: Uuid,
    pub content: String,
    pub weight: f64,
    /// preference, person, routine, goal or fact.
    pub kind: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// How many of Nomi's replies drew on this memory.
    pub uses: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    /// A strong memory nobody has confirmed (or edited) in a long while: worth asking whether
    /// it's still true.
    pub needs_check: bool,
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
    // Replaced, merged and faded memories stay out of the list (and out of recall).
    let rows: Vec<(Uuid, String, f64, String, DateTime<Utc>, DateTime<Utc>, i64, Option<DateTime<Utc>>, bool, String)> = sqlx::query_as(
        "SELECT m.id, m.content, m.weight, m.kind, m.created_at, m.updated_at, \
                (SELECT count(*) FROM message_memory_usage u WHERE u.memory_id = m.id), m.last_used_at, \
                (m.weight >= 2.5 AND COALESCE(m.confirmed_at, m.created_at) < now() - interval '90 days'), \
                m.embedding::text \
         FROM memory_items m WHERE m.user_id = $1 AND m.archived_at IS NULL ORDER BY m.updated_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load memories"))?;

    let memories = rows
        .into_iter()
        .map(|(id, content, weight, kind, created_at, updated_at, uses, last_used_at, needs_check, embedding_text)| MemoryItemResponse {
            id,
            content,
            weight,
            kind,
            created_at,
            updated_at,
            uses,
            last_used_at,
            needs_check,
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

#[derive(Deserialize)]
pub struct EditMemoryRequest {
    pub content: String,
    pub kind: Option<String>,
}

/// Rewrites one of the caller's memories (re-embedded so recall finds it by its new words).
pub async fn edit_my_memory(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(memory_id): Path<Uuid>,
    Json(req): Json<EditMemoryRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    use nomi_agent_core::memory::{edit, EditMemoryError};
    let embedder = crate::bootstrap::build_embedding_provider_from_settings_or_env(&state.pool, &state.settings_key, state.http_client.clone()).await;
    edit(&state.pool, embedder.as_ref(), claims.sub, memory_id, &req.content, req.kind.as_deref()).await.map_err(|e| match e {
        EditMemoryError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
        EditMemoryError::InvalidText | EditMemoryError::InvalidKind => (StatusCode::BAD_REQUEST, e.to_string()),
        EditMemoryError::Embedding(ref detail) => {
            tracing::warn!(error = %detail, "memory edit: embedding failed");
            (StatusCode::BAD_GATEWAY, "couldn't save the new wording right now".to_string())
        }
        EditMemoryError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to update memory".to_string()),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// The caller says a memory is still true.
pub async fn confirm_my_memory(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(memory_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    match nomi_agent_core::memory::confirm(&state.pool, claims.sub, memory_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((StatusCode::NOT_FOUND, "memory not found")),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to confirm memory")),
    }
}
