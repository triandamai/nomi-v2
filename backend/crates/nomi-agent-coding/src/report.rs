//! What a build showed when Koda's run ended, from what actually ran (`project_runs`, and the
//! preview's own check when it opens a project) rather than from what his summary says. A check
//! counts only for the version of the files it ran on, so a pass before the last edit isn't one.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use nomi_agent_core::Locale;

use crate::runtime::tail;

/// The checks a finished build has to pass, as Koda runs them.
const CHECKS: [&str; 2] = ["check", "build"];
const OUTPUT_TAIL: usize = 1500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildStatus {
    /// Every check passed on the final files.
    Verified,
    /// A check failed on the final files.
    Failing,
    /// Not every check ran on the final files (the preview wasn't open, a command didn't
    /// finish, or the files changed after it ran).
    Unverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    Passed,
    Failed,
    TimedOut,
    /// It ran during this build, but on files that changed after.
    Stale,
    NotRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSource {
    /// Koda ran it with run_command.
    Koda,
    /// The project page ran it when the person opened the project.
    Preview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    /// As typed: "npm run check".
    pub command: String,
    pub outcome: CheckOutcome,
    pub source: CheckSource,
    pub exit_code: Option<i32>,
    /// The end of its output, when it failed.
    pub output_tail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildReport {
    pub status: BuildStatus,
    /// The version of the project's files the report is about (the final one).
    pub files_version: i64,
    /// Files written or edited during the build.
    pub changed_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub checks: Vec<CheckResult>,
}

/// The report for `project_id` covering everything since `since` (when the build started).
pub async fn build_report(conn: &mut PgConnection, session_id: Uuid, project_id: Uuid, since: DateTime<Utc>) -> Result<BuildReport, sqlx::Error> {
    let (files_version, checked_version, check_ok, check_output): (i64, Option<i64>, Option<bool>, Option<String>) =
        sqlx::query_as("SELECT files_version, checked_version, check_ok, check_output FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&mut *conn)
            .await?;
    let changed_files: Vec<String> =
        sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 AND updated_at >= $2 ORDER BY path")
            .bind(project_id)
            .bind(since)
            .fetch_all(&mut *conn)
            .await?;
    // Deleted files leave no row behind, only the card Koda's delete_file posted.
    let deleted_files: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT b->>'path' FROM messages m CROSS JOIN LATERAL jsonb_array_elements(m.content_blocks) b \
         WHERE m.session_id = $1 AND m.created_at >= $2 AND jsonb_typeof(m.content_blocks) = 'array' \
           AND b->>'kind' = 'file_delete' AND b->>'project_id' = $3 \
           AND NOT EXISTS (SELECT 1 FROM project_files f WHERE f.project_id = $4 AND f.path = b->>'path') \
         ORDER BY 1",
    )
    .bind(session_id)
    .bind(since)
    .bind(project_id.to_string())
    .bind(project_id)
    .fetch_all(&mut *conn)
    .await?;

    let mut checks = Vec::new();
    for script in CHECKS {
        let command = format!("npm run {script}");
        let latest: Option<(String, Option<i32>, Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT status, exit_code, output, files_version FROM project_runs \
             WHERE project_id = $1 AND command = 'npm' AND args = $2 AND created_at >= $3 \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(project_id)
        .bind(serde_json::json!(["run", script]))
        .bind(since)
        .fetch_optional(&mut *conn)
        .await?;
        let from_koda = latest.map(|(status, exit_code, output, ran_on)| {
            let outcome = match status.as_str() {
                _ if ran_on != Some(files_version) => CheckOutcome::Stale,
                "done" if exit_code == Some(0) => CheckOutcome::Passed,
                "done" | "failed" => CheckOutcome::Failed,
                "timed_out" => CheckOutcome::TimedOut,
                _ => CheckOutcome::NotRun,
            };
            let output_tail = (outcome == CheckOutcome::Failed).then(|| tail(output.as_deref().unwrap_or_default().trim_end(), OUTPUT_TAIL).to_string());
            CheckResult { command: command.clone(), outcome, source: CheckSource::Koda, exit_code, output_tail }
        });
        // The page runs `npm run check` itself when the project opens, on the version it reports.
        let from_preview = (script == "check" && checked_version == Some(files_version)).then(|| match check_ok {
            Some(true) => CheckResult { command: command.clone(), outcome: CheckOutcome::Passed, source: CheckSource::Preview, exit_code: Some(0), output_tail: None },
            _ => CheckResult {
                command: command.clone(),
                outcome: CheckOutcome::Failed,
                source: CheckSource::Preview,
                exit_code: None,
                output_tail: Some(tail(check_output.as_deref().unwrap_or_default().trim_end(), OUTPUT_TAIL).to_string()),
            },
        });
        let current = |c: &CheckResult| matches!(c.outcome, CheckOutcome::Passed | CheckOutcome::Failed);
        checks.push(match (from_koda, from_preview) {
            (Some(koda), _) if current(&koda) => koda,
            (_, Some(preview)) => preview,
            (Some(koda), None) => koda,
            (None, None) => CheckResult { command, outcome: CheckOutcome::NotRun, source: CheckSource::Koda, exit_code: None, output_tail: None },
        });
    }

    let status = if checks.iter().any(|c| c.outcome == CheckOutcome::Failed) {
        BuildStatus::Failing
    } else if checks.iter().all(|c| c.outcome == CheckOutcome::Passed) {
        BuildStatus::Verified
    } else {
        BuildStatus::Unverified
    };
    Ok(BuildReport { status, files_version, changed_files, deleted_files, checks })
}

/// The report on a delegated build, covering everything since the delegation was picked up, kept
/// on the delegation.
pub async fn save_report(conn: &mut PgConnection, delegation_id: Uuid, session_id: Uuid, project_id: Uuid) -> Result<BuildReport, sqlx::Error> {
    let since: DateTime<Utc> = sqlx::query_scalar("SELECT COALESCE(claimed_at, created_at) FROM agent_delegations WHERE id = $1")
        .bind(delegation_id)
        .fetch_one(&mut *conn)
        .await?;
    let report = build_report(conn, session_id, project_id, since).await?;
    sqlx::query("UPDATE agent_delegations SET report = $2 WHERE id = $1")
        .bind(delegation_id)
        .bind(report.to_json())
        .execute(&mut *conn)
        .await?;
    Ok(report)
}

/// At the end of a delegated build that finished: its report (`save_report`), and the project
/// marked ready unless a check failed on the final files — it stays as it was until a later run
/// gets them passing.
pub async fn finish_build(conn: &mut PgConnection, delegation_id: Uuid, session_id: Uuid, project_id: Uuid) -> Result<BuildReport, sqlx::Error> {
    let report = save_report(conn, delegation_id, session_id, project_id).await?;
    if report.status != BuildStatus::Failing {
        sqlx::query("UPDATE projects SET status = 'ready' WHERE id = $1 AND status = 'building'")
            .bind(project_id)
            .execute(&mut *conn)
            .await?;
    }
    Ok(report)
}

impl BuildReport {
    /// One line for the chat, under Koda's own summary: how it stands and each check.
    pub fn summary_line(&self, locale: Locale) -> String {
        let heading = locale.t(match self.status {
            BuildStatus::Verified => "build.verified",
            BuildStatus::Failing => "build.failing",
            BuildStatus::Unverified => "build.unverified",
        });
        let checks: Vec<String> = self
            .checks
            .iter()
            .map(|check| {
                let key = match check.outcome {
                    CheckOutcome::Passed => "build.check_passed",
                    CheckOutcome::Failed => "build.check_failed",
                    CheckOutcome::TimedOut => "build.check_timed_out",
                    CheckOutcome::Stale => "build.check_stale",
                    CheckOutcome::NotRun => "build.check_not_run",
                };
                let code = check.exit_code.map_or("?".to_string(), |c| c.to_string());
                locale.tf(key, &[("command", &check.command), ("code", &code)])
            })
            .collect();
        let mut line = format!("{heading}: {}.", checks.join(" · "));
        if self.status == BuildStatus::Unverified {
            line.push(' ');
            line.push_str(&locale.t("build.unverified_hint"));
        }
        line
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}
