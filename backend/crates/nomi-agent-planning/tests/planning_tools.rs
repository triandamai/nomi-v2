//! Rena's settings and tools beyond creating a project: publishing without Shipyard, unknown
//! tools, and when the app-building playbook is part of her context.

use nomi_agent_core::SubAgent;
use nomi_agent_planning::PlanningAgent;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[test]
fn rena_describes_herself() {
    let agent = PlanningAgent::new();
    assert_eq!(agent.agent_type(), agent.intent_label());
    assert!(!agent.system_prompt().is_empty());
    assert!(agent.intent_description().contains("built"));
    assert!(agent.uses_memory() && agent.uses_personality() && agent.can_delegate() && agent.supports_todos() && agent.supports_plans() && agent.keeps_plans_in_drafts());
    assert!(!agent.surfaces_activity());
    assert!(!agent.tool_needs_approval("create_project"));
    assert!(agent.tool_needs_approval("publish_project"));
    assert_eq!(agent.tools().len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_playbook_comes_with_building_and_publishing_needs_shipyard(pool: PgPool) {
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'c') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let agent = PlanningAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let said = |text: &str| vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }];

    assert!(agent.domain_context(&mut conn, None, session, user, &said("build me a website for my bakery")).await.is_some());
    assert!(agent.domain_context(&mut conn, None, session, user, &said("plan a trip to Bali")).await.is_none());

    let published = agent.execute_tool(&mut conn, session, Uuid::new_v4(), user, "publish_project", json!({"project_id": Uuid::new_v4()})).await;
    assert!(published.is_err() || !published.unwrap().display_text.is_empty());
    assert!(agent.execute_tool(&mut conn, session, Uuid::new_v4(), user, "teleport", json!({})).await.is_err());
}
