//! The worker loops as the worker binary runs them: a chat turn, a hand-off to a crew member and
//! a background task each get picked up and finished. The model is the fake provider the tests
//! run with (LLM_PROVIDER=fake); MQTT is the local broker the realtime tests use.

use std::time::Duration;

use nomi_realtime::MqttPublisher;
use sqlx::PgPool;
use uuid::Uuid;

const KEY: [u8; 32] = nomi_test_support::TEST_SETTINGS_KEY;

async fn database_url(pool: &PgPool) -> String {
    let name: String = sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await.unwrap();
    let base = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://nomi:nomi@localhost:5432/nomi".into());
    let mut url = reqwest::Url::parse(&base).unwrap();
    url.set_path(&name);
    url.to_string()
}

fn mqtt() -> MqttPublisher {
    MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, &format!("test-workers-{}", Uuid::new_v4()))
}

/// Waits (up to 30s) until `sql` (a count) says the work is done.
async fn until(pool: &PgPool, sql: &str, id: Uuid) -> bool {
    for _ in 0..150 {
        let done: i64 = sqlx::query_scalar(sql).bind(id).fetch_one(pool).await.unwrap();
        if done > 0 {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    false
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_workers_pick_up_turns_hand_offs_and_background_tasks(pool: PgPool) {
    std::env::set_var("LLM_PROVIDER", "fake");
    std::env::set_var("EMBEDDING_PROVIDER", "fake");
    let url = database_url(&pool).await;
    let storage = nomi_test_support::test_project_storage();
    let turns = tokio::spawn(nomi_server::worker::run(pool.clone(), mqtt(), None, KEY, reqwest::Client::new(), url.clone(), storage.clone()));
    let delegations = tokio::spawn(nomi_server::delegation_worker::run(pool.clone(), mqtt(), None, KEY, reqwest::Client::new(), url.clone(), storage.clone()));
    let background = tokio::spawn(nomi_server::background_worker::run(pool.clone(), mqtt(), url.clone()));

    // A person and their chat, as the web app makes them.
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')").bind(org).bind(user).execute(&pool).await.unwrap();
    let identity = nomi_turn::bootstrap::ensure_web_channel_identity(&pool, user).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', $2, $3) RETURNING id")
        .bind(org)
        .bind(user.to_string())
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();

    // A turn.
    let mut tx = pool.begin().await.unwrap();
    let turn = nomi_turn::queue::enqueue(&mut tx, session, identity, "Hi there", Some(org)).await.unwrap();
    tx.commit().await.unwrap();
    assert!(until(&pool, "SELECT count(*) FROM turn_jobs WHERE id = $1 AND status NOT IN ('pending', 'processing')", turn).await, "the turn finished");

    // A hand-off to a crew member, and one to an agent that doesn't exist.
    let handed: Uuid = sqlx::query_scalar("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task) VALUES ($1, $2, 'supervisor', 'chitchat', 'Say hello') RETURNING id")
        .bind(session)
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    let unknown: Uuid = sqlx::query_scalar("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task) VALUES ($1, $2, 'supervisor', 'no-such-agent', 'x') RETURNING id")
        .bind(session)
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("SELECT pg_notify('agent_delegations_channel', $1)").bind(handed.to_string()).execute(&pool).await.unwrap();
    assert!(until(&pool, "SELECT count(*) FROM agent_delegations WHERE id = $1 AND status IN ('completed', 'failed', 'cancelled')", handed).await, "the hand-off finished");
    assert!(until(&pool, "SELECT count(*) FROM agent_delegations WHERE id = $1 AND status = 'failed'", unknown).await, "an unknown agent fails");

    // A background task (a command the crew asked to run).
    let task: Uuid = sqlx::query_scalar("INSERT INTO background_tasks (session_id, user_id, agent_type, label, command, timeout_seconds) VALUES ($1, $2, 'coding', 'Say hello', 'echo hello', 5) RETURNING id")
        .bind(session)
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("SELECT pg_notify('background_tasks_channel', $1)").bind(task.to_string()).execute(&pool).await.unwrap();
    let settled = until(&pool, "SELECT count(*) FROM background_tasks WHERE id = $1 AND status NOT IN ('pending', 'running')", task).await;

    turns.abort();
    delegations.abort();
    background.abort();
    assert!(settled, "the background task finished");
}
