//! Rena plans anything (trips, schedules, study plans) and, for apps, works with Koda. These pin
//! that general planning stays as it is while the app-building playbook only comes with build
//! requests and project chats.

use nomi_agent_core::SubAgent;
use nomi_agent_planning::PlanningAgent;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use sqlx::PgPool;
use uuid::Uuid;

fn said(role: LlmRole, text: &str) -> LlmMessage {
    LlmMessage { role, content: vec![ContentBlock::Text { text: text.to_string() }] }
}

async fn chat(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap()
}

#[test]
fn rena_still_plans_anything_with_the_same_tools() {
    let prompt = PlanningAgent::new().system_prompt();
    for kept in ["trips and itineraries", "schedules", "write_plan", "'- [ ]' checklist items", "question per card"] {
        assert!(prompt.contains(kept), "general planning keeps {kept:?}");
    }
    // Nothing is taken away: every tool she had is still offered, in every chat.
    let tools: Vec<String> = PlanningAgent::new().tools().into_iter().map(|t| t.name).collect();
    assert_eq!(tools, ["create_project", "publish_project"]);
}

#[test]
fn renas_own_prompt_no_longer_carries_the_app_playbook() {
    let prompt = PlanningAgent::new().system_prompt();
    assert!(!prompt.contains("delegate_to_agent"), "the playbook comes only with app requests");
    assert!(!prompt.contains("Project <project_id>"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_trip_plan_gets_no_app_building_playbook(pool: PgPool) {
    let session_id = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let messages = vec![said(LlmRole::User, "Plan a 3-day trip to Bali for two, mid-range budget")];
    assert_eq!(PlanningAgent::new().domain_context(&mut conn, None, session_id, Uuid::nil(), &messages).await, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_app_request_gets_the_playbook(pool: PgPool) {
    let session_id = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let messages = vec![said(LlmRole::User, "Can you build me a website for my bakery?")];
    let playbook = PlanningAgent::new().domain_context(&mut conn, None, session_id, Uuid::nil(), &messages).await.expect("a build request");
    for step in ["create_project", "write_plan", "ask_user", "delegate_to_agent", "'Project <project_id>: '"] {
        assert!(playbook.contains(step), "the playbook still says {step:?}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_playbook_stays_for_the_rest_of_an_app_conversation(pool: PgPool) {
    // A short answer to her questions ("Editorial") doesn't look like a build request on its own.
    let session_id = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let messages = vec![
        said(LlmRole::User, "make me a landing page for my yoga studio"),
        said(LlmRole::Assistant, "A few questions first."),
        said(LlmRole::User, "Editorial"),
    ];
    assert!(PlanningAgent::new().domain_context(&mut conn, None, session_id, Uuid::nil(), &messages).await.is_some());

    // In a project chat it's always there, whatever was said.
    let project_chat = chat(&pool).await;
    sqlx::query(
        "WITH u AS (INSERT INTO users DEFAULT VALUES RETURNING id) \
         INSERT INTO projects (user_id, session_id, org_id, name) SELECT u.id, s.id, s.org_id, 'Studio' FROM u, sessions s WHERE s.id = $1",
    )
    .bind(project_chat)
    .execute(&pool)
    .await
    .unwrap();
    let messages = vec![said(LlmRole::User, "Editorial")];
    assert!(PlanningAgent::new().domain_context(&mut conn, None, project_chat, Uuid::nil(), &messages).await.is_some());
}
