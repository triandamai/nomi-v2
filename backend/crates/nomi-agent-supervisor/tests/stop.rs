use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, SubAgent};
use nomi_agent_money::MoneyAgent;
use nomi_agent_supervisor::stop::handle_stop_message;
use nomi_agent_supervisor::SupervisorAgent;

struct Seed {
    user_id: Uuid,
    identity_id: Uuid,
    chat_a: Uuid,
    chat_b: Uuid,
}

async fn seed(pool: &PgPool) -> Seed {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let identity_id: Uuid =
        sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', 'w-1') RETURNING id")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let mut chats = Vec::new();
    for chat in ["chat-a", "chat-b"] {
        let id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id")
            .bind(org_id)
            .bind(chat)
            .fetch_one(pool)
            .await
            .unwrap();
        chats.push(id);
    }
    Seed { user_id, identity_id, chat_a: chats[0], chat_b: chats[1] }
}

/// A Money agent in `session_id`, paused on an approval card. Returns (agent_session_id, card id).
async fn seed_paused_money_agent(pool: &PgPool, seed: &Seed, session_id: Uuid) -> (Uuid, Uuid) {
    let card_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, content, content_blocks) VALUES ($1, 'Needs your OK', \
         '[{\"type\": \"approval\", \"status\": \"pending\", \"tool_name\": \"list_transactions\", \"description\": \"x\"}]') RETURNING id",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, state) \
         VALUES ($1, $2, 'money', 'active', jsonb_build_object('paused_for_approval', true, 'pending_approval_message_id', $3::text)) RETURNING id",
    )
    .bind(session_id)
    .bind(seed.identity_id)
    .bind(card_id.to_string())
    .fetch_one(pool)
    .await
    .unwrap();
    (agent_session_id, card_id)
}

async fn seed_delegation(pool: &PgPool, seed: &Seed, session_id: Uuid, target: &str, status: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) \
         VALUES ($1, $2, 'chitchat', $3, 'do a thing', $4) RETURNING id",
    )
    .bind(session_id)
    .bind(seed.user_id)
    .bind(target)
    .bind(status)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_turn_job(pool: &PgPool, seed: &Seed, session_id: Uuid, status: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO turn_jobs (session_id, sender_channel_identity_id, text, status) VALUES ($1, $2, 'hi', $3) RETURNING id")
        .bind(session_id)
        .bind(seed.identity_id)
        .bind(status)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn status_of(pool: &PgPool, table: &str, id: Uuid) -> String {
    sqlx::query_scalar(&format!("SELECT status FROM {table} WHERE id = $1")).bind(id).fetch_one(pool).await.unwrap()
}

fn registry() -> AgentRegistry {
    AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(SupervisorAgent)])
}

