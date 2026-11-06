//! Iris through her tools: what she reads of a project's app, blind until a project admin allows
//! more, and who can allow it.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::content_block::ContentBlock;
use nomi_agent_core::project_ops::{DataSource, ProjectOps, QueryRows, TableInfo};
use nomi_agent_core::{Locale, SubAgent, ToolOutcome};
use nomi_agent_inspector::InspectorAgent;

/// The project's app as the tests want it, recording what Iris asked for.
#[derive(Default)]
struct FakeOps {
    logs: Vec<String>,
    tables: Vec<TableInfo>,
    rows: QueryRows,
    asked: Mutex<Vec<String>>,
}

#[async_trait]
impl ProjectOps for FakeOps {
    async fn publish(&self, _: Uuid, _: Uuid, _: &str, _: Option<&str>) -> Result<String, String> {
        Err("not here".into())
    }
    async fn status(&self, _: Uuid, _: Uuid, environment: &str) -> Result<String, String> {
        self.asked.lock().unwrap().push(format!("status {environment}"));
        Ok(format!("{environment}: running"))
    }
    async fn logs(&self, _: Uuid, _: Uuid, environment: &str, tail: u32) -> Result<Vec<String>, String> {
        self.asked.lock().unwrap().push(format!("logs {environment} {tail}"));
        Ok(self.logs.clone())
    }
    async fn tables(&self, _: Uuid, _: Uuid, source: DataSource) -> Result<Vec<TableInfo>, String> {
        self.asked.lock().unwrap().push(format!("tables {source:?}"));
        Ok(self.tables.clone())
    }
    async fn query(&self, _: Uuid, _: Uuid, source: DataSource, sql: &str) -> Result<QueryRows, String> {
        self.asked.lock().unwrap().push(format!("query {source:?} {sql}"));
        Ok(self.rows.clone())
    }
}

struct World {
    pool: PgPool,
    owner: Uuid,
    stranger: Uuid,
    project: Uuid,
}

async fn world(pool: PgPool) -> World {
    let owner: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let stranger: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'x') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();
    let project: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Bakery') RETURNING id")
        .bind(owner)
        .bind(session)
        .fetch_one(&pool)
        .await
        .unwrap();
    World { pool, owner, stranger, project }
}

async fn run(agent: &InspectorAgent, w: &World, user: Uuid, tool: &str, input: Value) -> Result<ToolOutcome, String> {
    let mut conn = w.pool.acquire().await.unwrap();
    agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user, tool, input).await
}

