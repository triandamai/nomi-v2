//! Who has a project open right now, and who's typing in its chat. Both are short-lived: an
//! open project page sends a heartbeat every few seconds, and a person stays "here" until their
//! last one is older than `PRESENCE_TTL_SECONDS` (so a tab that closes without saying goodbye
//! still drops off). Changes go out on the project's live topic, which only people who can open
//! the project can subscribe to; nothing is written to the chat.

use axum::extract::{Path, State};
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
}

#[derive(Serialize)]
pub struct Presence {
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
pub async fn heartbeat(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    // Whether they'd already counted as here, so only an arrival tells the others.
    let was_here: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM project_presence WHERE project_id = $1 AND user_id = $2 \
         AND last_seen > now() - make_interval(secs => $3))",
    )
    .bind(project_id)
    .bind(claims.sub)
    .bind(PRESENCE_TTL_SECONDS as f64)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to check presence"))?;
    sqlx::query(
        "INSERT INTO project_presence (project_id, user_id, last_seen) VALUES ($1, $2, now()) \
         ON CONFLICT (project_id, user_id) DO UPDATE SET last_seen = now()",
    )
    .bind(project_id)
    .bind(claims.sub)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to record presence"))?;
    if !was_here {
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
        "SELECT pp.user_id, {PERSON_NAME} AS name, up.avatar_url \
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
    Ok(Json(Presence { people }))
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
