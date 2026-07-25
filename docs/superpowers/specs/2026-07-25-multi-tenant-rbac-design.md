# Multi-Tenant RBAC & Organizations

Date: 2026-07-25
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `2026-07-22-cross-channel-identity-design.md`, `2026-07-22-sub-agent-lifecycle-design.md`, `2026-07-25-frontend-chat-design.md` (this doc amends its Audience & Roles / Screens sections)

## Purpose

The frontend chat design assumed a simple `role: user | admin` claim. Real usage needs multi-tenancy: users belong to organizations, permissions are scoped per-org, and users can invite others into group conversations. This document designs the org/membership data model, the permission-string authorization scheme, and how onboarding/invites work — and reconciles this with two things the existing backend docs already committed to: lazy, signup-free channel bootstrap (Telegram/WhatsApp), and the implicit (message-sender-based) group participation model.

## 1. Data Model

```sql
CREATE TABLE organizations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    is_personal BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE memberships (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    role        TEXT NOT NULL,              -- 'owner' | 'admin' | 'member'
    status      TEXT NOT NULL DEFAULT 'active', -- 'invited' | 'active' | 'removed'
    invited_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, user_id)
);

CREATE TABLE org_invites (
    code        TEXT PRIMARY KEY,
    org_id      UUID NOT NULL REFERENCES organizations(id),
    role        TEXT NOT NULL,              -- role granted on acceptance
    invited_by  UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ
);                                          -- same shape/pattern as link_codes

ALTER TABLE sessions ADD COLUMN org_id UUID NOT NULL REFERENCES organizations(id);

CREATE TABLE session_participants (
    session_id  UUID NOT NULL REFERENCES sessions(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    invited_by  UUID REFERENCES users(id),
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, user_id)
);
```

- A `users` row can hold multiple `memberships` rows — multi-org, switchable, confirmed.
- `organizations.is_personal` flags the auto-created, single-member orgs described in §3 — they're an implementation device for channel isolation, not real teams, and are excluded from any "invite people" or "manage members" UI.
- `sessions.org_id` is `NOT NULL` uniformly — every session (DM or group, any channel) belongs to exactly one org. §3 resolves how that's satisfied for channel-native sessions and DMs.
- `session_participants` is **additive** to the cross-channel identity design, not a replacement: it only matters for the new "invite before first message" path on web-native group conversations (§4). Telegram/WhatsApp groups keep working exactly as already designed — participation inferred from `messages.sender_channel_identity_id` — see §5 for the explicit split.

## 2. Permission Model & Enforcement

- **Grammar**: `<app>:<scope>:<role>:[actions]`, confirmed. `scope` is `admin` (platform-wide) or an `org_id` (tenant-scoped).
- **Resources for MVP**: `user` (platform-admin scope only — managing users across the whole platform), `member` (org scope — managing an org's membership), `conversation` (org scope — creating group sessions, inviting participants). Actions: `view`, `manage` (implies create/update/delete for that resource). Deliberately minimal and extensible — new resources (e.g. `project`, once project-management-with-AI integration happens) slot into the same grammar later without a redesign.
- **Role → action mapping** (fixed lookup, not stored per-row):
  - `owner`: `manage` on `member` and `conversation`, plus org-level actions not yet designed (billing, org deletion — explicitly deferred, see Out of Scope).
  - `admin`: `manage` on `member` and `conversation`.
  - `member`: `view` on `member`, `view`+participate on `conversation` (participation itself — sending messages — isn't gated by the permission grammar, it's gated by `session_participants`/inferred membership per §5).
- **Claims lifecycle**: computed at login (and on org-switch) from the platform-admin flag plus all `active` `memberships` rows, baked into a short-lived session token (15–60 min) as a claims array. On expiry, refresh re-derives claims from current DB state, so a role change or org removal takes effect within one refresh cycle, not instantly. For anything sensitive (removing a member, changing a role), the backend re-checks the caller's current `memberships` row directly rather than trusting the token's claims array, so a stale token can't be used to perform an action after access was just revoked.
- **Enforcement is backend-only.** Every API handler declares its required permission string(s) and checks them against the caller's current authorization (claims token, revalidated for sensitive writes as above). The frontend route guards and hidden UI are reflection of this, never the boundary — consistent with the frontend design's existing note on this.
- **Tenant isolation as defense in depth**: every org-scoped query filters by `org_id` regardless of claims, so a permission-string bug can't leak cross-org rows.

## 3. Reconciling Org-Scoping With Existing Channel Bootstrap

The cross-channel identity design has no signup step — a Telegram/WhatsApp message lazily creates a `users` row with zero org context. To keep `sessions.org_id NOT NULL` uniform (no nullable special-casing throughout the codebase):

- **The first time any user is bootstrapped** (channel-native, per the existing lazy-creation flow, or web registration), an `organizations` row is auto-created with `is_personal = true`, and a `memberships` row makes that user its sole `owner`. This happens silently — channel users never see or interact with this org; it exists purely so every session has a valid `org_id`.
- **Channel-native sessions** (a Telegram/WhatsApp DM or group the bot is added to) use the sender's (or, for a pre-existing group, the first-ever sender's) personal org as `org_id`. This is an implementation detail, invisible to that channel's users.
- **Web-native sessions**: if a user goes through the create-or-join registration flow (§4) and joins/creates a real org, their *web-initiated* sessions use that real org's `id`. Their pre-existing channel sessions are unaffected — still tagged to their personal org, matching the cross-channel identity design's existing principle that linking never merges or moves session history, only resolves shared `user_id` for memory/personalization.
- **DMs require an explicit org context to start** (per your answer) — for a web-initiated DM between two users who share more than one real org together, the initiating user picks which org context it happens in (a small org-context selector when starting the DM, defaulting to the currently-active org from the switcher). For a channel-native DM (the common case — a person messaging the bot on Telegram), there's only ever one org in play (their personal org), so no picker is needed there.

