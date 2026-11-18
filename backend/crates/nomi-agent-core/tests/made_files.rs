use nomi_agent_core::made_files::make_file;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn a_made_file_is_stored_like_an_attachment_and_read_back(pool: PgPool) {
    std::env::set_var("ATTACHMENTS_DIR", std::env::temp_dir().join(format!("nomi-core-made-{}", std::process::id())));
    let me: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let source: Uuid = sqlx::query_scalar("INSERT INTO attachments (user_id, name, mime, kind, size_bytes, storage_key, status) VALUES ($1, 's.txt', 'text/plain', 'text', 1, 'k', 'ready') RETURNING id")
        .bind(me)
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let said = make_file(
        &mut conn,
        None,
        Uuid::new_v4(),
        me,
        "files",
        &json!({ "name": "may", "format": "csv", "summary": "May spending", "rows": [["a", "b"], [1, 2]], "sources": [source.to_string(), Uuid::new_v4().to_string()] }),
    )
    .await
    .unwrap();
    assert!(said.contains("may.csv"));
    let (id, mime, size, made_by, from): (Uuid, String, i64, String, Vec<Uuid>) =
        sqlx::query_as("SELECT id, mime, size_bytes, made_by, made_from FROM attachments WHERE user_id = $1 AND made_by IS NOT NULL").bind(me).fetch_one(&pool).await.unwrap();
    assert_eq!((mime.as_str(), made_by.as_str()), ("text/csv", "files"));
    assert_eq!(from, vec![source], "only the person's own files count as sources");
    let bytes = nomi_storage::blob::attachment_store().get_bytes(&nomi_attachments::made::storage_key(me, id)).await.unwrap().unwrap();
    assert_eq!(bytes.len() as i64, size);
    assert_eq!(String::from_utf8(bytes).unwrap(), "a,b\r\n1,2\r\n");
}
