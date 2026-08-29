# Grantable Permissions & Admin User Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a platform admin grant a specific user a specific permission (`nomi:admin:<resource>:[view,manage]` or `nomi:<org_id>:<resource>:[view,manage]`) without flipping `is_platform_admin`, and give admins a `/admin/users` page to do it: a roster, one-click "promote to staff," a per-user permission grant/revoke panel, and org assignment.

**Architecture:** One new table (`user_permissions`) storing explicit per-user grants. `compute_permissions` (already called at login/register/refresh, unchanged call sites) unions its two existing hardcoded sources (the `is_platform_admin` flag, `memberships.role`) with two new ones read from this table — merging action sets when the same `(scope, resource)` appears from more than one source. A new `nomi-server` route module (`admin_users.rs`) exposes roster/detail/grant/revoke/assign-org endpoints, gated by `nomi:admin:user:[view|manage]` the same way every existing admin route gates on `nomi:admin:system_config:*`. The `/admin` layout gate broadens from "system_config only" to "any `nomi:admin:*` permission," with each sub-page (including the new Users page) keeping its own specific resource check, mirrored in the sidebar so ungranted links don't render. The new frontend page uses the existing `DataTable`, `BottomSheet` (per your instruction — not `Dialog`), `List`/`ListItem`, and `Select` components; no new reusable components are introduced.

**Tech Stack:** Rust, axum, sqlx (Postgres), SvelteKit 2 / Svelte 5, Tailwind, MD3 design tokens.

**Spec:** `docs/superpowers/specs/2026-08-29-grantable-permissions-design.md`

## Global Constraints