#[sqlx::test(migrations = "../../migrations")]
async fn stop_in_a_chat_stops_only_that_chats_work(pool: PgPool) {
    let seed = seed(&pool).await;
    let (money_here, card) = seed_paused_money_agent(&pool, &seed, seed.chat_a).await;
    let (money_elsewhere, _) = seed_paused_money_agent(&pool, &seed, seed.chat_b).await;
    let delegation_here = seed_delegation(&pool, &seed, seed.chat_a, "coding", "processing").await;
    let queued_here = seed_turn_job(&pool, &seed, seed.chat_a, "pending").await;
    let queued_elsewhere = seed_turn_job(&pool, &seed, seed.chat_b, "pending").await;

    let outcome = handle_stop_message(&pool, &registry(), seed.chat_a, seed.identity_id, seed.user_id, "Stop!")
        .await
        .unwrap()
        .expect("a stop command");

    assert_eq!(outcome.report.stopped, vec!["Money".to_string(), "Coding".to_string()]);
    assert_eq!(outcome.report.dropped_messages, 1);
    assert!(outcome.reply.starts_with("Stopped Money and Coding."), "{}", outcome.reply);

    assert_eq!(status_of(&pool, "agent_sessions", money_here).await, "cancelled");
    assert_eq!(status_of(&pool, "agent_sessions", money_elsewhere).await, "active");
    assert_eq!(status_of(&pool, "agent_delegations", delegation_here).await, "cancelled");
    assert_eq!(status_of(&pool, "turn_jobs", queued_here).await, "cancelled");
    assert_eq!(status_of(&pool, "turn_jobs", queued_elsewhere).await, "pending");

    // The approval card is settled, and the approval endpoint's own pending check now says no.
    let card_status: String = sqlx::query_scalar("SELECT content_blocks->0->>'status' FROM messages WHERE id = $1")
        .bind(card)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(card_status, "cancelled");
    let still_pending: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_sessions WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true)",
    )
    .bind(card.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!still_pending);

    // Both the command and the supervisor's reply are in the chat, in that order.
    let messages: Vec<(Option<String>, String)> =
        sqlx::query_as("SELECT agent_display_name, content FROM messages WHERE session_id = $1 AND content_blocks IS NULL ORDER BY created_at")
            .bind(seed.chat_a)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(messages[0], (None, "Stop!".to_string()));
    assert_eq!(messages[1].0.as_deref(), Some("Supervisor"));

    // Turns already running see the stop.
    let requests: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_stop_requests WHERE user_id = $1 AND session_id = $2")
        .bind(seed.user_id)
        .bind(seed.chat_a)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(requests, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn stop_all_agents_stops_every_chat_and_counts_nomi_at_work(pool: PgPool) {
    let seed = seed(&pool).await;
    let (money_a, _) = seed_paused_money_agent(&pool, &seed, seed.chat_a).await;
    // Nomi (no agent session) is mid-turn in chat B.
    let running_b = seed_turn_job(&pool, &seed, seed.chat_b, "processing").await;

    let outcome = handle_stop_message(&pool, &registry(), seed.chat_a, seed.identity_id, seed.user_id, "stop all agents")
        .await
        .unwrap()
        .expect("a stop command");

    assert_eq!(outcome.report.stopped, vec!["Money".to_string(), "Nomi".to_string()]);
    assert_eq!(outcome.report.chats, 2);
    assert!(outcome.reply.starts_with("Stopped Money and Nomi across 2 chats."), "{}", outcome.reply);
    assert_eq!(status_of(&pool, "agent_sessions", money_a).await, "cancelled");
    // The running job itself finishes on its own once its turn sees the stop request.
    assert_eq!(status_of(&pool, "turn_jobs", running_b).await, "processing");
    let user_wide: bool = sqlx::query_scalar("SELECT session_id IS NULL AND agent_type IS NULL FROM agent_stop_requests WHERE user_id = $1")
        .bind(seed.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(user_wide);
}

#[sqlx::test(migrations = "../../migrations")]
async fn naming_an_agent_stops_only_that_agent_and_keeps_queued_messages(pool: PgPool) {
    let seed = seed(&pool).await;
    let (money, _) = seed_paused_money_agent(&pool, &seed, seed.chat_b).await;
    let coding = seed_delegation(&pool, &seed, seed.chat_a, "coding", "pending").await;
    let queued = seed_turn_job(&pool, &seed, seed.chat_a, "pending").await;

    let outcome = handle_stop_message(&pool, &registry(), seed.chat_a, seed.identity_id, seed.user_id, "please stop the money agent")
        .await
        .unwrap()
        .expect("a stop command");

    assert_eq!(outcome.report.stopped, vec!["Money".to_string()]);
    assert_eq!(status_of(&pool, "agent_sessions", money).await, "cancelled");
    assert_eq!(status_of(&pool, "agent_delegations", coding).await, "pending");
    assert_eq!(status_of(&pool, "turn_jobs", queued).await, "pending");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_ordinary_message_or_an_idle_crew_is_handled_sensibly(pool: PgPool) {
    let seed = seed(&pool).await;

    let none = handle_stop_message(&pool, &registry(), seed.chat_a, seed.identity_id, seed.user_id, "stop telling me jokes").await.unwrap();
    assert!(none.is_none(), "not a command, so it is left for the normal queue");
    let written: i64 = sqlx::query_scalar("SELECT count(*) FROM messages").fetch_one(&pool).await.unwrap();
    assert_eq!(written, 0);

    let idle = handle_stop_message(&pool, &registry(), seed.chat_a, seed.identity_id, seed.user_id, "stop").await.unwrap().unwrap();
    assert_eq!(idle.reply, "Nothing is running right now, so there's nothing to stop.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_supervisor_tool_stops_agents_but_not_its_own_turn(pool: PgPool) {
    let seed = seed(&pool).await;
    let (money, _) = seed_paused_money_agent(&pool, &seed, seed.chat_a).await;
    // The supervisor's own turn is the processing job in this chat; it must not count itself.
    seed_turn_job(&pool, &seed, seed.chat_a, "processing").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = SupervisorAgent
        .execute_tool(&mut conn, seed.chat_a, Uuid::new_v4(), seed.user_id, "stop_agents", serde_json::json!({"scope": "this_chat"}))
        .await
        .unwrap();

    assert!(result.display_text.starts_with("Stopped Money."), "{}", result.display_text);
    assert_eq!(status_of(&pool, "agent_sessions", money).await, "cancelled");
    let exempt: Option<String> = sqlx::query_scalar("SELECT exempt_agent_type FROM agent_stop_requests WHERE user_id = $1")
        .bind(seed.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(exempt.as_deref(), Some("supervisor"));
}
