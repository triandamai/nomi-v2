//! Which Relay a project's calls go to. A project made after Relay moved onto Shipyard has its own
//! Relay for each environment (a row in `project_relays`); one made before it, or on a Nomi
//! that hasn't set that up, uses the shared Relay saved in the admin. Everything that talks to
//! Relay for a project asks `config_for`, so nothing else needs to know which it is.

use nomi_relay_client::RelayConfig;
use nomi_settings::crypto::{decrypt, encrypt};
use sqlx::PgPool;
use uuid::Uuid;

pub const CREATING: &str = "creating";
pub const READY: &str = "ready";
pub const SLEEPING: &str = "sleeping";
pub const WAKING: &str = "waking";
pub const FAILED: &str = "failed";

/// What Nomi has recorded for one environment's Relay.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    pub url: Option<String>,
    pub version: String,
    pub state: String,
    pub step: Option<String>,
    pub error: Option<String>,
}

/// Why a project's own Relay can't be called right now, in words for the person.
fn unavailable(stack: &Stack) -> String {
    match stack.state.as_str() {
        CREATING => "This project's backend is still being set up.".to_string(),
        SLEEPING | WAKING => "This project's backend is waking up.".to_string(),
        _ => {
            let step = stack.step.as_deref().map(|s| format!(" at \"{s}\"")).unwrap_or_default();
            match stack.error.as_deref() {
                Some(error) => format!("This project's backend couldn't be set up{step}: {error}"),
                None => format!("This project's backend couldn't be set up{step}."),
            }
        }
    }
}

type Row = (Option<String>, Vec<u8>, String, String, Option<String>, Option<String>);

async fn row(pool: &PgPool, project_id: Uuid, env: &str) -> Result<Option<Row>, String> {
    sqlx::query_as("SELECT url, control_key, version, state, step, error FROM project_relays WHERE project_id = $1 AND env = $2")
        .bind(project_id)
        .bind(env)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("couldn't read the project's backend: {e}"))
}

/// What's recorded for the project's `env` Relay, if it has its own.
pub async fn stack(pool: &PgPool, project_id: Uuid, env: &str) -> Result<Option<Stack>, String> {
    Ok(row(pool, project_id, env).await?.map(|(url, _, version, state, step, error)| Stack { url, version, state, step, error }))
}

/// Where to call the project's `env` Relay: its own when it has one and it's ready, `fallback`
/// (the shared Relay) when it has none, and the reason in words when its own isn't ready.
pub async fn config_for(
    pool: &PgPool,
    settings_key: Option<&[u8; 32]>,
    fallback: Option<RelayConfig>,
    project_id: Uuid,
    env: &str,
) -> Result<Option<RelayConfig>, String> {
    let Some((url, sealed, version, state, step, error)) = row(pool, project_id, env).await? else {
        return Ok(fallback);
    };
    let stack = Stack { url, version, state, step, error };
    if stack.state != READY {
        return Err(unavailable(&stack));
    }
    let key = settings_key.ok_or_else(|| "this Nomi can't read its projects' Relay keys: SETTINGS_ENCRYPTION_KEY isn't set".to_string())?;
    let control_key = decrypt(key, &sealed).map_err(|_| "the project's Relay key can't be decrypted with this settings key".to_string())?;
    RelayConfig::from_values(stack.url.as_deref(), Some(&control_key)).map(Some).ok_or_else(|| "this project's backend has no address yet".to_string())
}

/// `config_for` with the shared Relay as saved in the admin (or the environment) as the fallback,
/// and a not-ready backend as a 503 with its reason, for route handlers.
pub async fn for_request(pool: &PgPool, settings_key: Option<&[u8; 32]>, project_id: Uuid, env: &str) -> Result<Option<RelayConfig>, (axum::http::StatusCode, String)> {
    let shared = crate::relay_connection::load(pool, settings_key).await;
    config_for(pool, settings_key, shared, project_id, env).await.map_err(|message| (axum::http::StatusCode::SERVICE_UNAVAILABLE, message))
}

/// Records a Relay about to be made (or made again): its key sealed, state `creating`. An
/// existing row keeps its key, so a retry doesn't change what Shipyard was given.
pub async fn begin(pool: &PgPool, settings_key: &[u8; 32], project_id: Uuid, env: &str, control_key: &str, version: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO project_relays (project_id, env, control_key, version, state, step, error) VALUES ($1, $2, $3, $4, $5, NULL, NULL) \
         ON CONFLICT (project_id, env) DO UPDATE SET version = EXCLUDED.version, state = EXCLUDED.state, step = NULL, error = NULL, updated_at = now()",
    )
    .bind(project_id)
    .bind(env)
    .bind(encrypt(settings_key, control_key))
    .bind(version)
    .bind(CREATING)
    .execute(pool)
    .await
    .map_err(|e| format!("couldn't record the project's backend: {e}"))?;
    Ok(())
}

/// The control key Nomi sealed for this Relay.
pub async fn control_key(pool: &PgPool, settings_key: &[u8; 32], project_id: Uuid, env: &str) -> Result<Option<String>, String> {
    match row(pool, project_id, env).await? {
        Some((_, sealed, ..)) => decrypt(settings_key, &sealed).map(Some).map_err(|_| "the project's Relay key can't be decrypted with this settings key".to_string()),
        None => Ok(None),
    }
}

/// Moves a Relay to `state`, with the step it's on and, when it failed, why.
pub async fn set_state(pool: &PgPool, project_id: Uuid, env: &str, state: &str, step: Option<&str>, error: Option<&str>) -> Result<(), String> {
    sqlx::query("UPDATE project_relays SET state = $3, step = $4, error = $5, updated_at = now() WHERE project_id = $1 AND env = $2")
        .bind(project_id)
        .bind(env)
        .bind(state)
        .bind(step)
        .bind(error)
        .execute(pool)
        .await
        .map_err(|e| format!("couldn't update the project's backend: {e}"))?;
    Ok(())
}

/// Sets the Relay's address once Shipyard has given it.
pub async fn set_url(pool: &PgPool, project_id: Uuid, env: &str, url: &str) -> Result<(), String> {
    sqlx::query("UPDATE project_relays SET url = $3, updated_at = now() WHERE project_id = $1 AND env = $2")
        .bind(project_id)
        .bind(env)
        .bind(url.trim_end_matches('/'))
        .execute(pool)
        .await
        .map_err(|e| format!("couldn't update the project's backend: {e}"))?;
    Ok(())
}