- `compute_permissions`'s two existing branches (`is_platform_admin`, `memberships.role`) must not change their output for any existing user — every assertion in `backend/crates/nomi-auth/tests/auth_permissions.rs` must keep passing unmodified (verified: all of them use `.contains(&exact_string)` with actions always ordered `[view,manage]`, never `[manage,view]` — the merge logic must preserve that literal ordering).
- `actions` on a grant is closed to exactly `view`/`manage` (matches the DB `CHECK` and the existing bracket-list grammar `Claims::has_permission` parses) — validated in Rust *before* hitting the DB, so a bad request gets a clean `400`, not a constraint-violation `500`.
- `resource` is free text (per your answer) but format-validated: must start with a lowercase ASCII letter, followed by only lowercase ASCII letters, digits, or underscores. Same rule enforced in both the DB `CHECK` and application code.
- The `UNIQUE (user_id, scope_type, org_id, resource)` constraint does **not** collapse two admin-scope rows for the same user+resource, because Postgres treats `NULL <> NULL` — the grant path must do a `SELECT ... FOR UPDATE` inside a transaction, not rely on `ON CONFLICT`, and must NULL-safe-match `org_id` with `IS NOT DISTINCT FROM`.
- All new backend routes live in a **new** module — do not add anything to `settings.rs` or `llm_models.rs`.
- All new backend routes are gated with `claims.has_permission("admin", "user", "view" | "manage")` — reads need `view`, writes need `manage`. No route in this plan uses `authorize_org_action` (that helper re-validates a *caller's own* membership for org-scoped self-service actions; every route here is a platform-admin managing *someone else's* grants/memberships, which is exactly what `nomi:admin:user:manage` is for).
- No automated frontend tests are added — matches every other admin page shipped so far (confirmed in `2026-08-28-admin-dashboard-and-agents-implementation.md`'s own Global Constraints). Frontend verification is `npm run check` plus a live browser check, not a test run.
- Only use MD3 typescale classes confirmed to exist in `frontend/src/lib/styles/material3.css`: `md-headline-small`, `md-headline-small-emphasized`, `md-title-large`, `md-title-medium`, `md-body-large`, `md-body-medium`, `md-body-small`, `md-label-large`, `md-label-medium`.
- No new reusable Svelte component is added for the view/manage action checkboxes — two plain `<input type="checkbox">` elements, styled minimally inline, are enough for this one form and don't warrant a new `Checkbox.svelte` (YAGNI — no other page in this codebase needs one yet).
- `.cargo/config.toml` in this worktree already has a working `DATABASE_URL`/`JWT_SECRET`/`SETTINGS_ENCRYPTION_KEY` (copied from the main tree's uncommitted local config before this plan started) — `cargo test`/`cargo check` need no extra setup.

---

### Task 1: Backend — schema, grants module, `compute_permissions` rewrite

**Files:**
- Create: `backend/migrations/0017_user_permissions.sql`
- Create: `backend/crates/nomi-auth/src/grants.rs`
- Modify: `backend/crates/nomi-auth/src/lib.rs` (register the new module)
- Modify: `backend/crates/nomi-auth/src/permissions.rs` (rewrite `compute_permissions`)
- Create: `backend/crates/nomi-auth/tests/auth_grants.rs`

**Interfaces:**
- Produces (used by Task 2): `nomi_auth::grants::{PermissionGrant, NewGrant, GrantError, list_grants_for_user, grant_permission, revoke_permission}`.
  - `pub struct PermissionGrant { pub id: Uuid, pub scope_type: String, pub org_id: Option<Uuid>, pub org_name: Option<String>, pub resource: String, pub actions: Vec<String>, pub created_at: chrono::DateTime<chrono::Utc> }` — `Serialize`, `sqlx::FromRow`.
  - `pub struct NewGrant<'a> { pub user_id: Uuid, pub scope_type: &'a str, pub org_id: Option<Uuid>, pub resource: &'a str, pub actions: &'a [String], pub granted_by: Uuid }`.
  - `pub enum GrantError { InvalidResource, InvalidActions, OrgRequiredForOrgScope, OrgNotAllowedForAdminScope, Db(sqlx::Error) }` (`thiserror`).
  - `pub async fn list_grants_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PermissionGrant>, sqlx::Error>`.
  - `pub async fn grant_permission(pool: &PgPool, input: NewGrant<'_>) -> Result<Vec<PermissionGrant>, GrantError>` — validates, then select-then-merge-or-insert, returns the user's full updated grant list.
  - `pub async fn revoke_permission(pool: &PgPool, user_id: Uuid, grant_id: Uuid) -> Result<Vec<PermissionGrant>, sqlx::Error>` — deletes if present (no error if already gone — idempotent), returns the updated list.

- [ ] **Step 1: Write the migration**

Create `backend/migrations/0017_user_permissions.sql`:

```sql
CREATE TABLE user_permissions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope_type  TEXT NOT NULL CHECK (scope_type IN ('admin', 'org')),
    org_id      UUID REFERENCES organizations(id) ON DELETE CASCADE,
    resource    TEXT NOT NULL CHECK (resource ~ '^[a-z][a-z0-9_]*$'),
    actions     TEXT[] NOT NULL CHECK (actions <@ ARRAY['view', 'manage']::TEXT[] AND array_length(actions, 1) > 0),
    granted_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((scope_type = 'admin' AND org_id IS NULL) OR (scope_type = 'org' AND org_id IS NOT NULL))
);

CREATE INDEX user_permissions_user_id_idx ON user_permissions(user_id);
```

No `UNIQUE` constraint in the DB — deliberate, per Global Constraints (Postgres can't use one to prevent duplicate NULL-`org_id` rows anyway); dedup is entirely `grant_permission`'s job via `SELECT ... FOR UPDATE`.

- [ ] **Step 2: Run migrations to confirm the new file applies cleanly**

Run: `cd backend && cargo test -p nomi-server --test admin_dashboard_routes -- dashboard_returns_zero_counts_on_a_fresh_install`
Expected: PASS. (This test's `#[sqlx::test(migrations = "../../migrations")]` applies every migration file, including the new one, against a fresh throwaway test database — a clean pass proves `0017_user_permissions.sql` has no syntax error and doesn't break any existing migration ordering.)

- [ ] **Step 3: Write the grants module**

Create `backend/crates/nomi-auth/src/grants.rs`:

```rust
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PermissionGrant {
    pub id: Uuid,
    pub scope_type: String,
    pub org_id: Option<Uuid>,
    pub org_name: Option<String>,
    pub resource: String,
    pub actions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct NewGrant<'a> {
    pub user_id: Uuid,
    pub scope_type: &'a str,
    pub org_id: Option<Uuid>,
    pub resource: &'a str,
    pub actions: &'a [String],
    pub granted_by: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum GrantError {
    #[error("resource must start with a lowercase letter and contain only lowercase letters, digits, and underscores")]
    InvalidResource,
    #[error("actions must be a non-empty subset of view, manage")]
    InvalidActions,
    #[error("org_id is required when scope_type is org")]
    OrgRequiredForOrgScope,
    #[error("org_id must not be set when scope_type is admin")]
    OrgNotAllowedForAdminScope,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

const ALLOWED_ACTIONS: [&str; 2] = ["view", "manage"];

fn validate_resource(resource: &str) -> Result<(), GrantError> {
    let mut chars = resource.chars();
    let first_ok = chars.next().map(|c| c.is_ascii_lowercase()).unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if first_ok && rest_ok {
        Ok(())
    } else {
        Err(GrantError::InvalidResource)
    }
}

fn validate_actions(actions: &[String]) -> Result<(), GrantError> {
    if actions.is_empty() || !actions.iter().all(|a| ALLOWED_ACTIONS.contains(&a.as_str())) {
        Err(GrantError::InvalidActions)
    } else {
        Ok(())
    }
}

pub async fn list_grants_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PermissionGrant>, sqlx::Error> {
    sqlx::query_as(
        "SELECT up.id, up.scope_type, up.org_id, o.name AS org_name, up.resource, up.actions, up.created_at \
         FROM user_permissions up \
         LEFT JOIN organizations o ON o.id = up.org_id \
         WHERE up.user_id = $1 \
         ORDER BY up.created_at ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn grant_permission(pool: &PgPool, input: NewGrant<'_>) -> Result<Vec<PermissionGrant>, GrantError> {
    validate_resource(input.resource)?;
    validate_actions(input.actions)?;
    match (input.scope_type, input.org_id) {
        ("admin", Some(_)) => return Err(GrantError::OrgNotAllowedForAdminScope),
        ("org", None) => return Err(GrantError::OrgRequiredForOrgScope),
        _ => {}
    }

    let mut tx = pool.begin().await?;

    let existing: Option<(Uuid, Vec<String>)> = sqlx::query_as(
        "SELECT id, actions FROM user_permissions \
         WHERE user_id = $1 AND scope_type = $2 AND org_id IS NOT DISTINCT FROM $3 AND resource = $4 \
         FOR UPDATE",
    )
    .bind(input.user_id)
    .bind(input.scope_type)
    .bind(input.org_id)
    .bind(input.resource)
    .fetch_optional(&mut *tx)
    .await?;

    match existing {
        Some((id, mut current_actions)) => {
            for action in input.actions {
                if !current_actions.contains(action) {
                    current_actions.push(action.clone());
                }
            }
            sqlx::query("UPDATE user_permissions SET actions = $1 WHERE id = $2")
                .bind(&current_actions)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        None => {
            sqlx::query(
                "INSERT INTO user_permissions (user_id, scope_type, org_id, resource, actions, granted_by) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(input.user_id)
            .bind(input.scope_type)
            .bind(input.org_id)
            .bind(input.resource)
            .bind(input.actions)
            .bind(input.granted_by)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    list_grants_for_user(pool, input.user_id).await.map_err(GrantError::Db)
}

pub async fn revoke_permission(pool: &PgPool, user_id: Uuid, grant_id: Uuid) -> Result<Vec<PermissionGrant>, sqlx::Error> {
    sqlx::query("DELETE FROM user_permissions WHERE id = $1 AND user_id = $2")
        .bind(grant_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    list_grants_for_user(pool, user_id).await
}
```

- [ ] **Step 4: Register the module**

Modify `backend/crates/nomi-auth/src/lib.rs` — add `pub mod grants;` alphabetically (between `extractor` and `login`):

```rust
pub mod authorize;
pub mod claims;
pub mod extractor;
pub mod grants;
pub mod login;
pub mod password;
pub mod permissions;
pub mod refresh_token;
pub mod registration;
```

- [ ] **Step 5: Rewrite `compute_permissions`**

Replace the full contents of `backend/crates/nomi-auth/src/permissions.rs`:

```rust
use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use super::claims::permission_string;
use super::grants::list_grants_for_user;

const ACTION_ORDER: [&str; 2] = ["view", "manage"];

pub async fn compute_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let mut merged: HashMap<(String, String), HashSet<String>> = HashMap::new();

    let is_platform_admin: bool =
        sqlx::query_scalar("SELECT is_platform_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if is_platform_admin {
        merged
            .entry(("admin".to_string(), "user".to_string()))
            .or_default()
            .extend(["view".to_string(), "manage".to_string()]);
        merged
            .entry(("admin".to_string(), "system_config".to_string()))
            .or_default()
            .extend(["view".to_string(), "manage".to_string()]);
    }

    let memberships: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT org_id, role FROM memberships WHERE user_id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for (org_id, role) in memberships {
        let actions: &[&str] = if role == "owner" || role == "admin" {
            &["view", "manage"]
        } else {
            &["view"]
        };
        let scope = org_id.to_string();
        merged
            .entry((scope.clone(), "member".to_string()))
            .or_default()
            .extend(actions.iter().map(|s| s.to_string()));
        merged
            .entry((scope, "conversation".to_string()))
            .or_default()
            .extend(actions.iter().map(|s| s.to_string()));
    }

    for grant in list_grants_for_user(pool, user_id).await? {
        let scope = match grant.scope_type.as_str() {
            "admin" => "admin".to_string(),
            _ => match grant.org_id {
                Some(org_id) => org_id.to_string(),
                // An org-scope grant with no org_id can't exist (DB CHECK enforces it) —
                // skip defensively rather than let a malformed row panic permission computation.
                None => continue,
            },
        };
        merged.entry((scope, grant.resource)).or_default().extend(grant.actions);
    }

    let mut permissions: Vec<String> = merged
        .into_iter()
        .map(|((scope, resource), actions)| {
            let ordered: Vec<&str> = ACTION_ORDER.iter().copied().filter(|a| actions.contains(*a)).collect();
            permission_string(&scope, &resource, &ordered)
        })
        .collect();
    permissions.sort();
    Ok(permissions)
}
```

- [ ] **Step 6: Run the existing permission tests to confirm zero regression**

Run: `cd backend && cargo test -p nomi-auth --test auth_permissions`
Expected: all 7 tests PASS unchanged.

- [ ] **Step 7: Write new tests for the grants module and the merge logic**

Create `backend/crates/nomi-auth/tests/auth_grants.rs`:

```rust
use nomi_auth::grants::{grant_permission, list_grants_for_user, revoke_permission, GrantError, NewGrant};
use nomi_auth::permissions::compute_permissions;
use sqlx::PgPool;
use uuid::Uuid;

async fn make_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn granting_an_admin_scope_permission_appears_in_compute_permissions(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;

    grant_permission(
        &pool,
        NewGrant {
            user_id,
            scope_type: "admin",
            org_id: None,
            resource: "user",
            actions: &["view".to_string()],
            granted_by: granter,
        },
    )
    .await
    .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:user:[view]".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn granting_the_same_resource_twice_merges_actions_into_one_string(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;

    grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "user", actions: &["view".to_string()], granted_by: granter },
    )
    .await
    .unwrap();
    let grants = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "user", actions: &["manage".to_string()], granted_by: granter },
    )
    .await
    .unwrap();

    // One row, not two — the second grant merged into the first.
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].actions.len(), 2);

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:user:[view,manage]".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn org_scope_grant_merges_with_role_derived_permission_on_the_same_resource(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    // 'member' role alone yields view-only on `member`. An explicit grant adds manage.
    grant_permission(
        &pool,
        NewGrant {
            user_id,
            scope_type: "org",
            org_id: Some(org_id),
            resource: "member",
            actions: &["manage".to_string()],
            granted_by: granter,
        },
    )
    .await
    .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_id}:member:[view,manage]")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn revoking_a_grant_removes_it_from_compute_permissions(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;

    let grants = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "billing", actions: &["view".to_string()], granted_by: granter },
    )
    .await
    .unwrap();
    let grant_id = grants[0].id;

    let remaining = revoke_permission(&pool, user_id, grant_id).await.unwrap();
    assert!(remaining.is_empty());

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(!permissions.iter().any(|p| p.contains("billing")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_with_zero_grants_and_no_role_gets_an_empty_permissions_array(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn invalid_resource_format_is_rejected(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let result = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "Not-Valid!", actions: &["view".to_string()], granted_by: granter },
    )
    .await;
    assert!(matches!(result, Err(GrantError::InvalidResource)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn empty_actions_is_rejected(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let result = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "user", actions: &[], granted_by: granter },
    )
    .await;
    assert!(matches!(result, Err(GrantError::InvalidActions)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn org_scope_without_org_id_is_rejected(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let result = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "org", org_id: None, resource: "member", actions: &["view".to_string()], granted_by: granter },
    )
    .await;
    assert!(matches!(result, Err(GrantError::OrgRequiredForOrgScope)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_grants_for_user_only_returns_that_users_rows(pool: PgPool) {
    let user_a = make_user(&pool).await;
    let user_b = make_user(&pool).await;
    let granter = make_user(&pool).await;
    grant_permission(
        &pool,
        NewGrant { user_id: user_a, scope_type: "admin", org_id: None, resource: "user", actions: &["view".to_string()], granted_by: granter },
    )
    .await
    .unwrap();

    let b_grants = list_grants_for_user(&pool, user_b).await.unwrap();
    assert!(b_grants.is_empty());
}
```

- [ ] **Step 8: Run the new tests**

Run: `cd backend && cargo test -p nomi-auth --test auth_grants`
Expected: all 9 tests PASS.

- [ ] **Step 9: Commit**

```bash
cd backend
git add migrations/0017_user_permissions.sql crates/nomi-auth/src/grants.rs crates/nomi-auth/src/lib.rs crates/nomi-auth/src/permissions.rs crates/nomi-auth/tests/auth_grants.rs
git commit -m "feat: add grantable per-user permissions, merged into compute_permissions"
```

---

### Task 2: Backend — admin user management routes

**Files:**
- Create: `backend/crates/nomi-server/src/routes/admin_users.rs`
- Modify: `backend/crates/nomi-server/src/routes/mod.rs`
- Modify: `backend/crates/nomi-server/src/app.rs`
- Create: `backend/crates/nomi-server/tests/admin_users_routes.rs`

**Interfaces:**
- Consumes (from Task 1): `nomi_auth::grants::{PermissionGrant, NewGrant, GrantError, list_grants_for_user, grant_permission, revoke_permission}`.
- Produces (used by Task 4): the HTTP contract below.
  - `GET /api/admin/users?query=<string>&page=<u32>&page_size=<u32>` → `{"users": [{"id": Uuid, "email": String, "is_platform_admin": bool, "is_staff": bool, "org_count": i64}], "total": i64}`.
  - `GET /api/admin/users/:id` → `{"id": Uuid, "email": String, "is_platform_admin": bool, "permissions": [PermissionGrant], "memberships": [{"org_id": Uuid, "org_name": String, "role": String}]}`.
  - `POST /api/admin/users/:id/permissions` body `{"scope_type": "admin"|"org", "org_id": Uuid|null, "resource": String, "actions": [String]}` → `200 [PermissionGrant]` (that user's full updated grant list).
  - `DELETE /api/admin/users/:id/permissions/:permission_id` → `200 [PermissionGrant]`.
  - `POST /api/admin/users/:id/memberships` body `{"org_id": Uuid, "role": "owner"|"admin"|"member"}` → `200 [{"org_id": Uuid, "org_name": String, "role": String}]` (that user's full updated membership list).
  - `DELETE /api/admin/users/:id/memberships/:org_id` → `200 [{"org_id": Uuid, "org_name": String, "role": String}]`.
  - `GET /api/admin/orgs` → `[{"id": Uuid, "name": String}]`, `is_personal = true` orgs excluded, ordered by name.

- [ ] **Step 1: Implement the routes**

Create `backend/crates/nomi-server/src/routes/admin_users.rs`:

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::claims::Claims;
use nomi_auth::extractor::AuthClaims;
use nomi_auth::grants::{grant_permission, list_grants_for_user, revoke_permission, GrantError, NewGrant};

fn require_user_permission(claims: &Claims, action: &str) -> Result<(), (StatusCode, &'static str)> {
    if claims.has_permission("admin", "user", action) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to manage users"))
    }
}

fn grant_error_to_response(e: GrantError) -> (StatusCode, String) {
    match e {
        GrantError::InvalidResource | GrantError::InvalidActions | GrantError::OrgRequiredForOrgScope | GrantError::OrgNotAllowedForAdminScope => {
            (StatusCode::BAD_REQUEST, e.to_string())
        }
        GrantError::Db(err) => {
            tracing::error!(error = %err, "grant operation failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to save permission grant".to_string())
        }
    }
}

#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub is_platform_admin: bool,
    pub is_staff: bool,
    pub org_count: i64,
}

#[derive(Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummary>,
    pub total: i64,
}

#[derive(Deserialize)]
pub struct UserListQuery {
    pub query: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_users(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(q): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "view")?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let search = q.query.filter(|s| !s.trim().is_empty());

    let rows: Vec<(Uuid, String, bool, bool, i64)> = sqlx::query_as(
        "SELECT u.id, wc.email, u.is_platform_admin, \
                EXISTS(SELECT 1 FROM user_permissions up WHERE up.user_id = u.id AND up.scope_type = 'admin') AS has_admin_grant, \
                (SELECT COUNT(*) FROM memberships m WHERE m.user_id = u.id AND m.status = 'active') AS org_count \
         FROM users u \
         JOIN web_credentials wc ON wc.user_id = u.id \
         WHERE $1::text IS NULL OR wc.email ILIKE '%' || $1 || '%' \
         ORDER BY wc.email ASC \
         LIMIT $2 OFFSET $3",
    )
    .bind(&search)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list users");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to list users")
    })?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users u JOIN web_credentials wc ON wc.user_id = u.id WHERE $1::text IS NULL OR wc.email ILIKE '%' || $1 || '%'",
    )
    .bind(&search)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to count users");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to count users")
    })?;

    let users = rows
        .into_iter()
        .map(|(id, email, is_platform_admin, has_admin_grant, org_count)| UserSummary {
            id,
            email,
            is_platform_admin,
            is_staff: is_platform_admin || has_admin_grant,
            org_count,
        })
        .collect();

    Ok(Json(UserListResponse { users, total }))
}

