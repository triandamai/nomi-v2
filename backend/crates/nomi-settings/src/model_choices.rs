//! Which model a chat runs on when someone picks one in the chat, and which models each person's
//! model selector shows (Manage models).

use std::collections::HashMap;
use std::fmt;

use sqlx::PgPool;
use uuid::Uuid;

/// A model as the selector names it: one of Nomi's models (`admin:<id>`) or a model from one of
/// the person's own connected providers (`conn:<connection id>:<model id>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelKey {
    Admin(Uuid),
    Connection { connection_id: Uuid, model_id: String },
}

/// Longest model id a key may carry (provider ids are far shorter; this only bounds input).
const MAX_MODEL_ID: usize = 300;

impl ModelKey {
    pub fn parse(key: &str) -> Option<Self> {
        if let Some(id) = key.strip_prefix("admin:") {
            return id.parse().ok().map(ModelKey::Admin);
        }
        let rest = key.strip_prefix("conn:")?;
        // The model id itself may contain colons ("vendor/model:variant"): split once.
        let (connection_id, model_id) = rest.split_once(':')?;
        if model_id.is_empty() || model_id.len() > MAX_MODEL_ID {
            return None;
        }
        Some(ModelKey::Connection { connection_id: connection_id.parse().ok()?, model_id: model_id.to_string() })
    }
}

impl fmt::Display for ModelKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelKey::Admin(id) => write!(f, "admin:{id}"),
            ModelKey::Connection { connection_id, model_id } => write!(f, "conn:{connection_id}:{model_id}"),
        }
    }
}

/// The model picked for a chat in the chat itself.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionModel {
    pub key: ModelKey,
    /// The name it was picked under (a connection's models have no stored name of their own).
    pub label: Option<String>,
    pub set_by: Option<Uuid>,
}

/// None until someone picks a model in the chat, or once the picked model or provider is gone.
pub async fn get_session_model(pool: &PgPool, session_id: Uuid) -> Result<Option<SessionModel>, sqlx::Error> {
    let row: Option<(Option<Uuid>, Option<Uuid>, Option<String>, Option<String>, Option<Uuid>)> =
        sqlx::query_as("SELECT model_admin_id, model_connection_id, model_id, model_label, model_set_by FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(pool)
            .await?;
    let Some((admin_id, connection_id, model_id, label, set_by)) = row else { return Ok(None) };
    let key = match (admin_id, connection_id, model_id) {
        (Some(id), _, _) => ModelKey::Admin(id),
        (None, Some(connection_id), Some(model_id)) => ModelKey::Connection { connection_id, model_id },
        _ => return Ok(None),
    };
    Ok(Some(SessionModel { key, label, set_by }))
}

pub async fn set_session_model(pool: &PgPool, session_id: Uuid, key: &ModelKey, label: Option<&str>, set_by: Uuid) -> Result<(), sqlx::Error> {
    let (admin_id, connection_id, model_id) = match key {
        ModelKey::Admin(id) => (Some(*id), None, None),
        ModelKey::Connection { connection_id, model_id } => (None, Some(*connection_id), Some(model_id.as_str())),
    };
    sqlx::query(
        "UPDATE sessions SET model_admin_id = $2, model_connection_id = $3, model_id = $4, model_label = $5, model_set_by = $6 WHERE id = $1",
    )
    .bind(session_id)
    .bind(admin_id)
    .bind(connection_id)
    .bind(model_id)
    .bind(label)
    .bind(set_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// The person's changes from the defaults, by model key.
pub async fn model_visibility(pool: &PgPool, user_id: Uuid) -> Result<HashMap<String, bool>, sqlx::Error> {
    let rows: Vec<(String, bool)> = sqlx::query_as("SELECT model_key, enabled FROM user_model_visibility WHERE user_id = $1").bind(user_id).fetch_all(pool).await?;
    Ok(rows.into_iter().collect())
}

pub async fn set_model_visibility(pool: &PgPool, user_id: Uuid, changes: &[(String, bool)]) -> Result<(), sqlx::Error> {
    let (keys, enabled): (Vec<String>, Vec<bool>) = changes.iter().cloned().unzip();
    sqlx::query(
        "INSERT INTO user_model_visibility (user_id, model_key, enabled) SELECT $1, k, e FROM unnest($2::text[], $3::bool[]) AS c(k, e) \
         ON CONFLICT (user_id, model_key) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = now()",
    )
    .bind(user_id)
    .bind(keys)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}
