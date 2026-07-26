use sqlx::PgPool;
use uuid::Uuid;

use super::types::TurnError;

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapResult {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub sender_channel_identity_id: Uuid,
    pub session_id: Uuid,
}

pub async fn bootstrap_identity_and_session(
    pool: &PgPool,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
) -> Result<BootstrapResult, TurnError> {
    let existing: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, user_id FROM channel_identities WHERE channel = $1 AND channel_user_id = $2",
    )
    .bind(channel)
    .bind(sender_channel_user_id)
    .fetch_optional(pool)
    .await?;

    let (sender_channel_identity_id, user_id, org_id) = match existing {
        Some((identity_id, user_id)) => {
            let org_id: Uuid = sqlx::query_scalar(
                "SELECT o.id FROM organizations o \
                 JOIN memberships m ON m.org_id = o.id \
                 WHERE m.user_id = $1 AND o.is_personal = true",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await?;
            (identity_id, user_id, org_id)
        }
        None => {
            let mut tx = pool.begin().await?;

            let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
                .fetch_one(&mut *tx)
                .await?;

            let org_id: Uuid = sqlx::query_scalar(
                "INSERT INTO organizations (name, is_personal) VALUES ('Personal', true) RETURNING id",
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(org_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            let identity_id: Uuid = sqlx::query_scalar(
                "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(user_id)
            .bind(channel)
            .bind(sender_channel_user_id)
            .fetch_one(&mut *tx)
            .await?;

            tx.commit().await?;
            (identity_id, user_id, org_id)
        }
    };

    let existing_session: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM sessions WHERE channel = $1 AND chat_id = $2")
            .bind(channel)
            .bind(chat_id)
            .fetch_optional(pool)
            .await?;

    let session_id = match existing_session {
        Some(id) => id,
        None => {
            sqlx::query_scalar(
                "INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind(org_id)
            .bind(channel)
            .bind(chat_type)
            .bind(chat_id)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(BootstrapResult { user_id, org_id, sender_channel_identity_id, session_id })
}
