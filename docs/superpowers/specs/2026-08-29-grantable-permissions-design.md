# Grantable Permissions & Admin User Management

Date: 2026-08-29
Status: Approved (pending user review of this doc)
Related: `2026-07-25-multi-tenant-rbac-design.md` (extends its §2 permission model — resources move from a fixed MVP list to admin-grantable, free-text feature strings), `2026-07-26-auth-claims-layer-design.md` (extends its `compute_permissions`/Claims computation, §3), `2026-08-28-admin-dashboard-and-agents-design.md` (adds a new page to the same admin shell, reusing `require_system_config_permission`'s pattern)

## Purpose

Permission strings (`nomi:<scope>:<resource>:[actions]`) exist and are enforced, but the only way to grant one is hardcoded in `compute_permissions`: `is_platform_admin` (a single boolean) yields exactly two admin-scope strings, and `memberships.role` yields exactly two org-scope strings via a fixed role→action table. There is no way to give one specific user one specific permission — e.g. letting someone manage the user roster without making them a full platform admin, or letting a non-owner manage a specific org's LLM settings. This design adds a real grant mechanism (a stored table, merged into claims at login/refresh) and the admin UI to use it: a user roster, "promote to staff," per-user permission management, and assigning a user into an org with a role.

## 1. Schema

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
    CHECK ((scope_type = 'admin' AND org_id IS NULL) OR (scope_type = 'org' AND org_id IS NOT NULL)),
    UNIQUE (user_id, scope_type, org_id, resource)
);

