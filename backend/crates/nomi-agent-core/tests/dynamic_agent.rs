use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::PgPool;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{CatalogTool, DynamicAgent, DynamicAgentRow, SubAgent, ToolCatalog, ToolOutcome};
use nomi_llm::ToolDefinition;

struct Echo;
#[async_trait]
impl CatalogTool for Echo {
    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, _user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        Ok(ToolOutcome::text(input.to_string()))
    }
}

fn catalog_with_echo() -> Arc<ToolCatalog> {
    let mut entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> = HashMap::new();
    entries.insert(
        "echo",
        (ToolDefinition { name: "echo".to_string(), description: "echoes input".to_string(), input_schema: json!({"type": "object"}) }, Box::new(Echo)),
    );
    Arc::new(ToolCatalog::new(entries))
}

fn sample_row() -> DynamicAgentRow {
    DynamicAgentRow {
        id: Uuid::new_v4(),
        name: "Echo Bot".to_string(),
        system_prompt: "You echo things.".to_string(),
        intent_label: "echo_intent".to_string(),
        intent_description: "the user wants something echoed".to_string(),
        granted_tools: vec!["echo".to_string()],
        supports_todos: true,
        supports_plans: false,
        can_delegate: false,
    }
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn constructs_correctly_from_a_row() {
    let row = sample_row();
    let id = row.id;
    let agent = DynamicAgent::from_row(row, catalog_with_echo());

    assert_eq!(agent.agent_type(), id.to_string());
    assert_eq!(agent.intent_label(), "echo_intent");
    assert_eq!(agent.intent_description(), "the user wants something echoed");
    assert_eq!(agent.system_prompt(), "You echo things.");
    assert!(agent.supports_todos());
    assert!(!agent.supports_plans());
    assert!(!agent.can_delegate());
    assert!(!agent.uses_memory());
    assert!(!agent.uses_personality());
}

#[tokio::test]
async fn tools_only_lists_granted_tools() {
    let agent = DynamicAgent::from_row(sample_row(), catalog_with_echo());
    let tools = agent.tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
}

#[sqlx::test(migrations = "../../migrations")]
async fn execute_tool_runs_a_granted_tool(pool: sqlx::PgPool) {
    let agent = DynamicAgent::from_row(sample_row(), catalog_with_echo());
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "echo", json!({"x": 1}))
        .await
        .unwrap();
    assert_eq!(result.display_text, json!({"x": 1}).to_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn execute_tool_rejects_an_ungranted_tool_name(pool: sqlx::PgPool) {
    let mut row = sample_row();
    row.granted_tools = vec![]; // granted nothing, even though the catalog knows "echo"
    let agent = DynamicAgent::from_row(row, catalog_with_echo());
    let mut conn = pool.acquire().await.unwrap();
    let result = agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "echo", json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not granted"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn fetch_active_dynamic_agents_excludes_inactive_rows(pool: sqlx::PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Active', 'p', 'active_label', 'd', '{}', true, $1), ('Inactive', 'p', 'inactive_label', 'd', '{}', false, $1)",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .unwrap();

    let rows = nomi_agent_core::dynamic_agent::fetch_active_dynamic_agents(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].intent_label, "active_label");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_dynamic_agent_by_id_finds_inactive_rows_too(pool: sqlx::PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Disabled', 'p', 'disabled_label', 'd', '{}', false, $1) RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await
    .unwrap();

    let found = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(&mut conn, id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().intent_label, "disabled_label");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_dynamic_agent_by_id_returns_none_for_unknown_id(pool: sqlx::PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let found = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(&mut conn, Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}
