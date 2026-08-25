# User-Selectable LLM Models (with Bring-Your-Own-Key)

Date: 2026-08-25
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-02-app-configurable-providers-design.md` (the single global `provider_settings` row and `build_llm_provider_from_settings_or_env` this design extends into a per-user-selectable catalog — that design's own scope note already anticipated an additive extension here rather than a rewrite).

## Purpose

Today there is exactly one LLM provider+model+key for the whole app, set once by an admin in `/admin/settings/llm` and used for every user's every turn. This design adds a model picker to the chat input area: an admin curates a small list of ready-to-use models (each with its own shared key), any user can pick one, and any user can instead bring their own provider+model+API key entirely outside the admin's list. The choice is a per-user preference — set once, applies to all of that user's sessions until changed — not a per-message control.

Embeddings are untouched: still the single global admin-configured row (`provider_settings`, `setting_type = 'embedding'`), since memory retrieval isn't something a user interacts with directly.

## 1. Data Model & Admin API

Migration adds two tables and retires the LLM half of `provider_settings` (the `embedding` row/type is unaffected):

```sql
CREATE TABLE admin_llm_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label TEXT NOT NULL,
    provider TEXT NOT NULL CHECK (provider IN ('anthropic', 'openai', 'gemini', 'fake')),
    model_id TEXT NOT NULL,
    api_key_encrypted BYTEA NOT NULL,
    base_url TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by UUID NOT NULL REFERENCES users(id)
);
-- Exactly one row may have is_default = true; enforced in application code (partial unique
-- index on is_default WHERE is_default is impractical to make atomically swap-safe across a
-- two-step "unset old default, set new default" update, so this is a transaction-guarded
-- application invariant instead, same trust level as other single-owner admin actions in this app).

