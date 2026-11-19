use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use nomi_agent_core::TurnError;

/// How long a deny can be taken back, on the server's clock. The turn doesn't carry on from a
/// deny until it has passed.
pub const UNDO_WINDOW_SECONDS: i64 = 10;

/// Whether a deny made at `decided_at` can still be undone at `now`.
pub fn undo_open(decided_at: chrono::DateTime<chrono::Utc>, now: chrono::DateTime<chrono::Utc>) -> bool {
    now - decided_at < chrono::Duration::seconds(UNDO_WINDOW_SECONDS)
}

/// What a decision carries beyond approve or deny, all optional.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extras {
    /// The person's edits to the tool's input; the tool runs on this.
    pub edited_input: Option<serde_json::Value>,
    /// "Don't ask again" limited to one thing (today: `recipient`); `remember` alone covers the tool.
    pub remember_scope: Option<String>,
    pub remember_value: Option<String>,
}

pub struct ClaimedApprovalResume {
    pub id: Uuid,
    pub message_id: Uuid,
    pub decision: String,
    pub remember: bool,
    pub extras: Extras,
}

pub async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<Uuid, TurnError> {
    enqueue_with(tx, message_id, decision, remember, &Extras::default()).await
}

pub async fn enqueue_with(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    decision: &str,
    remember: bool,
    extras: &Extras,
) -> Result<Uuid, TurnError> {
    // One decision per card: a second click (double-click, or two tabs) while the first is still
    // queued or running is dropped, instead of running a second resume that can only fail.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))").bind(format!("approval:{message_id}")).execute(&mut **tx).await?;
    let in_flight: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM approval_resumes WHERE message_id = $1 AND status IN ('pending', 'processing') LIMIT 1",
    )
    .bind(message_id)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(id) = in_flight {
        return Ok(id);
    }

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO approval_resumes (message_id, decision, remember, edited_input, remember_scope, remember_value, run_after) \
         VALUES ($1, $2, $3, $4, $5, $6, now() + CASE WHEN $2 = 'deny' THEN make_interval(secs => $7) ELSE interval '0' END) RETURNING id",
    )
    .bind(message_id)
    .bind(decision)
    .bind(remember)
    .bind(&extras.edited_input)
    .bind(&extras.remember_scope)
    .bind(&extras.remember_value)
    .bind(UNDO_WINDOW_SECONDS as f64)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query("SELECT pg_notify('approval_resumes_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **tx)
        .await?;

    Ok(id)
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedApprovalResume>, TurnError> {
    let row: Option<(Uuid, Uuid, String, bool, Option<serde_json::Value>, Option<String>, Option<String>)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM approval_resumes \
             WHERE status = 'pending' AND run_after <= now() \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE approval_resumes SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, message_id, decision, remember, edited_input, remember_scope, remember_value",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, message_id, decision, remember, edited_input, remember_scope, remember_value)| ClaimedApprovalResume {
        id,
        message_id,
        decision,
        remember,
        extras: Extras { edited_input, remember_scope, remember_value },
    }))
}

