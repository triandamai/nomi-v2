//! A person's own crew in the turn: a switched-off member is passed by (and the person is told),
//! each member's permission levels decide whether a tool runs, asks or is refused, and the
//! member's soul.md joins its instructions.

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};
use nomi_turn::handle_inbound_message;

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text {
            text: text.to_string(),
        }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse {
            id: id.to_string(),
            name: name.to_string(),
            input,
            thought_signature: None,
        }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn registry() -> AgentRegistry {
    AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)])
}

async fn seed_speaker(pool: &PgPool, channel_user_id: &str) -> Uuid {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (name, is_personal) VALUES ('Personal', true) RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(org_id)
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2)")
        .bind(user_id)
        .bind(channel_user_id)
        .execute(pool)
        .await
        .unwrap();
    user_id
}

async fn switch_off(pool: &PgPool, user_id: Uuid, agent_type: &str) {
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::personal_crew::set_enabled(&mut conn, user_id, agent_type, false)
        .await
        .unwrap();
}

async fn set_level(
    pool: &PgPool,
    user_id: Uuid,
    tool: &str,
    level: nomi_agent_core::personal_crew::Level,
) {
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::personal_crew::set_levels(
        &mut conn,
        user_id,
        "money",
        &[(tool.to_string(), level)],
    )
    .await
    .unwrap();
}

