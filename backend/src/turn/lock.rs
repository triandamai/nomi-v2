use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use crate::agent_core::TurnError;

pub async fn acquire_session_lock(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<PoolConnection<Postgres>, TurnError> {
    let mut conn = pool.acquire().await?;
    sqlx::query("SELECT pg_advisory_lock(hashtext($1))")
        .bind(session_id.to_string())
        .execute(&mut *conn)
        .await?;
    Ok(conn)
}

pub async fn release_session_lock(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<(), TurnError> {
    sqlx::query("SELECT pg_advisory_unlock(hashtext($1))")
        .bind(session_id.to_string())
        .execute(&mut **conn)
        .await?;
    Ok(())
}

pub async fn insert_inbound_message(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    text: &str,
) -> Result<Uuid, TurnError> {
    let mut tx = conn.begin().await?;
    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(message_id)
}
