use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_money::MoneyAgent;
use nomi_agent_core::SubAgent;

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn seed_transaction(
    pool: &PgPool,
    user_id: Uuid,
    occurred_at: chrono::DateTime<chrono::Utc>,
    amount_cents: i64,
    category: &str,
    description: &str,
) {
    sqlx::query(
        "INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(occurred_at)
    .bind(amount_cents)
    .bind(category)
    .bind(description)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_transactions_returns_recent_transactions_for_the_user(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let other_user_id = seed_user(&pool).await;
    let now = chrono::Utc::now();
    seed_transaction(&pool, user_id, now, 1500, "food", "lunch").await;
    seed_transaction(&pool, other_user_id, now, 9999, "food", "someone else's lunch").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_transactions", serde_json::json!({"limit": 10}))
        .await
        .unwrap();

    assert!(result.display_text.contains("lunch"));
    assert!(!result.display_text.contains("someone else's lunch"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_transactions_filters_by_category(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let now = chrono::Utc::now();
    seed_transaction(&pool, user_id, now, 1500, "food", "lunch").await;
    seed_transaction(&pool, user_id, now, 5000, "rent", "monthly rent").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_transactions", serde_json::json!({"limit": 10, "category": "food"}))
        .await
        .unwrap();

    assert!(result.display_text.contains("lunch"));
    assert!(!result.display_text.contains("monthly rent"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_transactions_with_no_matches_says_so(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_transactions", serde_json::json!({"limit": 10}))
        .await
        .unwrap();
    assert_eq!(result.display_text, "No transactions found.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn summarize_budget_groups_totals_by_category(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let now = chrono::Utc::now();
    seed_transaction(&pool, user_id, now, 1500, "food", "lunch").await;
    seed_transaction(&pool, user_id, now, 2500, "food", "dinner").await;
    seed_transaction(&pool, user_id, now, 5000, "rent", "monthly rent").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "summarize_budget", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.display_text.contains("food: 40.00"));
    assert!(result.display_text.contains("rent: 50.00"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn summarize_budget_respects_since_filter(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let old = chrono::Utc::now() - chrono::Duration::days(60);
    let recent = chrono::Utc::now();
    seed_transaction(&pool, user_id, old, 1000, "food", "old lunch").await;
    seed_transaction(&pool, user_id, recent, 2000, "food", "recent lunch").await;

    let mut conn = pool.acquire().await.unwrap();
    let since = (chrono::Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let result = MoneyAgent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "summarize_budget", serde_json::json!({"since": since}))
        .await
        .unwrap();

    assert!(result.display_text.contains("food: 20.00"));
    assert!(!result.display_text.contains("food: 30.00"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn summarize_budget_with_an_invalid_since_date_returns_an_error(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "summarize_budget", serde_json::json!({"since": "not-a-date"}))
        .await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "transfer_money", serde_json::json!({})).await;
    assert!(result.is_err());
}
