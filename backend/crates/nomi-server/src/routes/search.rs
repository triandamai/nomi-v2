//! `GET /api/search?q=&kinds=chat,memory,file&limit=`: the person's own chats (title and
//! messages) in the workspace they're in, their memory, and the paths of files in that
//! workspace's projects they can open. Case-insensitive substring matching; a hit in a title or
//! file name ranks above one in a message's text, then newest first.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, ApiError};

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;
/// Longest query looked for.
const MAX_QUERY_CHARS: usize = 200;
/// Characters of context on each side of the match in a snippet.
const SNIPPET_RADIUS: usize = 60;
/// Longest title shown for a memory (its text).
const MEMORY_TITLE_CHARS: usize = 80;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    /// Comma-separated: chat, memory, file. All when left out.
    pub kinds: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SearchResult {
    /// chat, memory or file.
    pub kind: &'static str,
    /// The chat's id, the memory's id, or the file's path.
    pub id: String,
    pub title: String,
    pub snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    title_hit: bool,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

/// The ILIKE pattern for `q`, with its wildcards taken literally.
pub fn like_pattern(q: &str) -> String {
    let escaped = q
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// Up to `SNIPPET_RADIUS` characters either side of the first case-insensitive match of `q` in
/// `text`, on one line, with an ellipsis where it was cut. The start of the text when `q` isn't in it.
pub fn snippet(text: &str, q: &str) -> String {
    let chars: Vec<char> = text
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    let needle: Vec<char> = q.chars().flat_map(|c| c.to_lowercase()).collect();
    // Lowercasing can change lengths (rarely); fall back to the start then.
    let at = if lower.len() == chars.len() && !needle.is_empty() {
        lower
            .windows(needle.len())
            .position(|w| w == needle.as_slice())
            .unwrap_or(0)
    } else {
        0
    };
    let start = at.saturating_sub(SNIPPET_RADIUS);
    let end = (at + needle.len() + SNIPPET_RADIUS).min(chars.len());
    let mut out: String = chars[start..end].iter().collect();
    out = out.trim().to_string();
    if start > 0 {
        out = format!("…{out}");
    }
    if end < chars.len() {
        out.push('…');
    }
    out
}

/// Title and file-name hits first, then newest; at most `limit`.
fn rank(mut results: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
    results.sort_by(|a, b| {
        b.title_hit
            .cmp(&a.title_hit)
            .then(b.updated_at.cmp(&a.updated_at))
    });
    results.truncate(limit);
    results
}

fn clip(text: &str, max: usize) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        flat
    } else {
        format!("{}…", flat.chars().take(max).collect::<String>().trim_end())
    }
}

pub async fn search(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    let q = query.q.unwrap_or_default();
    let q = q.trim();
    if q.is_empty() {
        return Ok(Json(SearchResponse {
            results: Vec::new(),
        }));
    }
    if q.chars().count() > MAX_QUERY_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("search for at most {MAX_QUERY_CHARS} characters"),
        ));
    }
    let wants = |kind: &str| match query.kinds.as_deref() {
        None | Some("") => true,
        Some(kinds) => kinds.split(',').any(|k| k.trim() == kind),
    };
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let pattern = like_pattern(q);
    let mut results = Vec::new();

    if wants("chat") {
        #[allow(clippy::type_complexity)]
        let rows: Vec<(Uuid, Option<String>, Option<String>, DateTime<Utc>, bool, Option<Uuid>)> = sqlx::query_as(
            "SELECT s.id, s.title, hit.content, COALESCE(hit.created_at, s.created_at), COALESCE(s.title ILIKE $3, false), chat_project(s.id) \
             FROM sessions s \
             LEFT JOIN LATERAL ( \
                 SELECT m.content, m.created_at FROM messages m \
                 WHERE m.session_id = s.id AND m.content ILIKE $3 AND m.content NOT LIKE '🧠%' \
                 ORDER BY m.created_at DESC LIMIT 1 \
             ) hit ON true \
             WHERE s.user_id = $1 AND s.org_id = $2 AND (s.title ILIKE $3 OR hit.content IS NOT NULL) \
             ORDER BY 5 DESC, 4 DESC LIMIT $4",
        )
        .bind(claims.sub)
        .bind(claims.active_org_id)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to search chats"))?;
        for (id, title, content, at, title_hit, project_id) in rows {
            results.push(SearchResult {
                kind: "chat",
                id: id.to_string(),
                title: title.unwrap_or_default(),
                snippet: content.map(|c| snippet(&c, q)),
                session_id: Some(id),
                project_id,
                updated_at: at,
                title_hit,
            });
        }
    }

    if wants("memory") {
        let rows: Vec<(Uuid, String, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, content, updated_at FROM memory_items \
             WHERE user_id = $1 AND archived_at IS NULL AND content ILIKE $2 ORDER BY updated_at DESC LIMIT $3",
        )
        .bind(claims.sub)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to search memory"))?;
        for (id, content, at) in rows {
            results.push(SearchResult {
                kind: "memory",
                id: id.to_string(),
                title: clip(&content, MEMORY_TITLE_CHARS),
                snippet: Some(snippet(&content, q)),
                session_id: None,
                project_id: None,
                updated_at: at,
                title_hit: false,
            });
        }
    }

    if wants("file") {
        let rows: Vec<(String, Uuid, String, Uuid, DateTime<Utc>, bool)> = sqlx::query_as(
            "SELECT pf.path, p.id, p.name, p.session_id, pf.updated_at, \
                    regexp_replace(pf.path, '^.*/', '') ILIKE $3 AS name_hit \
             FROM project_files pf JOIN projects p ON p.id = pf.project_id \
             WHERE p.org_id = $2 AND pf.path ILIKE $3 AND can_open_project(p.id, $1) \
             ORDER BY name_hit DESC, pf.updated_at DESC LIMIT $4",
        )
        .bind(claims.sub)
        .bind(claims.active_org_id)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to search files"))?;
        for (path, project_id, project_name, session_id, at, name_hit) in rows {
            results.push(SearchResult {
                kind: "file",
                id: path.clone(),
                title: path,
                snippet: Some(project_name),
                session_id: Some(session_id),
                project_id: Some(project_id),
                updated_at: at,
                title_hit: name_hit,
            });
        }
    }

    Ok(Json(SearchResponse {
        results: rank(results, limit as usize),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards_are_taken_literally() {
        assert_eq!(like_pattern("50%_off\\"), "%50\\%\\_off\\\\%");
    }

    #[test]
    fn a_snippet_is_cut_around_the_match() {
        let text = format!("{}needle{}", "a".repeat(100), "b".repeat(100));
        let s = snippet(&text, "NEEDLE");
        assert!(s.starts_with('…') && s.ends_with('…'), "{s}");
        assert!(s.contains("needle"));
        assert_eq!(snippet("short\ntext", "zzz"), "short text");
    }

    #[test]
    fn title_hits_rank_first_then_newest() {
        let item = |id: &str, title_hit, secs| SearchResult {
            kind: "chat",
            id: id.into(),
            title: String::new(),
            snippet: None,
            session_id: None,
            project_id: None,
            updated_at: DateTime::<Utc>::from_timestamp(secs, 0).unwrap(),
            title_hit,
        };
        let ranked = rank(
            vec![
                item("old", false, 1),
                item("new", false, 3),
                item("title", true, 2),
            ],
            2,
        );
        let ids: Vec<_> = ranked.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["title", "new"]);
    }
}
