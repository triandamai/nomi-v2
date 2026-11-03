//! Who can open a project. By default everyone active in its organization can; a project can
//! instead be limited to the people added to it. Added people are members (they build it) or
//! admins (they also publish, roll back, take it offline, see the database and manage members).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, require_admin, ApiError};

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectPerson {
    pub user_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    /// Their role in the organization (owner, admin, member).
    pub org_role: String,
    /// Their role on the project when added to it.
    pub project_role: Option<String>,
    /// Whether they started the project.
    pub is_owner: bool,
    pub added_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct ProjectMembers {
    /// `org`: everyone in the organization; `members`: only the people added.
    pub access: String,
    pub my_role: &'static str,
    pub me: Uuid,
    /// Everyone active in the organization, with their place on the project.
    pub people: Vec<ProjectPerson>,
}

async fn load(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<ProjectMembers, ApiError> {
    let my_role = project_role(&state.pool, project_id, user_id).await?;
    let access: String = sqlx::query_scalar("SELECT access FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load project"))?;
    let people: Vec<ProjectPerson> = sqlx::query_as(
        "SELECT m.user_id, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS name, \
                COALESCE(w.email, g.email) AS email, up.avatar_url, m.role AS org_role, \
                pm.role AS project_role, (p.user_id = m.user_id) AS is_owner, pm.created_at AS added_at \
         FROM projects p \
         JOIN memberships m ON m.org_id = p.org_id AND m.status = 'active' \
         LEFT JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = m.user_id \
         LEFT JOIN user_profiles up ON up.user_id = m.user_id \
         LEFT JOIN web_credentials w ON w.user_id = m.user_id \
         LEFT JOIN google_identities g ON g.user_id = m.user_id \
         WHERE p.id = $1 \
         ORDER BY (p.user_id = m.user_id) DESC, (pm.role IS NULL), pm.role, name",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load project members"))?;
    Ok(ProjectMembers { access, my_role, me: user_id, people })
}

/// `GET /api/projects/:id/members`
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectMembers>, ApiError> {
    Ok(Json(load(&state, project_id, claims.sub).await?))
}

#[derive(Deserialize)]
pub struct AccessRequest {
    pub access: String,
}

/// `PUT /api/projects/:id/access`: open to the organization, or only to the people added. The
/// person limiting it is added as an admin so they don't lock themselves out.
pub async fn set_access(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AccessRequest>,
) -> Result<Json<ProjectMembers>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if req.access != "org" && req.access != "members" {
        return Err((StatusCode::BAD_REQUEST, "access is org or members".to_string()));
    }
    let mut tx = state.pool.begin().await.map_err(internal("failed to change access"))?;
    sqlx::query("UPDATE projects SET access = $2 WHERE id = $1")
        .bind(project_id)
        .bind(&req.access)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to change access"))?;
    if req.access == "members" {
        sqlx::query(
            "INSERT INTO project_members (project_id, user_id, role, added_by) VALUES ($1, $2, 'admin', $2) \
             ON CONFLICT (project_id, user_id) DO UPDATE SET role = 'admin'",
        )
        .bind(project_id)
        .bind(claims.sub)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to change access"))?;
    }
    tx.commit().await.map_err(internal("failed to change access"))?;
    Ok(Json(load(&state, project_id, claims.sub).await?))
}

#[derive(Deserialize)]
pub struct MemberRequest {
    /// admin (builds and publishes), member (asks the crew; an admin publishes) or viewer
    /// (watches, changes nothing).
    pub role: String,
}

/// `PUT /api/projects/:id/members/:user_id`: adds someone from the organization, or changes
/// their role.
pub async fn put_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<MemberRequest>,
) -> Result<Json<ProjectMembers>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if !matches!(req.role.as_str(), "admin" | "member" | "viewer") {
        return Err((StatusCode::BAD_REQUEST, "role is admin, member or viewer".to_string()));
    }
    let in_org: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM projects p JOIN memberships m ON m.org_id = p.org_id \
         WHERE p.id = $1 AND m.user_id = $2 AND m.status = 'active')",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to add member"))?;
    if !in_org {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "invite them to the organization first".to_string()));
    }
    // `xmax = 0` only for a row this statement inserted: someone new to the project.
    let joined: bool = sqlx::query_scalar(
        "INSERT INTO project_members (project_id, user_id, role, added_by) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id, user_id) DO UPDATE SET role = $3 RETURNING (xmax = 0)",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(&req.role)
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to add member"))?;
    let members = load(&state, project_id, claims.sub).await?;
    if joined {
        let name = |id: Uuid| members.people.iter().find(|p| p.user_id == id).map(|p| p.name.clone()).unwrap_or_default();
        // Someone who can't publish has it published by the admin who added them.
        let publisher = (req.role != "admin").then(|| name(claims.sub));
        announce_joined(&state.pool, project_id, &name(user_id), &req.role, publisher.as_deref()).await;
    }
    Ok(Json(members))
}

