use sqlx::PgPool;

#[sqlx::test]
async fn required_extensions_are_enabled(pool: PgPool) {
    let exts: Vec<String> = sqlx::query_scalar(
        "SELECT extname FROM pg_extension WHERE extname IN ('pgcrypto', 'vector') ORDER BY extname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(exts, vec!["pgcrypto".to_string(), "vector".to_string()]);
}