async fn reply_from(pool: &PgPool, message_id: Option<Uuid>) -> Option<String> {
    sqlx::query_scalar("SELECT agent_display_name FROM messages WHERE id = $1")
        .bind(message_id.unwrap())
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn notices(pool: &PgPool, session_id: Uuid) -> Vec<String> {
    sqlx::query_scalar("SELECT content FROM messages WHERE session_id = $1 AND content LIKE '%switched off%' ORDER BY created_at")
        .bind(session_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

/// The last tool call's (result, is_error), as the turn recorded it.
async fn last_tool_call(pool: &PgPool, session_id: Uuid) -> (String, bool) {
    sqlx::query_as(
        "SELECT payload->>'result', (payload->>'is_error')::boolean FROM agent_events \
         WHERE session_id = $1 AND event_type = 'ToolCalled' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_router_passes_a_switched_off_member_by(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    switch_off(&pool, user_id, "money").await;
    // The classifier still answers "money": Nomi takes it all the same.
    let provider = FakeLlmProvider::sequence(vec![text_response("money"), text_response("Sure.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "how much did I spend?",
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        reply_from(&pool, outcome.message_id).await,
        Some("Nomi".to_string())
    );
    let classifier_prompt = provider.received_requests.lock().unwrap()[0]
        .system
        .clone()
        .unwrap();
    assert!(
        !classifier_prompt.contains("money:"),
        "a switched-off member is never offered to the classifier"
    );
    let sessions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE agent_type = 'money'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(sessions, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn naming_a_switched_off_member_says_so_and_nomi_answers(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    switch_off(&pool, user_id, "money").await;
    let provider =
        FakeLlmProvider::sequence(vec![text_response("chitchat"), text_response("Sure.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "@dana coffee 25k",
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        reply_from(&pool, outcome.message_id).await,
        Some("Nomi".to_string())
    );
    let notices = notices(&pool, outcome.session_id).await;
    assert_eq!(notices.len(), 1);
    assert!(
        notices[0].starts_with("Dana is switched off"),
        "{notices:?}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn switching_off_the_member_in_the_conversation_ends_its_session(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    let provider = FakeLlmProvider::sequence(vec![
        text_response("Logged."),
        text_response("chitchat"),
        text_response("Sure."),
    ]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "@dana coffee 25k",
        None,
    )
    .await
    .unwrap();
    switch_off(&pool, user_id, "money").await;
    let outcome = handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "and tea 10k",
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        reply_from(&pool, outcome.message_id).await,
        Some("Nomi".to_string())
    );
    let active: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(active, 0);
    assert_eq!(notices(&pool, outcome.session_id).await.len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_set_to_never_is_refused_without_a_card(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    set_level(
        &pool,
        user_id,
        "list_transactions",
        nomi_agent_core::personal_crew::Level::Never,
    )
    .await;
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("I can't look at those."),
    ]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "how much did I spend?",
        None,
    )
    .await
    .unwrap();

    assert_eq!(outcome.reply, "I can't look at those.");
    let (result, is_error) = last_tool_call(&pool, outcome.session_id).await;
    assert!(is_error);
    assert_eq!(result, nomi_agent_core::prompts::MEMBER_TOOL_NEVER_RESULT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_set_to_allow_runs_without_a_card(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    set_level(
        &pool,
        user_id,
        "list_transactions",
        nomi_agent_core::personal_crew::Level::Allow,
    )
    .await;
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("Here it is."),
    ]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "how much did I spend?",
        None,
    )
    .await
    .unwrap();

    assert_eq!(outcome.reply, "Here it is.");
    let (_, is_error) = last_tool_call(&pool, outcome.session_id).await;
    assert!(!is_error);
}

#[sqlx::test(migrations = "../../migrations")]
async fn ask_first_brings_the_approval_card_even_over_an_always_allow_rule(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'list_transactions', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    set_level(
        &pool,
        user_id,
        "list_transactions",
        nomi_agent_core::personal_crew::Level::Ask,
    )
    .await;
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
    ]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "how much did I spend?",
        None,
    )
    .await
    .unwrap();

    assert_eq!(outcome.reply, "Waiting for approval.");
    let paused: Option<bool> = sqlx::query_scalar(
        "SELECT (state->>'paused_for_approval')::boolean FROM agent_sessions WHERE session_id = $1",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(paused, Some(true));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_members_soul_joins_its_instructions(pool: PgPool) {
    let user_id = seed_speaker(&pool, "tg-1").await;
    {
        let mut conn = pool.acquire().await.unwrap();
        nomi_agent_core::personal_crew::save_soul(
            &mut conn,
            user_id,
            "money",
            "Always answer in rupiah.",
        )
        .await
        .unwrap();
    }
    let provider = FakeLlmProvider::sequence(vec![text_response("Logged.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    handle_inbound_message(
        &pool,
        None,
        &provider,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "@dana coffee 25k",
        None,
    )
    .await
    .unwrap();

    let system = provider.received_requests.lock().unwrap()[0]
        .system
        .clone()
        .unwrap();
    assert!(
        system.contains("Always answer in rupiah."),
        "the soul.md is in Dana's instructions"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_persons_own_member_is_routable_only_for_them(pool: PgPool) {
    let owner = seed_speaker(&pool, "tg-1").await;
    seed_speaker(&pool, "tg-2").await;
    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, created_by, owner_user_id) \
         VALUES ('Chef', 'You cook.', 'chefabc', 'cooking and recipes', '{}', $1, $1)",
    )
    .bind(owner)
    .execute(&pool)
    .await
    .unwrap();
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let mine = FakeLlmProvider::sequence(vec![text_response("Pasta.")]);
    let outcome = handle_inbound_message(
        &pool,
        None,
        &mine,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "@chef dinner?",
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        reply_from(&pool, outcome.message_id).await,
        Some("Chef".to_string())
    );

    // Someone else naming "Chef" is just talking to Nomi.
    let theirs = FakeLlmProvider::sequence(vec![text_response("chefabc"), text_response("Sure.")]);
    let outcome = handle_inbound_message(
        &pool,
        None,
        &theirs,
        &embedder,
        &registry(),
        &catalog,
        "telegram",
        "dm",
        "chat-2",
        "tg-2",
        "@chef dinner?",
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        reply_from(&pool, outcome.message_id).await,
        Some("Nomi".to_string())
    );
    let classifier_prompt = theirs.received_requests.lock().unwrap()[0]
        .system
        .clone()
        .unwrap();
    assert!(!classifier_prompt.contains("chefabc"));
}
