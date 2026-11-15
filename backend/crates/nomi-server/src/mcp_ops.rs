//! The person's own MCP servers for the crew (`nomi_agent_core::mcp_ops::McpOps`): the rows of
//! `mcp_servers`, reached with `nomi_mcp`. Only public addresses are called, each within a time
//! limit, and one server failing never hides the others.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use nomi_agent_core::mcp_ops::{McpListing, McpOps, McpToolInfo, SharedMcpOps};
use nomi_mcp::McpClient;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// How long listing one server's tools may take, in seconds: a slow server is skipped rather
/// than holding the agent up.
const LIST_TIMEOUT_SECS: u64 = 12;
/// The port an address without one is on, by scheme.
const HTTPS_PORT: u16 = 443;
const HTTP_PORT: u16 = 80;

pub struct ServerMcpOps {
    pool: PgPool,
    http: reqwest::Client,
}

impl ServerMcpOps {
    pub fn shared(pool: PgPool) -> SharedMcpOps {
        Arc::new(Self { pool, http: nomi_mcp::http_client() })
    }

    /// The person's servers (name, address), oldest first.
    async fn servers(&self, user_id: Uuid) -> Result<Vec<(String, String)>, sqlx::Error> {
        sqlx::query_as("SELECT name, url FROM mcp_servers WHERE user_id = $1 ORDER BY created_at").bind(user_id).fetch_all(&self.pool).await
    }
}

/// Refuses an address on the server's own network (see `nomi_agent_core::web::ensure_public_host`).
async fn ensure_public(url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "its address isn't valid".to_string())?;
    let host = parsed.host_str().ok_or_else(|| "its address isn't valid".to_string())?;
    let port = parsed.port().unwrap_or(if parsed.scheme() == "http" { HTTP_PORT } else { HTTPS_PORT });
    nomi_agent_core::web::ensure_public_host(host, port).await.map_err(|_| "its address is private or couldn't be found".to_string())
}

#[async_trait]
impl McpOps for ServerMcpOps {
    async fn tools(&self, user_id: Uuid) -> McpListing {
        let mut listing = McpListing::default();
        let servers = match self.servers(user_id).await {
            Ok(servers) => servers,
            Err(e) => {
                tracing::error!(error = %e, "mcp: couldn't load the person's servers");
                return listing;
            }
        };
        for (name, url) in servers {
            if let Err(why) = ensure_public(&url).await {
                listing.unavailable.push((name, why));
                continue;
            }
            let client = McpClient::new(&self.http, &url);
            match tokio::time::timeout(Duration::from_secs(LIST_TIMEOUT_SECS), client.list_tools()).await {
                Ok(Ok(tools)) => listing.tools.extend(tools.into_iter().map(|t| McpToolInfo {
                    server: name.clone(),
                    tool: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })),
                Ok(Err(e)) => {
                    tracing::warn!(server = %name, error = %e, "mcp: listing a server's tools failed");
                    listing.unavailable.push((name, e.to_string()));
                }
                Err(_) => listing.unavailable.push((name, "it took too long to answer".to_string())),
            }
        }
        listing
    }

    async fn call(&self, user_id: Uuid, server: &str, tool: &str, arguments: Value) -> Result<String, String> {
        let servers = self.servers(user_id).await.map_err(|e| {
            tracing::error!(error = %e, "mcp: couldn't load the person's servers");
            "the person's servers couldn't be loaded".to_string()
        })?;
        let (name, url) = servers
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(server.trim()))
            .ok_or_else(|| format!("the person has no connected server named \"{server}\""))?;
        ensure_public(&url).await?;
        McpClient::new(&self.http, &url).call_tool(tool, &arguments).await.map_err(|e| {
            tracing::warn!(server = %name, tool, error = %e, "mcp: a tool call failed");
            e.to_string()
        })
    }
}
