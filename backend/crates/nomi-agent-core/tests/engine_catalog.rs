//! The engine offers, approves and runs its generic tools (records, reminders, background, web,
//! voice, attachments) through the tool catalog, gated per agent exactly as before.

use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use nomi_agent_core::engine::{engine_catalog_tools, engine_tools_for, OfferInputs};
use nomi_agent_core::tools::ToolSwitches;
use nomi_agent_core::{run_agent_turn, AgentRegistry, DynamicAgent, DynamicAgentRow, SubAgent, ToolCatalog, ToolOutcome};
use nomi_llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};
use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

/// A test agent; `records` and `reminders` switch its gates, `default` makes it the router.
struct Agent {
    name: &'static str,
    default: bool,
    records: bool,
    reminders: bool,
}

#[async_trait]
impl SubAgent for Agent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.name)
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("test prompt")
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, name: &str, _: serde_json::Value) -> Result<ToolOutcome, String> {
        Err(format!("unknown tool: {name}"))
    }
    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.name)
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("test agent")
    }
    fn is_default(&self) -> bool {
        self.default
    }
    fn uses_records(&self) -> bool {
        self.records
    }
    fn supports_reminders(&self) -> bool {
        self.reminders
    }
}

const PLAIN: Agent = Agent { name: "plain", default: true, records: false, reminders: false };
const KEEPER: Agent = Agent { name: "keeper", default: true, records: true, reminders: false };
const REMINDER: Agent = Agent { name: "reminder", default: true, records: false, reminders: true };
const TARGET: Agent = Agent { name: "target_specialist", default: false, records: false, reminders: false };

fn catalog() -> Arc<ToolCatalog> {
    Arc::new(ToolCatalog::from_tools(engine_catalog_tools()))
}

fn registry(agents: Vec<Box<dyn SubAgent>>) -> AgentRegistry {
    AgentRegistry::new(agents).with_tool_catalog(catalog())
}

fn ready_inputs() -> OfferInputs {
    OfferInputs { switches: ToolSwitches::default(), not_ready: HashSet::new(), mentions_files: true, ..Default::default() }
}

fn offered(agent: &dyn SubAgent, registry: &AgentRegistry, inputs: &OfferInputs) -> Vec<String> {
    engine_tools_for(agent, registry, inputs).into_iter().map(|t| t.name).collect()
}

#[test]
fn records_not_offered_without_uses_records() {
    let without = offered(&PLAIN, &registry(vec![Box::new(PLAIN)]), &ready_inputs());
    assert!(!without.iter().any(|n| n.ends_with("_record") || n == "list_records"), "{without:?}");
    let with = offered(&KEEPER, &registry(vec![Box::new(KEEPER)]), &ready_inputs());
    for name in ["save_record", "list_records", "update_record", "delete_record"] {
        assert!(with.contains(&name.to_string()), "{name} missing from {with:?}");
    }
}

#[test]
fn web_search_off_offered_to_nobody() {
    let inputs = OfferInputs { switches: ToolSwitches::with_off(["web_search"]), ..ready_inputs() };
    let built_in = offered(&PLAIN, &registry(vec![Box::new(PLAIN)]), &inputs);
    assert!(!built_in.contains(&"web_search".to_string()), "{built_in:?}");
    assert!(built_in.contains(&"read_web_page".to_string()), "{built_in:?}");

    let row = DynamicAgentRow {
        id: Uuid::new_v4(),
        name: "Scout".to_string(),
        system_prompt: "You find things.".to_string(),
        intent_label: "scout".to_string(),
        intent_description: "the person wants something found".to_string(),
        granted_tools: vec![],
        supports_todos: false,
        supports_plans: false,
        can_delegate: false,
    };
    let custom = DynamicAgent::from_row(row, catalog());
    let registry = registry(vec![Box::new(PLAIN)]);
    let custom_offer = offered(&custom, &registry, &inputs);
    assert!(!custom_offer.contains(&"web_search".to_string()), "{custom_offer:?}");
}

#[test]
fn reminder_targets_reach_create_reminder() {
    let registry = registry(vec![Box::new(REMINDER), Box::new(TARGET)]);
    let tools = engine_tools_for(&REMINDER, &registry, &ready_inputs());
    let create = tools.iter().find(|t| t.name == "create_reminder").expect("create_reminder offered");
    assert!(create.input_schema.to_string().contains("target_specialist"), "{}", create.input_schema);
    assert!(!offered(&PLAIN, &registry, &ready_inputs()).contains(&"create_reminder".to_string()));
}

fn tool_use(name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: "t1".into(), name: name.into(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn text(reply: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: reply.into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

/// A session, its person, and an agent session for "plain".
async fn seed(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let identity_id: Uuid =
        sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'plain', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (session_id, user_id, agent_session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn records_call_refused_without_uses_records(pool: PgPool) {
    let (session_id, user_id, agent_session_id) = seed(&pool).await;
    let provider = FakeLlmProvider::sequence(vec![
        tool_use("save_record", serde_json::json!({"collection": "trips", "data": {"place": "Bali"}})),
        text("done"),
    ]);
    let embedding = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = registry(vec![Box::new(PLAIN)]);
    let mut conn = pool.acquire().await.unwrap();
    run_agent_turn(&mut conn, None, None, &provider, &embedding, &registry, &PLAIN, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let saved: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_records").fetch_one(&pool).await.unwrap();
    assert_eq!(saved, 0, "an agent without records must not store one");
    let (result, is_error): (String, bool) = sqlx::query_as(
        "SELECT payload->>'result', (payload->>'is_error')::bool FROM agent_events WHERE agent_session_id = $1 AND event_type = 'ToolCalled'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(is_error, "refused call must be an error: {result}");
}

#[test]
fn read_skill_offered_when_member_has_a_skill() {
    let inputs = OfferInputs {
        skills: vec![nomi_skills::SkillEntry { name: "menu".into(), description: "Use when planning meals for the week.".into() }],
        ..ready_inputs()
    };
    let tools = engine_tools_for(&PLAIN, &registry(vec![Box::new(PLAIN)]), &inputs);
    let read = tools.iter().find(|t| t.name == "read_skill").expect("offered");
    assert!(read.description.contains("- menu: Use when planning meals for the week."), "the index is in the description");
}

#[test]
fn read_skill_not_offered_without_one() {
    assert!(!offered(&PLAIN, &registry(vec![Box::new(PLAIN)]), &ready_inputs()).contains(&"read_skill".to_string()));
}
