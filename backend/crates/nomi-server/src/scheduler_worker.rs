use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, LoopOutcome, NotificationDelivery};
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};

const POLL_INTERVAL: Duration = Duration::from_secs(30);
const REMINDER_MAX_TOKENS: u32 = 1024;

// pub, not pub(crate): backend/crates/nomi-server/tests/scheduler_worker.rs is a separate
// integration-test crate and needs to call claim_next/process_claimed_job and construct/read
// ClaimedJob directly to test the claim loop and job processing without spinning the infinite
// polling loop in `run`.
pub struct ClaimedJob {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub target_agent_type: String,
    pub prompt: String,
    pub recurrence: Option<String>,
    pub recurrence_weekday: Option<i16>,
    pub recurrence_day_of_month: Option<i16>,
    pub run_at: DateTime<Utc>,
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedJob>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, Uuid, String, String, Option<String>, Option<i16>, Option<i16>, DateTime<Utc>)> =
        sqlx::query_as(
            "WITH claimed AS ( \
                 SELECT id FROM scheduled_jobs \
                 WHERE status = 'active' AND run_at <= now() AND claimed_at IS NULL \
                 ORDER BY run_at \
                 FOR UPDATE SKIP LOCKED \
                 LIMIT 1 \
             ) \
             UPDATE scheduled_jobs SET claimed_at = now() \
             WHERE id IN (SELECT id FROM claimed) \
             RETURNING id, session_id, user_id, target_agent_type, prompt, recurrence, \
                       recurrence_weekday, recurrence_day_of_month, run_at",
        )
        .fetch_optional(pool)
        .await?;

    Ok(row.map(
        |(id, session_id, user_id, target_agent_type, prompt, recurrence, recurrence_weekday, recurrence_day_of_month, run_at)| ClaimedJob {
            id,
            session_id,
            user_id,
            target_agent_type,
            prompt,
            recurrence,
            recurrence_weekday,
            recurrence_day_of_month,
            run_at,
        },
    ))
}

pub(crate) fn next_occurrence(
    from: DateTime<Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> DateTime<Utc> {
    match recurrence {
        "daily" => from + chrono::Duration::days(1),
        "weekly" => {
            // `weekday` is 0=Sunday..6=Saturday (matches the create_reminder tool schema's
            // description). chrono's Weekday::num_days_from_sunday() uses the same convention.
            let target = weekday.unwrap_or(0) as u32;
            let mut candidate = from + chrono::Duration::days(1);
            while candidate.weekday().num_days_from_sunday() != target {
                candidate += chrono::Duration::days(1);
            }
            candidate
        }
        "monthly" => {
            let day = day_of_month.unwrap_or(1) as u32;
            let (mut year, mut month) = (from.year(), from.month());
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            let last_day_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .checked_add_months(chrono::Months::new(1))
                .unwrap()
                .pred_opt()
                .unwrap()
                .day();
            let clamped_day = day.min(last_day_of_month);
            // Step through day=1 first: with_month()/with_year() reject a result that isn't a
            // real calendar date, and `from`'s own day-of-month (e.g. 31) may not exist in the
            // target month — day=1 always does, in every month.
            from.with_day(1).unwrap().with_year(year).unwrap().with_month(month).unwrap().with_day(clamped_day).unwrap()
        }
        _ => from,
    }
}

async fn finish_one_time_or_advance_recurring(pool: &PgPool, job: &ClaimedJob) {
    match &job.recurrence {
        Some(recurrence) => {
            let next_run_at = next_occurrence(job.run_at, recurrence, job.recurrence_weekday, job.recurrence_day_of_month);
            let _ = sqlx::query("UPDATE scheduled_jobs SET run_at = $2, claimed_at = NULL, last_fired_at = now() WHERE id = $1")
                .bind(job.id)
                .bind(next_run_at)
                .execute(pool)
                .await;
        }
        None => {
            let _ = sqlx::query("UPDATE scheduled_jobs SET status = 'completed', last_fired_at = now() WHERE id = $1")
                .bind(job.id)
                .execute(pool)
                .await;
        }
    }
}

/// Runs one claimed job to completion: looks up its target agent, runs a real agent turn with
/// `job.prompt` as the task, posts the result as a chat message authored by the target agent
/// itself (not "Supervisor" — a fired reminder is the literal deliverable the user asked for, not
/// a background status report), calls `notification.deliver` best-effort, then either completes
/// (one-time) or advances `run_at` (recurring). Exposed as a standalone function (rather than
/// inlined into `run`'s loop) so it's directly testable without spinning the infinite polling loop.
#[allow(clippy::too_many_arguments)]
pub async fn process_claimed_job(
    pool: &PgPool,
    mqtt: Option<&MqttPublisher>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn nomi_llm::LlmProvider,
    embedding_provider: &dyn nomi_embedding::EmbeddingProvider,
    registry: &AgentRegistry,
    notification: &dyn NotificationDelivery,
    job: ClaimedJob,
) {
    let Some(agent) = registry.find(&job.target_agent_type) else {
        tracing::warn!(job_id = %job.id, target = %job.target_agent_type, "scheduler worker: unknown target agent, cancelling job");
        let _ = sqlx::query("UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() WHERE id = $1")
            .bind(job.id)
            .execute(pool)
            .await;
        return;
    };

    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::warn!(job_id = %job.id, error = %e, "scheduler worker: failed to acquire connection");
            return;
        }
    };

    let messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: job.prompt.clone() }] }];

    let outcome = nomi_agent_core::run_agent_turn(
        &mut conn,
        mqtt.map(|m| (m, job.id)),
        s3,
        provider,
        embedding_provider,
        registry,
        agent.as_ref(),
        job.session_id,
        job.session_id,
        job.user_id,
        messages,
        REMINDER_MAX_TOKENS,
    )
    .await;

    match outcome {
        Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. }) => {
            let message_id: Option<Uuid> = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(job.session_id)
            .bind(&text)
            .bind(agent.display_name().as_ref())
            .fetch_one(&mut *conn)
            .await
            .ok();
            if let (Some(message_id), Some(publisher)) = (message_id, mqtt) {
                let _ = publisher.publish(job.session_id, &StreamEnvelope::MessageCreated { message_id }).await;
            }
            let _ = notification.deliver(job.user_id, &text).await;
            finish_one_time_or_advance_recurring(pool, &job).await;
        }
        Ok(LoopOutcome::AwaitingApproval { .. }) => {
            tracing::info!(job_id = %job.id, "scheduler worker: fired reminder needed a tool approval it can't get unattended");
            let notice = "Your reminder needed a tool approval it can't get automatically, so it didn't complete.";
            let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3)")
                .bind(job.session_id)
                .bind(notice)
                .bind(agent.display_name().as_ref())
                .execute(&mut *conn)
                .await;
            finish_one_time_or_advance_recurring(pool, &job).await;
        }
        Err(e) => {
            tracing::warn!(job_id = %job.id, error = %e, "scheduler worker: fired reminder failed");
            let sorry = format!("I wasn't able to complete your reminder — {e}.");
            let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3)")
                .bind(job.session_id)
                .bind(&sorry)
                .bind(agent.display_name().as_ref())
                .execute(&mut *conn)
                .await;
            finish_one_time_or_advance_recurring(pool, &job).await;
        }
    }
}