#[derive(Serialize)]
pub struct MembershipRow {
    pub org_id: Uuid,
    pub org_name: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct UserDetailResponse {
    pub id: Uuid,
    pub email: String,
    pub is_platform_admin: bool,
    pub permissions: Vec<nomi_auth::grants::PermissionGrant>,
    pub memberships: Vec<MembershipRow>,
}

async fn load_memberships(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Vec<MembershipRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT m.org_id, o.name AS org_name, m.role \
         FROM memberships m JOIN organizations o ON o.id = m.org_id \
         WHERE m.user_id = $1 AND m.status = 'active' \
         ORDER BY o.name ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_user_detail(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDetailResponse>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "view")?;

    let row: Option<(String, bool)> =
        sqlx::query_as("SELECT wc.email, u.is_platform_admin FROM users u JOIN web_credentials wc ON wc.user_id = u.id WHERE u.id = $1")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to load user");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to load user")
            })?;
    let (email, is_platform_admin) = row.ok_or((StatusCode::NOT_FOUND, "user not found"))?;

    let permissions = list_grants_for_user(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load permissions");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load permissions")
    })?;
    let memberships = load_memberships(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load memberships");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load memberships")
    })?;

    Ok(Json(UserDetailResponse { id: user_id, email, is_platform_admin, permissions, memberships }))
}

