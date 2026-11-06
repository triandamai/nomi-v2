//! Finley's tools through the agent: logging with and without items, budgets, the money month,
//! listing and summarizing, and what each refuses.

use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_money::MoneyAgent;

async fn person(pool: &PgPool) -> (Uuid, Uuid) {
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id").bind(org).bind(user.to_string()).fetch_one(pool).await.unwrap();
    (user, session)
}

async fn run(pool: &PgPool, user: Uuid, session: Uuid, tool: &str, input: Value) -> Result<String, String> {
    let mut conn = pool.acquire().await.unwrap();
    MoneyAgent.execute_tool(&mut conn, session, Uuid::new_v4(), user, tool, input).await.map(|o| o.display_text)
}

#[sqlx::test(migrations = "../../migrations")]
async fn finley_logs_budgets_and_sums_up(pool: PgPool) {
    let (user, session) = person(&pool).await;
    assert!(run(&pool, user, session, "log_transaction", json!({"amount": 45000, "category": "Food", "description": "Lunch"})).await.is_ok());
    let itemized = run(&pool, user, session, "log_transaction", json!({
        "category": "Groceries", "description": "Weekly shop", "kind": "expense",
        "items": [{"name": "Rice", "quantity": 2, "unit_amount": 15000}, {"name": "Eggs", "quantity": 1, "unit_amount": 30000}]
    }))
    .await;
    assert!(itemized.is_ok(), "{itemized:?}");
    assert!(run(&pool, user, session, "log_transaction", json!({"amount": 5000000, "category": "Salary", "description": "Pay", "kind": "income"})).await.is_ok());
    assert!(run(&pool, user, session, "log_transaction", json!({"category": "Food", "description": "x"})).await.is_err(), "an amount or items");
    assert!(run(&pool, user, session, "log_transaction", json!({"amount": 1, "category": "Food", "description": "x", "kind": "gift"})).await.is_err());

    assert!(run(&pool, user, session, "set_budget", json!({"category": "Food", "monthly_limit": 1000000})).await.is_ok());
    assert!(run(&pool, user, session, "set_budget", json!({"category": "Food", "monthly_limit": -5})).await.is_err());
    let budgets = run(&pool, user, session, "list_budgets", json!({})).await.unwrap();
    assert!(budgets.to_lowercase().contains("food"), "{budgets}");

    let listed = run(&pool, user, session, "list_transactions", json!({"category": "food", "limit": 5})).await.unwrap();
    assert!(listed.contains("Lunch"), "{listed}");
    assert!(run(&pool, user, session, "list_transactions", json!({})).await.unwrap().contains("Weekly shop"));
    let summary = run(&pool, user, session, "summarize_budget", json!({})).await.unwrap();
    assert!(!summary.is_empty());

    assert_eq!(run(&pool, user, session, "set_month_start", json!({"day": 25})).await.unwrap(), "Their money month now starts on day 25.");
    assert!(run(&pool, user, session, "set_month_start", json!({})).await.is_err());
    assert!(run(&pool, user, session, "set_month_start", json!({"day": 31})).await.is_err());
    assert!(run(&pool, user, session, "summarize_budget", json!({"month": "2026-11"})).await.is_ok());
    assert_eq!(run(&pool, user, session, "borrow", json!({})).await.unwrap_err(), "unknown tool: borrow");
}

#[test]
fn finley_describes_itself() {
    let agent = MoneyAgent;
    assert_eq!(agent.agent_type(), "money");
    assert!(!agent.system_prompt().is_empty());
    assert!(!agent.intent_label().is_empty() && !agent.intent_description().is_empty());
    assert!(agent.uses_memory());
    assert_eq!(agent.tools().len(), 6);
}
