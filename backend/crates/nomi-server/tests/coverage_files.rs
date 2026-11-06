//! A file in the composer (looked up and removed by its owner only) and the files model.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn a_composer_file_is_read_and_removed_by_its_owner(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let ben = app.person("ben@acme.test").await;
    let file: Uuid = sqlx::query_scalar("INSERT INTO attachments (user_id, name, mime, kind, size_bytes, storage_key, preview_key, status) VALUES ($1, 'receipt.png', 'image/png', 'image', 3, 'u/r.png', 'u/r.jpg', 'ready') RETURNING id")
        .bind(ana.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let one = format!("/api/attachments/{file}");
    let (status, body) = app.get(&one, &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "receipt.png");
    assert_eq!(body["has_preview"], true);
    assert_eq!(app.get(&one, &ben).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.call("DELETE", &one, Some(&ben.token), None).await.0, StatusCode::NOT_FOUND);

    // Sent with a message: it stays with the chat.
    let (_, created) = app.call("POST", "/api/sessions", Some(&ana.token), None).await;
    let session: Uuid = created["session_id"].as_str().unwrap().parse().unwrap();
    let message: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'here') RETURNING id").bind(session).fetch_one(&pool).await.unwrap();
    sqlx::query("UPDATE attachments SET message_id = $2 WHERE id = $1").bind(file).bind(message).execute(&pool).await.unwrap();
    assert_eq!(app.call("DELETE", &one, Some(&ana.token), None).await.0, StatusCode::CONFLICT);
    sqlx::query("UPDATE attachments SET message_id = NULL WHERE id = $1").bind(file).execute(&pool).await.unwrap();
    assert_eq!(app.call("DELETE", &one, Some(&ana.token), None).await.0, StatusCode::NO_CONTENT);
    assert_eq!(app.get(&one, &ana).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_files_model_is_set_and_cleared_by_admins(pool: PgPool) {
    let app = App::new(pool);
    let admin = app.admin("root@nomi.test").await;
    let person = app.person("ana@acme.test").await;
    let (status, made) = app
        .send("POST", "/api/admin/settings/llm/models", &admin, json!({"label": "Reader", "provider": "fake", "model_id": "fake-model", "api_key": "", "base_url": null}))
        .await;
    assert!(status.is_success(), "{status} {made}");
    let id = made["id"].as_str().unwrap();
    assert_eq!(app.send("PUT", "/api/admin/settings/llm/files-model", &person, json!({"id": id})).await.0, StatusCode::FORBIDDEN);
    assert_eq!(app.send("PUT", "/api/admin/settings/llm/files-model", &admin, json!({"id": id})).await.0, StatusCode::NO_CONTENT);
    assert_eq!(app.send("PUT", "/api/admin/settings/llm/files-model", &admin, json!({"id": Uuid::new_v4()})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.send("PUT", "/api/admin/settings/llm/files-model", &admin, json!({"id": null})).await.0, StatusCode::NO_CONTENT);
}