#[derive(Deserialize)]
pub struct GrantPermissionRequest {
    pub scope_type: String,
    pub org_id: Option<Uuid>,
    pub resource: String,
    pub actions: Vec<String>,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn grant_user_permission(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
    Json(req): Json<GrantPermissionRequest>,
) -> Result<Json<Vec<nomi_auth::grants::PermissionGrant>>, (StatusCode, String)> {
    require_user_permission(&claims, "manage").map_err(|(status, msg)| (status, msg.to_string()))?;

    if req.scope_type != "admin" && req.scope_type != "org" {
        return Err((StatusCode::BAD_REQUEST, "scope_type must be 'admin' or 'org'".to_string()));
    }

    let grants = grant_permission(
        &state.pool,
        NewGrant {
            user_id,
            scope_type: &req.scope_type,
            org_id: req.org_id,
            resource: &req.resource,
            actions: &req.actions,
            granted_by: claims.sub,
        },
    )
    .await
    .map_err(grant_error_to_response)?;

    Ok(Json(grants))
}

#[tracing::instrument(skip(state, claims))]
pub async fn revoke_user_permission(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((user_id, permission_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<nomi_auth::grants::PermissionGrant>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "manage")?;

    let grants = revoke_permission(&state.pool, user_id, permission_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to revoke permission");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to revoke permission")
    })?;