/// `DELETE /api/projects/:id/members/:user_id`. The person who started the project can't be
/// removed (they always keep it), and the last admin of a limited project stays.
pub async fn remove_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectMembers>, ApiError> {
    if user_id != claims.sub {
        require_admin(&state.pool, project_id, claims.sub).await?;
    } else {
        project_role(&state.pool, project_id, claims.sub).await?;
    }
    sqlx::query("DELETE FROM project_members WHERE project_id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to remove member"))?;
    if user_id == claims.sub && project_role(&state.pool, project_id, claims.sub).await.is_err() {
        // They left a project limited to its members: nothing more for them to see.
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }
    Ok(Json(load(&state, project_id, claims.sub).await?))
}

/// The design system's names for a project role: build (publishes), prompt (asks the crew),
/// view (watches).
pub fn design_role(role: &str) -> &'static str {
    match role {
        "admin" => "build",
        "viewer" => "view",
        _ => "prompt",
    }
}

/// Says in the project's chat that someone joined: what they can do and who publishes for them.
pub(crate) async fn announce_joined(pool: &sqlx::PgPool, project_id: Uuid, name: &str, role: &str, publisher: Option<&str>) {
    let Ok(Some(session_id)) = sqlx::query_scalar::<_, Uuid>("SELECT session_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(pool).await else {
        return;
    };
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(pool).await.ok().flatten();
    let locale = match owner {
        Some(owner) => crate::shipyard::person_locale(pool, owner).await,
        None => nomi_i18n::Locale::from_code_or_default(None),
    };
    let text = locale.tf(&format!("together.joined_{}", design_role(role)), &[("name", name)]);
    let block = serde_json::json!([{ "kind": "member_joined", "name": name, "role": design_role(role), "publisher": publisher }]);
    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(&text)
    .bind(&block)
    .fetch_one(pool)
    .await
    .ok();
    if let (Some(message_id), Some(publisher)) = (message_id, crate::project_events::publisher()) {
        let _ = publisher.publish(session_id, &nomi_realtime::StreamEnvelope::MessageCreated { message_id }).await;
    }
}

#[derive(Deserialize)]
pub struct ProjectInviteRequest {
    pub email: String,
    /// build, prompt or view.
    pub role: String,
}

#[derive(Serialize)]
pub struct ProjectInvite {
    /// Already in the organization: added to the project straight away, nothing to accept.
    pub added: bool,
    pub email: String,
    pub role: String,
    /// The invite's link, when one was sent.
    pub path: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// `POST /api/projects/:id/invites` with `{ email, role: build|prompt|view }` (the project's
/// admins): someone in the organization is added right away; anyone else gets an invite to the
/// organization that also puts them on this project.
pub async fn invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<ProjectInviteRequest>,
) -> Result<(StatusCode, Json<ProjectInvite>), ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let role = match req.role.as_str() {
        "build" => "admin",
        "prompt" => "member",
        "view" => "viewer",
        _ => return Err((StatusCode::BAD_REQUEST, "role is build, prompt or view".to_string())),
    };
    let email = req.email.trim().to_lowercase();
    let org_id: Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to invite"))?;
    let member: Option<Uuid> = sqlx::query_scalar(
        "SELECT m.user_id FROM memberships m \
         JOIN (SELECT user_id FROM web_credentials WHERE lower(email) = $2 UNION SELECT user_id FROM google_identities WHERE lower(email) = $2) a \
           ON a.user_id = m.user_id \
         WHERE m.org_id = $1 AND m.status = 'active' LIMIT 1",
    )
    .bind(org_id)
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to invite"))?;
    if let Some(user_id) = member {
        let _ = put_member(State(state.clone()), AuthClaims(claims), Path((project_id, user_id)), Json(MemberRequest { role: role.to_string() })).await?;
        return Ok((StatusCode::OK, Json(ProjectInvite { added: true, email, role: req.role, path: None, expires_at: None })));
    }
    let sent = crate::routes::orgs::send_invite(&state, claims.sub, org_id, &email, "member", Some((project_id, role)))
        .await
        .map_err(|(code, why)| (code, why.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(ProjectInvite { added: false, email, role: req.role, path: Some(format!("/invite/{}", sent.code)), expires_at: Some(sent.expires_at) }),
    ))
}
