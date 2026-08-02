pub mod crypto;

use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderSettingsRow {
    pub setting_type: String,
    pub provider: String,
    pub model_id: String,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<String>,
}

pub async fn get_settings(pool: &PgPool, setting_type: &str) -> Result<Option<ProviderSettingsRow>, sqlx::Error> {
    sqlx::query_as::<_, ProviderSettingsRow>(
        "SELECT setting_type, provider, model_id, api_key_encrypted, base_url \
         FROM provider_settings WHERE setting_type = $1",
    )
    .bind(setting_type)
    .fetch_optional(pool)
    .await
}

pub struct UpsertInput<'a> {
    pub setting_type: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<&'a str>,
    pub updated_by: Uuid,
}

pub async fn upsert_settings(pool: &PgPool, input: UpsertInput<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO provider_settings (setting_type, provider, model_id, api_key_encrypted, base_url, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (setting_type) DO UPDATE SET \
         provider = $2, model_id = $3, api_key_encrypted = $4, base_url = $5, updated_by = $6, updated_at = now()",
    )
    .bind(input.setting_type)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .bind(input.updated_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    let tail: String = key.chars().rev().take(4).collect::<Vec<char>>().into_iter().rev().collect();
    format!("...{tail}")
}
