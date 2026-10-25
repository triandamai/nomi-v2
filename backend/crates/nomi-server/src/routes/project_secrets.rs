//! A project's secrets: API keys and other values its published app reads from its
//! environment. Admins set and remove them; the value goes straight to Shipyard, which keeps it,
//! and Nomi keeps only the name, to bind it into the app on every publish. No one, model or
//! person, reads a value back through Nomi.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_shipyard::ShipyardError;

use crate::app::AppState;
use crate::routes::deploy::{client, ensure_error, internal, project_role, require_admin, shipyard_error, target, ApiError};

const MAX_NAME: usize = 64;
const MAX_VALUE: usize = 8 * 1024;
const MAX_SECRETS: i64 = 50;
/// Names the app's manifest sets itself.
const RESERVED: [&str; 4] = ["DATABASE_URL", "PORT", "NODE_ENV", "HOST"];

/// Whether `name` can be a secret: an environment variable name in capitals (`STRIPE_KEY`).
pub fn check_name(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    let shaped = matches!(chars.next(), Some(c) if c.is_ascii_uppercase() || c == '_')
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !shaped || name.len() > MAX_NAME {
        return Err(format!("use capitals, digits and _ (like STRIPE_KEY), up to {MAX_NAME} characters"));
    }
    if RESERVED.contains(&name) {
        return Err(format!("{name} is set by Nomi for every app"));
    }
    Ok(())
}

/// The names to bind into the project's app when it's published.
pub(crate) async fn names(pool: &sqlx::PgPool, project_id: Uuid) -> Result<Vec<String>, ApiError> {
    sqlx::query_scalar("SELECT name FROM project_secrets WHERE project_id = $1 ORDER BY name")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(internal("failed to load the project's secrets"))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Secret {
    pub name: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by_name: Option<String>,
}

#[derive(Serialize)]
pub struct Secrets {
    pub secrets: Vec<Secret>,
    pub can_edit: bool,
}

/// `GET /api/projects/:id/secrets`: the names (never the values), for everyone on the project.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<Secrets>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    let secrets: Vec<Secret> = sqlx::query_as(
        "SELECT s.name, s.updated_at, NULLIF(up.display_name, '') AS updated_by_name FROM project_secrets s \
         LEFT JOIN user_profiles up ON up.user_id = s.updated_by WHERE s.project_id = $1 ORDER BY s.name",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load the project's secrets"))?;
    Ok(Json(Secrets { secrets, can_edit: role == "admin" }))
}

#[derive(Deserialize)]
pub struct SetSecret {
    pub value: String,
}

/// `PUT /api/projects/:id/secrets/:name`: stores a new secret or replaces its value. The app
/// gets it on its next publish.
pub async fn set(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, name)): Path<(Uuid, String)>,
    Json(body): Json<SetSecret>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    check_name(&name).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    if body.value.is_empty() || body.value.len() > MAX_VALUE {
        return Err((StatusCode::BAD_REQUEST, format!("the value must be 1 to {MAX_VALUE} characters")));
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM project_secrets WHERE project_id = $1 AND name <> $2")
        .bind(project_id)
        .bind(&name)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to count the project's secrets"))?;
    if count >= MAX_SECRETS {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, format!("a project keeps up to {MAX_SECRETS} secrets")));
    }
    let client = client(&state)?;
    let target = target(&state, project_id).await?;
    let app = crate::shipyard::ensure_app(&state.pool, &client, &target).await.map_err(ensure_error)?;
    let existed: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM project_secrets WHERE project_id = $1 AND name = $2)")
        .bind(project_id)
        .bind(&name)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to look up the secret"))?;
    client.set_secret(&target.org_key(), &app, &name, Some(&body.value), None).await.map_err(shipyard_error)?;
    // A changed value reaches the running app on its next start; a new secret only on the next
    // publish, which binds it. Best-effort: the value is saved either way.
    if existed && published(&state.pool, project_id).await {
        if let Err(e) = client.restart_env(&target.org_key(), &app, LIVE_ENV).await {
            tracing::warn!(error = %e, %project_id, "couldn't restart the app after a secret changed");
        }
    }
    sqlx::query(
        "INSERT INTO project_secrets (project_id, name, updated_by) VALUES ($1, $2, $3) \
         ON CONFLICT (project_id, name) DO UPDATE SET updated_by = $3, updated_at = now()",
    )
    .bind(project_id)
    .bind(&name)
    .bind(claims.sub)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to save the secret"))?;
    tracing::info!(%project_id, user = %claims.sub, secret = %name, "project secret set");
    Ok(StatusCode::NO_CONTENT)
}

/// The environment a changed secret restarts.
const LIVE_ENV: &str = "production";

/// Whether the project has a version live in production.
async fn published(pool: &sqlx::PgPool, project_id: Uuid) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM project_publishes WHERE project_id = $1 AND environment = $2 AND status = 'succeeded')")
        .bind(project_id)
        .bind(LIVE_ENV)
        .fetch_one(pool)
        .await
        .unwrap_or(false)
}

/// `DELETE /api/projects/:id/secrets/:name`: removes it from Shipyard and from the app's next
/// publish.
pub async fn remove(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, name)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    if let Some(app) = target.shipyard_app.as_deref() {
        match client(&state)?.delete_secret(&target.org_key(), app, &name, None).await {
            Ok(_) | Err(ShipyardError::NotFound(_)) => {}
            Err(e) => return Err(shipyard_error(e)),
        }
    }
    let removed = sqlx::query("DELETE FROM project_secrets WHERE project_id = $1 AND name = $2")
        .bind(project_id)
        .bind(&name)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to remove the secret"))?
        .rows_affected();
    if removed == 0 {
        return Err((StatusCode::NOT_FOUND, "no such secret".to_string()));
    }
    tracing::info!(%project_id, user = %claims.sub, secret = %name, "project secret removed");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::check_name;

    #[test]
    fn names_are_environment_variables_nomi_doesnt_set() {
        assert!(check_name("STRIPE_KEY").is_ok());
        assert!(check_name("_PRIVATE2").is_ok());
        assert!(check_name("stripe_key").is_err());
        assert!(check_name("2FA_SECRET").is_err());
        assert!(check_name("API-KEY").is_err());
        assert!(check_name("").is_err());
        assert!(check_name(&"A".repeat(65)).is_err());
        assert!(check_name("DATABASE_URL").unwrap_err().contains("set by Nomi"));
    }
}
