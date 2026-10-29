//! Worker that executes `background_tasks` asynchronously and spawns Nomi (`turn_jobs` wakeup)
//! when each background task finishes.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::background::{
    execute_shell_command, format_wakeup_marker, BackgroundCommandOutcome, BackgroundWakeup,
    BACKGROUND_NOTIFY_CHANNEL,
};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedBackgroundTask {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub agent_type: String,
    pub label: String,
    pub command: String,
    pub project_id: Option<Uuid>,
    pub timeout_seconds: i32,
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedBackgroundTask>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, Uuid, String, String, String, Option<Uuid>, i32)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM background_tasks \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE background_tasks SET status = 'running', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, session_id, user_id, agent_type, label, command, project_id, timeout_seconds",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(id, session_id, user_id, agent_type, label, command, project_id, timeout_seconds)| ClaimedBackgroundTask {
            id,
            session_id,
            user_id,
            agent_type,
            label,
            command,
            project_id,
            timeout_seconds,
        },
    ))
}

async fn resolve_sender_identity(pool: &PgPool, session_id: Uuid, user_id: Uuid) -> Result<Uuid, String> {
    let from_session: Option<Uuid> = sqlx::query_scalar(
        "SELECT sender_channel_identity_id FROM messages \
         WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(id) = from_session {
        return Ok(id);
    }

    let from_user: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM channel_identities WHERE user_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(id) = from_user {
        return Ok(id);
    }

    nomi_turn::bootstrap::ensure_web_channel_identity(pool, user_id)
        .await
        .map_err(|e| e.to_string())
}

async fn run_task_command(pool: &PgPool, claimed: &ClaimedBackgroundTask) -> BackgroundCommandOutcome {
    let timeout = Duration::from_secs(claimed.timeout_seconds.max(1) as u64);

    if let Some(project_id) = claimed.project_id {
        if let Ok(mut conn) = pool.acquire().await {
            if nomi_agent_coding::runtime::runner_online(&mut conn, project_id).await.unwrap_or(false) {
                let input = json!({
                    "project_id": project_id.to_string(),
                    "command": claimed.command,
                    "timeout_seconds": claimed.timeout_seconds,
                });
                match nomi_agent_coding::runtime::run_command(&mut conn, claimed.user_id, input).await {
                    Ok(output) => return parse_project_runner_output(output),
                    Err(err) => {
                        return BackgroundCommandOutcome {
                            status: "failed",
                            exit_code: None,
                            output: err.clone(),
                            error: Some(err),
                        };
                    }
                }
            }
        }
    }

    execute_shell_command(&claimed.command, timeout).await
}

pub fn parse_project_runner_output(output: String) -> BackgroundCommandOutcome {
    let exit_code = output
        .lines()
        .nth(1)
        .and_then(|line| line.strip_prefix("exit code "))
        .and_then(|c| c.trim().parse::<i32>().ok());
    let failed = exit_code.is_some_and(|c| c != 0)
        || output.lines().nth(1).is_some_and(|l| l.starts_with("stopped waiting after"));
    BackgroundCommandOutcome {
        status: if failed { "failed" } else { "completed" },
        exit_code,
        output,
        error: if failed { Some("project command failed".to_string()) } else { None },
    }
}

pub fn build_wakeup_marker_for_outcome(
    claimed: &ClaimedBackgroundTask,
    outcome: &BackgroundCommandOutcome,
) -> String {
    let wakeup = BackgroundWakeup {
        task_id: claimed.id,
        agent_type: claimed.agent_type.clone(),
        label: claimed.label.clone(),
        command: claimed.command.clone(),
        status: outcome.status.to_string(),
        exit_code: outcome.exit_code,
        output: outcome.output.clone(),
    };
    format_wakeup_marker(&wakeup)
}

