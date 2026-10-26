//! Applying a project's migrations to one environment: names and lint checked first, an edited
//! migration refused, then each new file in its own transaction as the project's owner role,
//! with a statement timeout, recorded in `nomi.migrations` with its checksum.

use relay_core::names::{role, RoleKind};
use relay_core::Env;
use relay_migrate::lint::lint;
use relay_migrate::{checksum, line_at, valid_name, MigrationFile};
use serde_json::json;
use sqlx::postgres::{PgDatabaseError, PgErrorPosition};
use uuid::Uuid;

use crate::{Relay, RelayError};

/// How long one statement of a migration may run.
pub const STATEMENT_TIMEOUT: &str = "30s";

fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// Where in `content` Postgres said the error was, when it said.
fn error_line(error: &sqlx::Error, content: &str) -> Option<usize> {
    let pg = error.as_database_error()?.try_downcast_ref::<PgDatabaseError>()?;
    match pg.position()? {
        PgErrorPosition::Original(position) => Some(line_at(content, position)),
        PgErrorPosition::Internal { .. } => None,
    }
}

/// Applies what `env` doesn't have yet of `files`, in name order. Answers the names applied.
pub async fn apply(relay: &Relay, project: Uuid, env: Env, files: &[MigrationFile]) -> Result<Vec<String>, RelayError> {
    let mut files: Vec<&MigrationFile> = files.iter().collect();
    files.sort_by(|a, b| a.name().cmp(b.name()));

    if let Some(bad) = files.iter().find(|f| !valid_name(f.name())) {
        return Err(RelayError::Rejected {
            code: "LINT_FAILED",
            message: format!("{} isn't a migration name: use 0001_name.sql (lowercase letters, digits, underscores)", bad.name()),
            details: json!({ "file": bad.name() }),
        });
    }
    let owned: Vec<MigrationFile> = files.iter().map(|f| (*f).clone()).collect();
    if let Some(issue) = lint(&owned).into_iter().next() {
        return Err(RelayError::Rejected {
            code: "LINT_FAILED",
            message: format!("{} line {}: {}", issue.file, issue.line, issue.message),
            details: json!({ "file": issue.file, "line": issue.line, "rule": issue.rule }),
        });
    }

    let placement = relay.placement(project, env).await?;
    let applied: Vec<(String, String)> = sqlx::query_as("SELECT name, checksum FROM nomi.migrations WHERE project_id = $1")
        .bind(project)
        .fetch_all(&placement.pool)
        .await?;
    for file in &files {
        if let Some((_, recorded)) = applied.iter().find(|(name, _)| name == file.name()) {
            if *recorded != checksum(&file.content) {
                return Err(RelayError::Rejected {
                    code: "MIGRATION_EDITED",
                    message: format!("{} was already applied and has changed since: write a new migration instead of editing it", file.name()),
                    details: json!({ "file": file.name() }),
                });
            }
        }
    }

    let owner = quoted(&role(project, RoleKind::Owner));
    let mut ran = Vec::new();
    for file in files.iter().filter(|f| !applied.iter().any(|(name, _)| name == f.name())) {
        let mut tx = placement.pool.begin().await?;
        sqlx::query(&format!("SET LOCAL ROLE {owner}")).execute(&mut *tx).await?;
        sqlx::query(&format!("SET LOCAL search_path = {}", quoted(&placement.schema))).execute(&mut *tx).await?;
        sqlx::query(&format!("SET LOCAL statement_timeout = '{STATEMENT_TIMEOUT}'")).execute(&mut *tx).await?;
        if let Err(e) = sqlx::raw_sql(&file.content).execute(&mut *tx).await {
            let line = error_line(&e, &file.content);
            let message = e.as_database_error().map(|d| d.message().to_string()).unwrap_or_else(|| e.to_string());
            tx.rollback().await?;
            return Err(RelayError::Rejected {
                code: "MIGRATION_FAILED",
                message: match line {
                    Some(line) => format!("{} line {line}: {message}", file.name()),
                    None => format!("{}: {message}", file.name()),
                },
                details: json!({ "file": file.name(), "line": line, "applied": ran }),
            });
        }
        sqlx::query("RESET ROLE").execute(&mut *tx).await?;
        sqlx::query("INSERT INTO nomi.migrations (project_id, name, checksum) VALUES ($1, $2, $3)")
            .bind(project)
            .bind(file.name())
            .bind(checksum(&file.content))
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        ran.push(file.name().to_string());
    }
    if !ran.is_empty() {
        // PostgREST (from phase 2) re-reads the schema; harmless when nothing listens yet.
        sqlx::query("NOTIFY pgrst, 'reload schema'").execute(&placement.pool).await?;
    }
    Ok(ran)
}
