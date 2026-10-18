//! Providers connected once (llm_provider_connections), for the whole instance or for one person.
//! Keys and headers arrive and leave encrypted; callers encrypt with [`crate::crypto`].

use sqlx::PgPool;
use uuid::Uuid;

/// Whose connections: the instance's (shared, managed by admins) or one person's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Admin,
    User(Uuid),
}

impl Scope {
    fn user_id(self) -> Option<Uuid> {
        match self {
            Scope::Admin => None,
            Scope::User(id) => Some(id),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderConnectionRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub provider_id: String,
    pub label: String,
    pub kind: String,
    pub auth_type: String,
    pub api_key_encrypted: Option<Vec<u8>>,
    pub base_url: Option<String>,
    pub headers_encrypted: Option<Vec<u8>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

const COLUMNS: &str = "id, user_id, provider_id, label, kind, auth_type, api_key_encrypted, base_url, headers_encrypted, created_at, updated_at";

pub struct NewConnection<'a> {
    pub provider_id: &'a str,
    pub label: &'a str,
    pub kind: &'a str,
    pub auth_type: &'a str,
    pub api_key_encrypted: Option<Vec<u8>>,
    pub base_url: Option<&'a str>,
    pub headers_encrypted: Option<Vec<u8>>,
    pub updated_by: Uuid,
}

/// What to change; anything left `None` stays as it is.
#[derive(Default)]
pub struct ConnectionUpdate<'a> {
    pub label: Option<&'a str>,
    pub api_key_encrypted: Option<Vec<u8>>,
    pub base_url: Option<&'a str>,
    pub headers_encrypted: Option<Vec<u8>>,
    pub updated_by: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateConnectionError {
    /// The provider is already connected in this scope (or a custom endpoint has that name).
    #[error("already connected")]
    AlreadyConnected,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteConnectionError {
    #[error("not found")]
    NotFound,
    /// Admin models still come from it: the labels of those models.
    #[error("in use")]
    InUse(Vec<String>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

pub async fn list_connections(pool: &PgPool, scope: Scope) -> Result<Vec<ProviderConnectionRow>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM llm_provider_connections WHERE user_id IS NOT DISTINCT FROM $1 ORDER BY created_at, label"))
        .bind(scope.user_id())
        .fetch_all(pool)
        .await
}

/// The connection, only when it belongs to `scope`.
pub async fn get_connection(pool: &PgPool, scope: Scope, id: Uuid) -> Result<Option<ProviderConnectionRow>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM llm_provider_connections WHERE id = $1 AND user_id IS NOT DISTINCT FROM $2"))
        .bind(id)
        .bind(scope.user_id())
        .fetch_optional(pool)
        .await
}

pub async fn create_connection(pool: &PgPool, scope: Scope, new: NewConnection<'_>) -> Result<ProviderConnectionRow, CreateConnectionError> {
    sqlx::query_as(&format!(
        "INSERT INTO llm_provider_connections (user_id, provider_id, label, kind, auth_type, api_key_encrypted, base_url, headers_encrypted, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {COLUMNS}"
    ))
    .bind(scope.user_id())
    .bind(new.provider_id)
    .bind(new.label)
    .bind(new.kind)
    .bind(new.auth_type)
    .bind(new.api_key_encrypted)
    .bind(new.base_url)
    .bind(new.headers_encrypted)
    .bind(new.updated_by)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => CreateConnectionError::AlreadyConnected,
        _ => CreateConnectionError::Database(e),
    })
}

/// The updated connection, or None when it isn't in `scope`.
pub async fn update_connection(pool: &PgPool, scope: Scope, id: Uuid, update: ConnectionUpdate<'_>) -> Result<Option<ProviderConnectionRow>, sqlx::Error> {
    sqlx::query_as(&format!(
        "UPDATE llm_provider_connections SET \
            label = COALESCE($3, label), \
            api_key_encrypted = COALESCE($4, api_key_encrypted), \
            base_url = COALESCE($5, base_url), \
            headers_encrypted = COALESCE($6, headers_encrypted), \
            updated_by = $7, updated_at = now() \
         WHERE id = $1 AND user_id IS NOT DISTINCT FROM $2 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(scope.user_id())
    .bind(update.label)
    .bind(update.api_key_encrypted)
    .bind(update.base_url)
    .bind(update.headers_encrypted)
    .bind(update.updated_by)
    .fetch_optional(pool)
    .await
}

/// Labels of the admin models that come from this connection.
pub async fn models_using(pool: &PgPool, id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT label FROM admin_llm_models WHERE connection_id = $1 ORDER BY label").bind(id).fetch_all(pool).await
}

/// Removes the connection. A personal one also takes the person's saved own key with it, so their
/// chats go back to Nomi's models rather than quietly to the key saved the old way.
pub async fn delete_connection(pool: &PgPool, scope: Scope, id: Uuid) -> Result<(), DeleteConnectionError> {
    if get_connection(pool, scope, id).await?.is_none() {
        return Err(DeleteConnectionError::NotFound);
    }
    let models = models_using(pool, id).await?;
    if !models.is_empty() {
        return Err(DeleteConnectionError::InUse(models));
    }
    let mut tx = pool.begin().await?;
    if let Scope::User(user_id) = scope {
        sqlx::query(
            "UPDATE user_llm_selections SET custom_label = NULL, custom_provider = NULL, custom_model_id = NULL, \
                custom_api_key_encrypted = NULL, custom_base_url = NULL, custom_connection_id = NULL, updated_at = now() \
             WHERE user_id = $1 AND custom_connection_id = $2",
        )
        .bind(user_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM llm_provider_connections WHERE id = $1").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
