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

pub mod period;

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
        description: "Summarize the user's spending (not income) grouped by category, optionally since a given date.".to_string(),
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
        description: "Record money the user spent or received (e.g. \"I spent 45,000 on lunch\", \"got my salary, 8 million\", \
                      or a receipt). For a receipt or a purchase of several things, put each line in `items`. \
                      Records only; it never moves money."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "kind": {"type": "string", "enum": ["expense", "income"], "description": "expense (default) or income"},
                "amount": {"type": "number", "description": "The total, as a positive number (what the receipt says, tax and discounts included). Omit to use the sum of the items."},
                "category": {"type": "string", "description": "Lowercase category, e.g. food, transport, groceries, salary"},
                "description": {"type": "string", "description": "What it was, e.g. the shop or the source of the income"},
                "occurred_at": {"type": "string", "description": "ISO 8601 date or time; omit for now"},
                "items": {
                    "type": "array",
                    "description": "The lines of a receipt or purchase",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "quantity": {"type": "number", "description": "Defaults to 1"},
                            "unit_amount": {"type": "number", "description": "Price of one"},
                            "amount": {"type": "number", "description": "The line's total; defaults to quantity × unit_amount"},
                            "category": {"type": "string", "description": "Only when this line belongs to another category than the transaction"}
                        },
                        "required": ["name"]
                    }
                }
            },
            "required": ["category", "description"]
        }),
    }
}

pub fn set_month_start_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "set_month_start".to_string(),
        description: "Set the day the user's money month starts (1–28), e.g. 25 when they're paid on the 25th. Totals and \
                      budgets then run from that day to the day before it next month. 1 is the calendar month."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"day": {"type": "integer", "minimum": 1, "maximum": 28}},
            "required": ["day"]
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

/// One line of a transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub quantity: f64,
    pub unit_amount_cents: Option<i64>,
    pub amount_cents: i64,
    pub category: Option<String>,
}

fn cents(value: f64) -> i64 {
    (value * 100.0).round() as i64
}

/// The `items` of a log_transaction input: each needs a name, and an amount or a unit price.
pub fn items_of(input: &Value) -> Result<Vec<Item>, String> {
    let Some(list) = input.get("items").and_then(|v| v.as_array()) else { return Ok(Vec::new()) };
    list.iter()
        .enumerate()
        .map(|(i, item)| {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            if name.is_empty() {
                return Err(format!("item {} needs a name", i + 1));
            }
            let quantity = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(1.0);
            if !(quantity.is_finite() && quantity > 0.0) {
                return Err(format!("{name}: quantity must be more than zero"));
            }
            let unit = item.get("unit_amount").and_then(|v| v.as_f64());
            let amount = match (item.get("amount").and_then(|v| v.as_f64()), unit) {
                (Some(amount), _) => amount,
                (None, Some(unit)) => unit * quantity,
                (None, None) => return Err(format!("{name}: give its amount or its unit price")),
            };
            if !(amount.is_finite() && amount >= 0.0) || unit.is_some_and(|u| !(u.is_finite() && u >= 0.0)) {
                return Err(format!("{name}: amounts can't be negative"));
            }
            let category = item.get("category").and_then(|v| v.as_str()).map(|c| c.trim().to_lowercase()).filter(|c| !c.is_empty());
            Ok(Item { name, quantity, unit_amount_cents: unit.map(cents), amount_cents: cents(amount), category })
        })
        .collect()
}

fn kind_of(input: &Value) -> Result<&'static str, String> {
    match input.get("kind").and_then(|v| v.as_str()).map(str::trim) {
        None | Some("") | Some("expense") => Ok("expense"),
        Some("income") => Ok("income"),
        Some(other) => Err(format!("kind must be expense or income, not {other}")),
    }
}

