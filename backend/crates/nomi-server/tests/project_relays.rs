//! Which Relay a project's calls go to: its own when it has one, the shared one otherwise.
use nomi_agent_core::project_ops::{DataSource, ProjectOps};
use nomi_relay_client::RelayConfig;
use nomi_server::project_ops::ServerProjectOps;
use nomi_server::project_relays::{self, config_for, CREATING, FAILED, READY, SLEEPING};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: [u8; 32] = [3u8; 32];

async fn seed_project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, backend) VALUES ($1, $2, 'Todo', 'relay') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, project_id)
}

fn shared() -> Option<RelayConfig> {
    RelayConfig::from_values(Some("https://shared-relay.test"), Some("shared-key"))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_without_its_own_relay_uses_the_shared_one(pool: PgPool) {
    let (_, project) = seed_project(&pool).await;

    let config = config_for(&pool, Some(&KEY), shared(), project, "dev").await.unwrap().unwrap();

    assert_eq!(config.control_url, "https://shared-relay.test");
    assert_eq!(config.control_key, "shared-key");
    assert!(config_for(&pool, Some(&KEY), None, project, "dev").await.unwrap().is_none(), "no shared Relay either: not set up");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_ready_relay_is_called_with_its_own_address_and_key(pool: PgPool) {
    let (_, project) = seed_project(&pool).await;
    project_relays::begin(&pool, &KEY, project, "dev", "dev-control-key", "0.1.5").await.unwrap();
    project_relays::set_url(&pool, project, "dev", "https://todo-backend-dev.apps.test/").await.unwrap();
    project_relays::set_state(&pool, project, "dev", READY, None, None).await.unwrap();

    let dev = config_for(&pool, Some(&KEY), shared(), project, "dev").await.unwrap().unwrap();
    assert_eq!((dev.control_url.as_str(), dev.control_key.as_str()), ("https://todo-backend-dev.apps.test", "dev-control-key"));

    // Production has no Relay of its own yet, so it still goes to the shared one.
    let production = config_for(&pool, Some(&KEY), shared(), project, "production").await.unwrap().unwrap();
    assert_eq!(production.control_url, "https://shared-relay.test");

    let sealed: Vec<u8> = sqlx::query_scalar("SELECT control_key FROM project_relays WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&sealed).contains("dev-control-key"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_relay_that_isnt_ready_says_why_instead_of_falling_back(pool: PgPool) {
    let (_, project) = seed_project(&pool).await;
    project_relays::begin(&pool, &KEY, project, "dev", "k", "0.1.5").await.unwrap();

    let creating = config_for(&pool, Some(&KEY), shared(), project, "dev").await.unwrap_err();
    assert!(creating.contains("still being set up"), "{creating}");

    project_relays::set_state(&pool, project, "dev", SLEEPING, None, None).await.unwrap();
    assert!(config_for(&pool, Some(&KEY), shared(), project, "dev").await.unwrap_err().contains("waking up"));

    project_relays::set_state(&pool, project, "dev", FAILED, Some("deploy"), Some("the build failed")).await.unwrap();
    let failed = config_for(&pool, Some(&KEY), shared(), project, "dev").await.unwrap_err();
    assert!(failed.contains("couldn't be set up at \"deploy\": the build failed"), "{failed}");

    project_relays::begin(&pool, &KEY, project, "dev", "ignored", "0.1.6").await.unwrap();
    let again = project_relays::stack(&pool, project, "dev").await.unwrap().unwrap();
    assert_eq!((again.state.as_str(), again.step, again.error, again.version.as_str()), (CREATING, None, None, "0.1.6"));
    assert_eq!(project_relays::control_key(&pool, &KEY, project, "dev").await.unwrap().as_deref(), Some("ignored"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_the_settings_key_a_projects_own_relay_cant_be_opened(pool: PgPool) {
    let (_, project) = seed_project(&pool).await;
    project_relays::begin(&pool, &KEY, project, "dev", "k", "0.1.5").await.unwrap();
    project_relays::set_url(&pool, project, "dev", "https://x.test").await.unwrap();
    project_relays::set_state(&pool, project, "dev", READY, None, None).await.unwrap();

    let refused = config_for(&pool, None, shared(), project, "dev").await.unwrap_err();
    assert!(refused.contains("SETTINGS_ENCRYPTION_KEY"), "{refused}");
    let wrong = config_for(&pool, Some(&[9u8; 32]), shared(), project, "dev").await.unwrap_err();
    assert!(wrong.contains("can't be decrypted"), "{wrong}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn iris_reads_a_project_through_its_own_relay(pool: PgPool) {
    let (user, project) = seed_project(&pool).await;
    let own = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/control/projects/{project}/environments/production/schema")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "tables": [{ "name": "booking" }] }, "error": null })))
        .expect(1)
        .mount(&own)
        .await;
    project_relays::begin(&pool, &KEY, project, "production", "own-key", "0.1.5").await.unwrap();
    project_relays::set_url(&pool, project, "production", &own.uri()).await.unwrap();
    project_relays::set_state(&pool, project, "production", READY, None, None).await.unwrap();
    let ops = ServerProjectOps::with_relay(pool.clone(), nomi_storage::ProjectStore::at(std::env::temp_dir()), reqwest::Client::new(), Some(KEY));

    let tables = ops.tables(user, project, DataSource::Production).await.unwrap();

    assert_eq!(tables.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), vec!["booking"]);
}
