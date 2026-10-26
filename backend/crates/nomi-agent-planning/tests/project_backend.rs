use sqlx::PgPool;
use uuid::Uuid;

async fn seed_user_and_session(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_is_on_drizzle_with_no_relay_backend_until_told_otherwise(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let (backend, relay_project_id): (String, Option<Uuid>) = sqlx::query_as(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo') RETURNING backend, relay_project_id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(backend, "drizzle");
    assert_eq!(relay_project_id, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_backend_is_drizzle_or_relay_only(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let insert = |backend: &'static str| {
        sqlx::query("INSERT INTO projects (user_id, session_id, name, backend) VALUES ($1, $2, 'Todo', $3)")
            .bind(user_id)
            .bind(session_id)
            .bind(backend)
    };
    insert("relay").execute(&pool).await.unwrap();
    assert!(insert("firebase").execute(&pool).await.is_err());
}
