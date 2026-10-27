use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::CodingAgent;
use nomi_agent_core::relay_ops::{ProjectFile, RelayEnv, RelayOps, APPLY_MIGRATIONS_TOOL_NAME, DEPLOY_FUNCTIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME};
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

/// Records what Koda sent to Relay; told to, its migrations fail.
#[derive(Default)]
struct FakeRelay {
    applied: Mutex<Vec<(Uuid, RelayEnv, Vec<String>)>>,
    auth: Mutex<Vec<(RelayEnv, serde_json::Value)>>,
    storage: Mutex<Vec<(RelayEnv, serde_json::Value)>>,
    realtime: Mutex<Vec<(RelayEnv, serde_json::Value)>>,
    deployed: Mutex<Vec<(RelayEnv, Vec<String>, serde_json::Value)>>,
    fails: bool,
}

#[async_trait]
impl RelayOps for FakeRelay {
    async fn is_available(&self) -> bool {
        true
    }
    async fn ensure_backend(&self, _: Uuid, _: Uuid) -> Result<Uuid, String> {
        unimplemented!()
    }
    async fn apply_migrations(&self, _user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String> {
        self.applied.lock().unwrap().push((project_id, env, files.iter().map(|f| format!("{}={}", f.path, f.content)).collect()));
        if self.fails {
            return Err("0001_init.sql line 3: relation \"nope\" does not exist".into());
        }
        Ok("Applied to dev: 0001_init.sql.".into())
    }
    async fn describe_backend(&self, _: Uuid, _: Uuid, env: RelayEnv) -> Result<String, String> {
        if env == RelayEnv::Production {
            return Ok("tables in production".into());
        }
        Ok(json!({ "tables": [{ "name": "task", "rls_enabled": true, "columns": [{ "name": "id" }, { "name": "title" }], "policies": [{ "name": "own tasks" }] }] }).to_string())
    }
    async fn deploy_functions(&self, _: Uuid, _: Uuid, env: RelayEnv, files: &[ProjectFile], settings: &serde_json::Value) -> Result<String, String> {
        self.deployed.lock().unwrap().push((env, files.iter().map(|f| f.path.clone()).collect(), settings.clone()));
        Ok("Deployed to dev as build 1: place-order.".into())
    }
    async fn types(&self, _: Uuid, _: Uuid, _: RelayEnv) -> Result<String, String> {
        Ok("export interface Database { tables: {}; functions: Record<string, never> }\n".into())
    }
    async fn apply_auth(&self, _: Uuid, _: Uuid, env: RelayEnv, auth: &serde_json::Value) -> Result<String, String> {
        self.auth.lock().unwrap().push((env, auth.clone()));
        Ok("Sign-in settings saved to dev. Still needs the project secret NOMI_OAUTH_GITHUB_SECRET.".into())
    }
    async fn apply_storage(&self, _: Uuid, _: Uuid, env: RelayEnv, storage: &serde_json::Value) -> Result<String, String> {
        self.storage.lock().unwrap().push((env, storage.clone()));
        Ok("Buckets saved to dev: avatars.".into())
    }
    async fn apply_realtime(&self, _: Uuid, _: Uuid, env: RelayEnv, tables: &serde_json::Value) -> Result<String, String> {
        self.realtime.lock().unwrap().push((env, tables.clone()));
        Ok("Realtime on in dev for: message.".into())
    }
}

async fn write(agent: &CodingAgent, conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, seeded: &Seeded, path: &str, content: &str) -> String {
    agent
        .execute_tool(conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": path, "content": content}))
        .await
        .unwrap()
        .display_text
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
    assert_eq!(tool_names(&agent, &pool, &seeded).await, vec![APPLY_MIGRATIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME, DEPLOY_FUNCTIONS_TOOL_NAME]);
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

    assert!(said.display_text.starts_with("Applied to dev: 0001_init.sql."), "{}", said.display_text);
    let applied = relay.applied.lock().unwrap();
    assert_eq!(
        applied.last().cloned().into_iter().collect::<Vec<_>>(),
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

async fn written(storage: &ProjectStore, project_id: Uuid, path: &str) -> Option<String> {
    storage.get_object(&nomi_agent_coding::project_file_key(project_id, path)).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_a_migration_applies_it_to_dev_and_refreshes_the_types(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let storage = test_storage();
    let agent = CodingAgent::new(storage.clone()).with_relay_ops(relay.clone());
    let mut conn = pool.acquire().await.unwrap();

    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": "nomi/migrations/0001_init.sql", "content": "create table task (id int);"}))
        .await
        .unwrap();

    assert!(said.display_text.contains("Relay dev: Applied to dev: 0001_init.sql."), "{}", said.display_text);
    assert_eq!(relay.applied.lock().unwrap().len(), 1);
    assert_eq!(relay.applied.lock().unwrap()[0].1, RelayEnv::Dev);
    assert!(written(&storage, seeded.project_id, "src/lib/nomi.types.ts").await.unwrap().contains("export interface Database"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_apply_comes_back_with_the_write_for_koda_to_fix(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let storage = test_storage();
    let agent = CodingAgent::new(storage.clone()).with_relay_ops(Arc::new(FakeRelay { fails: true, ..Default::default() }));
    let mut conn = pool.acquire().await.unwrap();

    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": "nomi/migrations/0001_init.sql", "content": "select * from nope;"}))
        .await
        .unwrap();

    assert!(said.display_text.contains("0001_init.sql line 3"), "{}", said.display_text);
    assert!(said.block.is_some(), "the file was still written");
    assert!(written(&storage, seeded.project_id, "src/lib/nomi.types.ts").await.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn other_files_and_drizzle_projects_apply_nothing(pool: PgPool) {
    let relay = Arc::new(FakeRelay::default());
    let mut conn = pool.acquire().await.unwrap();
    for (backend, path) in [("relay", "src/routes/+page.svelte"), ("drizzle", "nomi/migrations/0001_init.sql")] {
        let seeded = seed_project(&pool, backend).await;
        let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
        agent
            .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": path, "content": "x"}))
            .await
            .unwrap();
    }
    assert!(relay.applied.lock().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_relay_projects_brief_has_its_tables_and_the_relay_guide(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let agent = CodingAgent::new(test_storage()).with_relay_ops(Arc::new(FakeRelay::default()));
    let mut conn = pool.acquire().await.unwrap();
    let messages = vec![nomi_llm::LlmMessage {
        role: nomi_llm::LlmRole::User,
        content: vec![nomi_llm::ContentBlock::Text { text: format!("Project {}: build it", seeded.project_id) }],
    }];

    let brief = agent.domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.expect("a brief");

    assert!(brief.contains("## Backend: Nomi Relay"), "{brief}");
    assert!(brief.contains("task (id, title)"), "{brief}");
    assert!(brief.contains("# Building on Nomi Relay"), "the relay guide, preloaded");
    assert!(!brief.contains("# Databases with Drizzle"), "not the Drizzle guide");
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_backend_json_sends_its_sign_in_settings_to_dev(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
    let mut conn = pool.acquire().await.unwrap();
    let backend = json!({ "auth": { "methods": ["email_code", "github"], "github": { "client_id": "gh" }, "redirect_urls": ["http://localhost:5173"] } });
    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": "nomi/backend.json", "content": backend.to_string()}))
        .await
        .unwrap();
    assert!(said.display_text.contains("NOMI_OAUTH_GITHUB_SECRET"), "{}", said.display_text);
    let sent = relay.auth.lock().unwrap().clone();
    assert_eq!(sent, vec![(RelayEnv::Dev, backend["auth"].clone())]);

    let broken = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": "nomi/backend.json", "content": "{ not json"}))
        .await
        .unwrap();
    assert!(broken.display_text.contains("isn't valid JSON"), "{}", broken.display_text);
    assert_eq!(relay.auth.lock().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_backend_json_sends_its_buckets_and_realtime_tables_to_dev(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
    let mut conn = pool.acquire().await.unwrap();
    let backend = json!({ "storage": { "buckets": [{ "name": "avatars", "public": true, "write": "owner" }] }, "realtime": ["message"] });

    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": seeded.project_id.to_string(), "path": "nomi/backend.json", "content": backend.to_string()}))
        .await
        .unwrap();

    assert!(said.display_text.contains("Relay dev storage: Buckets saved to dev: avatars."), "{}", said.display_text);
    assert!(said.display_text.contains("Relay dev realtime: Realtime on in dev for: message."), "{}", said.display_text);
    assert_eq!(relay.storage.lock().unwrap().clone(), vec![(RelayEnv::Dev, backend["storage"].clone())]);
    assert_eq!(relay.realtime.lock().unwrap().clone(), vec![(RelayEnv::Dev, backend["realtime"].clone())]);
    assert!(relay.auth.lock().unwrap().is_empty(), "no auth block, nothing sent for sign-in");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_migration_sends_the_realtime_tables_again(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
    let mut conn = pool.acquire().await.unwrap();
    let project = seeded.project_id.to_string();
    agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": project, "path": "nomi/backend.json", "content": json!({ "realtime": ["message"] }).to_string()}))
        .await
        .unwrap();

    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "write_file", json!({"project_id": project, "path": "nomi/migrations/0001_init.sql", "content": "create table message (id int primary key);"}))
        .await
        .unwrap();

    assert_eq!(relay.realtime.lock().unwrap().len(), 2, "sent again once the table exists");
    assert!(said.display_text.contains("Relay dev realtime:"), "{}", said.display_text);
    assert!(relay.storage.lock().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_a_function_deploys_every_function_file_to_dev_with_its_settings(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
    let mut conn = pool.acquire().await.unwrap();
    write(&agent, &mut conn, &seeded, "nomi/functions/_shared/money.ts", "export const cents = (n: number) => n * 100;").await;
    assert_eq!(relay.deployed.lock().unwrap().len(), 1, "a helper deploys too: it changes what the functions do");
    write(&agent, &mut conn, &seeded, "nomi/backend.json", &json!({ "functions": { "place-order": { "timeout": 10 } } }).to_string()).await;

    let said = write(&agent, &mut conn, &seeded, "nomi/functions/place-order.ts", "export default () => new Response('ok')").await;

    assert!(said.contains("Relay dev functions: Deployed to dev as build 1: place-order."), "{said}");
    let deployed = relay.deployed.lock().unwrap().clone();
    let (env, paths, settings) = deployed.last().unwrap();
    assert_eq!(*env, RelayEnv::Dev);
    assert_eq!(paths, &vec!["nomi/functions/_shared/money.ts".to_string(), "nomi/functions/place-order.ts".to_string()]);
    assert_eq!(settings, &json!({ "place-order": { "timeout": 10 } }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn koda_can_deploy_the_functions_himself(pool: PgPool) {
    let seeded = seed_project(&pool, "relay").await;
    let relay = Arc::new(FakeRelay::default());
    let agent = CodingAgent::new(test_storage()).with_relay_ops(relay.clone());
    assert!(tool_names(&agent, &pool, &seeded).await.contains(&"deploy_functions".to_string()));
    let mut conn = pool.acquire().await.unwrap();
    write(&agent, &mut conn, &seeded, "nomi/functions/place-order.ts", "export default () => new Response('ok')").await;

    let said = agent
        .execute_tool(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, "deploy_functions", json!({"project_id": seeded.project_id.to_string()}))
        .await
        .unwrap();

    assert!(said.display_text.contains("Deployed to dev"), "{}", said.display_text);
    assert_eq!(relay.deployed.lock().unwrap().len(), 2);
}
