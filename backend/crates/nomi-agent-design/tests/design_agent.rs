//! Sora through the engine's eyes: a tool she doesn't have is refused, and without a provider
//! nothing is connected.

use nomi_agent_core::SubAgent;
use nomi_agent_design::DesignAgent;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_sora_does_not_have_is_refused_by_name(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let refused = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "publish_preview", json!({})).await;
    assert_eq!(refused.err().as_deref(), Some("unknown tool: publish_preview"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_a_provider_listing_says_nothing_is_connected(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let listed = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "list_connected_tools", json!({})).await.unwrap();
    assert_eq!(listed.display_text, nomi_agent_core::mcp_ops::NOTHING_CONNECTED);
    let failed = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "use_connected_tool", json!({"server": "Refs", "tool": "search"})).await;
    assert!(failed.unwrap_err().contains("aren't available"));
}
