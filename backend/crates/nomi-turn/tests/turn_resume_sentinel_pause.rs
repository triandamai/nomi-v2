use std::sync::Arc;

// Regression coverage for approvals raised by a turn with no agent_sessions row of its own —
// the default agent, and every delegated turn (the delegation worker runs the target agent with
// the chat's session_id as a sentinel agent_session_id). Before the fix, the paused state was
// written to that non-existent row, so the approval card could never be resolved: every
// Approve/Deny answered "this action is no longer pending".

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmResponse, LlmRole, StopReason};
use nomi_turn::resume_paused_turn;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{run_agent_turn, AgentRegistry, LoopOutcome, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_realtime::MqttPublisher;
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

async fn seed(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Personal', true) RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', 'w-1')")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_delegated_turn_paused_for_approval_can_be_approved_and_finishes_its_delegation(pool: PgPool) {
    let (user_id, session_id) = seed(&pool).await;
    let delegation_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) \
         VALUES ($1, $2, 'chitchat', 'money', 'how much did I spend?', 'processing') RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})), // pauses
        text_response("You spent nothing this week"),                                   // resumed reply
    ]);

    // Exactly how delegation_worker runs a delegated turn: session_id doubles as the sentinel
    // agent_session_id.
    let mut conn = pool.acquire().await.unwrap();
    let outcome = run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedder,
        &registry,
        &MoneyAgent,
        session_id,
        session_id,
        user_id,
        vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "how much did I spend?".into() }] }],
        1024,
    )
    .await
    .unwrap();
    let message_id = match outcome {
        LoopOutcome::AwaitingApproval { message_id } => message_id,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };
    drop(conn);

    // What delegation_worker does on AwaitingApproval: tag the parked state with the delegation.
    sqlx::query("UPDATE agent_sessions SET state = state || jsonb_build_object('delegation_id', $1::text) WHERE state->>'pending_approval_message_id' = $2")
        .bind(delegation_id.to_string())
        .bind(message_id.to_string())
        .execute(&pool)
        .await
        .unwrap();

    // The approval endpoint's own pending check must now see it.
    let pending: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_sessions WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true)",
    )
    .bind(message_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(pending, "the approval card must be resolvable");

    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-sentinel-{}", Uuid::new_v4()));
    let resumed = resume_paused_turn(&pool, &mqtt, None, &provider, &embedder, &registry, &catalog, message_id, "approve", false)
        .await
        .unwrap();
    assert_eq!(resumed.reply, "You spent nothing this week");

    let card_status: String = sqlx::query_scalar("SELECT content_blocks->0->>'status' FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(card_status, "approved");

    let parked_status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(parked_status, "completed", "the parked row is closed once the turn finishes");

    let (delegation_status, result): (String, Option<String>) =
        sqlx::query_as("SELECT status, result FROM agent_delegations WHERE id = $1").bind(delegation_id).fetch_one(&pool).await.unwrap();
    assert_eq!(delegation_status, "completed");
    assert_eq!(result.as_deref(), Some("You spent nothing this week"));
}
