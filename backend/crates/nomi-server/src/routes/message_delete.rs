//! Deleting a message from a chat, with what it led to.
//!
//! A person's message goes with the crew's replies to it (its thread) and with what the crew
//! made while answering it: transactions logged, reminders set, plans drafted, tasks scheduled
//! (each row remembers the message that started its turn, `source_message_id`). A reply goes on
//! its own, with the plan or reminder its cards show. Files attached to deleted messages go too,
//! and so do recordings the crew read out loud in them.
//! `GET .../delete` lists all of it so the app can ask first; `DELETE` removes it for good.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

#[derive(Serialize, sqlx::FromRow, Debug, PartialEq)]
pub struct RelatedItem {
    pub id: Uuid,
    pub label: String,
    /// Transactions: the amount in cents, and "expense" or "income".
    pub amount_cents: Option<i64>,
    pub kind: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct Related {
    /// The messages that go: this one and, for a person's message, the replies to it.
    pub message_ids: Vec<Uuid>,
    pub transactions: Vec<RelatedItem>,
    pub reminders: Vec<RelatedItem>,
    pub plans: Vec<RelatedItem>,
    pub tasks: Vec<RelatedItem>,
    pub files: Vec<RelatedItem>,
}

/// The ids in cards of `kind` among these messages' content blocks, under `field`.
fn block_ids(blocks: &[Option<serde_json::Value>], kind: &str, field: &str) -> Vec<Uuid> {
    blocks
        .iter()
        .filter_map(|b| b.as_ref()?.as_array())
        .flatten()
        .filter(|block| block.get("kind").and_then(|k| k.as_str()) == Some(kind))
        .filter_map(|block| block.get(field)?.as_str()?.parse().ok())
        .collect()
}

/// What deleting `message_id` would remove. 404 for a message that isn't in this chat; 403 for
/// someone else's message (a teammate's, in a shared project chat).
pub async fn related_to(pool: &sqlx::PgPool, user_id: Uuid, session_id: Uuid, message_id: Uuid) -> Result<Related, ApiError> {
    let message: Option<(Option<Uuid>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT ci.user_id, m.created_at FROM messages m \
         LEFT JOIN channel_identities ci ON ci.id = m.sender_channel_identity_id \
         WHERE m.id = $1 AND m.session_id = $2",
    )
    .bind(message_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to look up message"))?;
    let (author, created_at) = message.ok_or((StatusCode::NOT_FOUND, "message not found"))?;
    let from_person = author.is_some();
    if author.is_some_and(|author| author != user_id) {
        return Err((StatusCode::FORBIDDEN, "only its author can delete this message"));
    }

    // A person's message takes the replies after it, up to their next message. A reminder going
    // off in between is not a reply to it.
    let message_ids: Vec<Uuid> = if from_person {
        sqlx::query_scalar(
            "SELECT id FROM messages WHERE session_id = $1 AND (id = $2 OR (sender_channel_identity_id IS NULL AND created_at > $3 \
               AND created_at < COALESCE((SELECT min(created_at) FROM messages WHERE session_id = $1 \
                   AND sender_channel_identity_id IS NOT NULL AND created_at > $3), 'infinity') \
               AND NOT COALESCE(content_blocks @> '[{\"kind\": \"reminder\"}]', false))) \
             ORDER BY created_at",
        )
        .bind(session_id)
        .bind(message_id)
        .bind(created_at)
        .fetch_all(pool)
        .await
        .map_err(internal("failed to look up message"))?
    } else {
        vec![message_id]
    };

    let blocks: Vec<Option<serde_json::Value>> = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE id = ANY($1)")
        .bind(&message_ids)
        .fetch_all(pool)
        .await
        .map_err(internal("failed to look up message"))?;
    let plan_ids = block_ids(&blocks, "plan", "plan_id");
    let reminder_ids = block_ids(&blocks, "reminder", "reminder_id");
    // Only a person's message started a turn, so only it can be the source of what the crew made.
    let source = from_person.then_some(message_id);

    let transactions: Vec<RelatedItem> = sqlx::query_as(
        "SELECT id, description AS label, amount_cents, kind FROM money_transactions \
         WHERE user_id = $1 AND source_message_id = $2 ORDER BY occurred_at",
    )
    .bind(user_id)
    .bind(source)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to look up related content"))?;
    let reminders: Vec<RelatedItem> = sqlx::query_as(
        "SELECT id, title AS label, NULL::bigint AS amount_cents, NULL::text AS kind FROM reminders \
         WHERE user_id = $1 AND (source_message_id = $2 OR id = ANY($3)) ORDER BY due_at",
    )
    .bind(user_id)
    .bind(source)
    .bind(&reminder_ids)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to look up related content"))?;
    let plans: Vec<RelatedItem> = sqlx::query_as(
        "SELECT id, title || ' (v' || version || ')' AS label, NULL::bigint AS amount_cents, NULL::text AS kind FROM agent_plans \
         WHERE session_id = $1 AND (source_message_id = $2 OR id = ANY($3)) ORDER BY created_at",
    )
    .bind(session_id)
    .bind(source)
    .bind(&plan_ids)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to look up related content"))?;
    let tasks: Vec<RelatedItem> = sqlx::query_as(
        "SELECT id, label, NULL::bigint AS amount_cents, NULL::text AS kind FROM scheduled_jobs \
         WHERE user_id = $1 AND source_message_id = $2 AND status <> 'cancelled' ORDER BY run_at",
    )
    .bind(user_id)
    .bind(source)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to look up related content"))?;
    let files: Vec<RelatedItem> = sqlx::query_as(
        "SELECT id, name AS label, NULL::bigint AS amount_cents, NULL::text AS kind FROM attachments \
         WHERE message_id = ANY($1) ORDER BY created_at",
    )
    .bind(&message_ids)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to look up related content"))?;

    Ok(Related { message_ids, transactions, reminders, plans, tasks, files })
}

/// `GET /api/sessions/:id/messages/:message_id/delete`: what deleting it would remove.
pub async fn preview_delete(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Related>, ApiError> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    Ok(Json(related_to(&state.pool, claims.sub, session_id, message_id).await?))
}

/// `DELETE /api/sessions/:id/messages/:message_id`: deletes it and everything `preview_delete`
/// listed, in one go.
pub async fn delete_message(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Related>, ApiError> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let related = related_to(&state.pool, claims.sub, session_id, message_id).await?;
    let mut tx = state.pool.begin().await.map_err(internal("failed to delete message"))?;
    let blobs = remove(&mut tx, &related, None).await?;
    tx.commit().await.map_err(internal("failed to delete message"))?;
    drop_blobs(blobs).await;

    Ok(Json(related))
}

/// Storage keys to drop once the rows that pointed at them are gone.
pub(crate) struct Blobs(Vec<String>);

/// Removes everything `related` lists, except `keep`: a person's message whose turn runs again
/// keeps itself and the files attached to it, and loses the replies and what the turn made.
pub(crate) async fn remove(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, related: &Related, keep: Option<Uuid>) -> Result<Blobs, ApiError> {
    let ids = |items: &[RelatedItem]| items.iter().map(|i| i.id).collect::<Vec<Uuid>>();
    let message_ids: Vec<Uuid> = related.message_ids.iter().copied().filter(|id| Some(*id) != keep).collect();
    let file_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM attachments WHERE id = ANY($1) AND message_id IS DISTINCT FROM $2")
        .bind(ids(&related.files))
        .bind(keep)
        .fetch_all(&mut **tx)
        .await
        .map_err(internal("failed to delete message"))?;

    // Recordings the crew read out loud in these messages (the speak tool's `/voice/...` files).
    let blocks: Vec<Option<serde_json::Value>> = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE id = ANY($1)")
        .bind(&message_ids)
        .fetch_all(&mut **tx)
        .await
        .map_err(internal("failed to delete message"))?;
    let mut keys: Vec<String> = blocks
        .iter()
        .filter_map(|b| b.as_ref()?.as_array())
        .flatten()
        .filter(|block| block.get("kind").and_then(|k| k.as_str()) == Some("voice"))
        .filter_map(|block| block.get("url")?.as_str()?.strip_prefix("/voice/").map(|rest| format!("voice/{rest}")))
        .collect();
    let file_keys: Vec<(String, Option<String>)> = sqlx::query_as("SELECT storage_key, preview_key FROM attachments WHERE id = ANY($1)")
        .bind(&file_ids)
        .fetch_all(&mut **tx)
        .await
        .map_err(internal("failed to delete message"))?;
    keys.extend(file_keys.into_iter().flat_map(|(key, preview)| std::iter::once(key).chain(preview)));

    for (sql, list) in [
        ("DELETE FROM money_transactions WHERE id = ANY($1)", ids(&related.transactions)),
        ("DELETE FROM reminders WHERE id = ANY($1)", ids(&related.reminders)),
        ("DELETE FROM agent_plans WHERE id = ANY($1)", ids(&related.plans)),
        ("DELETE FROM scheduled_jobs WHERE id = ANY($1)", ids(&related.tasks)),
        ("DELETE FROM attachments WHERE id = ANY($1)", file_ids),
        ("DELETE FROM approval_resumes WHERE message_id = ANY($1)", message_ids.clone()),
        ("DELETE FROM messages WHERE id = ANY($1)", message_ids),
    ] {
        if list.is_empty() {
            continue;
        }
        sqlx::query(sql).bind(&list).execute(&mut **tx).await.map_err(internal("failed to delete message"))?;
    }
    Ok(Blobs(keys))
}

/// Drops the files `remove` freed, after its transaction committed. Best effort: a file left
/// behind is only storage, and nothing points at it any more.
pub(crate) async fn drop_blobs(Blobs(keys): Blobs) {
    let store = nomi_storage::blob::attachment_store();
    for key in keys {
        let _ = store.delete(&key).await;
    }
}

