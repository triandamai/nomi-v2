use std::borrow::Cow;

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

pub fn list_transactions_tool_definition() -> ToolDefinition {
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
    }
}

pub fn summarize_budget_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "summarize_budget".to_string(),
        description: "Summarize the user's spending totals grouped by category, optionally since a given date.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "since": {"type": "string", "description": "ISO 8601 date; omit for all-time"}
            }
        }),
    }
}

pub fn log_transaction_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "log_transaction".to_string(),
        description: "Record a purchase or expense the user tells you about (e.g. \"I spent 45,000 on lunch\"). \
                      Records only; it never moves money."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "amount": {"type": "number", "description": "Amount spent, as a positive number"},
                "category": {"type": "string", "description": "Lowercase category, e.g. food, transport, subscriptions"},
                "description": {"type": "string", "description": "What it was for"},
                "occurred_at": {"type": "string", "description": "ISO 8601 date or time; omit for now"}
            },
            "required": ["amount", "category", "description"]
        }),
    }
}

pub fn set_budget_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "set_budget".to_string(),
        description: "Set (or change) the user's monthly spending limit for one category.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "category": {"type": "string"},
                "monthly_limit": {"type": "number", "description": "Positive amount per month"}
            },
            "required": ["category", "monthly_limit"]
        }),
    }
}

pub fn list_budgets_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_budgets".to_string(),
        description: "List the user's monthly budgets with how much of each is spent this month.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

fn amount_to_cents(input: &Value, field: &str) -> Result<i64, String> {
    let amount = input.get(field).and_then(|v| v.as_f64()).ok_or(format!("{field} is required"))?;
    if !(amount.is_finite() && amount > 0.0) {
        return Err(format!("{field} must be a positive number"));
    }
    Ok((amount * 100.0).round() as i64)
}

fn category_of(input: &Value) -> Result<String, String> {
    let category = input.get("category").and_then(|v| v.as_str()).unwrap_or_default().trim().to_lowercase();
    if category.is_empty() {
        return Err("category is required".to_string());
    }
    Ok(category)
}

/// Records an expense in the Money agent's own table. `source` is "manual" (the Money page) or
/// "agent" (this agent's tool).
pub async fn log_transaction(conn: &mut sqlx::PgConnection, user_id: Uuid, input: &Value, source: &str) -> Result<String, String> {
    let cents = amount_to_cents(input, "amount")?;
    let category = category_of(input)?;
    let description = input.get("description").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
    if description.is_empty() {
        return Err("description is required".to_string());
    }
    let occurred_at: DateTime<Utc> = match input.get("occurred_at").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        Some(text) => text
            .parse::<DateTime<Utc>>()
            .or_else(|_| chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").map(|d| d.and_hms_opt(12, 0, 0).unwrap_or_default().and_utc()))
            .map_err(|_| "occurred_at must be an ISO 8601 date or time".to_string())?,
        None => Utc::now(),
    };
    sqlx::query(
        "INSERT INTO money_transactions (user_id, occurred_at, amount_cents, category, description, source) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(occurred_at)
    .bind(cents)
    .bind(&category)
    .bind(&description)
    .bind(source)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    Ok(format!("Logged {:.2} for {description} under {category}.", cents as f64 / 100.0))
}

pub async fn set_budget(conn: &mut sqlx::PgConnection, user_id: Uuid, input: &Value) -> Result<String, String> {
    let cents = amount_to_cents(input, "monthly_limit")?;
    let category = category_of(input)?;
    sqlx::query(
        "INSERT INTO money_budgets (user_id, category, monthly_limit_cents) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, category) DO UPDATE SET monthly_limit_cents = EXCLUDED.monthly_limit_cents, updated_at = now()",
    )
    .bind(user_id)
    .bind(&category)
    .bind(cents)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    Ok(format!("Budget for {category} set to {:.2} a month.", cents as f64 / 100.0))
}

/// `(category, limit_cents, spent_cents)` for this calendar month (UTC), largest limit first.
pub async fn budgets_with_spending(conn: &mut sqlx::PgConnection, user_id: Uuid, month_start: DateTime<Utc>, month_end: DateTime<Utc>) -> Result<Vec<(String, i64, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT b.category, b.monthly_limit_cents, COALESCE(SUM(t.amount_cents), 0)::bigint \
         FROM money_budgets b LEFT JOIN money_transactions t \
           ON t.user_id = b.user_id AND t.category = b.category AND t.occurred_at >= $2 AND t.occurred_at < $3 \
         WHERE b.user_id = $1 GROUP BY b.category, b.monthly_limit_cents ORDER BY b.monthly_limit_cents DESC",
    )
    .bind(user_id)
    .bind(month_start)
    .bind(month_end)
    .fetch_all(&mut *conn)
    .await
}

pub async fn list_budgets(conn: &mut sqlx::PgConnection, user_id: Uuid) -> Result<String, String> {
    use chrono::Datelike;
    let now = Utc::now();
    let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or_default().and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();
    let end = start + chrono::Months::new(1);
    let rows = budgets_with_spending(conn, user_id, start, end).await.map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Ok("No budgets set yet.".to_string());
    }
    Ok(rows
        .iter()
        .map(|(category, limit, spent)| {
            let percent = (*spent as f64 / *limit as f64 * 100.0).round();
            format!("{category}: {:.2} of {:.2} spent this month ({percent}%)", *spent as f64 / 100.0, *limit as f64 / 100.0)
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

pub struct MoneyAgent;

#[async_trait]
impl SubAgent for MoneyAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(MONEY_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(MONEY_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            list_transactions_tool_definition(),
            summarize_budget_tool_definition(),
            log_transaction_tool_definition(),
            set_budget_tool_definition(),
            list_budgets_tool_definition(),
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
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "list_transactions" => list_transactions(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "summarize_budget" => summarize_budget(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "log_transaction" => log_transaction(conn, user_id, &input, "agent").await.map(nomi_agent_core::ToolOutcome::text),
            "set_budget" => set_budget(conn, user_id, &input).await.map(nomi_agent_core::ToolOutcome::text),
            "list_budgets" => list_budgets(conn, user_id).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("money")
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("the user wants to look at their transactions, spending or budget, log an expense, or set a budget")
    }
}

pub async fn list_transactions(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let limit = input.get("limit").and_then(|v| v.as_i64()).unwrap_or(DEFAULT_TRANSACTION_LIMIT);
    let category = input.get("category").and_then(|v| v.as_str());

    let rows: Vec<(DateTime<Utc>, i64, String, String)> = if let Some(category) = category {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description FROM money_transactions \
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
            "SELECT occurred_at, amount_cents, category, description FROM money_transactions \
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

pub async fn summarize_budget(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let since = input.get("since").and_then(|v| v.as_str());

    let rows: Vec<(String, i64)> = if let Some(since) = since {
        let since: DateTime<Utc> = since.parse().map_err(|_| "invalid 'since' date, expected ISO 8601".to_string())?;
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM money_transactions \
             WHERE user_id = $1 AND occurred_at >= $2 GROUP BY category ORDER BY category",
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM money_transactions \
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
