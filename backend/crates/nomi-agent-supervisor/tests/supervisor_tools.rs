//! The supervisor's stop tool, its report of background work, and the one-shot "started" line.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::content_block::ContentBlock;
use nomi_agent_core::SubAgent;
use nomi_agent_supervisor::{phrase_delegation_started, SupervisorAgent};
use nomi_llm::{ContentBlock as LlmBlock, LlmResponse, StopReason};
use nomi_test_support::FakeLlmProvider;

struct Chat {
    pool: PgPool,
    session: Uuid,
    user: Uuid,
}

async fn chat(pool: PgPool) -> Chat {
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'c1') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    Chat { pool, session, user }
}

impl Chat {
    async fn delegate(&self, target: &str, status: &str) {
        sqlx::query("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'supervisor', $3, 'work', $4)")
            .bind(self.session)
            .bind(self.user)
            .bind(target)
            .bind(status)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    async fn stop(&self, input: serde_json::Value) -> Result<nomi_agent_core::ToolOutcome, String> {
        let mut conn = self.pool.acquire().await.unwrap();
        SupervisorAgent.execute_tool(&mut conn, self.session, Uuid::new_v4(), self.user, "stop_agents", input).await
    }
}

fn reply(text: Option<&str>) -> LlmResponse {
    LlmResponse {
        content: text.map(|t| vec![LlmBlock::Text { text: t.to_string() }]).unwrap_or_default(),
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn stopping_needs_a_known_scope_and_names_an_agent(pool: PgPool) {
    let c = chat(pool).await;
    assert_eq!(c.stop(json!({})).await.unwrap_err(), "scope must be this_chat, everything or agent");
    assert_eq!(c.stop(json!({"scope": "agent", "agent": "  "})).await.unwrap_err(), "scope \"agent\" needs the agent's name");
    // Nothing running: said plainly, no card.
    let idle = c.stop(json!({"scope": "everything"})).await.unwrap();
    assert!(idle.block.is_none());
    assert!(!idle.display_text.is_empty());
    let idle_agent = c.stop(json!({"scope": "agent", "agent": "Money agent"})).await.unwrap();
    assert!(idle_agent.display_text.contains("Money agent"), "{}", idle_agent.display_text);
}

#[sqlx::test(migrations = "../../migrations")]
async fn stopping_this_chat_cancels_its_work_and_shows_what_stopped(pool: PgPool) {
    let c = chat(pool).await;
    c.delegate("money", "pending").await;
    c.delegate("planning", "processing").await;
    let outcome = c.stop(json!({"scope": "this_chat"})).await.unwrap();
    let Some(ContentBlock::Stopped { tasks }) = outcome.block else { panic!("a card of what stopped: {:?}", outcome.block) };
    assert_eq!(tasks.len(), 2);
    let open: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_delegations WHERE session_id = $1 AND status IN ('pending', 'processing')")
        .bind(c.session)
        .fetch_one(&c.pool)
        .await
        .unwrap();
    assert_eq!(open, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_custom_agent_is_stopped_by_its_name(pool: PgPool) {
    let c = chat(pool).await;
    let agent: Uuid = sqlx::query_scalar(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, created_by) VALUES ('Scout', 'p', 'scouting', 'd', $1) RETURNING id",
    )
    .bind(c.user)
    .fetch_one(&c.pool)
    .await
    .unwrap();
    c.delegate(&agent.to_string(), "pending").await;
    c.delegate("money", "pending").await;
    let outcome = c.stop(json!({"scope": "agent", "agent": "SCOUTING"})).await.unwrap();
    let Some(ContentBlock::Stopped { tasks }) = outcome.block else { panic!("Scout's work stopped") };
    assert_eq!(tasks.len(), 1);
    let money_left: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_delegations WHERE target_agent_type = 'money' AND status = 'pending'").fetch_one(&c.pool).await.unwrap();
    assert_eq!(money_left, 1, "only the named agent stops");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_activity_report_includes_background_tasks(pool: PgPool) {
    let c = chat(pool).await;
    for (label, status, output, error) in [("Build", "completed", Some("built in 3s"), None), ("Tests", "failed", Some("  "), Some("2 failed")), ("Lint", "pending", None, Some(" "))] {
        sqlx::query("INSERT INTO background_tasks (session_id, user_id, agent_type, label, command, status, output, error) VALUES ($1, $2, 'coding', $3, 'npm run x', $4, $5, $6)")
            .bind(c.session)
            .bind(c.user)
            .bind(label)
            .bind(status)
            .bind(output)
            .bind(error)
            .execute(&c.pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, error) VALUES ($1, $2, 'supervisor', 'money', 'sum', 'failed', 'no data')")
        .bind(c.session)
        .bind(c.user)
        .execute(&c.pool)
        .await
        .unwrap();
    let mut conn = c.pool.acquire().await.unwrap();
    let report = nomi_agent_supervisor::list_recent_agent_activity(&mut conn, c.session).await.unwrap();
    assert!(report.contains("- money (failed): asked to \"sum\" — error: no data"), "{report}");
    assert!(report.contains("coding background task (completed): \"Build\" (`npm run x`) — output: built in 3s"), "{report}");
    assert!(report.contains("\"Tests\" (`npm run x`) — error: 2 failed"), "{report}");
    assert!(report.contains("(pending): \"Lint\" (`npm run x`)\n"), "blank output and error are left out: {report}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_started_line_comes_from_the_model_in_the_crew_members_voice(pool: PgPool) {
    let c = chat(pool).await;
    sqlx::query("INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'ana@example.com', 'x')").bind(c.user).execute(&c.pool).await.unwrap();
    let provider = FakeLlmProvider::success(reply(Some("On it — checking your spending now.")));
    let mut conn = c.pool.acquire().await.unwrap();
    let line = phrase_delegation_started(&provider, &mut conn, c.user, c.session, "money", "Finley", "How much on food?").await.unwrap();
    assert_eq!(line, "On it — checking your spending now.");
    let request = provider.received_requests.lock().unwrap()[0].clone();
    let system = request.system.unwrap();
    assert!(system.contains("Right now you speak as Finley"));
    assert!(system.contains("What was asked: How much on food?"));
    assert!(system.contains("You're speaking with ana@example.com."), "the login email when there's no name");
    assert!(request.tools.is_empty());

    // A username beats the email, a display name beats both.
    sqlx::query("INSERT INTO user_profiles (user_id, username) VALUES ($1, 'ana_k')").bind(c.user).execute(&c.pool).await.unwrap();
    let provider = FakeLlmProvider::success(reply(None));
    let fallback = phrase_delegation_started(&provider, &mut conn, c.user, c.session, "money", "Finley", "x").await.unwrap();
    assert!(fallback.contains("money"), "with no text back, a plain line: {fallback}");
    assert!(provider.received_requests.lock().unwrap()[0].system.as_ref().unwrap().contains("You're speaking with ana_k."));
    sqlx::query("UPDATE user_profiles SET display_name = 'Ana' WHERE user_id = $1").bind(c.user).execute(&c.pool).await.unwrap();
    let provider = FakeLlmProvider::success(reply(Some("ok")));
    phrase_delegation_started(&provider, &mut conn, c.user, c.session, "money", "Finley", "x").await.unwrap();
    assert!(provider.received_requests.lock().unwrap()[0].system.as_ref().unwrap().contains("You're speaking with Ana."));

    let failing = FakeLlmProvider::failure("overloaded");
    assert!(phrase_delegation_started(&failing, &mut conn, c.user, c.session, "money", "Finley", "x").await.unwrap_err().contains("overloaded"));
}

#[test]
fn the_supervisor_describes_itself() {
    let agent = SupervisorAgent;
    assert_eq!(agent.agent_type(), "supervisor");
    assert_eq!(agent.intent_label(), "supervisor");
    assert!(!agent.system_prompt().is_empty());
    assert!(agent.intent_description().contains("stop"));
    assert_eq!(agent.tools().iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), ["list_recent_agent_activity", "stop_agents"]);
    assert!(agent.uses_personality());
    assert!(!agent.is_delegation_target());
}
