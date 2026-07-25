# Auth & Claims Layer Design

Date: 2026-07-26
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-25-multi-tenant-rbac-design.md` (this doc implements its §2 permission model and closes its deferred login mechanism), `docs/superpowers/specs/2026-07-25-frontend-chat-design.md` (closes its "login/identity-linking flow" out-of-scope item), `docs/superpowers/specs/2026-07-22-cross-channel-identity-design.md`

## Purpose

The multi-tenant RBAC design specified a permission-string claims model but explicitly deferred *how someone logs in* — that gap now blocks building anything else, since claims can't be issued without a real authentication step. This document designs the actual login/registration flow, the JWT-based claims token, and the backend enforcement layer (axum extractors and authorization helpers) that every future protected endpoint will use.

## 1. Schema Additions

Two gaps in the existing schema (`backend/migrations/0001`-`0006`) block this work:

```sql
-- migration: add platform-admin flag
ALTER TABLE users ADD COLUMN is_platform_admin BOOLEAN NOT NULL DEFAULT false;

-- migration: web login credentials, mirroring the channel_identities pattern
-- (one canonical `users` row; per-method identity/credentials live in their own table)
CREATE TABLE web_credentials (
    user_id       UUID PRIMARY KEY REFERENCES users(id),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- migration: refresh tokens, hash-only (never store the raw token)
CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    token_hash  TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ
);
```

`is_platform_admin` is a boolean column rather than a special org, since platform-wide admin is a rare, ops-only capability (per the frontend design's separate `/admin` route) — not something that benefits from org-shaped modeling like membership tiers do.

## 2. Registration & Login Flow

- **`POST /api/auth/register`**: body `{ email, password, org: { mode: "create", name: string } | { mode: "join", invite_code: string } }`.
  - Hashes `password` with `argon2` (default parameters).
  - Creates the `users` row and the `web_credentials` row.
  - `mode: "create"`: creates a new `organizations` row (`is_personal = false`), and a `memberships` row with `role = 'owner'`.
  - `mode: "join"`: resolves `org_invites` by `invite_code` (must be unexpired, unused), creates a `memberships` row with the invite's `role`, marks the invite `used_at`.
  - On success, immediately performs the same claims computation as login (§3) and returns the token pair.
- **`POST /api/auth/login`**: body `{ email, password }`. Looks up `web_credentials` by `email`, verifies the password against `password_hash` via `argon2`, computes claims (§3), returns `{ access_token, refresh_token }`. Wrong email or wrong password both return the same generic `401 invalid credentials` — never reveal which one was wrong.
- **Channel-only users are out of scope for login.** A user who only exists via a Telegram/WhatsApp `channel_identity` (per the cross-channel-identity design's lazy bootstrap) has a `users` row and an auto-created personal org, but no `web_credentials` row, and cannot log into the web app. Attaching a channel identity to a newly-registered web account is a future cross-channel-linking UI feature (already noted as deferred in the cross-channel-identity design) and is not built here.

## 3. Claims & Token Design

- **Access token**: a JWT, signed HS256 with a server-side secret read from an environment variable, 30-minute expiry. Payload:
  ```json
  {
    "sub": "<user_id>",
    "active_org_id": "<org_id>",
    "permissions": ["nomi:admin:user:[view,manage]", "nomi:<org_id>:owner:[view,manage]", "..."],
    "exp": 1234567890,
    "iat": 1234567890
  }
  ```
  `permissions` contains **every** permission string the user currently holds — `nomi:admin:user:[view,manage]` if `is_platform_admin`, plus `nomi:<org_id>:<role>:[view,manage]` for every `active` `memberships` row, not just the currently-active org. This makes the token self-sufficient for any read/view authorization check across every org the user belongs to, with no DB round-trip mid-request.
- **`active_org_id` is UI-context only, not a security boundary.** It records which org's chat list the frontend should default to; it does not gate anything by itself. Authorization checks always key on the specific `org_id` named in the request path, checked against `permissions`, regardless of what `active_org_id` says.
- **Switching orgs**: `POST /api/auth/switch-org { org_id }` re-issues the access token with a new `active_org_id` and the same `permissions` array (membership didn't change, so no need to recompute it) — no re-login required. Fails with `403` if the target `org_id` isn't among the caller's memberships.
- **Refresh token**: an opaque random value (32+ bytes, base64-encoded); only its hash (e.g. SHA-256) is stored in `refresh_tokens`, with a 30-day expiry. `POST /api/auth/refresh { refresh_token }` looks up the hash, rejects if `revoked_at IS NOT NULL` or `expires_at <= now()`, otherwise **recomputes the permissions array fresh from current DB state** (so a role change or org removal since the last login/refresh takes effect here) and issues a new access token.
- **Logout**: `POST /api/auth/logout { refresh_token }` sets `revoked_at = now()` on that refresh token. This is the real revocation point — an already-issued access token can't be invalidated early and simply expires within 30 minutes, which is the accepted blast radius (matches the RBAC design's own stated tradeoff on claims freshness).
- **Transport**: both tokens returned in the JSON response body; the frontend sends the access token as `Authorization: Bearer <token>` on subsequent requests. Cookie-based storage (httpOnly, CSRF handling) is a frontend-integration decision deferred to that plan — it doesn't change anything about how tokens are issued or verified here.

## 4. Enforcement Layer (axum)

- **`Claims` extractor** (`impl FromRequestParts<S> for Claims`): reads the `Authorization: Bearer` header, verifies the JWT signature and `exp`, and deserializes into a `Claims { sub: Uuid, active_org_id: Uuid, permissions: Vec<String> }` struct. A handler simply adds `claims: Claims` to its argument list to require auth; a missing, malformed, or expired token yields `401` before the handler body runs.
- **Cheap check (read/view endpoints)**: `Claims::has_permission(&self, scope: &str, resource: &str, action: &str) -> bool` — parses each string in `permissions` (format `nomi:<scope>:<role-or-resource>:[actions]` per the RBAC design's grammar) and checks for a match, purely in-memory. Used for endpoints like "list sessions in org X" or the `/org/:id/members` route guard.
- **Re-validated check (sensitive writes)**: `authorize_org_action(pool: &PgPool, user_id: Uuid, org_id: Uuid, required_role: &[&str]) -> Result<(), AuthError>` queries the **current** `memberships` row directly (`SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active'`), ignoring the token's cached `permissions` entirely. Used for anything that mutates membership/role state — removing a member, changing a role, generating an org invite — so that a caller whose membership was just revoked can't keep performing sensitive actions for the remainder of their still-valid access token's life. This is the concrete mechanism behind the RBAC design's "sensitive writes re-check current DB state, not the token" rule.
- **Cross-org rejection**: both the cheap and re-validated checks key on the *specific* `org_id` in the request path — a valid owner of a different org must still be rejected for an action scoped to an org they don't belong to. This is the same invariant the RBAC design's own route-guard test calls for.

