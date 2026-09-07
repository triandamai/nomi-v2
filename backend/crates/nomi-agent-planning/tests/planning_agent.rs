use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_planning::PlanningAgent;

async fn seed_user_and_session(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let chat_id = Uuid::new_v4().to_string();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(org_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_project_succeeds_and_stamps_the_calling_session(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let agent = PlanningAgent::new();
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "create_project",
            serde_json::json!({"name": "Todo app", "description": "A simple todo list"}),
        )
        .await
        .unwrap();

    let project_id: Uuid = result.display_text.parse().expect("create_project should return the new project id");
    let stored_session_id: Uuid = sqlx::query_scalar("SELECT session_id FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_session_id, session_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_project_renames_a_pre_existing_placeholder_instead_of_duplicating(pool: PgPool) {
    // The "+ Add new project" entry point creates a placeholder project row for the session
    // before any conversation happens (see routes::projects::create_project_session) — when the
    // planning agent later calls create_project with the real name, it must rename that same
    // row, not leave the placeholder behind as an orphan while inserting a second project.
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let placeholder_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'New project') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "create_project",
            serde_json::json!({"name": "Todo app", "description": "A simple todo list"}),
        )
        .await
        .unwrap();

    let project_id: Uuid = result.display_text.parse().unwrap();
    assert_eq!(project_id, placeholder_id, "should rename the existing row, not create a new one");

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM projects WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "must not leave an orphaned placeholder behind");

    let name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    assert_eq!(name, "Todo app");
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_updates_an_existing_project(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "write_plan",
            serde_json::json!({"project_id": project_id.to_string(), "plan": "1. index.html\n2. app.js"}),
        )
        .await
        .unwrap();
    assert_eq!(result.display_text, "plan saved");

    let stored_plan: Option<String> = sqlx::query_scalar("SELECT plan FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_plan.as_deref(), Some("1. index.html\n2. app.js"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_for_another_users_project_is_rejected(pool: PgPool) {
    let (owner_id, session_id) = seed_user_and_session(&pool).await;
    let (other_user_id, _) = seed_user_and_session(&pool).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(owner_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            other_user_id,
            "write_plan",
            serde_json::json!({"project_id": project_id.to_string(), "plan": "malicious overwrite"}),
        )
        .await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let agent = PlanningAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let result = agent.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "delete_everything", serde_json::json!({})).await;
    assert!(result.is_err());
}