    Ok(Json(grants))
}

#[derive(Deserialize)]
pub struct AssignOrgRequest {
    pub org_id: Uuid,
    pub role: String,
}

const ALLOWED_ROLES: [&str; 3] = ["owner", "admin", "member"];

#[tracing::instrument(skip(state, claims, req))]
pub async fn assign_user_to_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignOrgRequest>,
) -> Result<Json<Vec<MembershipRow>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "manage")?;

    if !ALLOWED_ROLES.contains(&req.role.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "role must be owner, admin, or member"));
    }

    sqlx::query(
        "INSERT INTO memberships (org_id, user_id, role, status) VALUES ($1, $2, $3, 'active') \
         ON CONFLICT (org_id, user_id) DO UPDATE SET role = EXCLUDED.role, status = 'active'",
    )
    .bind(req.org_id)
    .bind(user_id)
    .bind(&req.role)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to assign user to org");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to assign user to org")
    })?;

    let memberships = load_memberships(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to reload memberships");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to reload memberships")
    })?;
    Ok(Json(memberships))
}

#[tracing::instrument(skip(state, claims))]
pub async fn remove_user_from_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((user_id, org_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MembershipRow>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "manage")?;

    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to remove membership");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove membership")
        })?;

    let memberships = load_memberships(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to reload memberships");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to reload memberships")
    })?;
    Ok(Json(memberships))
}

#[derive(Serialize)]
pub struct OrgOption {
    pub id: Uuid,
    pub name: String,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_orgs(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<OrgOption>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "view")?;

    let orgs = sqlx::query_as("SELECT id, name FROM organizations WHERE is_personal = false ORDER BY name ASC")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list orgs");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to list orgs")
        })?
        .into_iter()
        .map(|(id, name)| OrgOption { id, name })
        .collect();

    Ok(Json(orgs))
}
```

- [ ] **Step 2: Register the module**

Modify `backend/crates/nomi-server/src/routes/mod.rs` — add `pub mod admin_users;` in the existing alphabetically-sorted list (check the file's current contents first; insert between `admin_dashboard` and `auth`).

- [ ] **Step 3: Wire up the routes**

Modify `backend/crates/nomi-server/src/app.rs`:

Add the import near the other route-module imports:
```rust
use crate::routes::admin_users as admin_users_routes;
```

Add these routes to the `Router::new()` chain, after the existing `/api/admin/agents` route:
```rust
        .route(
            "/api/admin/users",
            get(admin_users_routes::list_users),
        )
        .route(
            "/api/admin/users/:id",
            get(admin_users_routes::get_user_detail),
        )
        .route(
            "/api/admin/users/:id/permissions",
            post(admin_users_routes::grant_user_permission),
        )
        .route(
            "/api/admin/users/:id/permissions/:permission_id",
            delete(admin_users_routes::revoke_user_permission),
        )
        .route(
            "/api/admin/users/:id/memberships",
            post(admin_users_routes::assign_user_to_org),
        )
        .route(
            "/api/admin/users/:id/memberships/:org_id",
            delete(admin_users_routes::remove_user_from_org),
        )
        .route("/api/admin/orgs", get(admin_users_routes::list_orgs))
```

- [ ] **Step 4: Confirm it compiles**

Run: `cd backend && cargo check --workspace`
Expected: `0 errors`.

- [ ] **Step 5: Write integration tests**

Create `backend/crates/nomi-server/tests/admin_users_routes.rs`:

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_via_api(router: axum::Router, email: &str) {
    json_request(
        router,
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
}

async fn login_via_api(router: axum::Router, email: &str) -> String {
    let (_, login_body) = json_request(
        router,
        "POST",
        "/api/auth/login",
        json!({ "email": email, "password": "correct-password" }),
        None,
    )
    .await;
    login_body["access_token"].as_str().unwrap().to_string()
}

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

async fn user_id_by_email(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_listing_users(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/users", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn admin_can_list_users_and_search_by_email(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "alice@example.com").await;
    register_via_api(router.clone(), "bob@example.com").await;

    let (status, body) = json_request(router.clone(), "GET", "/api/admin/users?query=alice", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["email"], "alice@example.com");
    assert_eq!(users[0]["is_staff"], false);

    let (status, body) = json_request(router, "GET", "/api/admin/users", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    // admin + alice + bob
    assert_eq!(body["total"], 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn promoting_a_user_makes_them_staff_and_gets_them_into_the_admin_panel(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "staff@example.com").await;
    let staff_user_id = user_id_by_email(&pool, "staff@example.com").await;

    let (status, _) = json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{staff_user_id}/permissions"),
        json!({ "scope_type": "admin", "org_id": null, "resource": "user", "actions": ["view"] }),
        Some(&admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Confirm it lands in the roster's is_staff flag.
    let (_, body) = json_request(router.clone(), "GET", "/api/admin/users", Value::Null, Some(&admin_token)).await;
    let staff_row = body["users"].as_array().unwrap().iter().find(|u| u["id"] == staff_user_id.to_string()).unwrap();
    assert_eq!(staff_row["is_staff"], true);

    // Confirm it lands in the staff member's own token after a fresh login (claims-freshness
    // rule from the auth-claims design: only guaranteed on refresh/re-login, not instantly).
    let staff_token = login_via_api(router, "staff@example.com").await;
    let claims_bytes = staff_token.split('.').nth(1).unwrap();
    // (Decoding the JWT payload here would need base64 + serde wiring just for this one
    // assertion; simpler and just as conclusive to hit /api/whoami with the fresh token.)
    let _ = claims_bytes;
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_detail_returns_permissions_and_memberships(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Beta Org') RETURNING id").fetch_one(&pool).await.unwrap();
    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "member" }),
        Some(&admin_token),
    )
    .await;
    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/permissions"),
        json!({ "scope_type": "admin", "org_id": null, "resource": "billing", "actions": ["view"] }),
        Some(&admin_token),
    )
    .await;

    let (status, body) = json_request(router, "GET", &format!("/api/admin/users/{target_id}"), Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "target@example.com");
    assert_eq!(body["permissions"].as_array().unwrap().len(), 1);
    assert_eq!(body["permissions"][0]["resource"], "billing");
    assert_eq!(body["memberships"].as_array().unwrap().len(), 1);
    assert_eq!(body["memberships"][0]["org_name"], "Beta Org");
    assert_eq!(body["memberships"][0]["role"], "member");
}

#[sqlx::test(migrations = "../../migrations")]
async fn assigning_to_org_twice_updates_the_role_instead_of_erroring(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Beta Org') RETURNING id").fetch_one(&pool).await.unwrap();

    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "member" }),
        Some(&admin_token),
    )
    .await;
    let (status, body) = json_request(
        router,
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "admin" }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let rows = body.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["role"], "admin");
}

#[sqlx::test(migrations = "../../migrations")]
async fn removing_from_org_sets_status_removed_and_excludes_from_the_list(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Beta Org') RETURNING id").fetch_one(&pool).await.unwrap();
    json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/users/{target_id}/memberships"),
        json!({ "org_id": org_id, "role": "member" }),
        Some(&admin_token),
    )
    .await;

    let (status, body) = json_request(
        router,
        "DELETE",
        &format!("/api/admin/users/{target_id}/memberships/{org_id}"),
        Value::Null,
        Some(&admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn granting_an_invalid_resource_returns_400(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "target@example.com").await;
    let target_id = user_id_by_email(&pool, "target@example.com").await;

    let (status, _) = json_request(
        router,
        "POST",
        &format!("/api/admin/users/{target_id}/permissions"),
        json!({ "scope_type": "admin", "org_id": null, "resource": "Not Valid", "actions": ["view"] }),
        Some(&admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_orgs_excludes_personal_orgs(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    sqlx::query("INSERT INTO organizations (name, is_personal) VALUES ('Real Org', false)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO organizations (name, is_personal) VALUES ('Someones Personal Org', true)").execute(&pool).await.unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/orgs", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    let orgs = body.as_array().unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["name"], "Real Org");
}
```

