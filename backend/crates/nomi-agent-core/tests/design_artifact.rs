//! A chat's design previews in storage: every save is a new version with its own files, old
//! versions stay as they were, and a version remembers where it was published.

use nomi_agent_core::design_artifact::{self, DesignArtifact};
use nomi_agent_core::design_draft;
use nomi_agent_core::static_publish::{self, Published};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn chat(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    (session_id, user_id)
}

fn preview(direction: &str, heading: &str) -> DesignArtifact {
    let html = format!("<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body><h1 data-design-id=\"hero\">{heading}</h1></body></html>");
    DesignArtifact::parse(&json!({
        "direction_id": direction,
        "title": heading,
        "files": [{"path": "index.html", "content": html}, {"path": "styles.css", "content": "h1 { margin: 0 }"}]
    }))
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_save_is_a_new_version_with_its_own_files(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let (other_chat, _) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(design_artifact::latest(&mut conn, session_id).await.unwrap(), None);

    let first = preview("editorial", "Fresh at six");
    let second = preview("swiss", "Bread, daily");
    let third = preview("editorial", "Fresh at six, every day");
    assert_eq!(design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, Some(1), &first).await.unwrap(), 1);
    assert_eq!(design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, Some(1), &second).await.unwrap(), 2);
    assert_eq!(design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &third).await.unwrap(), 3);
    assert_eq!(design_artifact::save(&mut conn, other_chat, Uuid::new_v4(), user_id, None, &first).await.unwrap(), 1, "versions are per chat");

    assert_eq!(design_artifact::latest(&mut conn, session_id).await.unwrap(), Some((3, third.manifest.clone())));
    assert_eq!(design_artifact::latest_of_direction(&mut conn, session_id, "swiss").await.unwrap(), Some((2, second.manifest.clone())));
    assert_eq!(design_artifact::latest_of_direction(&mut conn, session_id, "brutalist").await.unwrap(), None);
    assert_eq!(design_artifact::version(&mut conn, session_id, 1).await.unwrap(), Some(first.manifest.clone()), "the first version is untouched");
    assert_eq!(design_artifact::version(&mut conn, session_id, 4).await.unwrap(), None);

    assert_eq!(design_artifact::files(&mut conn, session_id, 1).await.unwrap(), first.files);
    let (content_type, content) = design_artifact::file(&mut conn, session_id, 3, "index.html").await.unwrap().expect("the page");
    assert_eq!(content_type, "text/html; charset=utf-8");
    assert!(content.contains("Fresh at six, every day"));
    assert_eq!(design_artifact::file(&mut conn, session_id, 3, "nope.css").await.unwrap(), None);
    assert_eq!(design_artifact::file(&mut conn, other_chat, 3, "index.html").await.unwrap(), None, "another chat's version 3 doesn't exist");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_version_remembers_where_it_was_last_published(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &preview("editorial", "Fresh at six")).await.unwrap();
    assert_eq!(static_publish::latest(&mut conn, session_id, 1).await.unwrap(), None);

    let first = Published { provider: "fake".into(), url: "https://previews.example/a".into() };
    let second = Published { provider: "fake".into(), url: "https://previews.example/b".into() };
    assert!(static_publish::record(&mut conn, session_id, 1, &first).await.unwrap());
    assert!(static_publish::record(&mut conn, session_id, 1, &second).await.unwrap());
    assert!(!static_publish::record(&mut conn, session_id, 9, &second).await.unwrap(), "there is no version 9");
    assert_eq!(static_publish::latest(&mut conn, session_id, 1).await.unwrap(), Some(second));
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_the_chat_takes_its_previews_with_it(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &preview("editorial", "Fresh at six")).await.unwrap();
    sqlx::query("DELETE FROM sessions WHERE id = $1").bind(session_id).execute(&mut *conn).await.unwrap();
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM design_artifact_files").fetch_one(&mut *conn).await.unwrap();
    assert_eq!(left, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn versions_are_listed_oldest_first_each_with_its_own_unguessable_name(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let (other_chat, _) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    assert!(design_artifact::list(&mut conn, session_id).await.unwrap().is_empty());
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &preview("editorial", "Fresh at six")).await.unwrap();
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &preview("swiss", "Bread, daily")).await.unwrap();
    design_artifact::save(&mut conn, other_chat, Uuid::new_v4(), user_id, None, &preview("editorial", "Elsewhere")).await.unwrap();
    static_publish::record(&mut conn, session_id, 2, &Published { provider: "fake".into(), url: "https://previews.example/b".into() }).await.unwrap();

    let listed = design_artifact::list(&mut conn, session_id).await.unwrap();
    assert_eq!(listed.iter().map(|v| (v.version, v.manifest.direction_id.as_str())).collect::<Vec<_>>(), [(1, "editorial"), (2, "swiss")]);
    assert_eq!(listed.iter().map(|v| v.published_url.as_deref()).collect::<Vec<_>>(), [None, Some("https://previews.example/b")]);
    for stored in &listed {
        assert_eq!(stored.preview_token.len(), design_artifact::PREVIEW_TOKEN_CHARS);
        assert!(stored.preview_token.chars().all(|c| c.is_ascii_hexdigit()));
    }
    assert_ne!(listed[0].preview_token, listed[1].preview_token);

    let (content_type, content) = design_artifact::file_by_token(&mut conn, &listed[1].preview_token, "index.html").await.unwrap().expect("the page");
    assert_eq!(content_type, "text/html; charset=utf-8");
    assert!(content.contains("Bread, daily"));
    assert_eq!(design_artifact::file_by_token(&mut conn, &listed[1].preview_token, "nope.css").await.unwrap(), None);
    assert_eq!(design_artifact::file_by_token(&mut conn, "0".repeat(design_artifact::PREVIEW_TOKEN_CHARS).as_str(), "index.html").await.unwrap(), None);
}

