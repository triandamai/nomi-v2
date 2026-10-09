//! Each person's monthly allowance of tokens on Nomi's own models (their plan's, or an admin's
//! override). Checked whenever a model is built for them: under the allowance, Nomi's model
//! answers; over it, the API key they saved answers instead, and with no key nothing does
//! (`LlmError::QuotaExceeded`). They're told once a month at 80%, and once when it runs out.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_i18n::Locale;
use nomi_llm::{LlmError, LlmEventStream, LlmProvider, LlmRequest};

use crate::notifications::{self, Kind, Notice};

/// Share of the allowance at which people are warned.
const WARNING_SHARE: f64 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Within the allowance.
    Within,
    /// Allowance used up.
    UsedUp,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub month: String,
    pub used: i64,
    pub quota: i64,
}

impl Usage {
    pub fn standing(&self) -> Standing {
        if self.used >= self.quota {
            Standing::UsedUp
        } else {
            Standing::Within
        }
    }
}

pub async fn usage(pool: &PgPool, user_id: Uuid) -> Result<Usage, sqlx::Error> {
    let brief = nomi_usage::brief(pool, user_id).await?;
    Ok(Usage { month: brief.month, used: brief.tokens_used, quota: brief.plan.monthly_tokens })
}

/// `1234567` → "1,234,567" (English) or "1.234.567" (Indonesian).
pub fn format_tokens(n: i64, locale: Locale) -> String {
    let separator = if locale == Locale::Id { '.' } else { ',' };
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(c);
    }
    if n < 0 {
        out.insert(0, '-');
    }
    out
}

async fn locale_of(pool: &PgPool, user_id: Uuid) -> Locale {
    let code: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    Locale::from_code_or_default(code.as_deref())
}

/// Sends an alert the first time this month it applies; later calls do nothing.
async fn alert_once(pool: &PgPool, user_id: Uuid, month: &str, level: &str, notice: impl FnOnce(Locale) -> Notice) {
    let first: Option<String> = sqlx::query_scalar(
        "INSERT INTO quota_alerts (user_id, month, level) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING RETURNING level",
    )
    .bind(user_id)
    .bind(month)
    .bind(level)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, %user_id, "failed to record a quota alert");
        None
    });
    if first.is_some() {
        let locale = locale_of(pool, user_id).await;
        if let Err(e) = notifications::send(pool, user_id, notice(locale)).await {
            tracing::warn!(error = %e, %user_id, "failed to send a quota alert");
        }
    }
}

/// Warns at 80%, and says so once the allowance runs out: that Nomi switched to their own key
/// (`own_key_model` names it), or that it can't answer until next month.
pub async fn alert(pool: &PgPool, user_id: Uuid, usage: &Usage, own_key_model: Option<&str>) {
    let link = Some("/billing".to_string());
    match usage.standing() {
        Standing::Within if usage.quota > 0 && usage.used as f64 >= usage.quota as f64 * WARNING_SHARE => {
            let percent = ((usage.used as f64 / usage.quota as f64) * 100.0).floor() as i64;
            alert_once(pool, user_id, &usage.month, "warning", |l| Notice {
                kind: Kind::Quota,
                title: l.tf("quota.warning_title", &[("percent", &percent.to_string())]),
                body: l.tf("quota.warning_body", &[("used", &format_tokens(usage.used, l)), ("quota", &format_tokens(usage.quota, l))]),
                link: link.clone(),
            })
            .await
        }
        Standing::Within => {}
        Standing::UsedUp => match own_key_model {
            Some(model) => {
                alert_once(pool, user_id, &usage.month, "own_key", |l| Notice {
                    kind: Kind::Quota,
                    title: l.t("quota.own_key_title"),
                    body: l.tf("quota.own_key_body", &[("quota", &format_tokens(usage.quota, l)), ("model", model)]),
                    link: Some("/models".to_string()),
                })
                .await
            }
            None => {
                alert_once(pool, user_id, &usage.month, "used_up", |l| Notice {
                    kind: Kind::Quota,
                    title: l.t("quota.used_up_title"),
                    body: l.tf("quota.used_up_body", &[("quota", &format_tokens(usage.quota, l))]),
                    link: link.clone(),
                })
                .await
            }
        },
    }
}

/// Answers every request with `LlmError::QuotaExceeded`: what someone over their allowance with
/// no key of their own gets.
pub struct QuotaBlocked;

#[async_trait]
impl LlmProvider for QuotaBlocked {
    async fn complete_stream(&self, _request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        Err(LlmError::QuotaExceeded)
    }
}

pub fn blocked() -> Arc<dyn LlmProvider> {
    Arc::new(QuotaBlocked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_tokens_per_language() {
        assert_eq!(format_tokens(1_000_000, Locale::En), "1,000,000");
        assert_eq!(format_tokens(1_000_000, Locale::Id), "1.000.000");
        assert_eq!(format_tokens(999, Locale::En), "999");
    }

    #[test]
    fn standing_turns_at_the_allowance() {
        let usage = |used| Usage { month: "2026-10".into(), used, quota: 100 };
        assert_eq!(usage(99).standing(), Standing::Within);
        assert_eq!(usage(100).standing(), Standing::UsedUp);
    }
}