/// Records money spent or received in the Money agent's own table, with its items. `source` is
/// "manual" (the Money page) or "agent" (this agent's tool).
pub async fn log_transaction(conn: &mut sqlx::PgConnection, user_id: Uuid, input: &Value, source: &str) -> Result<String, String> {
    let kind = kind_of(input)?;
    let items = items_of(input)?;
    let total = match input.get("amount") {
        Some(amount) if !amount.is_null() => amount_to_cents(input, "amount")?,
        _ if !items.is_empty() => items.iter().map(|i| i.amount_cents).sum(),
        _ => return Err("amount is required (or the items it's made of)".to_string()),
    };
    if total <= 0 {
        return Err("amount must be a positive number".to_string());
    }
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
    let mut tx = sqlx::Connection::begin(&mut *conn).await.map_err(|e| e.to_string())?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO money_transactions (user_id, occurred_at, amount_cents, category, description, source, kind) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(user_id)
    .bind(occurred_at)
    .bind(total)
    .bind(&category)
    .bind(&description)
    .bind(source)
    .bind(kind)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    for (position, item) in items.iter().enumerate() {
        sqlx::query(
            "INSERT INTO money_transaction_items (transaction_id, user_id, position, name, quantity, unit_amount_cents, amount_cents, category) \
             VALUES ($1, $2, $3, $4, $5::numeric, $6, $7, $8)",
        )
        .bind(id)
        .bind(user_id)
        .bind(position as i32)
        .bind(&item.name)
        .bind(item.quantity)
        .bind(item.unit_amount_cents)
        .bind(item.amount_cents)
        .bind(&item.category)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    let verb = if kind == "income" { "Logged income of" } else { "Logged" };
    let lines = if items.is_empty() { String::new() } else { format!(" ({} items)", items.len()) };
    Ok(format!("{verb} {:.2} for {description} under {category}{lines}.", total as f64 / 100.0))
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

/// `(category, limit_cents, spent_cents)` between `month_start` and `month_end` (the person's
/// money month), largest limit first. Only spending counts.
pub async fn budgets_with_spending(conn: &mut sqlx::PgConnection, user_id: Uuid, month_start: DateTime<Utc>, month_end: DateTime<Utc>) -> Result<Vec<(String, i64, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT b.category, b.monthly_limit_cents, COALESCE(SUM(t.amount_cents), 0)::bigint \
         FROM money_budgets b LEFT JOIN money_transactions t \
           ON t.user_id = b.user_id AND t.category = b.category AND t.kind = 'expense' AND t.occurred_at >= $2 AND t.occurred_at < $3 \
         WHERE b.user_id = $1 GROUP BY b.category, b.monthly_limit_cents ORDER BY b.monthly_limit_cents DESC",
    )
    .bind(user_id)
    .bind(month_start)
    .bind(month_end)
    .fetch_all(&mut *conn)
    .await
}

pub async fn list_budgets(conn: &mut sqlx::PgConnection, user_id: Uuid) -> Result<String, String> {
    let (tz, start_day) = period::settings(conn, user_id).await.map_err(|e| e.to_string())?;
    let current = period::Period::containing(tz, start_day, Utc::now());
    let (start, end) = (current.start, current.end);
    let rows = budgets_with_spending(conn, user_id, start, end).await.map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Ok("No budgets set yet.".to_string());
    }
    Ok(rows
        .iter()
        .map(|(category, limit, spent)| {
            let percent = (*spent as f64 / *limit as f64 * 100.0).round();
            format!(
                "{category}: {:.2} of {:.2} spent this money month, {} to {} ({percent}%)",
                *spent as f64 / 100.0,
                *limit as f64 / 100.0,
                current.first_day(),
                current.last_day()
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

pub struct MoneyAgent;

#[async_trait]
impl SubAgent for MoneyAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

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
            set_month_start_tool_definition(),
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
            "set_month_start" => {
                let day = input.get("day").and_then(|v| v.as_i64()).ok_or("day is required")?;
                period::set_start_day(conn, user_id, day).await?;
                Ok(nomi_agent_core::ToolOutcome::text(format!("Their money month now starts on day {day}.")))
            }
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("money")
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("the user wants to look at their transactions, spending, income or budget, log an expense or income, set a budget, or change when their money month starts")
    }
}

pub async fn list_transactions(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let limit = input.get("limit").and_then(|v| v.as_i64()).unwrap_or(DEFAULT_TRANSACTION_LIMIT);
    let category = input.get("category").and_then(|v| v.as_str());

    let rows: Vec<(DateTime<Utc>, i64, String, String, String, Option<String>)> = if let Some(category) = category {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description, kind, \
                    (SELECT string_agg(i.name, ', ' ORDER BY i.position) FROM money_transaction_items i WHERE i.transaction_id = t.id) \
             FROM money_transactions t WHERE user_id = $1 AND category = $2 ORDER BY occurred_at DESC LIMIT $3",
        )
        .bind(user_id)
        .bind(category)
        .bind(limit)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description, kind, \
                    (SELECT string_agg(i.name, ', ' ORDER BY i.position) FROM money_transaction_items i WHERE i.transaction_id = t.id) \
             FROM money_transactions t WHERE user_id = $1 ORDER BY occurred_at DESC LIMIT $2",
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
        .map(|(occurred_at, amount_cents, category, description, kind, items)| {
            let sign = if kind == "income" { "+" } else { "" };
            let items = items.as_deref().map(|i| format!(" ({i})")).unwrap_or_default();
            format!("{} | {} | {sign}{:.2} | {description}{items}", occurred_at.format("%Y-%m-%d"), category, *amount_cents as f64 / 100.0)
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
             WHERE user_id = $1 AND kind = 'expense' AND occurred_at >= $2 GROUP BY category ORDER BY category",
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM money_transactions \
             WHERE user_id = $1 AND kind = 'expense' GROUP BY category ORDER BY category",
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