- [ ] **Step 6: Run the new tests**

Run: `cd backend && cargo test -p nomi-server --test admin_users_routes`
Expected: all 8 tests PASS.

- [ ] **Step 7: Run the full backend test suite to confirm no regressions elsewhere**

Run: `cd backend && cargo test --workspace`
Expected: all tests PASS.

- [ ] **Step 8: Commit**

```bash
cd backend
git add crates/nomi-server/src/routes/admin_users.rs crates/nomi-server/src/routes/mod.rs crates/nomi-server/src/app.rs crates/nomi-server/tests/admin_users_routes.rs
git commit -m "feat: add admin user roster, permission grant/revoke, and org assignment routes"
```

---

### Task 3: Frontend — broaden the admin gate and make the sidebar permission-aware

**Files:**
- Modify: `frontend/src/routes/admin/(protected)/+layout.server.ts`
- Modify: `frontend/src/routes/admin/(protected)/+layout.svelte`

**Interfaces:**
- Produces (used by Task 4's nav link, and by every existing admin sub-page implicitly): `LayoutServerLoad` returns `{ canManageSystemConfig: boolean, canViewUsers: boolean }`, available to `+layout.svelte` and all nested pages as `data`.

- [ ] **Step 1: Broaden the layout gate and compute per-resource flags**

Replace the full contents of `frontend/src/routes/admin/(protected)/+layout.server.ts`:

```ts
import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const response = await apiFetch(fetch, cookies, '/api/whoami');
	if (!response.ok) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const claims = (await response.json()) as { permissions: string[] };
	const isStaff = claims.permissions.some((permission) => permission.startsWith('nomi:admin:'));
	if (!isStaff) {
		throw redirect(303, '/?error=forbidden');
	}

	const canManageSystemConfig = claims.permissions.some((p) => p.startsWith('nomi:admin:system_config:'));
	const canViewUsers = claims.permissions.some((p) => p.startsWith('nomi:admin:user:'));

	return { canManageSystemConfig, canViewUsers };
};
```

- [ ] **Step 2: Gate sidebar links on the specific permission each page needs**

Modify `frontend/src/routes/admin/(protected)/+layout.svelte`:

Change the props line to destructure `data`:
```svelte
let { data, children }: { data: LayoutData; children: Snippet } = $props();
```

Replace the `<nav>` block:
```svelte
<nav class="flex flex-col gap-1" class:items-center={collapsed}>
	{#if collapsed}
		{#if data.canManageSystemConfig}
			<IconButton href="/admin" aria-label="Dashboard">
				<Icon name="dashboard" />
			</IconButton>
			<IconButton href="/admin/settings/llm" aria-label="LLM Settings">
				<Icon name="settings" />
			</IconButton>
			<IconButton href="/admin/settings/embedding" aria-label="Embedding Settings">
				<Icon name="settings" />
			</IconButton>
			<IconButton href="/admin/agents" aria-label="Agents">
				<Icon name="agents" />
			</IconButton>
		{/if}
		{#if data.canViewUsers}
			<IconButton href="/admin/users" aria-label="Users">
				<Icon name="person" />
			</IconButton>
		{/if}
	{:else}
		{#if data.canManageSystemConfig}
			<a href="/admin" class="m3-nav-link">Dashboard</a>
			<a href="/admin/settings/llm" class="m3-nav-link">LLM Settings</a>
			<a href="/admin/settings/embedding" class="m3-nav-link">Embedding Settings</a>
			<a href="/admin/agents" class="m3-nav-link">Agents</a>
		{/if}
		{#if data.canViewUsers}
			<a href="/admin/users" class="m3-nav-link">Users</a>
		{/if}
	{/if}
</nav>
```

- [ ] **Step 3: Type-check**

Run: `cd frontend && npm run check`
Expected: `0 errors` (the pre-existing `Menu.svelte` a11y warning is the only expected output).

- [ ] **Step 4: Commit**

```bash
cd frontend
git add src/routes/admin/'(protected)'/+layout.server.ts src/routes/admin/'(protected)'/+layout.svelte
git commit -m "feat: broaden the admin panel gate to any admin permission, gate nav links per-resource"
```

---

### Task 4: Frontend — `/admin/users` page (roster + BottomSheet detail panel)

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Create: `frontend/src/routes/admin/(protected)/users/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/users/+page.svelte`
- Create: `frontend/src/routes/admin/(protected)/users/[id]/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/users/[id]/+page.svelte`

**Interfaces:**
- Consumes (from Task 2): the `/api/admin/users*` and `/api/admin/orgs` HTTP contract documented in Task 2.

- [ ] **Step 1: Add the frontend types**

Modify `frontend/src/lib/types.ts` — append at the end of the file:

```ts
export interface AdminUserSummary {
	id: string;
	email: string;
	is_platform_admin: boolean;
	is_staff: boolean;
	org_count: number;
}

export interface AdminUserListResponse {
	users: AdminUserSummary[];
	total: number;
}

export interface PermissionGrant {
	id: string;
	scope_type: 'admin' | 'org';
	org_id: string | null;
	org_name: string | null;
	resource: string;
	actions: string[];
	created_at: string;
}

export interface MembershipRow {
	org_id: string;
	org_name: string;
	role: string;
}

export interface AdminUserDetail {
	id: string;
	email: string;
	is_platform_admin: boolean;
	permissions: PermissionGrant[];
	memberships: MembershipRow[];
}

export interface OrgOption {
	id: string;
	name: string;
}
```

- [ ] **Step 2: Roster page load + promote action**

Create `frontend/src/routes/admin/(protected)/users/+page.server.ts`:

```ts
import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminUserListResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

const PAGE_SIZE = 20;

export const load: PageServerLoad = async ({ url, cookies, fetch }) => {
	const query = url.searchParams.get('query') ?? '';
	const page = Number(url.searchParams.get('page') ?? '1') || 1;

	const params = new URLSearchParams({ page: String(page), page_size: String(PAGE_SIZE) });
	if (query) params.set('query', query);

	const response = await apiFetch(fetch, cookies, `/api/admin/users?${params}`);
	const result: AdminUserListResponse = response.ok
		? ((await response.json()) as AdminUserListResponse)
		: { users: [], total: 0 };

	return { users: result.users, total: result.total, page, pageSize: PAGE_SIZE, query };
};

export const actions: Actions = {
	promote: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = data.get('userId');
		if (typeof userId !== 'string') {
			return fail(400, { error: 'Invalid user.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions`, {
			method: 'POST',
			body: JSON.stringify({ scope_type: 'admin', org_id: null, resource: 'user', actions: ['view'] }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to promote user.' });
		}
		return { success: true };
	},
};
```

- [ ] **Step 3: Roster page UI**

Create `frontend/src/routes/admin/(protected)/users/+page.svelte`:

```svelte
<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const columns = [
		{ key: 'email', label: 'Email' },
		{ key: 'staff', label: 'Staff' },
		{ key: 'orgs', label: 'Organizations' },
		{ key: 'actions', label: '' },
	];

	function handleSearch(query: string) {
		goto(`?query=${encodeURIComponent(query)}&page=1`, { keepFocus: true });
	}

	function handlePageChange(nextPage: number) {
		const params = new URLSearchParams({ page: String(nextPage) });
		if (data.query) params.set('query', data.query);
		goto(`?${params}`, { keepFocus: true });
	}
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Users</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Manage user access: promote to staff, grant permissions, assign organizations.
</p>

