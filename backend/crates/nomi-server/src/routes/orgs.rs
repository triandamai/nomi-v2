//! Organizations people share projects in: the ones they're in, their members, and invites.
//!
//! Every organization has owners (who can do anything, including make other owners), admins
//! (who invite and remove members) and members (who work on its projects). An invite goes to
//! an email address and only works for someone signed in with that address. It's always
//! emailed; someone who already uses Nomi also finds it in their inbox, and someone who doesn't
//! is sent to create an account with that address first.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Duration, Utc};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_i18n::Locale;
use nomi_mail::layout::Action;

use crate::app::AppState;
use crate::notifications::{self, Kind, Notice};

type ApiError = (StatusCode, &'static str);

const INVITE_DAYS: i64 = 7;
/// How long a link anyone can join with works before it needs making again.
const INVITE_LINK_DAYS: i64 = 30;
const MAX_NAME_CHARS: usize = 80;
/// Faces shown on a workspace's card.
const MEMBER_AVATARS: i64 = 3;

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

/// The person's role in `org_id`, if they're an active member.
async fn role_in(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<Option<String>, ApiError> {
    sqlx::query_scalar(
        "SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to look up membership"))
}

/// Their role, or 404 when they're not in it (someone else's organization looks like none).
async fn require_member(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<String, ApiError> {
    role_in(pool, user_id, org_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "organization not found"))
}

async fn require_manager(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<String, ApiError> {
    let role = require_member(pool, user_id, org_id).await?;
    if role == "owner" || role == "admin" {
        Ok(role)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "only owners and admins can manage members",
        ))
    }
}

/// What people see for someone: their display name, else the start of their email.
async fn person_name(pool: &sqlx::PgPool, user_id: Uuid) -> String {
    sqlx::query_scalar(
        "SELECT COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) \
         FROM users u \
         LEFT JOIN user_profiles up ON up.user_id = u.id \
         LEFT JOIN web_credentials w ON w.user_id = u.id \
         LEFT JOIN google_identities g ON g.user_id = u.id \
         WHERE u.id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_default()
}

async fn person_locale(pool: &sqlx::PgPool, user_id: Uuid) -> Locale {
    let code: Option<String> =
        sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    Locale::from_code_or_default(code.as_deref())
}

fn clean_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty"));
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err((StatusCode::BAD_REQUEST, "name is too long"));
    }
    Ok(name.to_string())
}

#[derive(Serialize, sqlx::FromRow)]
pub struct OrgSummary {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub members: i64,
    pub projects: i64,
    /// Someone's own space, made with their account.
    pub is_personal: bool,
    /// The first few members (owners first), for the card's faces.
    #[sqlx(skip)]
    pub member_avatars: Vec<Avatar>,
    /// The newest thing that happened in its projects, if anything has.
    #[sqlx(skip)]
    pub last_activity: Option<OrgActivity>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Avatar {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct OrgActivity {
    /// "published" (a project went live) or "worked_on" (someone worked in a project's chat).
    pub kind: String,
    pub project: String,
    /// Who published, or whose project chat it was.
    pub by: Option<String>,
    pub at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OrgList {
    pub orgs: Vec<OrgSummary>,
    /// Where new projects go; null until the person picks one.
    pub project_org_id: Option<Uuid>,
    /// Where Nomi opens on sign-in; null is the workspace last used.
    pub open_org_id: Option<Uuid>,
}

/// A person's name as people see it, from the `users` row aliased `u`.
const NAME_SQL: &str =
    "COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1))";

async fn member_avatars(pool: &sqlx::PgPool, org_id: Uuid) -> Result<Vec<Avatar>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {NAME_SQL} AS name, up.avatar_url FROM memberships m \
         LEFT JOIN user_profiles up ON up.user_id = m.user_id \
         LEFT JOIN web_credentials w ON w.user_id = m.user_id \
         LEFT JOIN google_identities g ON g.user_id = m.user_id \
         WHERE m.org_id = $1 AND m.status = 'active' \
         ORDER BY CASE m.role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END, m.created_at LIMIT $2"
    ))
    .bind(org_id)
    .bind(MEMBER_AVATARS)
    .fetch_all(pool)
    .await
}

/// The newer of a workspace's last publish and the last message in one of its projects' chats.
/// Other chats in it are people's own, so they don't count.
async fn last_activity(
    pool: &sqlx::PgPool,
    org_id: Uuid,
) -> Result<Option<OrgActivity>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT kind, project, by, at FROM ( \
           (SELECT 'published' AS kind, p.name AS project, pp.created_by AS who, pp.created_at AS at \
            FROM project_publishes pp JOIN projects p ON p.id = pp.project_id \
            WHERE p.org_id = $1 AND pp.status = 'succeeded' ORDER BY pp.created_at DESC LIMIT 1) \
           UNION ALL \
           (SELECT 'worked_on', p.name, p.user_id, lm.created_at \
            FROM projects p JOIN LATERAL ( \
              SELECT created_at FROM messages m WHERE m.session_id = p.session_id ORDER BY created_at DESC LIMIT 1 \
            ) lm ON true \
            WHERE p.org_id = $1 ORDER BY lm.created_at DESC LIMIT 1) \
         ) a \
         LEFT JOIN LATERAL ( \
           SELECT {NAME_SQL} AS by FROM users u \
           LEFT JOIN user_profiles up ON up.user_id = u.id \
           LEFT JOIN web_credentials w ON w.user_id = u.id \
           LEFT JOIN google_identities g ON g.user_id = u.id \
           WHERE u.id = a.who \
         ) n ON true \
         ORDER BY at DESC LIMIT 1"
    ))
    .bind(org_id)
    .fetch_optional(pool)
    .await
}

