use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminLlmModel {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<String>,
    pub is_default: bool,
}

pub async fn list_admin_llm_models(pool: &PgPool) -> Result<Vec<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "SELECT id, label, provider, model_id, api_key_encrypted, base_url, is_default \
         FROM admin_llm_models ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_admin_llm_model(pool: &PgPool, id: Uuid) -> Result<Option<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "SELECT id, label, provider, model_id, api_key_encrypted, base_url, is_default \
         FROM admin_llm_models WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_default_admin_llm_model(pool: &PgPool) -> Result<Option<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "SELECT id, label, provider, model_id, api_key_encrypted, base_url, is_default \
         FROM admin_llm_models WHERE is_default = true",
    )
    .fetch_optional(pool)
    .await
}

pub struct NewAdminLlmModel<'a> {
    pub label: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<&'a str>,
    pub updated_by: Uuid,
}

/// The first model ever created becomes the default automatically (there must always be one
/// once at least one exists); later ones are created non-default until an admin explicitly
/// promotes them via `set_default_admin_llm_model`.
pub async fn create_admin_llm_model(pool: &PgPool, input: NewAdminLlmModel<'_>) -> Result<AdminLlmModel, sqlx::Error> {
    let existing_count: i64 = sqlx::query_scalar("SELECT count(*) FROM admin_llm_models").fetch_one(pool).await?;
    let is_default = existing_count == 0;

    sqlx::query_as::<_, AdminLlmModel>(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, base_url, is_default, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, label, provider, model_id, api_key_encrypted, base_url, is_default",
    )
    .bind(input.label)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .bind(is_default)
    .bind(input.updated_by)
    .fetch_one(pool)
    .await
}

pub struct UpdateAdminLlmModel<'a> {
    pub label: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Option<Vec<u8>>,
    pub base_url: Option<&'a str>,
    pub updated_by: Uuid,
}

pub async fn update_admin_llm_model(
    pool: &PgPool,
    id: Uuid,
    input: UpdateAdminLlmModel<'_>,
) -> Result<Option<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "UPDATE admin_llm_models SET \
         label = $2, provider = $3, model_id = $4, \
         api_key_encrypted = COALESCE($5, api_key_encrypted), \
         base_url = $6, updated_by = $7, updated_at = now() \
         WHERE id = $1 \
         RETURNING id, label, provider, model_id, api_key_encrypted, base_url, is_default",
    )
    .bind(id)
    .bind(input.label)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .bind(input.updated_by)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteAdminLlmModelError {
    #[error("model not found")]
    NotFound,
    #[error("cannot delete the current default; set a different default first")]
    IsDefault,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// There's no separate "last remaining model" guard: the first model ever created is always
/// the default (see `create_admin_llm_model`), and defaults can never be deleted here — so the
/// `IsDefault` check alone already guarantees at least one model always survives. A dedicated
/// row-count check would be dead code, since a lone remaining row is always the default by
/// construction.
pub async fn delete_admin_llm_model(pool: &PgPool, id: Uuid) -> Result<(), DeleteAdminLlmModelError> {
    let model = get_admin_llm_model(pool, id).await?.ok_or(DeleteAdminLlmModelError::NotFound)?;
    if model.is_default {
        return Err(DeleteAdminLlmModelError::IsDefault);
    }
    sqlx::query("DELETE FROM admin_llm_models WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SetDefaultAdminLlmModelError {
    #[error("model not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub async fn set_default_admin_llm_model(pool: &PgPool, id: Uuid) -> Result<(), SetDefaultAdminLlmModelError> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query("UPDATE admin_llm_models SET is_default = true WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if updated.rows_affected() == 0 {
        return Err(SetDefaultAdminLlmModelError::NotFound);
    }
    sqlx::query("UPDATE admin_llm_models SET is_default = false WHERE id != $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserLlmSelectionRow {
    pub admin_model_id: Option<Uuid>,
    pub custom_label: Option<String>,
    pub custom_provider: Option<String>,
    pub custom_model_id: Option<String>,
    pub custom_api_key_encrypted: Option<Vec<u8>>,
    pub custom_base_url: Option<String>,
}

pub async fn get_user_llm_selection(pool: &PgPool, user_id: Uuid) -> Result<Option<UserLlmSelectionRow>, sqlx::Error> {
    sqlx::query_as::<_, UserLlmSelectionRow>(
        "SELECT admin_model_id, custom_label, custom_provider, custom_model_id, \
         custom_api_key_encrypted, custom_base_url \
         FROM user_llm_selections WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn set_user_llm_selection_admin(pool: &PgPool, user_id: Uuid, admin_model_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, admin_model_id) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET \
         admin_model_id = $2, custom_label = NULL, custom_provider = NULL, custom_model_id = NULL, \
         custom_api_key_encrypted = NULL, custom_base_url = NULL, updated_at = now()",
    )
    .bind(user_id)
    .bind(admin_model_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub struct CustomLlmSelection<'a> {
    pub label: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<&'a str>,
}

pub async fn set_user_llm_selection_custom(
    pool: &PgPool,
    user_id: Uuid,
    input: CustomLlmSelection<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, custom_label, custom_provider, custom_model_id, \
         custom_api_key_encrypted, custom_base_url) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET \
         admin_model_id = NULL, custom_label = $2, custom_provider = $3, custom_model_id = $4, \
         custom_api_key_encrypted = $5, custom_base_url = $6, updated_at = now()",
    )
    .bind(user_id)
    .bind(input.label)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .execute(pool)
    .await?;
    Ok(())
}
