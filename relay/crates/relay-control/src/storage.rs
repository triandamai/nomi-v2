//! A project's buckets per environment, as `nomi/backend.json`'s `storage` says: kept in Relay's
//! state for the gateway, which checks every file request against them, and mirrored into the
//! project's `nomi.buckets` so its own SQL can see them.

use relay_core::backend::StorageConfig;
use relay_core::Env;
use uuid::Uuid;

use crate::{state, Relay, RelayError};

/// `backend.json` asks for something Relay can't do as written.
pub const BACKEND_INVALID: &str = "BACKEND_INVALID";

pub(crate) fn invalid(problems: Vec<String>) -> RelayError {
    RelayError::rejected(BACKEND_INVALID, problems.join("; "))
}

/// Replaces `env`'s buckets.
pub async fn save(relay: &Relay, project: Uuid, env: Env, config: &StorageConfig) -> Result<(), RelayError> {
    let problems = config.problems();
    if !problems.is_empty() {
        return Err(invalid(problems));
    }
    state::environment(relay, project, env).await?.ok_or_else(|| RelayError::NotFound("no backend for this project".into()))?;
    let placement = relay.placement(project, env).await?;
    let names: Vec<&str> = config.buckets.iter().map(|b| b.name.as_str()).collect();
    let mut tx = placement.pool.begin().await?;
    sqlx::query("DELETE FROM nomi.buckets WHERE project_id = $1 AND NOT (name = ANY($2))").bind(project).bind(&names).execute(&mut *tx).await?;
    for bucket in &config.buckets {
        sqlx::query("INSERT INTO nomi.buckets (project_id, name, public) VALUES ($1, $2, $3) ON CONFLICT (project_id, name) DO UPDATE SET public = EXCLUDED.public")
            .bind(project)
            .bind(&bucket.name)
            .bind(bucket.public)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    let value = serde_json::to_value(config).map_err(|e| RelayError::Internal(e.to_string()))?;
    sqlx::query("INSERT INTO relay.storage_configs (project_id, env, config) VALUES ($1, $2, $3) ON CONFLICT (project_id, env) DO UPDATE SET config = EXCLUDED.config")
        .bind(project)
        .bind(env.as_str())
        .bind(value)
        .execute(&relay.state)
        .await?;
    Ok(())
}

/// `env`'s buckets; none until `backend.json` names some.
pub async fn load(relay: &Relay, project: Uuid, env: Env) -> Result<StorageConfig, RelayError> {
    let value: Option<serde_json::Value> = sqlx::query_scalar("SELECT config FROM relay.storage_configs WHERE project_id = $1 AND env = $2")
        .bind(project)
        .bind(env.as_str())
        .fetch_optional(&relay.state)
        .await?;
    match value {
        Some(value) => serde_json::from_value(value).map_err(|e| RelayError::Internal(format!("saved buckets can't be read: {e}"))),
        None => Ok(StorageConfig::default()),
    }
}
