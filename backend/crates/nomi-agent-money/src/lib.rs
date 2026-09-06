use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;
use nomi_agent_core::prompts::MONEY_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;

pub const MONEY_AGENT_TYPE: &str = "money";

/// `list_transactions`'s default when the caller omits `limit`.
const DEFAULT_TRANSACTION_LIMIT: i64 = 10;

pub struct MoneyAgent;

#[async_trait]
impl SubAgent for MoneyAgent {
    fn agent_type(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        MONEY_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "list_transactions".to_string(),
                description: "List the user's most recent transactions, optionally filtered by category.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {"type": "integer", "description": "Max number of transactions to return"},
                        "category": {"type": "string", "description": "Optional category filter"}
                    },
                    "required": ["limit"]
                }),
            },
            ToolDefinition {
                name: "summarize_budget".to_string(),
                description: "Summarize the user's spending totals grouped by category, optionally since a given date.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "since": {"type": "string", "description": "ISO 8601 date; omit for all-time"}
                    }
                }),
            },
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "list_transactions" => list_transactions(conn, user_id, input).await,
            "summarize_budget" => summarize_budget(conn, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        "money"
    }

    fn intent_description(&self) -> &'static str {
        "the user wants to look at their transactions, spending, or budget"
    }
}

async fn list_transactions(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let limit = input.get("limit").and_then(|v| v.as_i64()).unwrap_or(DEFAULT_TRANSACTION_LIMIT);
    let category = input.get("category").and_then(|v| v.as_str());

    let rows: Vec<(DateTime<Utc>, i64, String, String)> = if let Some(category) = category {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description FROM mock_transactions \
             WHERE user_id = $1 AND category = $2 ORDER BY occurred_at DESC LIMIT $3",
        )
        .bind(user_id)
        .bind(category)
        .bind(limit)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description FROM mock_transactions \
             WHERE user_id = $1 ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    };

    if rows.is_empty() {
        return Ok("No transactions found.".to_string());
    }

    let lines: Vec<String> = rows
        .iter()
        .map(|(occurred_at, amount_cents, category, description)| {
            format!(
                "{} | {} | {:.2} | {}",
                occurred_at.format("%Y-%m-%d"),
                category,
                *amount_cents as f64 / 100.0,
                description
            )
        })
        .collect();

    Ok(lines.join("\n"))
}

async fn summarize_budget(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let since = input.get("since").and_then(|v| v.as_str());

    let rows: Vec<(String, i64)> = if let Some(since) = since {
        let since: DateTime<Utc> = since.parse().map_err(|_| "invalid 'since' date, expected ISO 8601".to_string())?;
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM mock_transactions \
             WHERE user_id = $1 AND occurred_at >= $2 GROUP BY category ORDER BY category",
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM mock_transactions \
             WHERE user_id = $1 GROUP BY category ORDER BY category",
        )
        .bind(user_id)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    };

    if rows.is_empty() {
        return Ok("No transactions found.".to_string());
    }

    let lines: Vec<String> = rows
        .iter()
        .map(|(category, total_cents)| format!("{}: {:.2}", category, *total_cents as f64 / 100.0))
        .collect();

    Ok(lines.join("\n"))
}
