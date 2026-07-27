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
    org_id_hint: Option<Uuid>,
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
            let org_id = match org_id_hint {
                Some(id) => id,
                None => {
                    sqlx::query_scalar(
                        "SELECT o.id FROM organizations o \
                         JOIN memberships m ON m.org_id = o.id \
                         WHERE m.user_id = $1 AND o.is_personal = true",
                    )
                    .bind(user_id)
                    .fetch_one(pool)
                    .await?
                }
            };
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

            let inserted_identity_id: Option<Uuid> = sqlx::query_scalar(
                "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, $2, $3) \
                 ON CONFLICT (channel, channel_user_id) DO NOTHING RETURNING id",
            )
            .bind(user_id)
            .bind(channel)
            .bind(sender_channel_user_id)
            .fetch_optional(&mut *tx)
            .await?;

            match inserted_identity_id {
                Some(identity_id) => {
                    tx.commit().await?;
                    (identity_id, user_id, org_id)
                }
                None => {
                    // Lost the race: another concurrent call already created this identity. Roll back
                    // our now-orphaned users/organizations/memberships rows and resolve the winner's
                    // identity, user, and personal org instead.
                    tx.rollback().await?;
                    let (identity_id, winner_user_id): (Uuid, Uuid) = sqlx::query_as(
                        "SELECT id, user_id FROM channel_identities WHERE channel = $1 AND channel_user_id = $2",
                    )
                    .bind(channel)
                    .bind(sender_channel_user_id)
                    .fetch_one(pool)
                    .await?;
                    let winner_org_id: Uuid = sqlx::query_scalar(
                        "SELECT o.id FROM organizations o \
                         JOIN memberships m ON m.org_id = o.id \
                         WHERE m.user_id = $1 AND o.is_personal = true",
                    )
                    .bind(winner_user_id)
                    .fetch_one(pool)
                    .await?;
                    (identity_id, winner_user_id, winner_org_id)
                }
            }
        }
    };

    let inserted_session_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (channel, chat_id) DO NOTHING RETURNING id",
    )
    .bind(org_id)
    .bind(channel)
    .bind(chat_type)
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;

    let session_id = match inserted_session_id {
        Some(id) => id,
        None => {
            sqlx::query_scalar("SELECT id FROM sessions WHERE channel = $1 AND chat_id = $2")
                .bind(channel)
                .bind(chat_id)
                .fetch_one(pool)
                .await?
        }
    };

    Ok(BootstrapResult { user_id, org_id, sender_channel_identity_id, session_id })
}
