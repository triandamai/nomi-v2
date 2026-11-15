//! Sora through the engine's eyes: no tools of her own yet, so any tool call to her is refused.

use nomi_agent_core::SubAgent;
use nomi_agent_design::DesignAgent;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_sora_does_not_have_is_refused_by_name(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let refused = DesignAgent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "publish_preview", json!({})).await;
    assert_eq!(refused.err().as_deref(), Some("unknown tool: publish_preview"));
}