## 5. Testing Approach

- **Unit**: `Claims::has_permission` against a hand-built permissions array — covers admin-scope match, org-scope match, wrong-org rejection, wrong-action rejection.
- **Integration** (real Postgres via `sqlx::test`, same style as the schema plan):
  - register (create mode) → login → access token round-trip decodes to the expected claims.
  - register (join mode) with a valid `org_invites` code → `memberships` row created with the invite's role; a second use of the same code is rejected.
  - refresh token issuance, then successful refresh; refresh rejected after `logout` revokes it; refresh rejected after `expires_at` has passed.
  - `authorize_org_action` rejects a caller whose `memberships` row was deleted *after* their still-valid access token was issued — the concrete revocation-latency scenario from §4.
- **axum integration tests** (via `tower::ServiceExt::oneshot`, no real HTTP server needed): a protected route rejects a missing/malformed/expired token with `401`; rejects a valid token lacking the required permission string with `403`; accepts a valid token with the right permission string.

## Trade-offs Accepted

- **Access tokens can't be revoked before they expire** (30-minute window). Accepted per the RBAC design's own stated tradeoff; sensitive writes are protected by the DB re-check in §4 regardless, so the blast radius is limited to read/view access during that window, not membership/role mutation.
- **Channel-only users can't log into the web app at all** in this iteration — building the cross-channel-linking UI that would let them attach a web login to their existing identity is deferred, consistent with the cross-channel-identity design's own "designed now, not built for v1" scoping for linking.
- **Cookie-based token transport is deferred** to the frontend-integration plan; this plan's contract (bearer token in a JSON body) is sufficient to build and test the backend in isolation.

## Out of Scope (deferred, not blocking this spec)

- Cross-channel-linking UI (attaching a `channel_identity` to a web-registered account).
- Cookie/CSRF handling for the eventual SvelteKit frontend.
- Password reset / email verification flows.
- Rate-limiting login attempts (worth adding before any public deployment, not needed to prove the design).