/// `GET /api/orgs`: the organizations the person is in.
pub async fn list_orgs(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<OrgList>, ApiError> {
    let failed = internal("failed to list organizations");
    let mut orgs: Vec<OrgSummary> = sqlx::query_as(
        "SELECT o.id, o.name, m.role, o.is_personal, \
                (SELECT count(*) FROM memberships x WHERE x.org_id = o.id AND x.status = 'active') AS members, \
                (SELECT count(*) FROM projects p WHERE p.org_id = o.id) AS projects \
         FROM organizations o JOIN memberships m ON m.org_id = o.id \
         WHERE m.user_id = $1 AND m.status = 'active' AND o.deleted_at IS NULL \
         ORDER BY m.created_at",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(&failed)?;
    for org in &mut orgs {
        org.member_avatars = member_avatars(&state.pool, org.id).await.map_err(&failed)?;
        org.last_activity = last_activity(&state.pool, org.id).await.map_err(&failed)?;
    }
    let project_org_id = super::projects::project_org(&state.pool, claims.sub)
        .await
        .map_err(&failed)?;
    let open_org_id = open_org(&state.pool, claims.sub).await.map_err(&failed)?;
    Ok(Json(OrgList {
        orgs,
        project_org_id,
        open_org_id,
    }))
}

#[derive(Deserialize)]
pub struct NameRequest {
    pub name: String,
}

/// `POST /api/orgs`: a new organization, with the person as its owner.
pub async fn create_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<NameRequest>,
) -> Result<(StatusCode, Json<OrgSummary>), ApiError> {
    let name = clean_name(&req.name)?;
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(internal("failed to create organization"))?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal, created_by) VALUES ($1, false, $2) RETURNING id")
        .bind(&name)
        .bind(claims.sub)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal("failed to create organization"))?;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(id)
        .bind(claims.sub)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to create organization"))?;
    tx.commit()
        .await
        .map_err(internal("failed to create organization"))?;
    let member_avatars = member_avatars(&state.pool, id)
        .await
        .map_err(internal("failed to create organization"))?;
    Ok((
        StatusCode::CREATED,
        Json(OrgSummary {
            id,
            name,
            role: "owner".into(),
            members: 1,
            projects: 0,
            is_personal: false,
            member_avatars,
            last_activity: None,
        }),
    ))
}