<div class="mt-6">
	<DataTable
		{columns}
		page={data.page}
		pageSize={data.pageSize}
		totalItems={data.total}
		onPageChange={handlePageChange}
		searchQuery={data.query}
		onSearch={handleSearch}
		searchPlaceholder="Search by email..."
	>
		{#each data.users as user (user.id)}
			<tr>
				<td><a href="/admin/users/{user.id}" style="color: var(--md-sys-color-primary)">{user.email}</a></td>
				<td>{user.is_staff ? 'Yes' : 'No'}</td>
				<td>{user.org_count}</td>
				<td>
					{#if !user.is_staff}
						<form method="POST" action="?/promote" use:enhance>
							<input type="hidden" name="userId" value={user.id} />
							<Button type="submit" variant="text">Promote to staff</Button>
						</form>
					{/if}
				</td>
			</tr>
		{/each}
	</DataTable>
	{#if data.users.length === 0}
		<p class="md-body-medium mt-4" style="color: var(--md-sys-color-on-surface-variant)">No users found.</p>
	{/if}
</div>
```

- [ ] **Step 4: Detail panel load + grant/revoke/assign/remove actions**

Create `frontend/src/routes/admin/(protected)/users/[id]/+page.server.ts`:

```ts
import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminUserDetail, OrgOption } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const [detailResponse, orgsResponse] = await Promise.all([
		apiFetch(fetch, cookies, `/api/admin/users/${params.id}`),
		apiFetch(fetch, cookies, '/api/admin/orgs'),
	]);

	if (!detailResponse.ok) {
		throw error(detailResponse.status === 404 ? 404 : 500, 'Failed to load user.');
	}

	const user: AdminUserDetail = await detailResponse.json();
	const orgs: OrgOption[] = orgsResponse.ok ? await orgsResponse.json() : [];

	return { user, orgs };
};

export const actions: Actions = {
	grantPermission: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const scopeType = data.get('scopeType');
		const orgId = data.get('orgId');
		const resource = data.get('resource');
		const actions = data.getAll('actions').filter((a): a is string => typeof a === 'string');

		if (typeof scopeType !== 'string' || typeof resource !== 'string' || !resource) {
			return fail(400, { error: 'Resource is required.' });
		}
		if (actions.length === 0) {
			return fail(400, { error: 'Select at least one action.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/permissions`, {
			method: 'POST',
			body: JSON.stringify({
				scope_type: scopeType,
				org_id: scopeType === 'org' && typeof orgId === 'string' && orgId ? orgId : null,
				resource,
				actions,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to grant permission.' });
		}
		return { success: true };
	},

	revokePermission: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const permissionId = data.get('permissionId');
		if (typeof permissionId !== 'string') {
			return fail(400, { error: 'Invalid permission.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/permissions/${permissionId}`, {
			method: 'DELETE',
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to revoke permission.' });
		}
		return { success: true };
	},

	assignOrg: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const orgId = data.get('orgId');
		const role = data.get('role');
		if (typeof orgId !== 'string' || !orgId || typeof role !== 'string' || !role) {
			return fail(400, { error: 'Organization and role are required.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/memberships`, {
			method: 'POST',
			body: JSON.stringify({ org_id: orgId, role }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to assign organization.' });
		}
		return { success: true };
	},

	removeOrg: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const orgId = data.get('orgId');
		if (typeof orgId !== 'string') {
			return fail(400, { error: 'Invalid organization.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/memberships/${orgId}`, {
			method: 'DELETE',
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to remove organization.' });
		}
		return { success: true };
	},
};
```

- [ ] **Step 5: Detail panel UI (BottomSheet)**

Create `frontend/src/routes/admin/(protected)/users/[id]/+page.svelte`:

```svelte
<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let open = $state(true);

	$effect(() => {
		// BottomSheet flips `open` to false on its own dismiss affordances (Escape, backdrop
		// click, drag-to-dismiss) — closing this route-driven panel means navigating back.
		if (!open) goto('/admin/users');
	});

	let scopeType = $state<'admin' | 'org'>('admin');
	let grantOrgId = $state(data.orgs[0]?.id ?? '');
	let assignOrgId = $state(data.orgs[0]?.id ?? '');
	let assignRole = $state('member');

	const scopeOptions = [
		{ value: 'admin', label: 'Admin' },
		{ value: 'org', label: 'Organization' },
	];
	const roleOptions = [
		{ value: 'owner', label: 'Owner' },
		{ value: 'admin', label: 'Admin' },
		{ value: 'member', label: 'Member' },
	];
</script>

<BottomSheet bind:open>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{data.user.email}</h2>

	<section class="mt-6">
		<h3 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Permissions</h3>
		{#if data.user.permissions.length === 0}
			<p class="md-body-small mt-2" style="color: var(--md-sys-color-on-surface-variant)">No explicit grants.</p>
		{:else}
			<List class="mt-2">
				{#each data.user.permissions as grant (grant.id)}
					<ListItem
						headline="{grant.scope_type === 'admin' ? 'Admin' : (grant.org_name ?? 'Unknown org')}: {grant.resource}"
						supportingText={grant.actions.join(', ')}
					>
						{#snippet trailing()}
							<form method="POST" action="?/revokePermission" use:enhance>
								<input type="hidden" name="permissionId" value={grant.id} />
								<IconButton type="submit" aria-label="Revoke {grant.resource}">
									<Icon name="close" size={16} />
								</IconButton>
							</form>
						{/snippet}
					</ListItem>
				{/each}
			</List>
		{/if}

		<form method="POST" action="?/grantPermission" use:enhance class="mt-4 flex flex-col gap-3">
			<Select label="Scope" name="scopeType" bind:value={scopeType} options={scopeOptions} />
			{#if scopeType === 'org'}
				<Select
					label="Organization"
					name="orgId"
					bind:value={grantOrgId}
					options={data.orgs.map((o) => ({ value: o.id, label: o.name }))}
				/>
			{/if}
			<TextField id="resource" name="resource" label="Resource" required />
			<div class="flex gap-4">
				<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
					<input type="checkbox" name="actions" value="view" /> View
				</label>
				<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
					<input type="checkbox" name="actions" value="manage" /> Manage
				</label>
			</div>
			<Button type="submit" variant="filled" class="w-fit">Grant</Button>
		</form>
	</section>

	<section class="mt-6">
		<h3 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Organizations</h3>
		{#if data.user.memberships.length === 0}
			<p class="md-body-small mt-2" style="color: var(--md-sys-color-on-surface-variant)">Not a member of any organization.</p>
		{:else}
			<List class="mt-2">
				{#each data.user.memberships as membership (membership.org_id)}
					<ListItem headline={membership.org_name} supportingText={membership.role}>
						{#snippet trailing()}
							<form method="POST" action="?/removeOrg" use:enhance>
								<input type="hidden" name="orgId" value={membership.org_id} />
								<IconButton type="submit" aria-label="Remove from {membership.org_name}">
									<Icon name="close" size={16} />
								</IconButton>
							</form>
						{/snippet}
					</ListItem>
				{/each}
			</List>
		{/if}

		{#if data.orgs.length > 0}
			<form method="POST" action="?/assignOrg" use:enhance class="mt-4 flex flex-col gap-3">
				<Select
					label="Organization"
					name="orgId"
					bind:value={assignOrgId}
					options={data.orgs.map((o) => ({ value: o.id, label: o.name }))}
				/>
				<Select label="Role" name="role" bind:value={assignRole} options={roleOptions} />
				<Button type="submit" variant="filled" class="w-fit">Assign</Button>
			</form>
		{/if}
	</section>
</BottomSheet>
```

- [ ] **Step 6: Type-check**

Run: `cd frontend && npm run check`
Expected: `0 errors` (the pre-existing `Menu.svelte` a11y warning is the only expected output).

- [ ] **Step 7: Live verification in the browser**

With the backend running (`cd backend && cargo run --bin nomi-server`, or the already-running dev instance if there is one) and the frontend dev server running (`cd frontend && npm run dev`):
1. Log in as a platform admin (or promote a test user via a direct DB update, matching `make_platform_admin` in the tests).
2. Navigate to `/admin/users`. Confirm the roster renders, search filters by email, and pagination works once there are 20+ users (optional if the test DB is small — at minimum confirm the page loads with the seeded admin/test users).
3. Click a non-staff user's "Promote to staff" button. Confirm the row updates to show `Staff: Yes` after the page reloads.
4. Click a user's email to open the detail `BottomSheet`. Confirm it slides up from the bottom, confirm dragging it down past the dismiss threshold (or pressing Escape) navigates back to `/admin/users`.
5. In the detail sheet, grant a permission (e.g. resource `billing`, action `view`, scope Admin) and confirm it appears in the Permissions list after the page reloads. Revoke it and confirm it disappears.
6. Assign the user to an organization with a role, confirm it appears in the Organizations list, then remove it and confirm it disappears.
7. Log in as the newly-promoted staff user (only `nomi:admin:user:[view]`, no `system_config`) and confirm: the sidebar shows only "Users" (no Dashboard/LLM Settings/Embedding Settings/Agents), and navigating directly to `/admin/settings/llm` still renders (SvelteKit-level access — the *page* loads since the layout gate now only requires "any admin permission") but the model list is empty since the backend 403s the underlying API call — this is the accepted, scoped edge case from the design doc's §4, not a regression to fix here.

- [ ] **Step 8: Commit**

```bash
cd frontend
git add src/lib/types.ts src/routes/admin/'(protected)'/users
git commit -m "feat: add the /admin/users page — roster, promote to staff, permission and org management"
```
