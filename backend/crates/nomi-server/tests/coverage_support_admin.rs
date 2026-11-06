//! Admin → Support: editing and removing articles, publishing and dismissing drafts, and keys.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn articles_and_drafts_are_kept_by_admins(pool: PgPool) {
    let app = App::new(pool.clone());
    let admin = app.admin("root@nomi.test").await;
    let person = app.person("ana@acme.test").await;

    let (status, article) = app.send("POST", "/api/admin/support/articles", &admin, json!({"title": "Exporting", "body": "Use Export.", "tags": ["projects"]})).await;
    assert_eq!(status, StatusCode::CREATED, "{article}");
    let one = format!("/api/admin/support/articles/{}", article["id"].as_str().unwrap());
    let (status, updated) = app.send("PUT", &one, &admin, json!({"title": "  Exporting a project ", "body": "Open the project, then Export.", "tags": [], "published": false})).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["title"], "Exporting a project");
    assert_eq!(updated["published"], false);
    assert_eq!(app.send("PUT", &one, &admin, json!({"title": " ", "body": "x"})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("PUT", &format!("/api/admin/support/articles/{}", Uuid::new_v4()), &admin, json!({"title": "x"})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.send("PUT", &one, &person, json!({"title": "x"})).await.0, StatusCode::FORBIDDEN);
    assert_eq!(app.call("DELETE", &one, Some(&admin.token), None).await.0, StatusCode::NO_CONTENT);

    let draft = |question: &str| {
        let pool = pool.clone();
        let question = question.to_string();
        async move {
            sqlx::query_scalar::<_, Uuid>("INSERT INTO kb_drafts (question, title, body) VALUES ($1, 'How to export', 'Use Export') RETURNING id")
                .bind(question)
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };
    let first = draft("How do I export?").await;
    let second = draft("Is it free?").await;
    let (status, drafts) = app.get("/api/admin/support/drafts", &admin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(drafts.as_array().unwrap().len(), 2);
    let (status, published) = app.send("POST", &format!("/api/admin/support/drafts/{first}/publish"), &admin, json!({"title": "Exporting", "body": "Use Export.", "tags": []})).await;
    assert!(status.is_success(), "{status} {published}");
    assert_eq!(app.send("POST", &format!("/api/admin/support/drafts/{first}/publish"), &admin, json!({"title": "Again", "body": "x"})).await.0, StatusCode::NOT_FOUND, "already handled");
    assert_eq!(app.call("POST", &format!("/api/admin/support/drafts/{second}/dismiss"), Some(&admin.token), None).await.0, StatusCode::NO_CONTENT);
    let (_, drafts) = app.get("/api/admin/support/drafts", &admin).await;
    assert!(drafts.as_array().unwrap().is_empty());

    let (status, keys) = app.get("/api/admin/support/keys", &admin).await;
    assert_eq!((status, keys), (StatusCode::OK, json!([])));
    let (status, questions) = app.get("/api/admin/support/questions", &admin).await;
    assert_eq!(status, StatusCode::OK, "{questions}");
}