/// `PATCH /api/orgs/:id`: renames it (owners and admins).
pub async fn rename_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(org_id): Path<Uuid>,
    Json(req): Json<NameRequest>,
) -> Result<StatusCode, ApiError> {
    require_manager(&state.pool, claims.sub, org_id).await?;
    let name = clean_name(&req.name)?;
    sqlx::query("UPDATE organizations SET name = $2 WHERE id = $1")
        .bind(org_id)
        .bind(&name)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to rename organization"))?;
    // Shipyard shows the same name; it follows in the background.
    if let Some(config) = state.shipyard.clone() {
        let (pool, http) = (state.pool.clone(), state.http_client.clone());
        tokio::spawn(async move {
            let client = nomi_shipyard::Shipyard::new(&http, &config);
            if let Err(e) = crate::shipyard::sync_org_name(&pool, &client, org_id).await {
                tracing::warn!(error = ?e, %org_id, "couldn't rename the organization on Shipyard");
            }
        });
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ProjectOrgRequest {
    pub org_id: Uuid,
}

/// `PUT /api/preferences/project-org`: where new projects go, and which projects are listed.
pub async fn set_project_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<ProjectOrgRequest>,
) -> Result<StatusCode, ApiError> {
    require_member(&state.pool, claims.sub, req.org_id).await?;
    remember_project_org(&state.pool, claims.sub, req.org_id)
        .await
        .map_err(internal("failed to save organization"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct OpenOrgRequest {
    /// The workspace Nomi opens in on sign-in; null opens the one last used.
    pub org_id: Option<Uuid>,
}

/// `PUT /api/preferences/open-org`: where Nomi opens on sign-in (a workspace, or the last used).
pub async fn set_open_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<OpenOrgRequest>,
) -> Result<StatusCode, ApiError> {
    if let Some(org_id) = req.org_id {
        require_member(&state.pool, claims.sub, org_id).await?;
    }
    sqlx::query(
        "INSERT INTO user_preferences (user_id, open_org_id) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET open_org_id = $2, updated_at = now()",
    )
    .bind(claims.sub)
    .bind(req.org_id)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to save organization"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// The workspace the person chose to open Nomi in, while they're still in it.
async fn open_org(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT p.open_org_id FROM user_preferences p \
         JOIN memberships m ON m.org_id = p.open_org_id AND m.user_id = p.user_id AND m.status = 'active' \
         WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

async fn remember_project_org(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_preferences (user_id, project_org_id) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET project_org_id = $2, updated_at = now()",
    )
    .bind(user_id)
    .bind(org_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Member {
    pub user_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    /// Projects in this workspace they started or were added to.
    pub projects: i64,
    /// When they last used Nomi (signed in, had a project open, or a device checked in).
    pub last_active_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PendingInvite {
    pub code: String,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// A link anyone can join with (no address), rather than an invite to one person.
    pub link: bool,
}

const PENDING_INVITE_COLUMNS: &str = "code, email, role, created_at, expires_at, is_link AS link";

#[derive(Serialize, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct OrgDetail {
    pub id: Uuid,
    pub name: String,
    pub is_personal: bool,
    /// Who made it; null when that account is gone.
    pub created_by: Option<Person>,
    pub created_at: DateTime<Utc>,
    pub my_role: String,
    pub me: Uuid,
    pub members: Vec<Member>,
    /// Only owners and admins see these.
    pub invites: Vec<PendingInvite>,
}

/// `GET /api/orgs/:id`: its members, and (for owners and admins) the invites still open.
pub async fn get_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrgDetail>, ApiError> {
    let my_role = require_member(&state.pool, claims.sub, org_id).await?;
    let (name, is_personal, created_at, creator): (String, bool, DateTime<Utc>, Option<Uuid>) =
        sqlx::query_as(
            "SELECT name, is_personal, created_at, created_by FROM organizations WHERE id = $1",
        )
        .bind(org_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load organization"))?;
    let created_by = match creator {
        Some(id) => Some(Person {
            id,
            name: person_name(&state.pool, id).await,
        }),
        None => None,
    };
    let members: Vec<Member> = sqlx::query_as(
        "SELECT m.user_id, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS name, \
                COALESCE(w.email, g.email) AS email, up.avatar_url, m.role, m.created_at AS joined_at, \
                (SELECT count(*) FROM projects p WHERE p.org_id = m.org_id AND (p.user_id = m.user_id \
                   OR EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = p.id AND pm.user_id = m.user_id))) AS projects, \
                GREATEST( \
                  (SELECT max(r.created_at) FROM refresh_tokens r WHERE r.user_id = m.user_id), \
                  (SELECT max(pp.last_seen) FROM project_presence pp JOIN projects p ON p.id = pp.project_id \
                     WHERE pp.user_id = m.user_id AND p.org_id = m.org_id), \
                  (SELECT max(d.last_seen_at) FROM devices d WHERE d.user_id = m.user_id) \
                ) AS last_active_at \
         FROM memberships m \
         LEFT JOIN user_profiles up ON up.user_id = m.user_id \
         LEFT JOIN web_credentials w ON w.user_id = m.user_id \
         LEFT JOIN google_identities g ON g.user_id = m.user_id \
         WHERE m.org_id = $1 AND m.status = 'active' \
         ORDER BY CASE m.role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END, m.created_at",
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load organization"))?;
    let invites: Vec<PendingInvite> = if my_role == "member" {
        Vec::new()
    } else {
        sqlx::query_as(&format!(
            "SELECT {PENDING_INVITE_COLUMNS} FROM org_invites \
             WHERE org_id = $1 AND used_at IS NULL AND declined_at IS NULL AND expires_at > now() ORDER BY created_at DESC"
        ))
        .bind(org_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to load organization"))?
    };
    Ok(Json(OrgDetail {
        id: org_id,
        name,
        is_personal,
        created_by,
        created_at,
        my_role,
        me: claims.sub,
        members,
        invites,
    }))
}

#[derive(Deserialize)]
pub struct InviteRequest {
    pub email: String,
    /// "member" (default) or "admin".
    pub role: Option<String>,
}

/// Percent-encodes a query value.
fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub(crate) fn valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.contains(char::is_whitespace)
}

/// `POST /api/orgs/:id/invites`: invites an address (owners and admins). Someone who already
/// has an account gets it in their inbox too; anyone else gets an email.
pub async fn create_invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(org_id): Path<Uuid>,
    Json(req): Json<InviteRequest>,
) -> Result<(StatusCode, Json<PendingInvite>), ApiError> {
    let my_role = require_manager(&state.pool, claims.sub, org_id).await?;
    let email = req.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err((StatusCode::BAD_REQUEST, "enter a valid email address"));
    }
    let role = req.role.as_deref().unwrap_or("member");
    if !["member", "admin"].contains(&role) {
        return Err((StatusCode::BAD_REQUEST, "role must be member or admin"));
    }
    if role == "admin" && my_role != "owner" && my_role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "only owners and admins can invite admins",
        ));
    }

    let invite = send_invite(&state, claims.sub, org_id, &email, role, None).await?;
    Ok((StatusCode::CREATED, Json(invite)))
}

/// Sends an invite to `email` to join `org_id` with `role` (and, with `project`, to build that
/// project with the given project role): in their inbox when they already use Nomi, by email
/// either way. A fresh invite replaces any still open for the same address.
pub(crate) async fn send_invite(
    state: &AppState,
    inviter_id: Uuid,
    org_id: Uuid,
    email: &str,
    role: &str,
    project: Option<(Uuid, &str)>,
) -> Result<PendingInvite, ApiError> {
    let claims_sub = inviter_id;
    let invitee = account_for(&state.pool, email).await?;
    if let Some(user_id) = invitee {
        if role_in(&state.pool, user_id, org_id).await?.is_some() {
            return Err((StatusCode::CONFLICT, "they're already a member"));
        }
    }

    // A fresh invite replaces any still open for the same address.
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let expires_at = Utc::now() + Duration::days(INVITE_DAYS);
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(internal("failed to invite"))?;
    sqlx::query("DELETE FROM org_invites WHERE org_id = $1 AND lower(email) = $2 AND used_at IS NULL AND NOT is_link")
        .bind(org_id)
        .bind(email)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to invite"))?;
    let invite: PendingInvite = sqlx::query_as(&format!(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at, email, project_id, project_role) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING {PENDING_INVITE_COLUMNS}"
    ))
    .bind(&code)
    .bind(org_id)
    .bind(role)
    .bind(claims_sub)
    .bind(expires_at)
    .bind(email)
    .bind(project.map(|(id, _)| id))
    .bind(project.map(|(_, role)| role))
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to invite"))?;
    tx.commit().await.map_err(internal("failed to invite"))?;

    deliver_invite(
        state,
        claims_sub,
        org_id,
        email,
        &code,
        invitee,
        project.map(|(id, _)| id),
    )
    .await?;
    Ok(invite)
}

/// Tells `email` about the invite `code`: in their inbox when they already use Nomi (`invitee`),
/// and by email either way.
async fn deliver_invite(
    state: &AppState,
    claims_sub: Uuid,
    org_id: Uuid,
    email: &str,
    code: &str,
    invitee: Option<Uuid>,
    project: Option<Uuid>,
) -> Result<(), ApiError> {
    let org_name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to invite"))?;
    let inviter = person_name(&state.pool, claims_sub).await;
    let path = format!("/invite/{code}");
    // Someone without an account signs up first (with this address), then lands on the invite.
    let open_path = match invitee {
        Some(_) => path.clone(),
        None => format!(
            "/register?redirect_to={}&email={}",
            urlencode(&path),
            urlencode(email)
        ),
    };
    let locale = match invitee {
        Some(user_id) => person_locale(&state.pool, user_id).await,
        None => person_locale(&state.pool, claims_sub).await,
    };
    let project_name: Option<String> = match project {
        Some(id) => sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten(),
        None => None,
    };
    let (title, body) = match &project_name {
        Some(name) => (
            locale.tf(
                "team.project_invite_title",
                &[("inviter", &inviter), ("project", name)],
            ),
            locale.tf(
                "team.project_invite_body",
                &[("project", name), ("org", &org_name)],
            ),
        ),
        None => (
            locale.tf(
                "team.invite_title",
                &[("inviter", &inviter), ("org", &org_name)],
            ),
            locale.tf("team.invite_body", &[("org", &org_name)]),
        ),
    };

    // In their inbox when they already use Nomi; by email either way.
    if let Some(user_id) = invitee {
        let notice = Notice {
            kind: Kind::Account,
            title: title.clone(),
            body: body.clone(),
            link: Some(path.clone()),
        };
        if let Err(e) = notifications::send_to_inbox(&state.pool, user_id, notice).await {
            tracing::warn!(error = %e, "failed to notify an invited member");
        }
    }
    let (action, note) = match (invitee.is_some(), crate::public_url::link(&open_path)) {
        (true, Some(url)) => (
            Some(Action {
                label: locale.t("team.invite_button"),
                url,
            }),
            locale.t("team.invite_link"),
        ),
        (true, None) => (None, locale.tf("team.invite_in_app", &[("where", &path)])),
        (false, Some(url)) => (
            Some(Action {
                label: locale.t("team.invite_signup_button"),
                url,
            }),
            locale.t("team.invite_signup_link"),
        ),
        (false, None) => (
            None,
            locale.tf("team.invite_signup_in_app", &[("where", &path)]),
        ),
    };
    notifications::email_address(
        email.to_string(),
        title,
        vec![body],
        action,
        vec![note],
        locale.tf("team.invite_reason", &[("inviter", &inviter)]),
        locale,
    );
    Ok(())
}

/// Whose account `email` is, if anyone's.
async fn account_for(pool: &sqlx::PgPool, email: &str) -> Result<Option<Uuid>, ApiError> {
    sqlx::query_scalar(
        "SELECT user_id FROM web_credentials WHERE lower(email) = $1 \
         UNION SELECT user_id FROM google_identities WHERE lower(email) = $1 LIMIT 1",
    )
    .bind(email.to_lowercase())
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to invite"))
}

/// `POST /api/orgs/:id/invites/:code/resend`: sends an open invite again and gives it another
/// week (owners and admins). Links have no address to send to.
pub async fn resend_invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org_id, code)): Path<(Uuid, String)>,
) -> Result<Json<PendingInvite>, ApiError> {
    require_manager(&state.pool, claims.sub, org_id).await?;
    let expires_at = Utc::now() + Duration::days(INVITE_DAYS);
    let invite: PendingInvite = sqlx::query_as(&format!(
        "UPDATE org_invites SET expires_at = $3, declined_at = NULL \
         WHERE org_id = $1 AND code = $2 AND used_at IS NULL AND NOT is_link AND email IS NOT NULL \
         RETURNING {PENDING_INVITE_COLUMNS}"
    ))
    .bind(org_id)
    .bind(&code)
    .bind(expires_at)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to resend invite"))?
    .ok_or((StatusCode::NOT_FOUND, "invite not found"))?;
    let email = invite.email.clone().unwrap_or_default();
    let invitee = account_for(&state.pool, &email).await?;
    let project: Option<Uuid> =
        sqlx::query_scalar("SELECT project_id FROM org_invites WHERE code = $1")
            .bind(&code)
            .fetch_one(&state.pool)
            .await
            .map_err(internal("failed to resend invite"))?;
    deliver_invite(&state, claims.sub, org_id, &email, &code, invitee, project).await?;
    Ok(Json(invite))
}

