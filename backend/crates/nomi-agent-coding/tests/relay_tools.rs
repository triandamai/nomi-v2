use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::CodingAgent;
use nomi_agent_core::relay_ops::{ProjectFile, RelayEnv, RelayOps, APPLY_MIGRATIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME};
use nomi_agent_core::SubAgent;
use nomi_storage::ProjectStore;

fn test_storage() -> ProjectStore {
    ProjectStore::at(std::env::temp_dir().join(format!("nomi-agent-coding-relay-{}", Uuid::new_v4())))
}

struct Seeded {
    user_id: Uuid,
    session_id: Uuid,
    project_id: Uuid,
}

async fn seed_project(pool: &PgPool, backend: &str) -> Seeded {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, backend) VALUES ($1, $2, 'Todo', $3) RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .bind(backend)
        .fetch_one(pool)
        .await
        .unwrap();
    Seeded { user_id, session_id, project_id }
}

/// Records what Koda sent to Relay.
#[derive(Default)]
struct FakeRelay {
    applied: Mutex<Vec<(Uuid, RelayEnv, Vec<String>)>>,
}

#[async_trait]
impl RelayOps for FakeRelay {
    fn is_available(&self) -> bool {
        true
    }
    async fn ensure_backend(&self, _: Uuid, _: Uuid) -> Result<Uuid, String> {
        unimplemented!()
    }
    async fn apply_migrations(&self, _user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String> {
        self.applied.lock().unwrap().push((project_id, env, files.iter().map(|f| format!("{}={}", f.path, f.content)).collect()));
        Ok("Applied to dev: 0001_init.sql.".into())
    }
    async fn describe_backend(&self, _: Uuid, _: Uuid, env: RelayEnv) -> Result<String, String> {
        Ok(format!("tables in {}", env.as_str()))
    }
    async fn deploy_functions(&self, _: Uuid, _: Uuid, _: RelayEnv, _: &[ProjectFile]) -> Result<String, String> {
        unimplemented!()
    }
    async fn types(&self, _: Uuid, _: Uuid, _: RelayEnv) -> Result<String, String> {
        unimplemented!()
    }
}

async fn tool_names(agent: &CodingAgent, pool: &PgPool, seeded: &Seeded) -> Vec<String> {
    let mut conn = pool.acquire().await.unwrap();
    agent.chat_tools(&mut conn, seeded.session_id, seeded.user_id, &[]).await.into_iter().map(|t| t.name).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn koda_has_no_relay_tools_on_a_drizzle_project(pool: PgPool) {
    let seeded = seed_project(&pool, "drizzle").await;
    let agent = CodingAgent::new(test_storage()).with_relay_ops(Arc::new(FakeRelay::default()));
    assert!(tool_names(&agent, &pool, &seeded).await.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn koda_gets_relay_tools_on_a_relay_project_without_asking_first(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let agent = CodingAgent::new(test_storage()).with_relay_ops(Arc::new(FakeRelay::default()));
    assert_eq!(tool_names(&agent, &pool, &seeded).await, vec![APPLY_MIGRATIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME]);
    assert!(!agent.tool_needs_approval(APPLY_MIGRATIONS_TOOL_NAME));
    assert!(!agent.tool_needs_approval(DESCRIBE_BACKEND_TOOL_NAME));
}

#[sqlx::test(migrations = "../../migrations")]
async fn apply_migrations_sends_the_migration_files_in_order_to_dev(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
    let mut conn = pool.acquire().await.unwrap();
    let project = seeded.project_id.to_string();
    for (path, content) in [
        ("nomi/migrations/0002_tags.sql", "create table tag (id int);"),
        ("nomi/migrations/0001_init.sql", "create table task (id int);"),
        ("nomi/backend.json", "{}"),
        ("src/routes/+page.svelte", "<h1>hi</h1>"),
    ] {
        agent
            .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": project, "path": path, "content": content}))
            .await
            .unwrap();
    }

    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, APPLY_MIGRATIONS_TOOL_NAME, json!({"project_id": project}))
        .await
        .unwrap();

    assert_eq!(said.display_text, "Applied to dev: 0001_init.sql.");
    let applied = relay.applied.lock().unwrap();
    assert_eq!(
        *applied,
        vec![(
            seeded.project_id,
            RelayEnv::Dev,
            vec![
                "nomi/migrations/0001_init.sql=create table task (id int);".to_string(),
                "nomi/migrations/0002_tags.sql=create table tag (id int);".to_string()
            ]
        )]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn describe_backend_reads_the_environment_asked_for(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let agent = CodingAgent::new(test_storage()).with_relay_ops(Arc::new(FakeRelay::default()));
    let mut conn = pool.acquire().await.unwrap();
    let said = agent
        .execute_tool(
            &mut conn,
            seeded.session_id,
            Uuid::new_v4(),
            seeded.user_id,
            DESCRIBE_BACKEND_TOOL_NAME,
            json!({"project_id": seeded.project_id.to_string(), "environment": "production"}),
        )
        .await
        .unwrap();
    assert_eq!(said.display_text, "tables in production");
}

#[sqlx::test(migrations = "../../migrations")]
async fn relay_tools_refuse_a_drizzle_project_and_someone_else_s(pool: PgPool) {
    let drizzle = seed_project(&pool, "drizzle").await;
    let relay_project = seed_project(&pool, "relay").await;
    let agent = CodingAgent::new(test_storage()).with_relay_ops(Arc::new(FakeRelay::default()));
    let mut conn = pool.acquire().await.unwrap();

    let on_drizzle = agent
        .execute_tool(&mut conn, drizzle.session_id, Uuid::new_v4(), drizzle.user_id, APPLY_MIGRATIONS_TOOL_NAME, json!({"project_id": drizzle.project_id.to_string()}))
        .await;
    assert!(on_drizzle.unwrap_err().contains("Relay"));

    let stranger = agent
        .execute_tool(&mut conn, drizzle.session_id, Uuid::new_v4(), drizzle.user_id, DESCRIBE_BACKEND_TOOL_NAME, json!({"project_id": relay_project.project_id.to_string()}))
        .await;
    assert!(stranger.is_err());
}
