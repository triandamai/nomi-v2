//! What Settings › Profile works out from the person's own usage (`llm_usage`), with no model
//! call: the hours they're most active, and insights such as a cheaper model that would have
//! answered their quick questions, or that most of their tokens land late at night.

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_i18n::Locale;

/// How far back the hours and insights look.
pub const INSIGHT_WINDOW_DAYS: i32 = 30;
/// The width of "most active" (e.g. 21:00–23:00).
pub const ACTIVE_WINDOW_HOURS: usize = 2;
/// Fewer tokens than this in the window say too little to name hours or habits.
pub const MIN_TOKENS_FOR_HOURS: i64 = 1_000;
/// "Late" is from this local hour…
pub const LATE_HOURS_FROM: usize = 21;
/// …until this one, after midnight.
pub const LATE_HOURS_UNTIL: usize = 4;
/// The share of tokens (percent) that has to land late to call it a habit.
pub const LATE_SHARE_MIN_PERCENT: i64 = 50;
/// A reply this short (output tokens) counts as a quick question.
pub const QUICK_QUESTION_MAX_OUTPUT_TOKENS: i32 = 300;
/// How many recent quick questions are looked at.
pub const QUICK_QUESTION_SAMPLE: i64 = 10;
/// At least this many of them must have been cheaper on the other model.
pub const MIN_CHEAPER_ANSWERS: usize = 5;
/// The saving (percent of the window's spend) worth mentioning.
pub const MIN_SAVING_PERCENT: f64 = 5.0;
/// Model prices are per million tokens.
const TOKENS_PER_MTOK: f64 = 1_000_000.0;
const HOURS_PER_DAY: usize = 24;
const PERCENT: f64 = 100.0;
/// Where the person picks the model their chats use.
const MODELS_PAGE: &str = "/models";