#[derive(Deserialize)]
pub struct InviteLinkRequest {
    /// "member" (default) or "admin".
    pub role: Option<String>,
}

/// `POST /api/orgs/:id/invite-links`: a link anyone signed in can join with, as `role` (owners
/// and admins). It replaces the open link for that role, works for many people, and lasts
/// `INVITE_LINK_DAYS`; revoking it is `DELETE /api/orgs/:id/invites/:code`.
pub async fn create_invite_link(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(org_id): Path<Uuid>,
    Json(req): Json<InviteLinkRequest>,
) -> Result<(StatusCode, Json<PendingInvite>), ApiError> {
    require_manager(&state.pool, claims.sub, org_id).await?;
    let role = req.role.as_deref().unwrap_or("member");
    if !["member", "admin"].contains(&role) {
        return Err((StatusCode::BAD_REQUEST, "role must be member or admin"));
    }
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let expires_at = Utc::now() + Duration::days(INVITE_LINK_DAYS);
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(internal("failed to make an invite link"))?;
    sqlx::query("DELETE FROM org_invites WHERE org_id = $1 AND is_link AND role = $2")
        .bind(org_id)
        .bind(role)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to make an invite link"))?;
    let invite: PendingInvite = sqlx::query_as(&format!(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at, is_link) \
         VALUES ($1, $2, $3, $4, $5, true) RETURNING {PENDING_INVITE_COLUMNS}"
    ))
    .bind(&code)
    .bind(org_id)
    .bind(role)
    .bind(claims.sub)
    .bind(expires_at)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to make an invite link"))?;
    tx.commit()
        .await
        .map_err(internal("failed to make an invite link"))?;
    Ok((StatusCode::CREATED, Json(invite)))
}

