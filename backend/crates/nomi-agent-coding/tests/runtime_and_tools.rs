use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::runtime::{self, NOT_OPEN};
use nomi_agent_coding::{execute, CODING_AGENT_TYPE};
use nomi_storage::ProjectStore;

fn storage() -> ProjectStore {
    ProjectStore::at(std::env::temp_dir().join(format!("nomi-koda-test-{}", Uuid::new_v4())))
}

/// (user, session, project)
async fn seed(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Tasks') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, session_id, project_id)
}

async fn files_version(pool: &PgPool, project_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn new_projects_default_to_sveltekit_and_koda_sees_the_stack(pool: PgPool) {
    let (user_id, _, project_id) = seed(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let out = execute(&mut conn, &storage(), user_id, "list_files", json!({ "project_id": project_id.to_string() })).await.unwrap();
    assert!(out.display_text.starts_with("Stack: sveltekit"), "{}", out.display_text);
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_many_files_then_editing_one_bumps_the_version_each_time(pool: PgPool) {
    let (user_id, _, project_id) = seed(&pool).await;
    let store = storage();
    let mut conn = pool.acquire().await.unwrap();
    let pid = project_id.to_string();

    let out = execute(
        &mut conn,
        &store,
        user_id,
        "write_files",
        json!({ "project_id": pid, "files": [
            { "path": "package.json", "content": "{\"name\":\"app\"}" },
            { "path": "src/routes/+page.svelte", "content": "<h1>Hello</h1>\n<p>Hello again</p>\n" }
        ]}),
    )
    .await
    .unwrap();
    assert!(out.display_text.contains("Wrote 2 files"));
    assert_eq!(files_version(&pool, project_id).await, 1);

    // Ambiguous, then unique.
    let err = execute(&mut conn, &store, user_id, "edit_file", json!({ "project_id": pid, "path": "src/routes/+page.svelte", "old_string": "Hello", "new_string": "Hi" }))
        .await
        .unwrap_err();
    assert!(err.contains("2 times"), "{err}");
    execute(&mut conn, &store, user_id, "edit_file", json!({ "project_id": pid, "path": "src/routes/+page.svelte", "old_string": "<h1>Hello</h1>", "new_string": "<h1>Tasks</h1>" }))
        .await
        .unwrap();
    let content = execute(&mut conn, &store, user_id, "read_file", json!({ "project_id": pid, "path": "src/routes/+page.svelte" })).await.unwrap();
    assert_eq!(content.display_text, "<h1>Tasks</h1>\n<p>Hello again</p>\n");
    assert_eq!(files_version(&pool, project_id).await, 2);

    let found = execute(&mut conn, &store, user_id, "search_files", json!({ "project_id": pid, "query": "hello AGAIN" })).await.unwrap();
    assert_eq!(found.display_text, "src/routes/+page.svelte:2: <p>Hello again</p>");

    let guide = execute(&mut conn, &store, user_id, "read_guide", json!({ "topic": "database" })).await.unwrap();
    assert!(guide.display_text.contains("PGlite"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_batch_with_a_bad_path_writes_nothing(pool: PgPool) {
    let (user_id, _, project_id) = seed(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let err = execute(
        &mut conn,
        &storage(),
        user_id,
        "write_files",
        json!({ "project_id": project_id.to_string(), "files": [
            { "path": "ok.ts", "content": "" },
            { "path": "../escape.ts", "content": "" }
        ]}),
    )
    .await
    .unwrap_err();
    assert!(err.contains("../escape.ts"), "{err}");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_files").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn commands_wait_for_an_open_project_page(pool: PgPool) {
    let (user_id, _, project_id) = seed(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let input = json!({ "project_id": project_id.to_string(), "command": "npm run check" });

    // Nobody has the project open.
    let out = runtime::run_command(&mut conn, user_id, input.clone()).await.unwrap();
    assert_eq!(out, NOT_OPEN);

    // The page checks in, then answers whatever Koda asks it to run.
    let mut page = pool.acquire().await.unwrap();
    runtime::check_in(&mut page, project_id).await.unwrap();
    let page_pool = pool.clone();
    let page_task = tokio::spawn(async move {
        let mut page = page_pool.acquire().await.unwrap();
        loop {
            let (_, runs) = runtime::check_in(&mut page, project_id).await.unwrap();
            if let Some(run) = runs.first() {
                assert_eq!((run.command.as_str(), run.args.as_slice()), ("npm", &["run".to_string(), "check".to_string()][..]));
                runtime::finish_run(&mut page, project_id, run.id, Some(1), "src/routes/+page.svelte:3 Type error").await.unwrap();
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    let out = runtime::run_command(&mut conn, user_id, input).await.unwrap();
    page_task.await.unwrap();
    assert_eq!(out, "$ npm run check\nexit code 1\nsrc/routes/+page.svelte:3 Type error");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_check_goes_to_koda_once_per_version(pool: PgPool) {
    let (_, session_id, project_id) = seed(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    // Passing checks and failures of files that changed since aren't handed over.
    assert_eq!(runtime::record_check(&mut conn, project_id, 0, true, "ok").await.unwrap(), None);
    sqlx::query("UPDATE projects SET files_version = 3 WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
    assert_eq!(runtime::record_check(&mut conn, project_id, 2, false, "old error").await.unwrap(), None);

    let fix = runtime::record_check(&mut conn, project_id, 3, false, "Cannot find module 'zod'").await.unwrap().unwrap();
    assert_eq!(fix.session_id, session_id);
    assert!(fix.task.starts_with(&format!("Project {project_id}:")) && fix.task.contains("Cannot find module 'zod'"));
    // Again for the same version: already handed over.
    assert_eq!(runtime::record_check(&mut conn, project_id, 3, false, "Cannot find module 'zod'").await.unwrap(), None);

    // Koda already on it: not handed over again.
    sqlx::query("UPDATE projects SET files_version = 4 WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'planning', $3, 'Project x: build', 'processing')")
        .bind(session_id)
        .bind(user_id)
        .bind(CODING_AGENT_TYPE)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(runtime::record_check(&mut conn, project_id, 4, false, "still broken").await.unwrap(), None);
}
