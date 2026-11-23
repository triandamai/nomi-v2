//! Dana's tools as `Tool`s: what `MoneyAgent` runs, and what custom agents can be granted.

use nomi_tool::ToolContext;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

use nomi_tool::{Tool, ToolOutcome};
use nomi_llm::ToolDefinition;

use crate::MONEY_AGENT_TYPE;

/// Every money tool, in the order Dana is offered them. All of them run for any agent.
pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(ListTransactions),
        Box::new(SummarizeBudget),
        Box::new(LogTransaction),
        Box::new(SetBudget),
        Box::new(ListBudgets),
        Box::new(SetMonthStart),
    ]
}

pub struct ListTransactions;

#[async_trait]
impl Tool for ListTransactions {
    fn definition(&self) -> ToolDefinition {
        crate::list_transactions_tool_definition()
    }
    fn group(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["expenses", "spending", "history", "payments"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::list_transactions(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

pub struct SummarizeBudget;

#[async_trait]
impl Tool for SummarizeBudget {
    fn definition(&self) -> ToolDefinition {
        crate::summarize_budget_tool_definition()
    }
    fn group(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["spending", "categories", "totals", "report"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::summarize_budget(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

pub struct LogTransaction;

#[async_trait]
impl Tool for LogTransaction {
    fn definition(&self) -> ToolDefinition {
        crate::log_transaction_tool_definition()
    }
    fn group(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["expense", "income", "receipt", "record", "spent"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::log_transaction_outcome(conn, user_id, &input, session_id).await
    }
}

pub struct SetBudget;

#[async_trait]
impl Tool for SetBudget {
    fn definition(&self) -> ToolDefinition {
        crate::set_budget_tool_definition()
    }
    fn group(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["limit", "monthly", "category", "spending"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::set_budget(conn, user_id, &input).await.map(ToolOutcome::text)
    }
}

pub struct ListBudgets;

#[async_trait]
impl Tool for ListBudgets {
    fn definition(&self) -> ToolDefinition {
        crate::list_budgets_tool_definition()
    }
    fn group(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["limits", "monthly", "spent", "progress"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::list_budgets(conn, user_id).await.map(ToolOutcome::text)
    }
}

pub struct SetMonthStart;

#[async_trait]
impl Tool for SetMonthStart {
    fn definition(&self) -> ToolDefinition {
        crate::set_month_start_tool_definition()
    }
    fn group(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["payday", "salary", "period", "cycle"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        let day = input.get("day").and_then(|v| v.as_i64()).ok_or("day is required")?;
        crate::period::set_start_day(conn, user_id, day).await?;
        Ok(ToolOutcome::text(format!("Their money month now starts on day {day}.")))
    }
}