/// `DELETE /api/orgs/:id/invites/:code`: takes an open invite back.
pub async fn revoke_invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org_id, code)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    require_manager(&state.pool, claims.sub, org_id).await?;
    sqlx::query("DELETE FROM org_invites WHERE org_id = $1 AND code = $2 AND used_at IS NULL")
        .bind(org_id)
        .bind(&code)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to revoke invite"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct RoleRequest {
    pub role: String,
}

async fn owner_count(pool: &sqlx::PgPool, org_id: Uuid) -> Result<i64, ApiError> {
    sqlx::query_scalar("SELECT count(*) FROM memberships WHERE org_id = $1 AND role = 'owner' AND status = 'active'")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .map_err(internal("failed to count owners"))
}

/// `PATCH /api/orgs/:id/members/:user_id`: changes someone's role. Admins move people between
/// member and admin; only owners make or unmake owners, and the last owner stays one.
pub async fn set_member_role(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<RoleRequest>,
) -> Result<StatusCode, ApiError> {
    let my_role = require_manager(&state.pool, claims.sub, org_id).await?;
    if !["owner", "admin", "member"].contains(&req.role.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "role must be owner, admin or member",
        ));
    }
    let their_role = role_in(&state.pool, user_id, org_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "member not found"))?;
    if (req.role == "owner" || their_role == "owner") && my_role != "owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "only owners can change an owner's role or make owners",
        ));
    }
    if their_role == "owner" && req.role != "owner" && owner_count(&state.pool, org_id).await? <= 1
    {
        return Err((
            StatusCode::CONFLICT,
            "an organization needs at least one owner",
        ));
    }
    sqlx::query("UPDATE memberships SET role = $3 WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .bind(&req.role)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to change role"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/orgs/:id/members/:user_id`: removes someone (owners and admins), or leaves when
/// it's the person themselves. Only owners remove owners, and the last owner can't go.
pub async fn remove_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let my_role = require_member(&state.pool, claims.sub, org_id).await?;
    let their_role = role_in(&state.pool, user_id, org_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "member not found"))?;
    if user_id != claims.sub {
        if my_role == "member" {
            return Err((
                StatusCode::FORBIDDEN,
                "only owners and admins can remove members",
            ));
        }
        if their_role == "owner" && my_role != "owner" {
            return Err((StatusCode::FORBIDDEN, "only owners can remove an owner"));
        }
    }
    if their_role == "owner" && owner_count(&state.pool, org_id).await? <= 1 {
        return Err((
            StatusCode::CONFLICT,
            "an organization needs at least one owner",
        ));
    }
    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to remove member"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct InviteInfo {
    pub org_id: Uuid,
    pub org_name: String,
    pub inviter: String,
    pub role: String,
    /// "open", "expired", "used", "member" (already in it) or "other_address".
    pub state: &'static str,
    pub email: Option<String>,
    /// An invite to build one project together: which, and as what (build, prompt or view).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<InviteProject>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, Clone)]
pub struct InviteProject {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub role: &'static str,
}

/// The project an invite is for, if it's for one.
async fn invite_project(
    pool: &sqlx::PgPool,
    code: &str,
) -> Option<(Uuid, String, Option<String>, String)> {
    sqlx::query_as(
        "SELECT p.id, p.name, p.description, i.project_role FROM org_invites i JOIN projects p ON p.id = i.project_id \
         WHERE i.code = $1 AND i.project_role IS NOT NULL",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

async fn my_emails(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Vec<String>, ApiError> {
    sqlx::query_scalar(
        "SELECT lower(email) FROM web_credentials WHERE user_id = $1 UNION SELECT lower(email) FROM google_identities WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to look up invite"))
}

#[derive(sqlx::FromRow)]
struct InviteRow {
    org_id: Uuid,
    org_name: String,
    invited_by: Uuid,
    role: String,
    email: Option<String>,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    declined_at: Option<DateTime<Utc>>,
    is_link: bool,
}

async fn load_invite(pool: &sqlx::PgPool, code: &str) -> Result<InviteRow, ApiError> {
    sqlx::query_as(
        "SELECT i.org_id, o.name AS org_name, i.invited_by, i.role, i.email, i.expires_at, i.used_at, i.declined_at, i.is_link \
         FROM org_invites i JOIN organizations o ON o.id = i.org_id AND o.deleted_at IS NULL WHERE i.code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to look up invite"))?
    .ok_or((StatusCode::NOT_FOUND, "invite not found"))
}

async fn invite_state(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    row: &InviteRow,
) -> Result<&'static str, ApiError> {
    if role_in(pool, user_id, row.org_id).await?.is_some() {
        return Ok("member");
    }
    if row.used_at.is_some() {
        return Ok("used");
    }
    if row.declined_at.is_some() {
        return Ok("declined");
    }
    if row.expires_at <= Utc::now() {
        return Ok("expired");
    }
    if let Some(email) = &row.email {
        if !my_emails(pool, user_id)
            .await?
            .contains(&email.to_lowercase())
        {
            return Ok("other_address");
        }
    }
    Ok("open")
}

/// `GET /api/invites/:code`: what an invite is for, and whether this person can take it.
pub async fn get_invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(code): Path<String>,
) -> Result<Json<InviteInfo>, ApiError> {
    let row = load_invite(&state.pool, &code).await?;
    let invite_state = invite_state(&state.pool, claims.sub, &row).await?;
    let InviteRow {
        org_id,
        org_name,
        invited_by,
        role,
        email,
        expires_at,
        ..
    } = row;
    let project = invite_project(&state.pool, &code)
        .await
        .map(|(id, name, description, role)| InviteProject {
            id,
            name,
            description,
            role: super::project_members::design_role(&role),
        });
    Ok(Json(InviteInfo {
        org_id,
        org_name,
        inviter: person_name(&state.pool, invited_by).await,
        role,
        state: invite_state,
        email: email.map(|e| nomi_auth::email_code::mask_email(&e)),
        project,
        expires_at,
    }))
}

#[derive(Serialize)]
pub struct AcceptedInvite {
    pub org_id: Uuid,
    /// The project's chat, when the invite was to build a project: where to go next.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_session_id: Option<Uuid>,
}

/// `POST /api/invites/:code/accept`: joins the organization, and makes it the one Projects shows.
pub async fn accept_invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(code): Path<String>,
) -> Result<Json<AcceptedInvite>, ApiError> {
    let row = load_invite(&state.pool, &code).await?;
    match invite_state(&state.pool, claims.sub, &row).await? {
        "open" => {}
        "member" => {
            return Ok(Json(AcceptedInvite {
                org_id: row.org_id,
                project_session_id: None,
            }))
        }
        "used" => return Err((StatusCode::GONE, "this invite has already been used")),
        "declined" => return Err((StatusCode::GONE, "this invite was declined")),
        "expired" => return Err((StatusCode::GONE, "this invite has expired")),
        _ => {
            return Err((
                StatusCode::FORBIDDEN,
                "this invite was sent to another address",
            ))
        }
    }
    let InviteRow {
        org_id,
        org_name,
        invited_by,
        role,
        is_link,
        ..
    } = row;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(internal("failed to accept invite"))?;
    // Claimed once: a second accept of the same code finds it used. A link stays open for others.
    if !is_link {
        let claimed = sqlx::query("UPDATE org_invites SET used_at = now(), used_by = $2 WHERE code = $1 AND used_at IS NULL")
            .bind(&code)
            .bind(claims.sub)
            .execute(&mut *tx)
            .await
            .map_err(internal("failed to accept invite"))?;
        if claimed.rows_affected() == 0 {
            return Err((StatusCode::GONE, "this invite has already been used"));
        }
    }
    sqlx::query(
        "INSERT INTO memberships (org_id, user_id, role, invited_by) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (org_id, user_id) DO UPDATE SET status = 'active', role = $3, invited_by = $4",
    )
    .bind(org_id)
    .bind(claims.sub)
    .bind(&role)
    .bind(invited_by)
    .execute(&mut *tx)
    .await
    .map_err(internal("failed to accept invite"))?;
    // An invite to build a project also puts them on it, as what they were invited as.
    let project = invite_project(&state.pool, &code).await;
    if let Some((project_id, _, _, project_role)) = &project {
        sqlx::query(
            "INSERT INTO project_members (project_id, user_id, role, added_by) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (project_id, user_id) DO UPDATE SET role = $3",
        )
        .bind(project_id)
        .bind(claims.sub)
        .bind(project_role)
        .bind(invited_by)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to accept invite"))?;
    }
    tx.commit()
        .await
        .map_err(internal("failed to accept invite"))?;

    remember_project_org(&state.pool, claims.sub, org_id)
        .await
        .map_err(internal("failed to accept invite"))?;
    let mut project_session_id = None;
    if let Some((project_id, _, _, project_role)) = &project {
        let publisher = (project_role != "admin").then_some(invited_by);
        let publisher_name = match publisher {
            Some(id) => Some(person_name(&state.pool, id).await),
            None => None,
        };
        let name = person_name(&state.pool, claims.sub).await;
        super::project_members::announce_joined(
            &state.pool,
            *project_id,
            &name,
            project_role,
            publisher_name.as_deref(),
        )
        .await;
        project_session_id = sqlx::query_scalar("SELECT session_id FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();
    }

    let name = person_name(&state.pool, claims.sub).await;
    let locale = person_locale(&state.pool, invited_by).await;
    let notice = Notice {
        kind: Kind::Account,
        title: locale.tf("team.joined_title", &[("name", &name), ("org", &org_name)]),
        body: locale.tf("team.joined_body", &[("org", &org_name)]),
        link: Some(format!("/organizations/{org_id}")),
    };
    if let Err(e) = notifications::send(&state.pool, invited_by, notice).await {
        tracing::warn!(error = %e, "failed to tell the inviter someone joined");
    }

    Ok(Json(AcceptedInvite {
        org_id,
        project_session_id,
    }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MyInvite {
    pub code: String,
    pub org_id: Uuid,
    pub org_name: String,
    pub inviter: String,
    pub role: String,
    /// Projects in the workspace.
    pub projects: i64,
    pub expires_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub project: Option<MyInviteProject>,
    #[serde(skip)]
    pub project_id: Option<Uuid>,
    #[serde(skip)]
    pub project_name: Option<String>,
}

#[derive(Serialize)]
pub struct MyInviteProject {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct MyInvites {
    pub invites: Vec<MyInvite>,
}

/// `GET /api/invites`: the open invites to the signed-in person's addresses, newest first, for
/// workspaces they aren't in yet.
pub async fn list_my_invites(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<MyInvites>, ApiError> {
    let emails = my_emails(&state.pool, claims.sub).await?;
    let mut invites: Vec<MyInvite> = sqlx::query_as(&format!(
        "SELECT i.code, i.org_id, o.name AS org_name, i.role, i.expires_at, \
                (SELECT count(*) FROM projects p WHERE p.org_id = i.org_id) AS projects, \
                i.project_id, pr.name AS project_name, \
                (SELECT {NAME_SQL} FROM users u \
                   LEFT JOIN user_profiles up ON up.user_id = u.id \
                   LEFT JOIN web_credentials w ON w.user_id = u.id \
                   LEFT JOIN google_identities g ON g.user_id = u.id \
                   WHERE u.id = i.invited_by) AS inviter \
         FROM org_invites i \
         JOIN organizations o ON o.id = i.org_id AND o.deleted_at IS NULL \
         LEFT JOIN projects pr ON pr.id = i.project_id \
         WHERE lower(i.email) = ANY($1) AND i.used_at IS NULL AND i.declined_at IS NULL AND i.expires_at > now() \
           AND NOT EXISTS (SELECT 1 FROM memberships m WHERE m.org_id = i.org_id AND m.user_id = $2 AND m.status = 'active') \
         ORDER BY i.created_at DESC"
    ))
    .bind(&emails)
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list invites"))?;
    for invite in &mut invites {
        if let (Some(id), Some(name)) = (invite.project_id, invite.project_name.clone()) {
            invite.project = Some(MyInviteProject { id, name });
        }
    }
    Ok(Json(MyInvites { invites }))
}

/// `POST /api/invites/:code/decline`: says no to an open invite sent to the person's address.
/// It's gone from their list and from the workspace's open invites.
pub async fn decline_invite(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(code): Path<String>,
) -> Result<StatusCode, ApiError> {
    let row = load_invite(&state.pool, &code).await?;
    if row.is_link {
        return Err((StatusCode::BAD_REQUEST, "a link can't be declined"));
    }
    match invite_state(&state.pool, claims.sub, &row).await? {
        "open" => {}
        "declined" => return Ok(StatusCode::NO_CONTENT),
        "member" => return Err((StatusCode::CONFLICT, "you're already a member")),
        "used" | "expired" => return Err((StatusCode::GONE, "this invite is no longer open")),
        _ => {
            return Err((
                StatusCode::FORBIDDEN,
                "this invite was sent to another address",
            ))
        }
    }
    sqlx::query("UPDATE org_invites SET declined_at = now() WHERE code = $1 AND used_at IS NULL")
        .bind(&code)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to decline invite"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/orgs/:id`: closes a workspace (owners only). Its projects must be moved or
/// deleted first (409), and it can't be someone's personal space or the only workspace one of
/// its members has (409). Each member's own chats in it move to their next workspace; open
/// invites and links stop working; everyone leaves it.
pub async fn delete_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(org_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let failed = internal("failed to delete organization");
    if require_member(&state.pool, claims.sub, org_id).await? != "owner" {
        return Err((StatusCode::FORBIDDEN, "only owners can delete a workspace"));
    }
    let (is_personal, projects): (bool, i64) =
        sqlx::query_as("SELECT is_personal, (SELECT count(*) FROM projects WHERE org_id = $1) FROM organizations WHERE id = $1")
            .bind(org_id)
            .fetch_one(&state.pool)
            .await
            .map_err(&failed)?;
    if is_personal {
        return Err((
            StatusCode::CONFLICT,
            "a personal workspace can't be deleted",
        ));
    }
    if projects > 0 {
        return Err((StatusCode::CONFLICT, "move or delete its projects first"));
    }
    let stranded: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM memberships m WHERE m.org_id = $1 AND m.status = 'active' AND NOT EXISTS ( \
           SELECT 1 FROM memberships x JOIN organizations o ON o.id = x.org_id AND o.deleted_at IS NULL \
           WHERE x.user_id = m.user_id AND x.org_id <> $1 AND x.status = 'active'))",
    )
    .bind(org_id)
    .fetch_one(&state.pool)
    .await
    .map_err(&failed)?;
    if stranded {
        return Err((
            StatusCode::CONFLICT,
            "it's the only workspace one of its members has",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(&failed)?;
    // Each member's own chats go with them to the first other workspace they're in.
    sqlx::query(
        "UPDATE sessions s SET org_id = ( \
           SELECT x.org_id FROM memberships x JOIN organizations o ON o.id = x.org_id AND o.deleted_at IS NULL \
           WHERE x.user_id = s.user_id AND x.org_id <> $1 AND x.status = 'active' ORDER BY x.created_at LIMIT 1) \
         WHERE s.org_id = $1 AND EXISTS (SELECT 1 FROM memberships m WHERE m.org_id = $1 AND m.user_id = s.user_id AND m.status = 'active')",
    )
    .bind(org_id)
    .execute(&mut *tx)
    .await
    .map_err(&failed)?;
    for statement in [
        "DELETE FROM org_invites WHERE org_id = $1 AND used_at IS NULL",
        "UPDATE memberships SET status = 'removed' WHERE org_id = $1",
        "UPDATE users SET active_org_id = NULL WHERE active_org_id = $1",
        "UPDATE user_preferences SET project_org_id = NULL WHERE project_org_id = $1",
        "UPDATE user_preferences SET open_org_id = NULL WHERE open_org_id = $1",
        "UPDATE organizations SET deleted_at = now() WHERE id = $1",
    ] {
        sqlx::query(statement)
            .bind(org_id)
            .execute(&mut *tx)
            .await
            .map_err(&failed)?;
    }
    tx.commit().await.map_err(&failed)?;
    Ok(StatusCode::NO_CONTENT)
}

/// A new account made with a project invite (password, emailed code or Google sign-up) is put on
/// the project as it's made (`registration::place_in_org`); this tells the project's chat, as
/// accepting does.
pub(crate) async fn announce_signup_joins(pool: &sqlx::PgPool, user_id: Uuid) {
    let joined: Vec<(Uuid, String, Uuid)> = match sqlx::query_as(
        "SELECT project_id, project_role, invited_by FROM org_invites \
         WHERE used_by = $1 AND project_id IS NOT NULL AND project_role IS NOT NULL",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "couldn't look up the projects a new account joined");
            return;
        }
    };
    let name = person_name(pool, user_id).await;
    for (project_id, project_role, invited_by) in joined {
        let publisher = match project_role.as_str() {
            "admin" => None,
            _ => Some(person_name(pool, invited_by).await),
        };
        super::project_members::announce_joined(
            pool,
            project_id,
            &name,
            &project_role,
            publisher.as_deref(),
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::valid_email;

    #[test]
    fn checks_the_shape_of_an_address() {
        assert!(valid_email("ana@example.com"));
        assert!(!valid_email("ana@example"));
        assert!(!valid_email("@example.com"));
        assert!(!valid_email("ana example@example.com"));
        assert!(!valid_email("ana@.com"));
    }
}
