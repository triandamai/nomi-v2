//! Who has a project open right now, and who's typing in its chat. Both are short-lived: an
//! open project page sends a heartbeat every few seconds, and a person stays "here" until their
//! last one is older than `PRESENCE_TTL_SECONDS` (so a tab that closes without saying goodbye
//! still drops off). Changes go out on the project's live topic, which only people who can open
//! the project can subscribe to; nothing is written to the chat.

use axum::extract::{Path, State};
use serde::Deserialize;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, ApiError};

/// A heartbeat older than this means the person closed the project (or their device slept).
pub const PRESENCE_TTL_SECONDS: i64 = 45;

#[derive(Serialize, sqlx::FromRow)]
pub struct PresentPerson {
    pub user_id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    /// What they're looking at: `preview`, `code` or `terminal`.
    pub view: Option<String>,
    /// The file open in their code view.
    pub path: Option<String>,
}

/// Where someone is in the project, sent with each heartbeat (and as soon as it changes).
#[derive(Deserialize, Default)]
pub struct Whereabouts {
    pub view: Option<String>,
    pub path: Option<String>,
}

/// Only the project's own views; a path is at most a file path's length.
fn clean(whereabouts: Whereabouts) -> (Option<String>, Option<String>) {
    let view = whereabouts.view.filter(|v| matches!(v.as_str(), "preview" | "code" | "terminal"));
    let path = whereabouts.path.map(|p| p.chars().take(300).collect::<String>()).filter(|p| !p.is_empty());
    (view, path)
}

#[derive(Serialize)]
pub struct Presence {
    /// Who's asking, so their page can tell its own typing pings from others'.
    pub me: Uuid,
    /// Everyone else who has the project open, longest-present first.
    pub people: Vec<PresentPerson>,
}

/// The name people know someone by, as the members list shows it.
const PERSON_NAME: &str = "COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1))";

fn tell_project(project_id: Uuid, event: &serde_json::Value) {
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&crate::project_events::topic(project_id), event) {
        tracing::debug!(error = %e, %project_id, "couldn't tell the project's pages about presence");
    }
}

/// `POST /api/projects/:id/presence`: this person has the project open.
pub async fn heartbeat(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    body: Option<Json<Whereabouts>>,
) -> Result<StatusCode, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let (view, path) = clean(body.map(|Json(b)| b).unwrap_or_default());
    // Whether they'd already counted as here, and where they were, so only an arrival or a move
    // tells the others.
    let before: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT view, path FROM project_presence WHERE project_id = $1 AND user_id = $2 \
         AND last_seen > now() - make_interval(secs => $3)",
    )
    .bind(project_id)
    .bind(claims.sub)
    .bind(PRESENCE_TTL_SECONDS as f64)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to check presence"))?;
    sqlx::query(
        "INSERT INTO project_presence (project_id, user_id, last_seen, view, path) VALUES ($1, $2, now(), $3, $4) \
         ON CONFLICT (project_id, user_id) DO UPDATE SET last_seen = now(), view = COALESCE($3, project_presence.view), \
         path = CASE WHEN $3 IS NULL THEN project_presence.path ELSE $4 END",
    )
    .bind(project_id)
    .bind(claims.sub)
    .bind(&view)
    .bind(&path)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to record presence"))?;
    let moved = matches!(&before, Some((was_view, was_path)) if view.is_some() && (was_view != &view || was_path != &path));
    if before.is_none() || moved {
        tell_project(project_id, &json!({ "kind": "PresenceChanged" }));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/projects/:id/presence`: this person closed the project.
pub async fn leave(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    sqlx::query("DELETE FROM project_presence WHERE project_id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to clear presence"))?;
    tell_project(project_id, &json!({ "kind": "PresenceChanged" }));
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/projects/:id/presence`: who else has the project open.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<Presence>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let people: Vec<PresentPerson> = sqlx::query_as(&format!(
        "SELECT pp.user_id, {PERSON_NAME} AS name, up.avatar_url, pp.view, pp.path \
         FROM project_presence pp \
         LEFT JOIN user_profiles up ON up.user_id = pp.user_id \
         LEFT JOIN web_credentials w ON w.user_id = pp.user_id \
         LEFT JOIN google_identities g ON g.user_id = pp.user_id \
         WHERE pp.project_id = $1 AND pp.user_id <> $2 AND pp.last_seen > now() - make_interval(secs => $3) \
           AND can_open_project(pp.project_id, pp.user_id) \
         ORDER BY pp.last_seen"
    ))
    .bind(project_id)
    .bind(claims.sub)
    .bind(PRESENCE_TTL_SECONDS as f64)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list presence"))?;
    Ok(Json(Presence { me: claims.sub, people }))
}

/// `POST /api/projects/:id/typing`: this person is writing in the project's chat. The pages
/// show it for a few seconds; the next ping keeps it up.
pub async fn typing(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let name: String = sqlx::query_scalar(&format!(
        "SELECT {PERSON_NAME} FROM users u \
         LEFT JOIN user_profiles up ON up.user_id = u.id \
         LEFT JOIN web_credentials w ON w.user_id = u.id \
         LEFT JOIN google_identities g ON g.user_id = u.id WHERE u.id = $1"
    ))
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to look up who's typing"))?;
    tell_project(project_id, &json!({ "kind": "Typing", "user_id": claims.sub, "name": name }));
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CursorAt {
    /// Where the pointer is over the view, as fractions of its width and height (0–1).
    pub x: f64,
    pub y: f64,
    pub view: String,
}

/// `POST /api/projects/:id/cursor` with `{ x, y, view }`: where this person's pointer is, passed
/// straight to the others' pages (nothing is stored). Pages send it a few times a second at most.
pub async fn cursor(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(at): Json<CursorAt>,
) -> Result<StatusCode, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    if !(0.0..=1.0).contains(&at.x) || !(0.0..=1.0).contains(&at.y) || !matches!(at.view.as_str(), "preview" | "code" | "terminal") {
        return Err((StatusCode::BAD_REQUEST, "x and y are fractions of the view, and view is preview, code or terminal".to_string()));
    }
    let name: String = sqlx::query_scalar(&format!(
        "SELECT {PERSON_NAME} FROM users u \
         LEFT JOIN user_profiles up ON up.user_id = u.id \
         LEFT JOIN web_credentials w ON w.user_id = u.id \
         LEFT JOIN google_identities g ON g.user_id = u.id WHERE u.id = $1"
    ))
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to look up whose pointer it is"))?;
    tell_project(project_id, &json!({ "kind": "Cursor", "user_id": claims.sub, "name": name, "x": at.x, "y": at.y, "view": at.view }));
    Ok(StatusCode::NO_CONTENT)
}
