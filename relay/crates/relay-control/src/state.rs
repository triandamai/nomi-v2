//! Relay's own records of each project's backend (the `relay` schema in its state database).

use relay_core::crypto::{decrypt, encrypt};
use relay_core::keys::{issued_now, new_signing_secret};
use relay_core::{Env, Tier};
use uuid::Uuid;

use crate::{Relay, RelayError};

pub struct EnvironmentRow {
    pub tier: Tier,
    pub slug: String,
    /// A dedicated project's database; `None` for the shared one.
    pub database_url: Option<String>,
    pub signing_secret: String,
    pub keys_issued_at: u64,
}

/// Records the project (keeping its slug and tier once set) and each environment (keeping its
/// secret once made).
pub async fn record(relay: &Relay, project: Uuid, name: &str, tier: Tier, databases: Option<(&str, &str)>) -> Result<(), RelayError> {
    sqlx::query(
        "INSERT INTO relay.projects (id, name, slug, tier) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name",
    )
    .bind(project)
    .bind(name)
    .bind(relay_core::names::slug(name, project))
    .bind(tier.as_str())
    .execute(&relay.state)
    .await?;
    for env in Env::ALL {
        let url = databases.map(|(dev, production)| if env == Env::Dev { dev } else { production });
        sqlx::query(
            "INSERT INTO relay.environments (project_id, env, database_url_sealed, signing_secret_sealed, keys_issued_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (project_id, env) DO NOTHING",
        )
        .bind(project)
        .bind(env.as_str())
        .bind(url.map(|u| encrypt(&relay.encryption_key, u)))
        .bind(encrypt(&relay.encryption_key, &new_signing_secret()))
        .bind(issued_now() as i64)
        .execute(&relay.state)
        .await?;
    }
    Ok(())
}

pub async fn environment(relay: &Relay, project: Uuid, env: Env) -> Result<Option<EnvironmentRow>, RelayError> {
    let row: Option<(String, String, Option<Vec<u8>>, Vec<u8>, i64)> = sqlx::query_as(
        "SELECT p.tier, p.slug, e.database_url_sealed, e.signing_secret_sealed, e.keys_issued_at \
         FROM relay.environments e JOIN relay.projects p ON p.id = e.project_id WHERE e.project_id = $1 AND e.env = $2",
    )
    .bind(project)
    .bind(env.as_str())
    .fetch_optional(&relay.state)
    .await?;
    let Some((tier, slug, url, secret, issued)) = row else { return Ok(None) };
    let open = |sealed: &[u8]| decrypt(&relay.encryption_key, sealed).map_err(|e| RelayError::Internal(e.to_string()));
    Ok(Some(EnvironmentRow {
        tier: Tier::parse(&tier).unwrap_or(Tier::Free),
        slug,
        database_url: url.as_deref().map(open).transpose()?,
        signing_secret: open(&secret)?,
        keys_issued_at: issued as u64,
    }))
}

/// The project an app's hostname names.
pub async fn by_slug(relay: &Relay, slug: &str) -> Result<Option<Uuid>, RelayError> {
    Ok(sqlx::query_scalar("SELECT id FROM relay.projects WHERE slug = $1").bind(slug).fetch_optional(&relay.state).await?)
}

pub async fn forget(relay: &Relay, project: Uuid) -> Result<(), RelayError> {
    sqlx::query("DELETE FROM relay.projects WHERE id = $1").bind(project).execute(&relay.state).await?;
    Ok(())
}
