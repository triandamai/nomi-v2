use sqlx::PgPool;
use uuid::Uuid;

pub async fn ensure_web_channel_identity(pool: &PgPool, user_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM channel_identities WHERE channel = 'web' AND channel_user_id = $1",
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let inserted: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', $2) \
         ON CONFLICT (channel, channel_user_id) DO NOTHING RETURNING id",
    )
    .bind(user_id)
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    match inserted {
        Some(id) => Ok(id),
        None => {
            // Lost a race against a concurrent call for the same user (e.g. two browser tabs
            // opened at once) — re-fetch the winner. Same pattern already proven in
            // bootstrap_identity_and_session's own concurrent-first-contact handling.
            sqlx::query_scalar(
                "SELECT id FROM channel_identities WHERE channel = 'web' AND channel_user_id = $1",
            )
            .bind(user_id.to_string())
            .fetch_one(pool)
            .await
        }
    }
}
