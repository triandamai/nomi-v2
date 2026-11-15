//! find_tools and create_agent against a real database: the catalog table, word and meaning
//! matches, who carries a tool, and members made the way Settings › Crew makes them.
use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_agent_core::tool_finder::{self, embed_catalog_entries, sync_catalog_table};
use nomi_agent_core::tools::ToolSwitches;
use nomi_agent_core::{AgentRegistry, SubAgent, Tool, ToolCatalog, ToolOutcome};
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use nomi_llm::ToolDefinition;

/// The width of a stored embedding.
const DIMENSIONS: usize = 1536;
/// The plan's limit on members of one's own, in the plan-limit test.
const CREW_LIMIT: i32 = 1;

struct Fake {
    name: &'static str,
    description: &'static str,
    keywords: &'static [&'static str],
}

#[async_trait]
impl Tool for Fake {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition { name: self.name.to_string(), description: self.description.to_string(), input_schema: json!({"type": "object"}) }
    }
    fn group(&self) -> &'static str {
        "test"
    }
    fn keywords(&self) -> &'static [&'static str] {
        self.keywords
    }
    async fn execute(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: Value) -> Result<ToolOutcome, String> {
        Ok(ToolOutcome::text(self.name))
    }
}

fn catalog() -> Arc<ToolCatalog> {
    Arc::new(ToolCatalog::from_tools(vec![
        Box::new(Fake { name: "list_budgets", description: "List the monthly budgets.", keywords: &["spending"] }),
        Box::new(Fake { name: "set_budget", description: "Set a monthly budget.", keywords: &["limit"] }),
        Box::new(Fake { name: "gmail_send", description: "Send an email.", keywords: &["mail"] }),
    ]))
}

/// A built-in member: `default` is Nomi.
struct Member {
    agent_type: &'static str,
    name: &'static str,
    tools: &'static [&'static str],
    default: bool,
}

#[async_trait]
impl SubAgent for Member {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.agent_type)
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|name| ToolDefinition { name: name.to_string(), description: String::new(), input_schema: json!({}) }).collect()
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<ToolOutcome, String> {
        Err("unused".into())
    }
    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.agent_type)
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn display_name(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.name)
    }
    fn is_default(&self) -> bool {
        self.default
    }
    fn creates_crew_members(&self) -> bool {
        self.default
    }
}

fn registry() -> AgentRegistry {
    AgentRegistry::new(vec![
        Box::new(Member { agent_type: "chitchat", name: "Nomi", tools: &[], default: true }),
        Box::new(Member { agent_type: "money", name: "Dana", tools: &["set_budget", "list_budgets"], default: false }),
    ])
    .with_tool_catalog(catalog())
}

/// Embeds anything about money on one axis and everything else on another.
struct Axes;