/// Takes back a deny that hasn't been carried out yet: the queued resume is cancelled, so the
/// card can be decided again. `true` when it did; `false` when there was no deny waiting (the
/// window passed and the turn went on, or it was already undone), in which case nothing changed.
pub async fn cancel_waiting_deny(pool: &PgPool, message_id: Uuid) -> Result<bool, TurnError> {
    let cancelled = sqlx::query(
        "UPDATE approval_resumes SET status = 'cancelled', completed_at = now() \
         WHERE message_id = $1 AND decision = 'deny' AND status = 'pending' AND run_after > now()",
    )
    .bind(message_id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(cancelled > 0)
}

pub async fn mark_completed(pool: &PgPool, id: Uuid) -> Result<(), TurnError> {
    sqlx::query("UPDATE approval_resumes SET status = 'completed', completed_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_failed(pool: &PgPool, id: Uuid, error: &str) -> Result<(), TurnError> {
    sqlx::query("UPDATE approval_resumes SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deny_can_be_undone_inside_the_window_only() {
        let decided = chrono::Utc::now();
        let at = |secs: i64| decided + chrono::Duration::seconds(secs);
        assert!(undo_open(decided, at(0)));
        assert!(undo_open(decided, at(UNDO_WINDOW_SECONDS - 1)));
        assert!(!undo_open(decided, at(UNDO_WINDOW_SECONDS)));
        assert!(!undo_open(decided, at(UNDO_WINDOW_SECONDS + 30)));
    }

    #[test]
    fn a_clock_behind_the_decision_does_not_open_the_window_forever() {
        let decided = chrono::Utc::now();
        // Not trusted: a decision "from the future" is treated as just made, not as always open.
        assert!(undo_open(decided, decided - chrono::Duration::seconds(1)));
        assert!(!undo_open(decided, decided + chrono::Duration::seconds(UNDO_WINDOW_SECONDS)));
    }

    async fn seed_message(pool: &PgPool) -> Uuid {
        let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(pool)
            .await
            .unwrap();
        let session_id: Uuid = sqlx::query_scalar(
            "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id",
        )
        .bind(org_id)
        .bind(format!("chat-{}", Uuid::new_v4()))
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'pending action') RETURNING id")
            .bind(session_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn enqueue_then_claim_next_marks_it_processing(pool: PgPool) {
        let message_id = seed_message(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        let resume_id = enqueue(&mut tx, message_id, "approve", false).await.unwrap();
        tx.commit().await.unwrap();

        let claimed = claim_next(&pool).await.unwrap().expect("expected a claimable resume");
        assert_eq!(claimed.id, resume_id);
        assert_eq!(claimed.message_id, message_id);
        assert_eq!(claimed.decision, "approve");
        assert!(!claimed.remember);

        let status: String = sqlx::query_scalar("SELECT status FROM approval_resumes WHERE id = $1")
            .bind(resume_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "processing");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn claim_next_returns_none_when_nothing_is_pending(pool: PgPool) {
        let message_id = seed_message(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        enqueue(&mut tx, message_id, "approve", true).await.unwrap();
        tx.commit().await.unwrap();

        // First claim takes the only pending row (an approve: a deny waits out its undo window).
        assert!(claim_next(&pool).await.unwrap().is_some());
        // Second claim finds nothing left pending.
        assert!(claim_next(&pool).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn mark_completed_updates_status_and_completed_at(pool: PgPool) {
        let message_id = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let resume_id = enqueue(&mut tx, message_id, "approve", false).await.unwrap();
        tx.commit().await.unwrap();
        claim_next(&pool).await.unwrap();

        mark_completed(&pool, resume_id).await.unwrap();

        let (status, completed_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
            sqlx::query_as("SELECT status, completed_at FROM approval_resumes WHERE id = $1")
                .bind(resume_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "completed");
        assert!(completed_at.is_some());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn mark_failed_updates_status_and_error(pool: PgPool) {
        let message_id = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let resume_id = enqueue(&mut tx, message_id, "approve", false).await.unwrap();
        tx.commit().await.unwrap();
        claim_next(&pool).await.unwrap();

        mark_failed(&pool, resume_id, "boom").await.unwrap();

        let (status, error): (String, Option<String>) =
            sqlx::query_as("SELECT status, error FROM approval_resumes WHERE id = $1")
                .bind(resume_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "failed");
        assert_eq!(error.as_deref(), Some("boom"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_deny_waits_out_the_undo_window_but_an_approve_does_not(pool: PgPool) {
        let message_id = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        enqueue(&mut tx, message_id, "deny", false).await.unwrap();
        tx.commit().await.unwrap();
        assert!(claim_next(&pool).await.unwrap().is_none(), "a fresh deny isn't released yet");

        sqlx::query("UPDATE approval_resumes SET run_after = now() - interval '1 second'").execute(&pool).await.unwrap();
        assert!(claim_next(&pool).await.unwrap().is_some());

        let other = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        enqueue(&mut tx, other, "approve", false).await.unwrap();
        tx.commit().await.unwrap();
        assert!(claim_next(&pool).await.unwrap().is_some(), "an approve runs at once");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_waiting_deny_can_be_cancelled_once_and_the_edits_ride_along(pool: PgPool) {
        let message_id = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        enqueue(&mut tx, message_id, "deny", false).await.unwrap();
        tx.commit().await.unwrap();
        assert!(cancel_waiting_deny(&pool, message_id).await.unwrap());
        assert!(!cancel_waiting_deny(&pool, message_id).await.unwrap(), "the second time changes nothing");
        assert!(claim_next(&pool).await.unwrap().is_none());

        let extras = Extras {
            edited_input: Some(serde_json::json!({"to": "new@x.co"})),
            remember_scope: Some("recipient".to_string()),
            remember_value: Some("new@x.co".to_string()),
        };
        let mut tx = pool.begin().await.unwrap();
        enqueue_with(&mut tx, message_id, "approve", false, &extras).await.unwrap();
        tx.commit().await.unwrap();
        let claimed = claim_next(&pool).await.unwrap().expect("the approve is claimable");
        assert_eq!(claimed.extras, extras);
    }

    #[test]
    fn undo_open_respects_the_ten_second_window() {
        let now = chrono::Utc::now();
        let fresh = now - chrono::Duration::seconds(5);
        assert!(undo_open(fresh, now));

        let boundary = now - chrono::Duration::seconds(UNDO_WINDOW_SECONDS);
        assert!(!undo_open(boundary, now));

        let expired = now - chrono::Duration::seconds(UNDO_WINDOW_SECONDS + 5);
        assert!(!undo_open(expired, now));
    }
}