/// Runs the scheduler-worker loop forever: every `POLL_INTERVAL`, claims and fires every due
/// `scheduled_jobs` row. Poll-only, unlike `delegation_worker.rs`'s LISTEN/NOTIFY — the trigger
/// here is elapsed time, not a row insert, so there's nothing useful for a notify to wake early.
pub async fn run(
    pool: PgPool,
    mqtt: MqttPublisher,
    s3: Option<nomi_storage::S3Config>,
    settings_key: [u8; 32],
    http_client: reqwest::Client,
    project_storage: nomi_storage::LocalFsStore,
    notification: Arc<dyn NotificationDelivery>,
) {
    tracing::info!("scheduler worker: polling for due reminders every {:?}", POLL_INTERVAL);
    let registry = crate::build_agent_registry(project_storage);

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        loop {
            let claimed = match claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "scheduler worker: failed to claim next job");
                    break;
                }
            };

            let provider = build_llm_provider_for_user(&pool, claimed.user_id, &settings_key, http_client.clone()).await;
            let embedding_provider = build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            process_claimed_job(&pool, Some(&mqtt), s3.as_ref(), provider.as_ref(), embedding_provider.as_ref(), &registry, notification.as_ref(), claimed).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn daily_advances_by_exactly_24_hours() {
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "daily", None, None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 21, 11, 0, 0).unwrap());
    }

    #[test]
    fn weekly_advances_to_the_next_matching_weekday() {
        // 2026-09-20 is a Sunday (weekday 0). Target weekday 3 = Wednesday.
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "weekly", Some(3), None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 23, 11, 0, 0).unwrap());
    }

    #[test]
    fn weekly_rolls_a_full_week_when_today_already_matches() {
        // 2026-09-20 is a Sunday (weekday 0). Target weekday 0 = Sunday.
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "weekly", Some(0), None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 27, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_advances_to_the_next_month_same_day() {
        let from = Utc.with_ymd_and_hms(2026, 9, 15, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(15));
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 15, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_clamps_to_the_target_months_last_day() {
        // day_of_month = 31 scheduled from January lands on February's last valid day (28, 2026
        // is not a leap year), not an invalid date and not rolling into March.
        let from = Utc.with_ymd_and_hms(2026, 1, 31, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(31));
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 2, 28, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_wraps_from_december_into_january_of_the_next_year() {
        let from = Utc.with_ymd_and_hms(2026, 12, 10, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(10));
        assert_eq!(next, Utc.with_ymd_and_hms(2027, 1, 10, 11, 0, 0).unwrap());
    }
}
