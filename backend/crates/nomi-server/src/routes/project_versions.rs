//! A project's versions (migration 0107, with their files from 0120): listed newest first, and
//! any one put back. A restore is a version of its own, made by the person, so it can be undone
//! by restoring the one before it.

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_agent_coding::files::{remove_file, save_bytes, version_blob_key};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, require_builder, ApiError};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;
/// A version's summary (the prompt it answered) is shown up to this many characters.
const SUMMARY_CHARS: i32 = 140;

#[derive(Deserialize)]
pub struct VersionsQuery {
    /// Versions older than this one (for paging).
    pub before: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct VersionBy {
    /// The person, for a change of their own (or the prompt's sender is not named here).
    pub name: Option<String>,
    /// The crew member, when it was their reply ("Koda").
    pub agent: Option<String>,
}

#[derive(Serialize)]
pub struct VersionItem {
    pub version: i64,
    pub by: VersionBy,
    pub at: DateTime<Utc>,
    /// The person's prompt the crew answered with this version.
    pub summary: Option<String>,
    pub files: i64,
    /// False when it was made before versions kept their files.
    pub restorable: bool,
    /// The version this one put back, when it was a restore.
    pub restored_from: Option<i64>,
    pub current: bool,
}

#[derive(Serialize)]
pub struct Versions {
    pub versions: Vec<VersionItem>,
    pub current: i64,
    pub has_more: bool,
}

#[derive(sqlx::FromRow)]
struct Row {
    version: i64,
    agent: Option<String>,
    person: Option<String>,
    at: DateTime<Utc>,
    summary: Option<String>,
    files: i64,
    restorable: bool,
    restored_from: Option<i64>,
}

/// `$1` project, `$2` before (or null), `$3` limit + 1, `$4` summary length.
const VERSIONS_SQL: &str = "
WITH p AS (SELECT id, session_id FROM projects WHERE id = $1),
chats AS (
    SELECT s.id FROM sessions s, p WHERE s.id = p.session_id OR s.in_project_id = p.id
    UNION SELECT i.session_id FROM project_issues i, p WHERE i.project_id = p.id AND i.session_id IS NOT NULL
)
SELECT v.version, v.agent, v.restored_from, v.created_at AS at,
       COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS person,
       (SELECT count(*) FROM project_version_files f WHERE f.project_id = v.project_id AND f.version = v.version) AS files,
       NOT EXISTS (SELECT 1 FROM project_version_files f WHERE f.project_id = v.project_id AND f.version = v.version AND f.blob_sha IS NULL) AS restorable,
       CASE WHEN v.agent IS NULL THEN NULL ELSE (
           SELECT left(m.content, $4) FROM messages m
           WHERE m.session_id IN (SELECT id FROM chats) AND m.sender_channel_identity_id IS NOT NULL AND m.created_at <= v.created_at
           ORDER BY m.created_at DESC LIMIT 1
       ) END AS summary
FROM project_versions v
LEFT JOIN user_profiles up ON up.user_id = v.user_id
LEFT JOIN web_credentials w ON w.user_id = v.user_id
LEFT JOIN google_identities g ON g.user_id = v.user_id
WHERE v.project_id = $1 AND ($2::bigint IS NULL OR v.version < $2)
ORDER BY v.version DESC
LIMIT $3";

/// `GET /api/projects/:id/versions?before=&limit=`: newest first, for anyone who can open it.
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<VersionsQuery>,
) -> Result<Json<Versions>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let current: i64 = sqlx::query_scalar("SELECT version FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load the project's versions"))?;
    let mut rows: Vec<Row> = sqlx::query_as(VERSIONS_SQL)
        .bind(project_id)
        .bind(query.before)
        .bind(limit + 1)
        .bind(SUMMARY_CHARS)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to load the project's versions"))?;
    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let versions = rows
        .into_iter()
        .map(|row| {
            let agent = row
                .agent
                .as_deref()
                .map(|a| nomi_agent_core::crew::crew_name(a).unwrap_or(a).to_string());
            VersionItem {
                version: row.version,
                by: VersionBy {
                    name: if agent.is_some() { None } else { row.person },
                    agent,
                },
                at: row.at,
                summary: row.summary,
                files: row.files,
                restorable: row.restorable,
                restored_from: row.restored_from,
                current: row.version == current,
            }
        })
        .collect();
    Ok(Json(Versions {
        versions,
        current,
        has_more,
    }))
}

/// `POST /api/projects/:id/versions/:version/restore`: puts the project's files back as they
/// were at that version, as a new version. 202 with `{ version, restored_from }`.
pub async fn restore(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, version)): Path<(Uuid, i64)>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    require_builder(&state.pool, project_id, claims.sub).await?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM project_versions WHERE project_id = $1 AND version = $2)",
    )
    .bind(project_id)
    .bind(version)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to load the version"))?;
    if !exists {
        return Err((StatusCode::NOT_FOUND, "version not found".to_string()));
    }
    let wanted: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT path, blob_sha FROM project_version_files WHERE project_id = $1 AND version = $2",
    )
    .bind(project_id)
    .bind(version)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load the version"))?;
    if wanted.iter().any(|(_, sha)| sha.is_none()) {
        return Err((
            StatusCode::CONFLICT,
            "that version was made before versions kept their files".to_string(),
        ));
    }
    let now: HashMap<String, Option<String>> = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT path, blob_sha FROM project_files WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load the project's files"))?
    .into_iter()
    .collect();

    // Every changed file is read first, so a missing one leaves the project as it was.
    let mut writes = Vec::new();
    for (path, sha) in &wanted {
        let sha = sha.as_deref().unwrap_or_default();
        if now.get(path).and_then(|s| s.as_deref()) == Some(sha) {
            continue;
        }
        let bytes = state
            .project_storage
            .get_bytes(&version_blob_key(project_id, sha))
            .await
            .map_err(internal("failed to read the version's files"))?
            .ok_or((
                StatusCode::GONE,
                "that version's files weren't kept".to_string(),
            ))?;
        writes.push((path.clone(), bytes));
    }
    let removes: Vec<&String> = now
        .keys()
        .filter(|path| !wanted.iter().any(|(p, _)| p == *path))
        .collect();

    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to restore the version"))?;
    for path in removes {
        remove_file(&mut conn, &state.project_storage, project_id, path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    for (path, bytes) in &writes {
        save_bytes(&mut conn, &state.project_storage, project_id, path, bytes)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    let new_version =
        nomi_agent_coding::files::bump_files_version_by(&mut conn, project_id, claims.sub)
            .await
            .map_err(internal("failed to restore the version"))?;
    sqlx::query(
        "UPDATE project_versions SET restored_from = $3 WHERE project_id = $1 AND version = $2",
    )
    .bind(project_id)
    .bind(new_version)
    .bind(version)
    .execute(&mut *conn)
    .await
    .map_err(internal("failed to restore the version"))?;
    crate::project_events::changed(project_id, "version.restored");
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "version": new_version, "restored_from": version })),
    ))
}