/// Executes one claimed background task, updates its row in `background_tasks`, and enqueues a
/// wakeup turn job to spawn Nomi with the result (unless the task was cancelled while running).
pub async fn process_claimed_task(
    pool: &PgPool,
    mqtt: Option<&MqttPublisher>,
    claimed: ClaimedBackgroundTask,
) -> Result<Option<Uuid>, String> {
    let outcome = run_task_command(pool, &claimed).await;

    let updated = sqlx::query(
        "UPDATE background_tasks \
         SET status = $2, exit_code = $3, output = $4, error = $5, completed_at = now() \
         WHERE id = $1 AND status = 'running'",
    )
    .bind(claimed.id)
    .bind(outcome.status)
    .bind(outcome.exit_code)
    .bind(&outcome.output)
    .bind(&outcome.error)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();

    if let Some(publisher) = mqtt {
        let _ = publisher
            .publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id })
            .await;
    }

    // If the task was cancelled while running, do not spawn Nomi.
    if updated == 0 {
        return Ok(None);
    }

    let sender_channel_identity_id = resolve_sender_identity(pool, claimed.session_id, claimed.user_id).await?;
    let marker = build_wakeup_marker_for_outcome(&claimed, &outcome);

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let turn_job_id = nomi_turn::queue::enqueue(&mut tx, claimed.session_id, sender_channel_identity_id, &marker, None)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(Some(turn_job_id))
}

fn background_concurrency() -> usize {
    std::env::var("BACKGROUND_TASK_CONCURRENCY")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(8)
}

pub async fn run(pool: PgPool, mqtt: MqttPublisher, database_url: String) {
    let mut listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "background worker: failed to connect LISTEN client; not started");
            return;
        }
    };
    if let Err(e) = listener.listen(BACKGROUND_NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "background worker: failed to LISTEN on background_tasks_channel; not started");
        return;
    }
    tracing::info!("background worker: listening for background tasks");

    let slots = Arc::new(tokio::sync::Semaphore::new(background_concurrency()));
    let finished = Arc::new(tokio::sync::Notify::new());

    loop {
        tokio::select! {
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()) => {}
            _ = finished.notified() => {}
        }

        loop {
            let Ok(slot) = slots.clone().try_acquire_owned() else {
                break;
            };
            let claimed = match claim_next(&pool).await {
                Ok(Some(task)) => task,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "background worker: failed to claim next background task");
                    break;
                }
            };
            let pool = pool.clone();
            let mqtt = mqtt.clone();
            let finished = finished.clone();
            tokio::spawn(async move {
                let _slot = slot;
                if let Err(e) = process_claimed_task(&pool, Some(&mqtt), claimed).await {
                    tracing::warn!(error = %e, "background worker: failed to process background task");
                }
                finished.notify_one();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_agent_core::background::parse_wakeup_marker;

    #[test]
    fn parse_project_runner_output_distinguishes_success_failure_and_timeout() {
        let ok = parse_project_runner_output("$ npm test\nexit code 0\n\nall pass".to_string());
        assert_eq!(ok.status, "completed");
        assert_eq!(ok.exit_code, Some(0));
        assert!(ok.error.is_none());

        let failed = parse_project_runner_output("$ npm test\nexit code 1\n\n1 failing".to_string());
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.exit_code, Some(1));
        assert!(failed.error.is_some());

        let timed_out = parse_project_runner_output("$ sleep 99\nstopped waiting after 10s\n\n".to_string());
        assert_eq!(timed_out.status, "failed");
        assert_eq!(timed_out.exit_code, None);
        assert!(timed_out.error.is_some());
    }

    #[test]
    fn build_wakeup_marker_for_outcome_round_trips_for_nomi_spawn() {
        let claimed = ClaimedBackgroundTask {
            id: Uuid::nil(),
            session_id: Uuid::nil(),
            user_id: Uuid::nil(),
            agent_type: "money".to_string(),
            label: "monthly-ledger-reconcile".to_string(),
            command: "echo reconciled".to_string(),
            project_id: None,
            timeout_seconds: 120,
        };
        let outcome = BackgroundCommandOutcome {
            status: "completed",
            exit_code: Some(0),
            output: "$ echo reconciled\nexit code 0\n\nreconciled".to_string(),
            error: None,
        };

        let marker = build_wakeup_marker_for_outcome(&claimed, &outcome);
        let parsed = parse_wakeup_marker(&marker).expect("valid wakeup marker");
        assert_eq!(parsed.task_id, claimed.id);
        assert_eq!(parsed.agent_type, "money");
        assert_eq!(parsed.label, "monthly-ledger-reconcile");
        assert_eq!(parsed.status, "completed");
        assert!(parsed.prompt_note().contains("You are Nomi, spawned because this background task finished"));
    }
}
