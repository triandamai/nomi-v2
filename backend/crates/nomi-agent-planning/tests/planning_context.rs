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
    for kept in ["trips and itineraries", "schedules", "write_plan", "'- [ ]' checklist items", "in one card"] {
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

/// The `design` setting is read through the process-wide settings cache and pool, so the tests
/// that set it take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Switches the design workflow on for everyone, or off, as the admin page would.
async fn design(pool: &PgPool, on: bool) {
    let key: String = nomi_test_support::TEST_SETTINGS_KEY.iter().map(|b| format!("{b:02x}")).collect();
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", key);
    nomi_settings::config::install(pool.clone());
    let admin: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let values = [("enabled".to_string(), if on { "on" } else { "off" }.to_string())].into_iter().collect();
    nomi_settings::config::save(pool, &nomi_test_support::TEST_SETTINGS_KEY, &nomi_settings::config::DESIGN, &values, admin).await.unwrap();
    nomi_settings::config::forget();
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_design_an_app_request_gets_exactly_the_playbook_it_always_did(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    design(&pool, false).await;
    let session_id = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let messages = vec![said(LlmRole::User, "Can you build me a website for my bakery?")];
    let context = PlanningAgent::new().domain_context(&mut conn, None, session_id, Uuid::new_v4(), &messages).await;
    assert_eq!(context.as_deref(), Some(nomi_agent_core::prompts::PLANNING_APP_PLAYBOOK));
}

#[sqlx::test(migrations = "../../migrations")]
async fn with_design_an_app_request_also_gets_discovery_and_what_this_chat_already_asked(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    design(&pool, true).await;
    let session_id = chat(&pool).await;
    let other_chat = chat(&pool).await;
    for (chat, question) in [(session_id, "Who is the bakery for?"), (other_chat, "Asked somewhere else?")] {
        sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) VALUES ($1, NULL, $2, $3, 'Rena')")
            .bind(chat)
            .bind(question)
            .bind(serde_json::json!([{"kind": "questions", "intro": null, "questions": [{"question": question, "options": []}]}]))
            .execute(&pool)
            .await
            .unwrap();
    }
    let mut conn = pool.acquire().await.unwrap();
    let messages = vec![said(LlmRole::User, "Can you build me a website for my bakery?")];
    let context = PlanningAgent::new().domain_context(&mut conn, None, session_id, Uuid::new_v4(), &messages).await.expect("a build request");
    assert!(context.starts_with(nomi_agent_core::prompts::PLANNING_APP_PLAYBOOK));
    assert!(context.contains(nomi_agent_core::prompts::PLANNING_DESIGN_DISCOVERY));
    assert!(context.contains("- Who is the bakery for?") && !context.contains("Asked somewhere else?"));

    // A trip is still planned as a trip.
    let trip = vec![said(LlmRole::User, "Plan a 3-day trip to Bali for two")];
    assert_eq!(PlanningAgent::new().domain_context(&mut conn, None, chat(&pool).await, Uuid::new_v4(), &trip).await, None);
    drop(conn);
    design(&pool, false).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn rena_has_the_brief_tool_only_where_design_is_on_and_only_for_an_app(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let session_id = chat(&pool).await;
    let app = vec![said(LlmRole::User, "Can you build me a website for my bakery?")];
    let trip = vec![said(LlmRole::User, "Plan a 3-day trip to Bali for two")];
    let names = |tools: Vec<nomi_llm::ToolDefinition>| tools.into_iter().map(|t| t.name).collect::<Vec<_>>();

    design(&pool, false).await;
    let mut conn = pool.acquire().await.unwrap();
    assert!(PlanningAgent::new().chat_tools(&mut conn, session_id, Uuid::new_v4(), &app).await.is_empty());
    drop(conn);

    design(&pool, true).await;
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(names(PlanningAgent::new().chat_tools(&mut conn, session_id, Uuid::new_v4(), &app).await), ["save_design_brief"]);
    assert!(PlanningAgent::new().chat_tools(&mut conn, session_id, Uuid::new_v4(), &trip).await.is_empty());
    assert!(!PlanningAgent::new().tool_needs_approval("save_design_brief"));
    drop(conn);
    design(&pool, false).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn saving_a_brief_keeps_versions_and_refuses_one_that_is_not_valid(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    design(&pool, true).await;
    let session_id = chat(&pool).await;
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let rena = PlanningAgent::new();
    let brief = serde_json::json!({
        "company": [{"value": "Roti Pagi", "source": "confirmed"}],
        "audience": [{"value": "Neighbours", "source": "assumed"}],
        "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
    });

    let first = rena.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_design_brief", brief.clone()).await.unwrap();
    assert!(first.display_text.starts_with("Design brief saved as version 1."));
    let second = rena.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_design_brief", brief).await.unwrap();
    assert!(second.display_text.starts_with("Design brief saved as version 2."));

    let invalid = serde_json::json!({"company": [{"value": "Roti Pagi", "source": "confirmed"}]});
    let refused = rena.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_design_brief", invalid).await.unwrap_err();
    assert!(refused.starts_with("the brief needs audience and objectives"));
    let (version, _) = nomi_agent_core::design_brief::latest(&mut conn, session_id).await.unwrap().unwrap();
    assert_eq!(version, 2, "a refused brief saves nothing");
    drop(conn);

    // With design off the tool isn't there to call.
    design(&pool, false).await;
    let mut conn = pool.acquire().await.unwrap();
    let off = rena.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_design_brief", serde_json::json!({})).await.unwrap_err();
    assert_eq!(off, "unknown tool: save_design_brief");
}