fn publish_error_of(listed: &[design_artifact::StoredVersion], version: i32) -> Option<String> {
    listed.iter().find(|v| v.version == version).unwrap().publish_error.clone()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_publish_is_listed_until_a_later_one_succeeds(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &preview("editorial", "Fresh at six")).await.unwrap();
    assert!(static_publish::record_failure(&mut conn, session_id, 1, "publishing previews isn't set up").await.unwrap());
    assert!(!static_publish::record_failure(&mut conn, session_id, 9, "nope").await.unwrap(), "there is no version 9");
    let listed = design_artifact::list(&mut conn, session_id).await.unwrap();
    assert_eq!(publish_error_of(&listed, 1).as_deref(), Some("publishing previews isn't set up"));
    assert_eq!(listed[0].published_url, None);

    static_publish::record(&mut conn, session_id, 1, &Published { provider: "fake".into(), url: "https://previews.example/a".into() }).await.unwrap();
    let listed = design_artifact::list(&mut conn, session_id).await.unwrap();
    assert_eq!(publish_error_of(&listed, 1), None, "the later success wins");
    assert_eq!(listed[0].published_url.as_deref(), Some("https://previews.example/a"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failure_after_a_success_keeps_the_address(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &preview("editorial", "Fresh at six")).await.unwrap();
    static_publish::record(&mut conn, session_id, 1, &Published { provider: "fake".into(), url: "https://previews.example/a".into() }).await.unwrap();
    static_publish::record_failure(&mut conn, session_id, 1, "Shipyard refused it").await.unwrap();
    let listed = design_artifact::list(&mut conn, session_id).await.unwrap();
    assert_eq!(listed[0].published_url.as_deref(), Some("https://previews.example/a"));
    assert_eq!(publish_error_of(&listed, 1).as_deref(), Some("Shipyard refused it"));
}

struct Answers(Result<Published, String>);

#[async_trait::async_trait]
impl static_publish::StaticPublisher for Answers {
    async fn publish(&self, _request: &static_publish::PublishRequest<'_>) -> Result<Published, String> {
        self.0.clone()
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn publishing_records_what_happened(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let saved = preview("editorial", "Fresh at six");
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), user_id, None, &saved).await.unwrap();
    let request = static_publish::PublishRequest { user_id, session_id, version: 1, title: "Fresh at six", files: &saved.files };

    let failed = static_publish::publish_and_record(&mut conn, &Answers(Err("the service is down".into())), &request).await;
    assert_eq!(failed.unwrap_err(), "the service is down");
    assert_eq!(publish_error_of(&design_artifact::list(&mut conn, session_id).await.unwrap(), 1).as_deref(), Some("the service is down"));

    let at = Published { provider: "fake".into(), url: "https://previews.example/a".into() };
    assert_eq!(static_publish::publish_and_record(&mut conn, &Answers(Ok(at.clone())), &request).await.unwrap(), at);
    assert_eq!(static_publish::latest(&mut conn, session_id, 1).await.unwrap(), Some(at));
}

fn file(path: &str, content: &str) -> design_artifact::ArtifactFile {
    design_artifact::ArtifactFile { path: path.into(), content: content.into() }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_draft_keeps_the_latest_content_of_each_file(pool: PgPool) {
    let (session_id, _) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_draft::stage(&mut conn, session_id, "editorial", &[file("a.css", "1")]).await.unwrap();
    design_draft::stage(&mut conn, session_id, "editorial", &[file("a.css", "2"), file("b.css", "b")]).await.unwrap();
    let mut draft = design_draft::draft(&mut conn, session_id, "editorial").await.unwrap();
    draft.sort_by(|x, y| x.path.cmp(&y.path));
    assert_eq!(draft, vec![file("a.css", "2"), file("b.css", "b")]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn drafts_are_per_direction_and_cleared_on_request(pool: PgPool) {
    let (session_id, _) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_draft::stage(&mut conn, session_id, "editorial", &[file("a.css", "1")]).await.unwrap();
    assert_eq!(design_draft::draft(&mut conn, session_id, "brutalist").await.unwrap(), vec![]);
    design_draft::stage(&mut conn, session_id, "editorial", &[file("late.css", "2")]).await.unwrap();
    design_draft::clear(&mut conn, session_id, "editorial", &["a.css".to_string()]).await.unwrap();
    assert_eq!(design_draft::draft(&mut conn, session_id, "editorial").await.unwrap(), vec![file("late.css", "2")], "only what became a version is dropped");
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_the_chat_takes_its_drafts_with_it(pool: PgPool) {
    let (session_id, _) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    design_draft::stage(&mut conn, session_id, "editorial", &[file("a.css", "1")]).await.unwrap();
    sqlx::query("DELETE FROM sessions WHERE id = $1").bind(session_id).execute(&pool).await.unwrap();
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM design_preview_drafts").fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0);
}
