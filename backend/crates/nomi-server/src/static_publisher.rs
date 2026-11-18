//! Design previews published through Shipyard's static artifacts
//! (`nomi_agent_core::static_publish::StaticPublisher`): one static project per chat, in the
//! chat's organization, served by Shipyard as the files are. Each preview version gets the
//! address pinned to its own deployment, so a version's link never changes under it.

use std::sync::Arc;

use async_trait::async_trait;
use nomi_agent_core::static_publish::{NoStaticPublisher, PublishRequest, Published, SharedStaticPublisher, StaticPublisher};
use nomi_shipyard::{Shipyard, ShipyardConfig, StaticPublish, StaticPublished};
use sqlx::PgPool;
use uuid::Uuid;

/// The provider's name on a publication.
pub const PROVIDER: &str = "shipyard";
/// The alias that follows a chat's newest preview.
const ALIAS: &str = "latest";
/// What a chat's static project is called on Shipyard, before the chat's id.
const SLUG_PREFIX: &str = "design-";

/// The chat's static project on Shipyard.
pub fn project_slug(session_id: Uuid) -> String {
    format!("{SLUG_PREFIX}{}", session_id.simple())
}

/// The address of what Shipyard is serving: the deployment's own, which never moves.
pub fn address_of(config: &ShipyardConfig, published: &StaticPublished) -> Result<String, String> {
    if !published.is_ready() {
        return Err(format!("Shipyard isn't serving it yet ({})", published.status));
    }
    config.address(&published.preview_url).ok_or_else(|| "Shipyard gave an address that can't be used".to_string())
}

pub struct ShipyardStaticPublisher {
    pool: PgPool,
    http: reqwest::Client,
    config: ShipyardConfig,
}

impl ShipyardStaticPublisher {
    pub fn new(pool: PgPool, http: reqwest::Client, config: ShipyardConfig) -> Self {
        Self { pool, http, config }
    }

    /// Shipyard from `SHIPYARD_*`; without it, previews are saved and not published.
    pub fn shared(pool: PgPool) -> SharedStaticPublisher {
        match ShipyardConfig::from_env() {
            Some(config) => Arc::new(Self::new(pool, crate::http_client(), config)),
            None => {
                // Said once per process: every preview saved here stays unpublished until it is set up.
                static MISSING: std::sync::Once = std::sync::Once::new();
                MISSING.call_once(|| tracing::warn!("SHIPYARD_URL / SHIPYARD_KEY aren't set: design previews are saved but not published"));
                Arc::new(NoStaticPublisher)
            }
        }
    }
}

#[async_trait]
impl StaticPublisher for ShipyardStaticPublisher {
    async fn publish(&self, request: &PublishRequest<'_>) -> Result<Published, String> {
        let unreadable = |e: sqlx::Error| {
            tracing::error!(error = %e, "design preview: couldn't read the chat's organization");
            "the chat's organization couldn't be read".to_string()
        };
        let org: Option<(Uuid, String)> = sqlx::query_as("SELECT o.id, o.name FROM sessions s JOIN organizations o ON o.id = s.org_id WHERE s.id = $1")
            .bind(request.session_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(unreadable)?;
        let (org_id, org_name) = org.ok_or("this chat couldn't be found")?;
        let org_key = org_id.to_string();

        let client = Shipyard::new(&self.http, &self.config);
        client.put_org(&org_key, &org_name).await.map_err(|e| e.to_string())?;
        if let Err(e) = sqlx::query("UPDATE organizations SET shipyard_synced_at = now() WHERE id = $1").bind(org_id).execute(&self.pool).await {
            tracing::warn!(error = %e, "design preview: couldn't note the organization is on Shipyard");
        }

        let files: Vec<(String, Vec<u8>)> = request.files.iter().map(|f| (f.path.clone(), f.content.clone().into_bytes())).collect();
        let artifact = nomi_shipyard::bundle::zip_static(&files)?;
        let slug = project_slug(request.session_id);
        let publish = StaticPublish { project_slug: &slug, alias: ALIAS, spa_fallback: false };
        let published = client.static_publish(&org_key, &publish, artifact).await.map_err(|e| e.to_string())?;
        Ok(Published { provider: PROVIDER.to_string(), url: address_of(&self.config, &published)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ShipyardConfig {
        ShipyardConfig { base_url: "https://ship.example.com/openapi/v1/integration".into(), key: "k".into(), reveal_key: None, webhook_secret: None }
    }

    fn published(status: &str, preview_url: &str) -> StaticPublished {
        StaticPublished {
            project_id: "p1".into(),
            project_slug: "x".into(),
            deployment_id: "d1".into(),
            status: status.into(),
            preview_url: preview_url.into(),
            alias: Some(ALIAS.into()),
            alias_url: Some("/alias/p1/latest".into()),
        }
    }

    #[test]
    fn a_chat_has_one_static_project() {
        let session_id = Uuid::parse_str("0193309a-4f5b-7b00-8cb5-123456789abc").unwrap();
        assert_eq!(project_slug(session_id), "design-0193309a4f5b7b008cb5123456789abc");
    }

    #[test]
    fn a_version_gets_its_deployments_own_address() {
        assert_eq!(address_of(&config(), &published("READY", "/dpl/d1")).as_deref(), Ok("https://ship.example.com/dpl/d1"));
        assert_eq!(address_of(&config(), &published("ready", "/dpl/d1")).as_deref(), Ok("https://ship.example.com/dpl/d1"));
    }

    #[test]
    fn nothing_is_given_out_until_shipyard_serves_it_at_a_usable_address() {
        assert_eq!(address_of(&config(), &published("BUILDING", "/dpl/d1")).unwrap_err(), "Shipyard isn't serving it yet (BUILDING)");
        assert_eq!(address_of(&config(), &published("READY", "//evil.example/dpl")).unwrap_err(), "Shipyard gave an address that can't be used");
        assert_eq!(address_of(&config(), &published("READY", "")).unwrap_err(), "Shipyard gave an address that can't be used");
    }
}
