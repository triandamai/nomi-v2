use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::CodingAgent;
use nomi_agent_core::{ContentBlock, SubAgent};
use nomi_storage::LocalFsStore;

async fn seed_project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let chat_id = Uuid::new_v4().to_string();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(org_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Test project') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, project_id)
}

fn temp_storage() -> LocalFsStore {
    let dir = std::env::temp_dir().join(format!("nomi-coding-test-{}", Uuid::new_v4()));
    LocalFsStore::at(dir)
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_a_new_file_produces_a_file_write_block_with_no_previous_content(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    let outcome = agent
        .execute_tool(
            &mut conn,
            Uuid::new_v4(),
            Uuid::new_v4(),
            user_id,
            "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": "<h1>hi</h1>"}),
        )
        .await
        .unwrap();

    match outcome.block {
        Some(ContentBlock::FileWrite { path, content, previous_content, .. }) => {
            assert_eq!(path, "index.html");
            assert_eq!(content, "<h1>hi</h1>");
            assert_eq!(previous_content, None);
        }
        other => panic!("expected a FileWrite block, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn overwriting_an_existing_file_captures_its_previous_content_for_the_diff(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    let input = |content: &str| {
        serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": content})
    };

    agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file", input("<h1>old</h1>")).await.unwrap();

    let outcome = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file", input("<h1>new</h1>"))
        .await
        .unwrap();

    match outcome.block {
        Some(ContentBlock::FileWrite { content, previous_content, .. }) => {
            assert_eq!(content, "<h1>new</h1>");
            assert_eq!(previous_content, Some("<h1>old</h1>".to_string()));
        }
        other => panic!("expected a FileWrite block, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_file_produces_a_file_delete_block(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    agent
        .execute_tool(
            &mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "old.js", "content": "x"}),
        )
        .await
        .unwrap();

    let outcome = agent
        .execute_tool(
            &mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "delete_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "old.js"}),
        )
        .await
        .unwrap();

    match outcome.block {
        Some(ContentBlock::FileDelete { path, .. }) => assert_eq!(path, "old.js"),
        other => panic!("expected a FileDelete block, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn read_file_and_list_files_never_produce_a_block(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    agent
        .execute_tool(
            &mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "a.txt", "content": "x"}),
        )
        .await
        .unwrap();

    let read_outcome = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "read_file", serde_json::json!({"project_id": project_id.to_string(), "path": "a.txt"}))
        .await
        .unwrap();
    assert!(read_outcome.block.is_none());

    let list_outcome = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_files", serde_json::json!({"project_id": project_id.to_string()}))
        .await
        .unwrap();
    assert!(list_outcome.block.is_none());
}
