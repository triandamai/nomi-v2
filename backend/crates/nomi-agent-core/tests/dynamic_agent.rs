use std::sync::Arc;

use nomi_agent_core::ToolContext;
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::PgPool;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{DynamicAgent, DynamicAgentRow, SubAgent, Tool, ToolCatalog, ToolOutcome};
use nomi_llm::ToolDefinition;

struct Echo;
#[async_trait]
impl Tool for Echo {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition { name: "echo".to_string(), description: "echoes input".to_string(), input_schema: json!({"type": "object"}) }
    }
    fn group(&self) -> &'static str {
        "test"
    }
    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, _user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        Ok(ToolOutcome::text(input.to_string()))
    }
}

fn catalog_with_echo() -> Arc<ToolCatalog> {
    Arc::new(ToolCatalog::from_tools(vec![Box::new(Echo)]))
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
    assert_eq!(agent.display_name(), "Echo Bot");
    assert!(agent.supports_todos());
    assert!(!agent.supports_plans());
    assert!(!agent.can_delegate());
    assert!(agent.uses_memory());
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

/// The front door, so a registry can be built around a custom agent.
struct Front;

#[async_trait]
impl SubAgent for Front {
    fn agent_type(&self) -> std::borrow::Cow<'static, str> {
        "chitchat".into()
    }
    fn system_prompt(&self) -> std::borrow::Cow<'static, str> {
        "front".into()
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, name: &str, _: Value) -> Result<ToolOutcome, String> {
        Err(format!("unknown tool: {name}"))
    }
    fn intent_label(&self) -> std::borrow::Cow<'static, str> {
        "chitchat".into()
    }
    fn intent_description(&self) -> std::borrow::Cow<'static, str> {
        "default".into()
    }
    fn is_default(&self) -> bool {
        true
    }
}

/// The echo tool plus the engine's generic tools, as the server's catalog holds them.
fn full_catalog() -> Arc<ToolCatalog> {
    let mut tools: Vec<Box<dyn Tool>> = vec![Box::new(Echo)];
    tools.extend(nomi_agent_core::engine::engine_catalog_tools());
    Arc::new(ToolCatalog::from_tools(tools))
}

fn custom_with(granted: &[&str]) -> (DynamicAgent, nomi_agent_core::AgentRegistry) {
    let catalog = full_catalog();
    let row = DynamicAgentRow { granted_tools: granted.iter().map(|g| g.to_string()).collect(), ..sample_row() };
    let registry = nomi_agent_core::AgentRegistry::new(vec![Box::new(Front)]).with_tool_catalog(catalog.clone());
    (DynamicAgent::from_row(row, catalog), registry)
}

fn engine_offer(agent: &DynamicAgent, registry: &nomi_agent_core::AgentRegistry) -> Vec<ToolDefinition> {
    let inputs = nomi_agent_core::engine::OfferInputs { mentions_files: true, ..Default::default() };
    nomi_agent_core::engine::engine_tools_for(agent, registry, &inputs)
}

#[test]
fn granted_area_tool_still_offered() {
    let (agent, registry) = custom_with(&["echo"]);
    assert_eq!(agent.tools().iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), ["echo"]);
    assert!(!engine_offer(&agent, &registry).iter().any(|t| t.name == "echo"), "offered once, as the agent's own tool");
}

#[test]
fn granted_create_reminder_offered_without_flag() {
    let (agent, registry) = custom_with(&["echo", "create_reminder"]);
    assert!(!agent.supports_reminders());
    let offered = engine_offer(&agent, &registry);
    let create = offered.iter().find(|t| t.name == "create_reminder").expect("a granted create_reminder is offered");
    assert!(create.input_schema.to_string().contains("target_agent"));
    assert!(!offered.iter().any(|t| t.name == "list_reminders"), "only the granted reminder tool");
    assert!(!agent.tools().iter().any(|t| t.name == "create_reminder"), "offered by the engine, not twice");
}

#[test]
fn custom_agent_keeps_automatic_web_and_records() {
    let (agent, registry) = custom_with(&[]);
    let offered: Vec<String> = engine_offer(&agent, &registry).into_iter().map(|t| t.name).collect();
    for name in ["read_web_page", "save_record", "list_records", "run_background_task", "read_attachment"] {
        assert!(offered.contains(&name.to_string()), "{name} missing from {offered:?}");
    }
}