This keeps one uniform rule (`org_id` always set, never nullable) while leaving the existing channel-bootstrap and cross-channel-linking designs untouched.

## 4. Onboarding & Invite Flows

- **Registration** (web): create-or-join. "Create an organization" → new `organizations` row (`is_personal = false`), caller becomes `owner`. "I have an invite" → enter/accept a code from `org_invites`, creating a `memberships` row (`status='invited'` → `active` on first successful login).
- **Org invites** (bringing a new person into the org): `owner`/`admin` generates an `org_invites` code/link for a proposed `role`; accepting creates or activates the `memberships` row. Same short-lived, single-use pattern as `link_codes`.
- **Conversation invites** ("add to group", scoped to existing org members only, per your answer): picker queries `memberships` for the current org; selecting someone inserts a `session_participants` row directly — no separate accept/decline step, since they're already a vetted, active member of the same org.
- **Org switching**: a user with multiple active `memberships` sees an org switcher; switching re-scopes claims to the selected `org_id` and reloads the chat list for that org only (§6 — no unified cross-org inbox in this iteration). Personal orgs (`is_personal = true`) never appear in this switcher — if a user's only org is their personal one, the switcher itself is hidden.

## 5. Participation Model: Explicit vs. Inferred

Two different rules coexist, split by channel, stated explicitly to avoid ambiguity:

- **Channel-native groups** (Telegram, WhatsApp, any future non-web channel): participation is exactly as the cross-channel identity design already specifies — inferred from `messages.sender_channel_identity_id`. No `session_participants` rows are created or checked for these; the external platform (Telegram/WhatsApp) is the real source of truth for who's in the group.
- **Web-native groups** (`sessions.channel = 'web'`): participation is explicit via `session_participants`. Viewing or posting to a web group conversation requires a row there; membership is granted only through the "add to group" flow in §4. This is the path that needed inventing, since a web-native group has no external platform tracking who's in it.

## 6. Frontend Changes

Amends `2026-07-25-frontend-chat-design.md` §1 (Audience & Roles) and §2 (Screens & Navigation):

- **Audience & Roles**: replace the simple `role: user | admin` claim with the permission-string claims array described in §2 of this doc. Route guards and conditional UI check for specific permission strings (e.g. presence of `nomi:admin:*` vs. `nomi:<currentOrgId>:owner` or `:admin`) instead of a flat role.
- **Two admin surfaces, two routes** (per your answer): `/admin` is platform-superuser-only (`nomi:admin:user:[view,manage]`) — every org/session on the platform, rare/ops-only, structurally unchanged from the original spec otherwise. A new `/org/:id/members` screen (inside the normal app shell, not the platform admin shell) is for an org's `owner`/`admin` managing their own org's membership and generating invites — reuses the app's normal navigation chrome rather than the platform-admin one.
- **New: org switcher** — visible in the app header/sidebar whenever a user has more than one active, non-personal `memberships` row; switching re-scopes the chat list and claims as described in §4.
- **New: onboarding flow** — a create-or-join screen shown post-registration/pre-first-chat, previously explicitly out of scope in the chat design doc and now in scope.
- **Chat list scoping**: always scoped to (current user, current active org), not a unified cross-org inbox — matches the org-switcher model rather than adding cross-org aggregation.
- **Conversation "add to group" UI**: a member picker scoped to `memberships` of the current org, invoked from the conversation header, per §4 — no separate invite-acceptance screen needed for this path.

## 7. Testing Approach

- Unit: role→action lookup table; claims-array construction from platform-admin flag + active memberships; org-resolution logic for channel-native vs. web-native session creation (§3).
- Integration: `org_invites` code accept flow creates/activates a `memberships` row exactly once (replay rejected, same as `link_codes`); a sensitive action (remove member) re-validates against current DB state even when passed a still-valid-but-stale claims token, confirming the "backend re-checks, doesn't trust the token" rule in §2.
- Route-guard tests: `/admin` rejects anyone without `nomi:admin:*`; `/org/:id/members` rejects anyone without `owner`/`admin` in that specific org (including a valid owner of a *different* org — cross-org authorization must fail).

## Out of Scope (deferred, not blocking this spec)

- Billing and org deletion — mentioned as owner-only in principle, no system designed yet.
- Project-management-with-AI integration — explicitly flagged by you as a future direction; the resource/action grammar is deliberately left extensible for it, nothing more.
- Removing/demoting a member mid-conversation (what happens to their existing `session_participants` rows) — worth a follow-up once the base invite flow is built and this edge case is observed in practice rather than speculated on now.
- Cross-org unified inbox — deferred per §6; may be revisited if single-org-at-a-time proves annoying in practice.