fn rows(n: usize) -> QueryRows {
    QueryRows {
        columns: vec!["id".into(), "email".into()],
        rows: (0..n).map(|i| vec![json!(i), if i % 2 == 0 { json!(format!("user{i}@x.test")) } else { Value::Null }]).collect(),
        truncated: n > 60,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn status_and_tables_are_plain_reads(pool: PgPool) {
    let w = world(pool).await;
    let ops = Arc::new(FakeOps {
        tables: vec![TableInfo { schema: "public".into(), name: "orders".into(), rows_estimate: 120 }, TableInfo { schema: "public".into(), name: "new".into(), rows_estimate: -1 }],
        ..Default::default()
    });
    let agent = InspectorAgent::new().with_project_ops(ops.clone());
    let p = w.project.to_string();
    assert_eq!(run(&agent, &w, w.owner, "app_status", json!({"project_id": p})).await.unwrap().display_text, "production: running");
    assert_eq!(run(&agent, &w, w.owner, "app_status", json!({"project_id": p, "environment": " staging "})).await.unwrap().display_text, "staging: running");
    assert_eq!(run(&agent, &w, w.owner, "list_tables", json!({"project_id": p, "source": "development"})).await.unwrap().display_text, "public.orders (~120 rows)\npublic.new (~0 rows)");
    assert_eq!(ops.asked.lock().unwrap().last().unwrap(), "tables Development");

    assert_eq!(run(&agent, &w, w.owner, "list_tables", json!({"project_id": p, "source": "prod?"})).await.unwrap_err(), "source is production or development");
    assert_eq!(run(&agent, &w, w.owner, "app_status", json!({})).await.unwrap_err(), "project_id is required");
    assert_eq!(run(&agent, &w, w.owner, "app_status", json!({"project_id": "nope"})).await.unwrap_err(), "project_id is not a valid UUID");
    assert_eq!(run(&agent, &w, w.owner, "fly", json!({"project_id": p})).await.unwrap_err(), "unknown tool: fly");

    let empty = InspectorAgent::new().with_project_ops(Arc::new(FakeOps::default()));
    assert_eq!(run(&empty, &w, w.owner, "list_tables", json!({"project_id": p})).await.unwrap().display_text, "The database has no tables yet.");
    assert_eq!(run(&empty, &w, w.owner, "read_logs", json!({"project_id": p})).await.unwrap().display_text, "The app printed nothing in that window.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn logs_stay_with_the_person_until_reading_them_is_allowed(pool: PgPool) {
    let w = world(pool).await;
    let logs: Vec<String> = (0..150).map(|i| match i {
        10 => "WARN slow query".to_string(),
        140 => "Error: connect ECONNREFUSED token=sk-live-abcdef1234567890abcdef".to_string(),
        _ => format!("GET /page/{i} 200"),
    }).collect();
    let ops = Arc::new(FakeOps { logs, ..Default::default() });
    let agent = InspectorAgent::new().with_project_ops(ops.clone());
    let p = w.project.to_string();

    let blind = run(&agent, &w, w.owner, "read_logs", json!({"project_id": p, "lines": 5000})).await.unwrap();
    assert_eq!(ops.asked.lock().unwrap().last().unwrap(), "logs production 1000", "at most 1000 lines");
    assert!(blind.display_text.starts_with("150 log lines: 1 look like errors, 1 like warnings."), "{}", blind.display_text);
    assert!(blind.display_text.contains("The newest error is 10 lines from the end."));
    assert!(!blind.display_text.contains("ECONNREFUSED"), "Iris doesn't see the lines");
    let Some(ContentBlock::Table { rows, source, .. }) = blind.block else { panic!("the person sees a table") };
    assert_eq!(rows.len(), 100);
    assert_eq!(source.as_deref(), Some("The newest 100 of 150 lines"));

    // Someone outside the project can't allow it; the owner can.
    let refused = run(&agent, &w, w.stranger, "request_access", json!({"project_id": p, "permission": "read_logs"})).await.unwrap_err();
    assert!(refused.starts_with("Only the project's admins"));
    assert_eq!(run(&agent, &w, w.owner, "request_access", json!({"project_id": p, "permission": "write_all"})).await.unwrap_err(), "permission is read_logs or read_data");
    let allowed = run(&agent, &w, w.owner, "request_access", json!({"project_id": p, "permission": "read_logs"})).await.unwrap();
    assert!(allowed.display_text.contains("read what the app's logs say"));

    let reading = run(&agent, &w, w.owner, "read_logs", json!({"project_id": p})).await.unwrap();
    assert!(reading.block.is_none());
    assert!(reading.display_text.contains("ECONNREFUSED"));
    assert!(!reading.display_text.contains("sk-live-abcdef1234567890abcdef"), "secrets are redacted even then");
}

#[sqlx::test(migrations = "../../migrations")]
async fn query_results_show_their_shape_until_reading_data_is_allowed(pool: PgPool) {
    let w = world(pool).await;
    let ops = Arc::new(FakeOps { rows: rows(120), ..Default::default() });
    let agent = InspectorAgent::new().with_project_ops(ops.clone());
    let p = w.project.to_string();

    assert_eq!(run(&agent, &w, w.owner, "query_database", json!({"project_id": p, "sql": "  "})).await.unwrap_err(), "sql is required");
    let blind = run(&agent, &w, w.owner, "query_database", json!({"project_id": p, "sql": "select * from users"})).await.unwrap();
    assert_eq!(
        blind.display_text,
        "120 rows (more matched). Columns: id (120 with a value), email (60 with a value). The person sees the rows in the chat; you can't read the values without read_data access."
    );
    let Some(ContentBlock::Table { rows: shown, source, .. }) = blind.block else { panic!("a table for the person") };
    assert_eq!(shown.len(), 100);
    assert_eq!(source.as_deref(), Some("120 rows, more matched than shown"));

    let one = InspectorAgent::new().with_project_ops(Arc::new(FakeOps { rows: rows(1), ..Default::default() }));
    let single = run(&one, &w, w.owner, "query_database", json!({"project_id": p, "sql": "select 1"})).await.unwrap();
    assert!(single.display_text.starts_with("1 row. Columns"), "{}", single.display_text);
    let none = InspectorAgent::new().with_project_ops(Arc::new(FakeOps { rows: QueryRows::default(), ..Default::default() }));
    assert!(run(&none, &w, w.owner, "query_database", json!({"project_id": p, "sql": "select"})).await.unwrap().display_text.contains("Columns: none."));

    run(&agent, &w, w.owner, "request_access", json!({"project_id": p, "permission": "read_data"})).await.unwrap();
    let reading = run(&agent, &w, w.owner, "query_database", json!({"project_id": p, "sql": "select * from users"})).await.unwrap();
    assert!(reading.block.is_none());
    assert!(reading.display_text.starts_with("120 row(s):\n{\"email\":\"user0@x.test\",\"id\":0}") || reading.display_text.starts_with("120 row(s):\n{\"id\":0"), "{}", &reading.display_text[..80]);
    assert!(reading.display_text.ends_with("(more rows matched than are shown)"));
    assert_eq!(reading.display_text.lines().count(), 52, "a header, 50 rows and the note");
}

#[test]
fn iris_asks_before_seeing_and_says_why() {
    let agent = InspectorAgent::default();
    assert_eq!(agent.agent_type(), "inspector");
    assert_eq!(agent.intent_label(), "inspector");
    assert!(!agent.system_prompt().is_empty());
    assert!(agent.intent_description().contains("logs"));
    assert_eq!(agent.tools().len(), 5);
    assert_eq!(
        agent.describe_action("request_access", &json!({"permission": "read_data", "reason": " to check the signup bug "}), Locale::En).as_deref(),
        Some("Let Iris see the values in the app's database in this project: to check the signup bug")
    );
    assert_eq!(agent.describe_action("request_access", &json!({"permission": "read_logs", "reason": ""}), Locale::En).as_deref(), Some("Let Iris read what the app's logs say in this project"));
    assert_eq!(agent.describe_action("read_logs", &json!({}), Locale::En), None);
}