/// The tokens counted for a call, as the profile counts them elsewhere (cached input at a tenth).
const TOKENS_SQL: &str = "COALESCE(ROUND(SUM(input_tokens - cached_input_tokens + output_tokens + cached_input_tokens * 0.1)), 0)::bigint";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ActiveHours {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InsightAction {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Insight {
    /// "cheaper_model" or "late_hours".
    pub kind: &'static str,
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<InsightAction>,
}

fn hour_label(hour: usize) -> String {
    format!("{:02}:00", hour % HOURS_PER_DAY)
}

/// The `width` consecutive hours (running over midnight) with the most tokens: their first hour.
/// `None` when there's too little to say.
pub fn busiest_window(tokens_by_hour: &[i64; HOURS_PER_DAY], width: usize) -> Option<usize> {
    let total: i64 = tokens_by_hour.iter().sum();
    if total < MIN_TOKENS_FOR_HOURS || width == 0 {
        return None;
    }
    let window = |start: usize| {
        (0..width)
            .map(|i| tokens_by_hour[(start + i) % HOURS_PER_DAY])
            .sum::<i64>()
    };
    // The earliest start wins a tie.
    (0..HOURS_PER_DAY).max_by(|a, b| window(*a).cmp(&window(*b)).then(b.cmp(a)))
}

/// The share of tokens (percent, rounded) that land from `LATE_HOURS_FROM` until
/// `LATE_HOURS_UNTIL`; `None` when there's too little to say.
pub fn late_share_percent(tokens_by_hour: &[i64; HOURS_PER_DAY]) -> Option<i64> {
    let total: i64 = tokens_by_hour.iter().sum();
    if total < MIN_TOKENS_FOR_HOURS {
        return None;
    }
    let late: i64 = (LATE_HOURS_FROM..HOURS_PER_DAY)
        .chain(0..LATE_HOURS_UNTIL)
        .map(|h| tokens_by_hour[h])
        .sum();
    Some(((late as f64 / total as f64) * PERCENT).round() as i64)
}

async fn tokens_by_hour(pool: &PgPool, user_id: Uuid) -> Result<[i64; HOURS_PER_DAY], sqlx::Error> {
    let rows: Vec<(i32, i64)> = sqlx::query_as(&format!(
        "SELECT EXTRACT(HOUR FROM created_at AT TIME ZONE COALESCE((SELECT timezone FROM user_preferences WHERE user_id = $1), 'UTC'))::int AS hour, \
                {TOKENS_SQL} \
         FROM llm_usage WHERE user_id = $1 AND created_at >= now() - make_interval(days => $2) GROUP BY hour"
    ))
    .bind(user_id)
    .bind(INSIGHT_WINDOW_DAYS)
    .fetch_all(pool)
    .await?;
    let mut hours = [0; HOURS_PER_DAY];
    for (hour, tokens) in rows {
        if let Some(slot) = usize::try_from(hour).ok().and_then(|h| hours.get_mut(h)) {
            *slot = tokens;
        }
    }
    Ok(hours)
}

/// The cheapest chat model Nomi offers with known prices: id, label, input and output USD per
/// million tokens.
async fn cheapest_model(pool: &PgPool) -> Result<Option<(Uuid, String, f64, f64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, label, input_usd_per_mtok::float8, output_usd_per_mtok::float8 FROM admin_llm_models \
         WHERE enabled AND NOT is_files_model AND NOT is_coding_model \
           AND input_usd_per_mtok IS NOT NULL AND output_usd_per_mtok IS NOT NULL \
         ORDER BY input_usd_per_mtok + output_usd_per_mtok, label LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

/// "Save ~N% on Nomi": the cheapest model would have answered most of the recent quick questions
/// for less, and over the window that adds up to a share of the spend worth mentioning.
async fn cheaper_model(
    pool: &PgPool,
    user_id: Uuid,
    locale: Locale,
) -> Result<Option<Insight>, sqlx::Error> {
    let Some((cheap_id, cheap_label, input_price, output_price)) = cheapest_model(pool).await?
    else {
        return Ok(None);
    };
    let cheap_cost = |input: i64, output: i64| {
        (input as f64 * input_price + output as f64 * output_price) / TOKENS_PER_MTOK
    };

    let sample: Vec<(Option<Uuid>, i64, i64, f64)> = sqlx::query_as(
        "SELECT admin_model_id, input_tokens::bigint, output_tokens::bigint, cost_usd::float8 FROM llm_usage \
         WHERE user_id = $1 AND source = 'nomi' AND output_tokens <= $2 AND created_at >= now() - make_interval(days => $3) \
         ORDER BY created_at DESC LIMIT $4",
    )
    .bind(user_id)
    .bind(QUICK_QUESTION_MAX_OUTPUT_TOKENS)
    .bind(INSIGHT_WINDOW_DAYS)
    .bind(QUICK_QUESTION_SAMPLE)
    .fetch_all(pool)
    .await?;
    let cheaper = sample
        .iter()
        .filter(|(model, input, output, cost)| {
            *model != Some(cheap_id) && cheap_cost(*input, *output) < *cost
        })
        .count();
    if cheaper < MIN_CHEAPER_ANSWERS {
        return Ok(None);
    }

    let (spend, saving): (f64, f64) = sqlx::query_as(
        "SELECT COALESCE(SUM(cost_usd), 0)::float8, \
                COALESCE(SUM(GREATEST(cost_usd::float8 - (input_tokens * $4 + output_tokens * $5) / $6, 0)) \
                  FILTER (WHERE source = 'nomi' AND output_tokens <= $3 AND admin_model_id IS DISTINCT FROM $7), 0)::float8 \
         FROM llm_usage WHERE user_id = $1 AND created_at >= now() - make_interval(days => $2)",
    )
    .bind(user_id)
    .bind(INSIGHT_WINDOW_DAYS)
    .bind(QUICK_QUESTION_MAX_OUTPUT_TOKENS)
    .bind(input_price)
    .bind(output_price)
    .bind(TOKENS_PER_MTOK)
    .bind(cheap_id)
    .fetch_one(pool)
    .await?;
    if spend <= 0.0 {
        return Ok(None);
    }
    let percent = (saving / spend * PERCENT).round();
    if percent < MIN_SAVING_PERCENT {
        return Ok(None);
    }
    let percent = format!("{percent:.0}");
    Ok(Some(Insight {
        kind: "cheaper_model",
        title: locale.tf(
            "profile_insights.cheaper_model_title",
            &[("percent", &percent)],
        ),
        body: locale.tf(
            "profile_insights.cheaper_model_body",
            &[
                ("model", &cheap_label),
                ("count", &cheaper.to_string()),
                ("sample", &sample.len().to_string()),
            ],
        ),
        action: Some(InsightAction {
            label: locale.t("profile_insights.cheaper_model_action"),
            href: MODELS_PAGE.to_string(),
        }),
    }))
}

/// The hours the person is most active, and the insights, for their own profile.
pub async fn load(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(Option<ActiveHours>, Vec<Insight>), sqlx::Error> {
    let locale = {
        let mut conn = pool.acquire().await?;
        nomi_agent_core::locale::user_locale(&mut conn, user_id).await
    };
    let hours = tokens_by_hour(pool, user_id).await?;
    let active_hours = busiest_window(&hours, ACTIVE_WINDOW_HOURS).map(|start| ActiveHours {
        from: hour_label(start),
        to: hour_label(start + ACTIVE_WINDOW_HOURS),
    });

    let mut insights = Vec::new();
    if let Some(insight) = cheaper_model(pool, user_id, locale).await? {
        insights.push(insight);
    }
    if let Some(percent) = late_share_percent(&hours).filter(|p| *p >= LATE_SHARE_MIN_PERCENT) {
        insights.push(Insight {
            kind: "late_hours",
            title: locale.t("profile_insights.late_hours_title"),
            body: locale.tf(
                "profile_insights.late_hours_body",
                &[
                    ("percent", &percent.to_string()),
                    ("hour", &hour_label(LATE_HOURS_FROM)),
                ],
            ),
            action: None,
        });
    }
    Ok((active_hours, insights))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hours(pairs: &[(usize, i64)]) -> [i64; HOURS_PER_DAY] {
        let mut hours = [0; HOURS_PER_DAY];
        for (hour, tokens) in pairs {
            hours[*hour] = *tokens;
        }
        hours
    }

    #[test]
    fn the_busiest_window_can_run_over_midnight() {
        assert_eq!(
            busiest_window(&hours(&[(21, 900), (22, 800), (9, 300)]), 2),
            Some(21)
        );
        assert_eq!(
            busiest_window(&hours(&[(23, 900), (0, 800), (9, 300)]), 2),
            Some(23)
        );
        assert_eq!(
            busiest_window(&hours(&[(9, 10)]), 2),
            None,
            "too little to say"
        );
        assert_eq!(hour_label(23 + ACTIVE_WINDOW_HOURS), "01:00");
    }

    #[test]
    fn late_share_counts_the_hours_after_nine_until_four() {
        assert_eq!(
            late_share_percent(&hours(&[(22, 600), (2, 80), (10, 320)])),
            Some(68)
        );
        assert_eq!(late_share_percent(&hours(&[(4, 1_000)])), Some(0));
        assert_eq!(late_share_percent(&hours(&[(22, 10)])), None);
    }
}
