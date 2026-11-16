//! Reading a built project for a design check: only for someone who can open the project, and
//! only its source text.

use nomi_agent_core::design_review::ProjectSource;
use nomi_agent_coding::files::project_file_key;
use nomi_server::design_project_source::StoreProjectSource;
use sqlx::PgPool;
use uuid::Uuid;

async fn project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, stack) VALUES ($1, $2, 'Bakery', 'sveltekit') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, project_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_projects_source_is_read_for_the_people_who_can_open_it(pool: PgPool) {
    let (owner, project_id) = project(&pool).await;
    let stranger: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let storage = nomi_test_support::test_project_storage();
    for (path, content) in [("src/app.css", "h1 { color: #B5532A }"), ("src/routes/+page.svelte", "<h1 data-design-id=\"hero\">Hi</h1>"), ("static/logo.png", "not text"), ("node_modules/x/index.js", "vendored")] {
        storage.put_object(&project_file_key(project_id, path), content, "text/plain").await.unwrap();
        sqlx::query("INSERT INTO project_files (project_id, path, size_bytes) VALUES ($1, $2, $3)").bind(project_id).bind(path).bind(content.len() as i32).execute(&pool).await.unwrap();
    }
    let source = StoreProjectSource::new(pool.clone(), storage);

    let files = source.files(owner, project_id).await.unwrap();
    assert_eq!(files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), ["src/app.css", "src/routes/+page.svelte"], "source text only, nothing vendored");
    assert_eq!(files[0].content, "h1 { color: #B5532A }");

    assert_eq!(source.files(stranger, project_id).await.unwrap_err(), "there is no such project, or it isn't yours to open");
    assert_eq!(source.files(owner, Uuid::new_v4()).await.unwrap_err(), "there is no such project, or it isn't yours to open");
}
