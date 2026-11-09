//! Keeps the provider free of sandboxes nobody owns. Deleting a project (or its owner) copies its
//! sandbox id into the `sandbox_deletions` outbox in the same transaction (a database trigger on
//! `project_sandboxes`), so the cascade never loses it and deleting a project never waits on the
//! provider. `drain_deletions` works the outbox off; `reap_orphans` finds sandboxes that leaked
//! past every other path. Both need the provider's API key, so they only run while the sandbox
//! runtime is switched on; outbox rows simply wait while it is off.

use std::collections::HashSet;
use std::time::Duration;

use nomi_sandbox::plan::backoff;
use nomi_sandbox::{SandboxError, SandboxId, SandboxProvider};
use sqlx::PgPool;
use uuid::Uuid;

use crate::configuration::ReaperMode;
use crate::sandbox_manager::{PROJECT_LABEL, RETRY_BASE, RETRY_CAP};

/// Most outbox rows worked in one pass.
const DELETION_BATCH: i64 = 20;
/// How often the orphan sweep runs.
pub const REAP_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Delete the sandboxes in the outbox that are due, returning how many were removed from it. A
/// sandbox the provider no longer knows counts as deleted. A failure keeps the row, with the
/// attempt counted and the next try pushed back.
pub async fn drain_deletions(pool: &PgPool, provider: &dyn SandboxProvider) -> usize {
    match try_drain(pool, provider).await {
        Ok(done) => done,
        Err(e) => {
            tracing::warn!(error = %e, "sandbox cleanup: working the deletion outbox failed");
            0
        }
    }
}

async fn try_drain(pool: &PgPool, provider: &dyn SandboxProvider) -> Result<usize, sqlx::Error> {
    // The row locks keep a second worker from taking the same rows while the provider is called.
    let mut tx = pool.begin().await?;
    let due: Vec<(Uuid, String, i32)> = sqlx::query_as(
        "SELECT id, sandbox_id, attempts FROM sandbox_deletions WHERE retry_at IS NULL OR retry_at <= now() \
         ORDER BY created_at LIMIT $1 FOR UPDATE SKIP LOCKED",
    )
    .bind(DELETION_BATCH)
    .fetch_all(&mut *tx)
    .await?;
    let mut done = 0;
    for (id, sandbox_id, attempts) in due {
        match provider.delete(&SandboxId(sandbox_id.clone())).await {
            Ok(()) | Err(SandboxError::NotFound) => {
                sqlx::query("DELETE FROM sandbox_deletions WHERE id = $1").bind(id).execute(&mut *tx).await?;
                done += 1;
            }
            Err(e) => {
                tracing::warn!(%sandbox_id, error = %e, "sandbox cleanup: deleting at the provider failed; will retry");
                let wait = e.retry_after().unwrap_or_else(|| backoff(attempts.max(0) as u32, RETRY_BASE, RETRY_CAP));
                sqlx::query("UPDATE sandbox_deletions SET attempts = attempts + 1, retry_at = now() + make_interval(secs => $2), last_error = $3 WHERE id = $1")
                    .bind(id)
                    .bind(wait.as_secs_f64())
                    .bind(e.to_string())
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }
    tx.commit().await?;
    Ok(done)
}

/// Find provider sandboxes labelled `nomi-project` that no project row and no outbox row names,
/// and delete them (or only log them in dry-run). Sandboxes without the label are never touched,
/// nor are those of a project whose row is still being created (its sandbox id isn't saved yet).
/// Returns how many orphans were deleted, or found in dry-run.
pub async fn reap_orphans(pool: &PgPool, provider: &dyn SandboxProvider, mode: ReaperMode) -> usize {
    let listed = match provider.list().await {
        Ok(listed) => listed,
        Err(e) => {
            tracing::warn!(error = %e, "sandbox reaper: listing sandboxes failed");
            return 0;
        }
    };
    // Read after the listing, so a sandbox created while we listed is already recorded.
    let known = async {
        let ids: Vec<String> = sqlx::query_scalar("SELECT sandbox_id FROM project_sandboxes WHERE sandbox_id IS NOT NULL UNION SELECT sandbox_id FROM sandbox_deletions").fetch_all(pool).await?;
        let creating: Vec<Uuid> = sqlx::query_scalar("SELECT project_id FROM project_sandboxes WHERE sandbox_id IS NULL").fetch_all(pool).await?;
        Ok::<_, sqlx::Error>((ids.into_iter().collect::<HashSet<_>>(), creating.into_iter().collect::<HashSet<_>>()))
    }
    .await;
    let (known, creating) = match known {
        Ok(known) => known,
        Err(e) => {
            tracing::warn!(error = %e, "sandbox reaper: reading the known sandboxes failed");
            return 0;
        }
    };
    let mut handled = 0;
    for sandbox in listed {
        let Some(label) = sandbox.labels.get(PROJECT_LABEL) else { continue };
        let Ok(project_id) = label.parse::<Uuid>() else { continue };
        if known.contains(&sandbox.id.0) || creating.contains(&project_id) {
            continue;
        }
        match mode {
            ReaperMode::DryRun => {
                tracing::warn!(sandbox_id = %sandbox.id.0, %project_id, "sandbox reaper (dry run): would delete an orphaned sandbox");
                handled += 1;
            }
            ReaperMode::Delete => match provider.delete(&sandbox.id).await {
                Ok(()) | Err(SandboxError::NotFound) => {
                    tracing::warn!(sandbox_id = %sandbox.id.0, %project_id, "sandbox reaper: deleted an orphaned sandbox");
                    handled += 1;
                }
                Err(e) => tracing::warn!(sandbox_id = %sandbox.id.0, error = %e, "sandbox reaper: deleting an orphan failed"),
            },
        }
    }
    handled
}