#[async_trait]
impl EmbeddingProvider for Axes {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut v = vec![0.0; DIMENSIONS];
        let money = ["budget", "cash", "spending"].iter().any(|w| text.contains(w));
        v[usize::from(!money)] = 1.0;
        Ok(v)
    }
    fn provider_name(&self) -> &'static str {
        "test"
    }
    fn model_id(&self) -> &str {
        "axes"
    }
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn names(result: &str) -> Vec<String> {
    let body: Value = serde_json::from_str(result).unwrap();
    body["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_catalog_table_follows_the_catalog(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("INSERT INTO tool_catalog_entries (name, description, tool_group, keywords, content_hash) VALUES ('retired', 'Gone.', 'old', '{}', 'x')")
        .execute(&mut *conn)
        .await
        .unwrap();
    sync_catalog_table(&mut conn, &catalog()).await.unwrap();
    let rows: Vec<String> = sqlx::query_scalar("SELECT name FROM tool_catalog_entries ORDER BY name").fetch_all(&mut *conn).await.unwrap();
    assert_eq!(rows, ["gmail_send", "list_budgets", "set_budget"]);

    assert_eq!(embed_catalog_entries(&pool, &Axes).await, rows.len());
    assert_eq!(embed_catalog_entries(&pool, &Axes).await, 0, "nothing left to embed");

    // A changed tool loses its embedding; an unchanged one keeps it.
    sqlx::query("UPDATE tool_catalog_entries SET content_hash = 'stale' WHERE name = 'set_budget'").execute(&mut *conn).await.unwrap();
    sync_catalog_table(&mut conn, &catalog()).await.unwrap();
    let unembedded: Vec<String> =
        sqlx::query_scalar("SELECT name FROM tool_catalog_entries WHERE embedding IS NULL").fetch_all(&mut *conn).await.unwrap();
    assert_eq!(unembedded, ["set_budget"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_tools_says_who_has_each_tool_and_skips_switched_off_ones(pool: PgPool) {
    let user = seed_user(&pool).await;
    let registry = registry();
    let catalog = registry.tool_catalog().unwrap().clone();
    let mut conn = pool.acquire().await.unwrap();
    let mine: Uuid = sqlx::query_scalar(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, created_by, owner_user_id) \
         VALUES ('Penny', 'p', 'penny1', 'budgets', ARRAY['list_budgets'], $1, $1) RETURNING id",
    )
    .bind(user)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    nomi_agent_core::tools::set_enabled(&mut conn, "gmail_send", false).await.unwrap();
    let switches = ToolSwitches::load(&mut conn).await;
    let nomi = registry.default_agent();

    let result = tool_finder::find_tools(&mut conn, &registry, &catalog, None, &switches, nomi.as_ref(), user, &json!({"query": "budget"}))
        .await
        .unwrap();
    let body: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(names(&result), ["list_budgets", "set_budget"]);
    let list = &body["tools"][0];
    assert_eq!(list["granted"], false);
    assert_eq!(list["carried_by"], json!(["Dana", "Penny"]));
    assert_eq!(body["tools"][1]["carried_by"], json!(["Dana"]));
    assert!(body["note"].as_str().is_some());

    let none = tool_finder::find_tools(&mut conn, &registry, &catalog, None, &switches, nomi.as_ref(), user, &json!({"query": "mail"})).await.unwrap();
    assert!(names(&none).is_empty(), "an admin switched gmail_send off");

    // Dana has set_budget herself, and isn't listed as carrying it.
    let dana = registry.find("money").unwrap();
    let result = tool_finder::find_tools(&mut conn, &registry, &catalog, None, &switches, dana.as_ref(), user, &json!({"query": "set_budget"}))
        .await
        .unwrap();
    let body: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(body["tools"][0]["granted"], true);
    assert_eq!(body["tools"][0]["carried_by"], json!([]));

    sqlx::query("DELETE FROM dynamic_agents WHERE id = $1").bind(mine).execute(&mut *conn).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_tools_fills_with_meaning_matches_and_does_without_them(pool: PgPool) {
    let user = seed_user(&pool).await;
    let registry = registry();
    let catalog = registry.tool_catalog().unwrap().clone();
    let mut conn = pool.acquire().await.unwrap();
    sync_catalog_table(&mut conn, &catalog).await.unwrap();
    embed_catalog_entries(&pool, &Axes).await;
    let switches = ToolSwitches::default();
    let nomi = registry.default_agent();
    let input = json!({"query": "cash"});

    let words_only = tool_finder::find_tools(&mut conn, &registry, &catalog, None, &switches, nomi.as_ref(), user, &input).await.unwrap();
    assert!(names(&words_only).is_empty());
    let with_meaning =
        tool_finder::find_tools(&mut conn, &registry, &catalog, Some(&Axes), &switches, nomi.as_ref(), user, &input).await.unwrap();
    let mut found = names(&with_meaning);
    found.sort();
    assert_eq!(found, ["list_budgets", "set_budget"], "gmail_send is too far from it");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_agent_makes_a_member_of_the_persons_own(pool: PgPool) {
    let user = seed_user(&pool).await;
    let registry = registry();
    let catalog = registry.tool_catalog().unwrap().clone();
    let mut conn = pool.acquire().await.unwrap();

    let made = tool_finder::create_agent(
        &mut conn,
        &registry,
        &catalog,
        user,
        &json!({"name": "Penny", "role": "Keeps the budget", "soul": "Use @list_budgets.", "tools": ["list_budgets"]}),
    )
    .await
    .unwrap();
    let made: Value = serde_json::from_str(&made).unwrap();
    assert_eq!(made["name"], "Penny");
    assert_eq!(made["tools"], json!(["list_budgets"]));
    let id: Uuid = made["agent_type"].as_str().unwrap().parse().unwrap();
    let (owner, tools): (Uuid, Vec<String>) =
        sqlx::query_as("SELECT owner_user_id, granted_tools FROM dynamic_agents WHERE id = $1").bind(id).fetch_one(&mut *conn).await.unwrap();
    assert_eq!((owner, tools), (user, vec!["list_budgets".to_string()]));
    let soul = nomi_agent_core::personal_crew::current_soul(&mut conn, user, &id.to_string()).await.unwrap();
    assert_eq!(soul, Some(("Use @list_budgets.".to_string(), 1)));

    let unknown = tool_finder::create_agent(&mut conn, &registry, &catalog, user, &json!({"name": "Otto", "role": "r", "tools": ["nope"]})).await;
    assert_eq!(unknown.unwrap_err(), "unknown tool nope");
    let taken = tool_finder::create_agent(&mut conn, &registry, &catalog, user, &json!({"name": "dana", "role": "r", "tools": []})).await;
    assert_eq!(taken.unwrap_err(), "dana is already in the crew");
    let again = tool_finder::create_agent(&mut conn, &registry, &catalog, user, &json!({"name": "penny", "role": "r", "tools": []})).await;
    assert!(again.is_err());

    sqlx::query("UPDATE plans SET max_custom_crew = $1 WHERE is_default").bind(CREW_LIMIT).execute(&mut *conn).await.unwrap();
    let limited = tool_finder::create_agent(&mut conn, &registry, &catalog, user, &json!({"name": "Otto", "role": "r", "tools": []})).await;
    assert!(limited.unwrap_err().contains("Upgrade your plan"));

    for table in ["user_agent_soul_versions", "user_agent_souls"] {
        sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1")).bind(user).execute(&mut *conn).await.unwrap();
    }
    sqlx::query("DELETE FROM dynamic_agents WHERE owner_user_id = $1").bind(user).execute(&mut *conn).await.unwrap();
}