CREATE INDEX user_permissions_user_id_idx ON user_permissions(user_id);
```

- `resource` is free text (per your answer) rather than a curated enum — an admin can grant `nomi:admin:whatever_feature:[view]` for a feature that doesn't exist yet. It's still format-validated (lowercase, starts with a letter, alphanumeric/underscore only) so it stays a clean identifier consistent with the existing `user`/`member`/`conversation`/`system_config` resources; nothing stops granting a resource string with no enforcement point behind it yet, same tradeoff the RBAC design already accepted for extensibility.
- `actions` stays closed to `view`/`manage` — this isn't the free-text axis; it's what `Claims::has_permission` and the existing bracket-list grammar already expect, and every current call site checks one of exactly these two.
- The `UNIQUE (user_id, scope_type, org_id, resource)` constraint means granting the same resource twice is an upsert (merge actions), not a duplicate row — see §2.
- Postgres treats `NULL <> NULL`, so the unique constraint does **not** actually collapse multiple `(user_id, 'admin', NULL, resource)` rows — two admin-scope grants for the same user+resource would both insert successfully. The grant endpoint (§4) is written as select-then-update-or-insert rather than relying on `ON CONFLICT` for the admin-scope case, precisely to route around this.

## 2. Permission Computation

`nomi-auth/src/permissions.rs`'s `compute_permissions(pool, user_id)` keeps its two existing branches completely unchanged (zero regression risk — every existing platform admin and every existing org role keeps exactly the permissions they have today) and unions in two new ones:

```rust
pub async fn compute_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let mut grants: HashMap<(ScopeKey, String), HashSet<Action>> = HashMap::new();

    // 1. existing: is_platform_admin -> admin:user, admin:system_config (unchanged)
    // 2. existing: memberships.role -> org:member, org:conversation (unchanged)
    // 3. new: explicit admin-scope grants from user_permissions
    // 4. new: explicit org-scope grants from user_permissions

    // merge into `grants`, unioning the action set for any (scope, resource)
    // pair that appears from more than one source, then format each entry
    // via the existing `permission_string(scope, resource, actions)`
}
```

Sources 3 and 4 are one query (`SELECT scope_type, org_id, resource, actions FROM user_permissions WHERE user_id = $1`), split into the two merge buckets by `scope_type`. Actions merge as a set union (e.g. a role-derived `view` plus an explicit grant of `manage` on the same resource yields `[view,manage]`, not two separate strings for the same resource — `Claims::has_permission` already expects at most one string per `(scope, resource)`).

This function is called in the same three places it already is (`login`, `register`, `POST /api/auth/refresh`), unchanged — the merge is invisible to every existing caller.

## 3. Backend API

New handlers in `backend/crates/nomi-server/src/routes/admin_users.rs`, all gated by `Claims::has_permission("admin", "user", ...)` — `view` for reads, `manage` for writes (the same two-tier pattern `system_config` already uses):

- **`GET /api/admin/users`** — roster, paginated/searchable by email, reusing the pagination/search query shape already built for the DataTable component's backing endpoints (LLM models, agents). Returns `{ id, email, is_platform_admin, permission_count, org_count }` per row — enough for the roster table without an N+1 fetch per row.
- **`GET /api/admin/users/:id`** — one user's full detail: every `user_permissions` row (formatted as `resource`/`actions`/`scope`) and every `memberships` row (`org_id`, `org_name`, `role`).
- **`POST /api/admin/users/:id/permissions`** — body `{ scope_type: "admin" | "org", org_id: Uuid | null, resource: string, actions: ["view" | "manage", ...] }`. Validates `resource` against the same format check as the DB constraint (defense in depth, and a cleaner 400 than a DB constraint violation), then select-then-upsert per §1's note (merge `actions` if the `(user_id, scope_type, org_id, resource)` row already exists). `granted_by` is set from the caller's `claims.sub`.
- **`DELETE /api/admin/users/:id/permissions/:permission_id`** — revoke one grant row outright.
- **`POST /api/admin/users/:id/memberships`** — body `{ org_id: Uuid, role: "owner" | "admin" | "member" }`. Creates (or updates the role on) a `memberships` row for that user — this is the "assign staff into an organization with a role" flow, and it's genuinely new: every existing way a `memberships` row gets created is self-service (registration, invite acceptance), never admin-initiated. Upserts on the existing `UNIQUE (org_id, user_id)` constraint.
- **`DELETE /api/admin/users/:id/memberships/:org_id`** — remove a user from an org (sets `status = 'removed'`, matching how the RBAC design already models membership lifecycle rather than hard-deleting the row).
- **`GET /api/admin/orgs`** — minimal `{ id, name }` list for the org picker in the "assign to organization" dialog. (`is_personal = true` orgs excluded, same rule the RBAC design already applies to the org switcher — a staff member should never be assignable into someone's auto-created personal org.)

Every mutating route re-derives the target user's `user_permissions` after the write and returns the updated set, so the frontend never has to guess whether a merge happened.

## 4. Admin Panel Access Gate

Today `admin/(protected)/+layout.server.ts` requires `nomi:admin:system_config:*` specifically — the single admin surface that existed before this feature. That's too narrow once a "staff" user can hold `nomi:admin:user:[view]` and nothing else: they'd be bounced at the front door.

- **Layout gate broadens** to "any `nomi:admin:*` permission" (`claims.permissions.some(p => p.startsWith('nomi:admin:'))`) — this is the front-door check only, gating whether `/admin`'s shell renders at all.
- **Each sub-page keeps its own specific check**, unchanged in spirit from how `system_config`-gated pages already work: LLM settings, embedding settings, and the agents list all still require `system_config` specifically (`require_system_config_permission` server-side, mirrored client-side). The new Users page (§5) requires `user` specifically (`view` to load the roster, `manage` to see/use the grant and org-assignment actions).
- A staff member with only `nomi:admin:user:[view]` now gets into `/admin`, sees the sidebar, but a click into LLM Settings still 403s server-side and the link itself is simply not rendered client-side — same "frontend reflects, backend is the boundary" rule the RBAC design already states in its §2.
- The dashboard's aggregate-count page stays gated on `system_config` as it is today (unchanged) — it surfaces platform-wide operational numbers (token usage, running agents), which is a `system_config`-shaped concern, not a `user`-shaped one.

## 5. Frontend: `/admin/users`

New page, added to the admin sidebar nav (`admin/(protected)/+layout.svelte`) alongside the existing four links, gated per §4.

- **Roster**: the existing `DataTable` component (pagination + search, already built), columns: email, staff status (derived: has any `nomi:admin:*` grant), org count. Row click opens the detail view.
- **Promote to staff** (roster row action, shown only for users with zero `nomi:admin:*` permissions): a single button, no dialog — grants `nomi:admin:user:[view]` immediately via §3's grant endpoint (minimal default: they can now get into `/admin` and see the roster, nothing else). Per your answer, this is deliberately a separate, later step from granting anything further.
- **User detail view** (its own panel, using the existing `BottomSheet` component rather than `Dialog` — consistent with this app's pattern of using bottom sheets for content-heavy, scrollable panels on both desktop and mobile widths): two sections.
  - *Permissions*: list of current `user_permissions` rows (scope, resource, actions) each with a revoke button, plus a small form to add a new one — scope toggle (Admin / Organization, with an org picker from `GET /api/admin/orgs` when Organization is selected), a free-text resource field, and view/manage checkboxes. This is where `system_config`, `manage` on `user`, or any other grant gets added after the initial "promote to staff" — the second of your two separate steps.
  - *Organizations*: list of current `memberships` rows (org name, role) each with a remove button, plus a small form (org picker + role dropdown) to assign the user into another org — this is the "assign into organization, role-based" flow, kept as a role picker against the existing `memberships.role` enum rather than a free-text grant, per the earlier design discussion.

## 6. Testing Approach

- **Unit** (`nomi-auth`): `compute_permissions` merge logic — an explicit admin-scope grant with no `is_platform_admin`; an explicit org-scope grant merging actions with a role-derived one on the same resource (e.g. role gives `view`, explicit grant adds `manage` → single string with both); a user with zero grants of any kind still gets an empty (not error) permissions array.
- **Integration** (`nomi-server`, real Postgres): granting a permission via `POST /api/admin/users/:id/permissions` and then confirming it appears in that user's *next* token (after `/api/auth/refresh`, per the auth-claims design's existing "refresh recomputes fresh" rule) — not the currently-held token, which is expected to lag until refresh/re-login, consistent with the existing claims-freshness tradeoff. Revoke removes it the same way. `POST /api/admin/users/:id/memberships` both creates a new row and updates the role on an existing one (upsert path). Every admin_users route rejects a caller lacking `nomi:admin:user:manage`/`:view` as appropriate, mirroring the existing `system_config` route tests.
- **Frontend**: `/admin` layout gate lets in a user with only `nomi:admin:user:[view]` (new, broadened case) but still blocks LLM Settings for that same user (per-page check still narrow) — the two halves of §4 need to be tested as separate assertions, since a regression could pass one and fail the other silently.

## Out of Scope

- Exposing `is_platform_admin` itself in this UI (toggling the boolean) — it stays a DB-level bootstrap flag, unchanged from today; this feature is additive (grantable permissions alongside it), not a replacement for it.
- Bulk permission operations (grant one resource to many users at once) — the roster is one-row-at-a-time for this iteration.
- Auditing/history of grant changes beyond `granted_by`/`created_at` on the row itself (no separate audit log table) — `granted_by` is enough to answer "who granted this" today; a full history-of-changes view is a future iteration if it's ever needed.
- Notifying a user when their permissions change — no email/in-app notification is sent; they simply see new capabilities on their next token refresh.
