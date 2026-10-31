//! The Money page (`routes/money.rs`; bodies parsed by `nomi_agent_money`). Amounts come back in
//! cents; the backend takes them in currency units, so writes convert.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::Services;
use crate::error::NomiError;

const MONEY: &str = "money";
const TRANSACTIONS: &str = "transactions";
const BUDGETS: &str = "budgets";
const PARAM_MONTH: &str = "month";
const CENTS_PER_UNIT: f64 = 100.0;
const DEFAULT_TIMEZONE: &str = "UTC";
const DEFAULT_START_DAY: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct CategoryTotal {
    pub category: String,
    pub cents: i64,
    pub count: i64,
}

/// `date` is `YYYY-MM-DD`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct DayTotal {
    pub date: String,
    pub cents: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct TransactionItem {
    pub name: String,
    pub quantity: f64,
    pub unit_amount_cents: Option<i64>,
    pub amount_cents: i64,
    pub category: Option<String>,
}

/// `kind` is `expense` or `income`; `source` is `manual`, `agent` or `import`.
#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct Transaction {
    pub id: String,
    pub occurred_at: String,
    pub amount_cents: i64,
    pub category: String,
    pub description: String,
    pub kind: String,
    pub source: String,
    pub created_at: String,
    #[serde(default)]
    pub items: Vec<TransactionItem>,
}

/// A category's monthly limit and what was spent against it in the summary's month.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct Budget {
    pub category: String,
    pub limit_cents: i64,
    pub spent_cents: i64,
}

/// One money month (`month` is `YYYY-MM`, the month it starts in).
#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct MoneySummary {
    pub month: String,
    #[serde(default = "default_start_day")]
    pub start_day: u32,
    pub period_start: String,
    pub period_end: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub total_cents: i64,
    pub income_cents: i64,
    pub previous_total_cents: i64,
    pub transaction_count: i64,
    #[serde(default)]
    pub by_category: Vec<CategoryTotal>,
    #[serde(default)]
    pub by_day: Vec<DayTotal>,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
    /// Months that have transactions, newest first.
    #[serde(default)]
    pub months: Vec<String>,
    #[serde(default)]
    pub budgets: Vec<Budget>,
}

/// A line of a new transaction: its amount, or its unit price (times `quantity`).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NewTransactionItem {
    pub name: String,
    pub quantity: f64,
    pub unit_amount_cents: Option<i64>,
    pub amount_cents: Option<i64>,
    pub category: Option<String>,
}

/// A transaction to add. Without `amount_cents`, the items add up to it. `occurred_at` (RFC 3339)
/// defaults to now; `kind` is `expense` or `income`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct NewTransaction {
    pub kind: String,
    pub amount_cents: Option<i64>,
    pub category: String,
    pub description: String,
    pub occurred_at: Option<String>,
    pub items: Vec<NewTransactionItem>,
}

#[derive(Serialize)]
struct TransactionBody<'a> {
    kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<f64>,
    category: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    occurred_at: Option<&'a str>,
    items: Vec<ItemBody<'a>>,
}

#[derive(Serialize)]
struct ItemBody<'a> {
    name: &'a str,
    quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<&'a str>,
}

#[derive(Serialize)]
struct BudgetBody<'a> {
    category: &'a str,
    monthly_limit: f64,
}

fn units(cents: i64) -> f64 {
    cents as f64 / CENTS_PER_UNIT
}

fn default_timezone() -> String {
    DEFAULT_TIMEZONE.to_owned()
}

fn default_start_day() -> u32 {
    DEFAULT_START_DAY
}

impl Services {
    /// The money month starting in `month` (`YYYY-MM`), or the current one.
    pub async fn money_summary(&self, month: Option<String>) -> Result<MoneySummary, NomiError> {
        match month.as_deref() {
            Some(month) => self.get(&[MONEY], &[(PARAM_MONTH, month)]).await,
            None => self.get(&[MONEY], &[]).await,
        }
    }

    pub async fn add_transaction(&self, transaction: NewTransaction) -> Result<(), NomiError> {
        let body = TransactionBody {
            kind: &transaction.kind,
            amount: transaction.amount_cents.map(units),
            category: &transaction.category,
            description: &transaction.description,
            occurred_at: transaction.occurred_at.as_deref(),
            items: transaction
                .items
                .iter()
                .map(|item| ItemBody {
                    name: &item.name,
                    quantity: item.quantity,
                    unit_amount: item.unit_amount_cents.map(units),
                    amount: item.amount_cents.map(units),
                    category: item.category.as_deref(),
                })
                .collect(),
        };
        self.call(Method::POST, &[MONEY, TRANSACTIONS], Some(&body))
            .await
    }

    /// Sets `category`'s monthly budget.
    pub async fn set_budget(&self, category: String, limit_cents: i64) -> Result<(), NomiError> {
        let body = BudgetBody {
            category: &category,
            monthly_limit: units(limit_cents),
        };
        self.call(Method::PUT, &[MONEY, BUDGETS], Some(&body)).await
    }
}
