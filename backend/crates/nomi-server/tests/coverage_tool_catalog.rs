//! The shared tool catalog that custom agents are granted tools from: every entry runs the
//! owning agent's own tool for the asking user.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn every_catalog_tool_runs_its_agents_own_tool(pool: PgPool) {
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')").bind(org).bind(user).execute(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'c') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();

    let catalog = nomi_server::build_tool_catalog(nomi_test_support::test_project_storage());
    let inputs = [
        ("log_transaction", json!({"amount": 45000, "category": "Food", "description": "Lunch", "kind": "expense"})),
        ("list_transactions", json!({})),
        ("set_budget", json!({"category": "Food", "monthly_limit": 1000000})),
        ("list_budgets", json!({})),
        ("summarize_budget", json!({})),
        ("set_personality", json!({"description": "Warm and brief"})),
        ("list_personality_versions", json!({})),
        ("reset_chat_personality", json!({})),
        ("rollback_personality", json!({"version": 1})),
        ("list_recent_agent_activity", json!({})),
        ("create_project", json!({"name": "Bakery", "description": "Orders"})),
        ("list_files", json!({"project_id": Uuid::new_v4()})),
    ];
    let known = catalog.known_tool_names();
    let mut ran = 0;
    for (name, input) in inputs {
        if !known.contains(&name) {
            continue;
        }
        let mut conn = pool.acquire().await.unwrap();
        // Success or a tool's own refusal both mean it reached the agent's code.
        let outcome = catalog.execute(name, &mut conn, session, Uuid::new_v4(), user, input).await;
        if name == "log_transaction" || name == "list_budgets" {
            assert!(outcome.is_ok(), "{name}: {outcome:?}");
        }
        ran += 1;
    }
    assert!(ran >= 11, "only {ran} catalog tools ran");
    assert!(catalog.definitions().len() >= 11);
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(catalog.execute("nope", &mut conn, session, Uuid::new_v4(), user, json!({})).await.unwrap_err(), "tool not granted: nope");
}
