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
const MAX_NAME_CHARS: usize = 80;

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

/// The person's role in `org_id`, if they're an active member.
async fn role_in(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<Option<String>, ApiError> {
    sqlx::query_scalar("SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active'")
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal("failed to look up membership"))
}

/// Their role, or 404 when they're not in it (someone else's organization looks like none).
async fn require_member(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<String, ApiError> {
    role_in(pool, user_id, org_id).await?.ok_or((StatusCode::NOT_FOUND, "organization not found"))
}

async fn require_manager(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<String, ApiError> {
    let role = require_member(pool, user_id, org_id).await?;
    if role == "owner" || role == "admin" {
        Ok(role)
    } else {
        Err((StatusCode::FORBIDDEN, "only owners and admins can manage members"))
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
    let code: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
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
}

#[derive(Serialize)]
pub struct OrgList {
    pub orgs: Vec<OrgSummary>,
    /// Where new projects go; null until the person picks one.
    pub project_org_id: Option<Uuid>,
}

/// `GET /api/orgs`: the organizations the person is in.
pub async fn list_orgs(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<OrgList>, ApiError> {
    let orgs: Vec<OrgSummary> = sqlx::query_as(
        "SELECT o.id, o.name, m.role, \
                (SELECT count(*) FROM memberships x WHERE x.org_id = o.id AND x.status = 'active') AS members, \
                (SELECT count(*) FROM projects p WHERE p.org_id = o.id) AS projects \
         FROM organizations o JOIN memberships m ON m.org_id = o.id \
         WHERE m.user_id = $1 AND m.status = 'active' \
         ORDER BY m.created_at",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list organizations"))?;
    let project_org_id = super::projects::project_org(&state.pool, claims.sub).await.map_err(internal("failed to list organizations"))?;
    Ok(Json(OrgList { orgs, project_org_id }))
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
    let mut tx = state.pool.begin().await.map_err(internal("failed to create organization"))?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ($1, false) RETURNING id")
        .bind(&name)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal("failed to create organization"))?;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(id)
        .bind(claims.sub)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to create organization"))?;
    tx.commit().await.map_err(internal("failed to create organization"))?;
    Ok((StatusCode::CREATED, Json(OrgSummary { id, name, role: "owner".into(), members: 1, projects: 0 })))
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
    remember_project_org(&state.pool, claims.sub, req.org_id).await.map_err(internal("failed to save organization"))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remember_project_org(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<(), sqlx::Error> {
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
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PendingInvite {
    pub code: String,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OrgDetail {
    pub id: Uuid,
    pub name: String,
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
    let name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load organization"))?;
    let members: Vec<Member> = sqlx::query_as(
        "SELECT m.user_id, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS name, \
                COALESCE(w.email, g.email) AS email, up.avatar_url, m.role, m.created_at AS joined_at \
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
        sqlx::query_as(
            "SELECT code, email, role, created_at, expires_at FROM org_invites \
             WHERE org_id = $1 AND used_at IS NULL AND expires_at > now() ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to load organization"))?
    };
    Ok(Json(OrgDetail { id: org_id, name, my_role, me: claims.sub, members, invites }))
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
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else { return false };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.') && !email.contains(char::is_whitespace)
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
        return Err((StatusCode::FORBIDDEN, "only owners and admins can invite admins"));
    }

    // Whose address it is, if anyone's.
    let invitee: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM web_credentials WHERE lower(email) = $1 \
         UNION SELECT user_id FROM google_identities WHERE lower(email) = $1 LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to invite"))?;
    if let Some(user_id) = invitee {
        if role_in(&state.pool, user_id, org_id).await?.is_some() {
            return Err((StatusCode::CONFLICT, "they're already a member"));
        }
    }

    // A fresh invite replaces any still open for the same address.
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let expires_at = Utc::now() + Duration::days(INVITE_DAYS);
    let mut tx = state.pool.begin().await.map_err(internal("failed to invite"))?;
    sqlx::query("DELETE FROM org_invites WHERE org_id = $1 AND lower(email) = $2 AND used_at IS NULL")
        .bind(org_id)
        .bind(&email)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to invite"))?;
    let invite: PendingInvite = sqlx::query_as(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at, email) VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING code, email, role, created_at, expires_at",
    )
    .bind(&code)
    .bind(org_id)
    .bind(role)
    .bind(claims.sub)
    .bind(expires_at)
    .bind(&email)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to invite"))?;
    tx.commit().await.map_err(internal("failed to invite"))?;

    let org_name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to invite"))?;
    let inviter = person_name(&state.pool, claims.sub).await;
    let path = format!("/invite/{code}");
    // Someone without an account signs up first (with this address), then lands on the invite.
    let open_path = match invitee {
        Some(_) => path.clone(),
        None => format!("/register?redirect_to={}&email={}", urlencode(&path), urlencode(&email)),
    };
    let locale = match invitee {
        Some(user_id) => person_locale(&state.pool, user_id).await,
        None => person_locale(&state.pool, claims.sub).await,
    };
    let title = locale.tf("team.invite_title", &[("inviter", &inviter), ("org", &org_name)]);
    let body = locale.tf("team.invite_body", &[("org", &org_name)]);

    // In their inbox when they already use Nomi; by email either way.
    if let Some(user_id) = invitee {
        let notice = Notice { kind: Kind::Account, title: title.clone(), body: body.clone(), link: Some(path.clone()) };
        if let Err(e) = notifications::send_to_inbox(&state.pool, user_id, notice).await {
            tracing::warn!(error = %e, "failed to notify an invited member");
        }
    }
    let (action, note) = match (invitee.is_some(), crate::public_url::link(&open_path)) {
        (true, Some(url)) => (Some(Action { label: locale.t("team.invite_button"), url }), locale.t("team.invite_link")),
        (true, None) => (None, locale.tf("team.invite_in_app", &[("where", &path)])),
        (false, Some(url)) => (Some(Action { label: locale.t("team.invite_signup_button"), url }), locale.t("team.invite_signup_link")),
        (false, None) => (None, locale.tf("team.invite_signup_in_app", &[("where", &path)])),
    };
    notifications::email_address(
        email.clone(),
        title,
        vec![body],
        action,
        vec![note],
        locale.tf("team.invite_reason", &[("inviter", &inviter)]),
        locale,
    );

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
        return Err((StatusCode::BAD_REQUEST, "role must be owner, admin or member"));
    }
    let their_role = role_in(&state.pool, user_id, org_id).await?.ok_or((StatusCode::NOT_FOUND, "member not found"))?;
    if (req.role == "owner" || their_role == "owner") && my_role != "owner" {
        return Err((StatusCode::FORBIDDEN, "only owners can change an owner's role or make owners"));
    }
    if their_role == "owner" && req.role != "owner" && owner_count(&state.pool, org_id).await? <= 1 {
        return Err((StatusCode::CONFLICT, "an organization needs at least one owner"));
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
    let their_role = role_in(&state.pool, user_id, org_id).await?.ok_or((StatusCode::NOT_FOUND, "member not found"))?;
    if user_id != claims.sub {
        if my_role == "member" {
            return Err((StatusCode::FORBIDDEN, "only owners and admins can remove members"));
        }
        if their_role == "owner" && my_role != "owner" {
            return Err((StatusCode::FORBIDDEN, "only owners can remove an owner"));
        }
    }
    if their_role == "owner" && owner_count(&state.pool, org_id).await? <= 1 {
        return Err((StatusCode::CONFLICT, "an organization needs at least one owner"));
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

type InviteRow = (Uuid, String, Uuid, String, Option<String>, DateTime<Utc>, Option<DateTime<Utc>>);

async fn load_invite(pool: &sqlx::PgPool, code: &str) -> Result<InviteRow, ApiError> {
    sqlx::query_as(
        "SELECT i.org_id, o.name, i.invited_by, i.role, i.email, i.expires_at, i.used_at \
         FROM org_invites i JOIN organizations o ON o.id = i.org_id WHERE i.code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to look up invite"))?
    .ok_or((StatusCode::NOT_FOUND, "invite not found"))
}

async fn invite_state(pool: &sqlx::PgPool, user_id: Uuid, row: &InviteRow) -> Result<&'static str, ApiError> {
    let (org_id, _, _, _, email, expires_at, used_at) = row;
    if role_in(pool, user_id, *org_id).await?.is_some() {
        return Ok("member");
    }
    if used_at.is_some() {
        return Ok("used");
    }
    if *expires_at <= Utc::now() {
        return Ok("expired");
    }
    if let Some(email) = email {
        if !my_emails(pool, user_id).await?.contains(&email.to_lowercase()) {
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
    let (org_id, org_name, invited_by, role, email, _, _) = row;
    Ok(Json(InviteInfo {
        org_id,
        org_name,
        inviter: person_name(&state.pool, invited_by).await,
        role,
        state: invite_state,
        email: email.map(|e| nomi_auth::email_code::mask_email(&e)),
    }))
}

#[derive(Serialize)]
pub struct AcceptedInvite {
    pub org_id: Uuid,
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
        "member" => return Ok(Json(AcceptedInvite { org_id: row.0 })),
        "used" => return Err((StatusCode::GONE, "this invite has already been used")),
        "expired" => return Err((StatusCode::GONE, "this invite has expired")),
        _ => return Err((StatusCode::FORBIDDEN, "this invite was sent to another address")),
    }
    let (org_id, org_name, invited_by, role, _, _, _) = row;

    let mut tx = state.pool.begin().await.map_err(internal("failed to accept invite"))?;
    // Claimed once: a second accept of the same code finds it used.
    let claimed = sqlx::query("UPDATE org_invites SET used_at = now(), used_by = $2 WHERE code = $1 AND used_at IS NULL")
        .bind(&code)
        .bind(claims.sub)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to accept invite"))?;
    if claimed.rows_affected() == 0 {
        return Err((StatusCode::GONE, "this invite has already been used"));
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
    tx.commit().await.map_err(internal("failed to accept invite"))?;

    remember_project_org(&state.pool, claims.sub, org_id).await.map_err(internal("failed to accept invite"))?;

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

    Ok(Json(AcceptedInvite { org_id }))
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