CREATE TABLE user_llm_selections (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    admin_model_id UUID REFERENCES admin_llm_models(id) ON DELETE SET NULL,
    custom_label TEXT,
    custom_provider TEXT CHECK (custom_provider IN ('anthropic', 'openai', 'gemini', 'fake')),
    custom_model_id TEXT,
    custom_api_key_encrypted BYTEA,
    custom_base_url TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT exactly_one_source CHECK (
        (admin_model_id IS NOT NULL AND custom_provider IS NULL) OR
        (admin_model_id IS NULL AND custom_provider IS NOT NULL) OR
        (admin_model_id IS NULL AND custom_provider IS NULL)
    )
);
```

A user row with everything `NULL` means "no preference set yet — use the default." `user_llm_selections` stores a *reference* to `admin_llm_models` (not a copy of its key): rotating an admin model's key updates every user pointing at it automatically, with no fan-out job, and deleting an admin model (`ON DELETE SET NULL`) cleanly falls back everyone referencing it to the default without an explicit migration step. A user has at most one active custom config at a time — switching to a different BYOK setup overwrites the existing `custom_*` columns rather than accumulating a list; this keeps the schema and resolution logic simple, and re-entering a key is a rare enough action not to warrant a saved-multiple-keys catalog.

**Admin endpoints** (`/api/admin/settings/llm/models`, same `system_config:manage` permission check `require_system_config_permission` already used by every other `/api/admin/settings/*` route):
- `GET /api/admin/settings/llm/models` — list all, `api_key_masked` via the existing `settings::mask_api_key`
- `POST /api/admin/settings/llm/models` — create; first-ever row is automatically `is_default = true`
- `PUT /api/admin/settings/llm/models/:id` — update label/provider/model_id/api_key (blank api_key = keep existing, matching today's `resolve_api_key` semantics)/base_url
- `DELETE /api/admin/settings/llm/models/:id` — `400` if it's the last remaining row (there must always be a default) or if it's currently the default (must reassign default first)
- `PUT /api/admin/settings/llm/models/:id/default` — transactionally unset the old default and set this one

The existing singular `GET/PUT /api/admin/settings/llm` (and its admin page) are removed, replaced by the list-based routes above — this was always an internal admin surface, not a versioned public API, so there's no compatibility shim to keep.

**User endpoints** (any authenticated user):
- `GET /api/llm/models` — the admin list (label/provider/model_id, no keys) + the user's current selection (an `admin_model_id` reference, or their one custom config with key masked, or neither if unset)
- `PUT /api/llm/selection` — `{"kind":"admin","admin_model_id":"..."}` or `{"kind":"custom","label":"...","provider":"...","model_id":"...","api_key":"...","base_url":"..."}` (the custom case validates before saving — see §2)

**Migration/rollout:** the migration that creates these tables also seeds `admin_llm_models` from the existing `provider_settings` row where `setting_type = 'llm'`, if one exists (label `"Default"`, `is_default = true`), so upgrading doesn't strand every user without a working model. On a fresh install with nothing configured yet, no seed row is created — the env-var fallback in §2 covers that until an admin adds one.

## 2. Provider Resolution & BYOK Validation

`build_llm_provider_from_settings_or_env` becomes `build_llm_provider_for_user(pool, user_id, settings_key, http_client)`. The worker loop (`backend/src/worker.rs`) already resolves `user_id` per claimed job before calling `process_turn` — this just threads that same value one call further. Resolution order:

1. `user_llm_selections` row has `admin_model_id` set → join `admin_llm_models`, decrypt its key.
2. Row has `custom_*` set → decrypt `custom_api_key_encrypted`, use those fields directly.
3. No row, or the row's `admin_model_id` pointed at a since-deleted model → whichever `admin_llm_models` row has `is_default = true`.
4. No `admin_llm_models` rows exist at all → today's existing `LLM_PROVIDER`/`LLM_MODEL_ID`/`LLM_API_KEY`/`LLM_BASE_URL` env-var fallback, byte-for-byte unchanged (keeps `docker-compose`'s `LLM_PROVIDER=fake` dev default working with zero admin setup).

**BYOK validation on save:** before `PUT /api/llm/selection` with `kind: "custom"` persists anything, it builds a `ModelConfig` from the submitted fields, runs it through the existing `build_provider()` + `crate::llm::complete()` with a minimal request (no system prompt, one short user message, `max_tokens: 8`). A validation failure returns `400` with the provider's actual error text — surfaced inline on the "add custom model" form, not a generic toast — and nothing is saved. Success encrypts and stores. The same allowed-provider list as admin models (`anthropic`/`openai`/`gemini`/`fake`) applies; `fake` validates trivially with no network call, useful for testing the picker itself (and is what the e2e test in §4 uses).

## 3. Frontend UI

A small pill/button near the Send button in the chat input area (`frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`) showing the active model's label. Clicking opens a popover:
- Admin's models, current selection checked
- "Your custom model" section: the user's active custom config if set, or an empty state
- "+ Use your own API key" → a small form (provider select, model_id, API key, optional base URL) → `PUT /api/llm/selection`; inline error on validation failure; closes and updates the pill on success

This is a **standing preference, not a per-message control** — selecting a model doesn't attach to whatever's in the input box; it changes what the *next* turn (whenever sent, from any of the user's sessions) uses. No per-message plumbing through `send_message`/`turn_jobs` is needed.

The admin's `/admin/settings/llm` page becomes a list view (add/edit/delete/set-default) instead of the current single-form page. There's no existing list-management UI elsewhere in this admin area to follow (`frontend/src/routes/admin/(protected)/` currently only has the single-form `settings/llm` page and the top-level dashboard) — this is a new pattern for the admin area, not a reuse of one.

## 4. Error Handling

| Condition | Behavior |
|---|---|
| BYOK key/provider/model invalid at save time | `400` with the provider's error text, inline on the form, nothing persisted |
| Selecting a stale/deleted `admin_model_id` (client's list was out of date) | `404` at `PUT /api/llm/selection` |
| A previously-valid key later revoked/expired | Surfaces through the existing `TurnFailed` → inline chat error path, same as any other LLM provider failure today — no special handling |
| Admin deletes a model other users are actively using | Those users silently fall back to the default on their next turn (§2 step 3) — no error, no notification (out of scope, see below) |
| Admin deletes the last remaining model, or the current default without reassigning | `400`, rejected — there must always be a default |

## 5. Testing

**Backend** (`#[sqlx::test]`, matching existing patterns in this codebase): admin CRUD endpoints (list/create/update/delete/set-default, including the last-row and current-default delete guards); the user selection endpoint (admin pick; custom pick with `FakeLlmProvider`-backed validation success and failure; stale `admin_model_id` → 404); the resolution logic directly, exercising all four branches in §2's order (admin ref → custom → no-row/deleted-ref fallback to default → no-admin-models-at-all env fallback).

**Frontend e2e** (extends the existing suite): open the picker, switch the admin model, send a message, confirm it renders under the new selection (proving the choice actually takes effect, not just that the UI updated); a BYOK flow using `fake` as the custom provider (validates with no real network call) to prove save → validate → select end to end in CI without a real API key.

## Out of Scope

- Per-message model override — this is a standing per-user preference only.
- Multiple saved BYOK configs per user — one active custom config at a time; switching overwrites it.
- Per-user embedding model selection — embeddings stay the single global admin-configured row.
- Notifying a user when the admin model they were using gets deleted and they silently fall back to the default.
- Rate limiting, usage tracking, or cost accounting for BYOK usage.
- Redesigning the admin settings page's visual style beyond what the list-based CRUD needs.
