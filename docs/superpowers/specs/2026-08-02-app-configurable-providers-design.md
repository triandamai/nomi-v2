# App-Configurable LLM & Embedding Providers

Date: 2026-08-02
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md` (the `LlmProvider` trait and `build_provider` this design now drives from the DB instead of env vars), `docs/superpowers/specs/2026-07-26-rag-memory-design.md` (the `EmbeddingProvider` this also covers)

## Purpose

Today `LLM_PROVIDER`/`LLM_MODEL_ID`/`LLM_API_KEY`/`LLM_BASE_URL` and their `EMBEDDING_*` counterparts are read once in `main.rs` and baked into an `Arc<dyn LlmProvider>` / `Arc<dyn EmbeddingProvider>` for the life of the process. Changing provider, model, or key requires an env var change and a redeploy.

This design moves that configuration into the database, editable via an admin-only HTTP API and a new frontend admin page, and swaps the live provider at runtime with no restart. Scope is global (one provider/model/key for the whole app, not per-organization) per explicit decision — this app doesn't yet need per-tenant model choice, and adding it later is a additive change to the same table (add an `org_id` column) rather than a rewrite.

## 1. Database

Migration `0010_provider_settings.sql`:

```sql
CREATE TABLE provider_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_type TEXT NOT NULL UNIQUE CHECK (setting_type IN ('llm', 'embedding')),
    provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    api_key_encrypted BYTEA NOT NULL,
    base_url TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by UUID NOT NULL REFERENCES users(id)
);
```

One row per `setting_type`; `UNIQUE` makes "save settings" an upsert (`ON CONFLICT (setting_type) DO UPDATE`). `provider`/`model_id`/`base_url` stay unencrypted (not secret); `api_key_encrypted` is nonce‖ciphertext from AES-256-GCM (below). `fake` provider rows store an empty ciphertext.

## 2. Encryption

New `backend/src/settings/crypto.rs`, using the `aes-gcm` crate (new dependency). A single master key comes from `SETTINGS_ENCRYPTION_KEY` (32 raw bytes, base64-encoded in the env var) — this key protects the DB-stored provider keys, so unlike the provider keys themselves it *has* to live outside the DB. Read once at startup, stored in `AppState`.

```rust
pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Vec<u8>;   // random 12-byte nonce ‖ ciphertext
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<String, CryptoError>;
```

A decrypt failure at startup (e.g. `SETTINGS_ENCRYPTION_KEY` rotated without re-saving settings) panics with a clear message — same fail-fast convention as other required env vars in `main.rs` today. Key rotation is out of scope; noted below.

## 3. AppState & Live Reload

```rust
// app.rs
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub provider: Arc<RwLock<Arc<dyn LlmProvider>>>,
    pub embedding_provider: Arc<RwLock<Arc<dyn EmbeddingProvider>>>,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
}
```

Read path (`routes/sessions.rs`, unchanged call site): `state.provider.read().await.clone()` replaces `state.provider.as_ref()` — one extra clone of an `Arc`, negligible cost per message.

Write path (settings save handler): build the new provider from the submitted config, then `*state.provider.write().await = new_provider` — the swap is atomic from a reader's perspective (readers either see the old or new `Arc`, never a partial state).

## 4. Startup (`main.rs`)

For each of `llm`/`embedding`: query `provider_settings` for that `setting_type`.
- **Row exists**: decrypt `api_key_encrypted`, build `ModelConfig`/`EmbeddingConfig` from the row, construct the provider.
- **No row**: fall back to today's env-var path exactly as it works now (non-breaking for existing deployments). The row is *not* auto-seeded from env — the DB only takes over once an admin saves settings via the API, per your bootstrap decision.

`SETTINGS_ENCRYPTION_KEY` becomes a new required env var (`main.rs` panics if unset, same style as `DATABASE_URL`/`JWT_SECRET`).

## 5. API Endpoints

Added to `routes/settings.rs`, mounted in `app.rs`:

- `GET /api/admin/settings/llm` / `GET /api/admin/settings/embedding` — returns `{ provider, model_id, base_url, api_key_masked }`. `api_key_masked` is derived at read time from the decrypted key (e.g. `sk-...ab12`, last 4 chars), never the full key.
- `PUT /api/admin/settings/llm` / `PUT /api/admin/settings/embedding` — body `{ provider, model_id, api_key: Option<String>, base_url: Option<String> }`.
  - `api_key: None` or omitted → keep the existing encrypted key (only `provider`/`model_id`/`base_url` change). This matters because `GET` never exposes the real key, so an admin editing just the model shouldn't have to re-paste the secret.
  - `api_key: Some(new_key)` → re-encrypt and replace.
  - Unknown `provider` string, or missing `model_id`/`api_key` for a non-`fake` provider when there's no existing row to fall back to → `400`.
  - On success: upsert the DB row, build the new provider, swap it into `AppState` (§3), return the same shape as `GET`.

Both routes require `AuthClaims` and `claims.has_permission("admin", "system_config", "manage")` (view-vs-manage is not split for MVP — both GET and PUT require `manage`), returning `403` otherwise — same inline style as `remove_member` in `routes/auth.rs` checking `authorize_org_action`.

## 6. Permissions

`auth/permissions.rs::compute_permissions` gains one line: platform admins (`is_platform_admin`) also get `permission_string("admin", "system_config", &["view", "manage"])`, alongside the existing `admin:user` permission.

## 7. Frontend

New `(admin)`-style area under `frontend/src/routes/admin/`, parallel to but independent of the existing `(app)` group:

- **`admin/login/+page.svelte` + `+page.server.ts`** — same shape as the existing `login/+page.server.ts` (form action posts credentials to `/api/auth/login`, sets `access_token`/`refresh_token`/`user_email` cookies — same session mechanism, this is just a distinct entry point), redirects to `/admin/settings/llm` on success.
- **`admin/+layout.server.ts`** — guards everything under `/admin` except `/admin/login`. No `accessToken` → redirect `/admin/login`. Otherwise `apiFetch(fetch, cookies, '/api/whoami')`; if the response isn't ok, or the returned `permissions` array has no `admin:system_config` entry, redirect to `/admin/login?error=forbidden`.
- **`admin/+layout.svelte`** — minimal shell: title bar, logout button posting to `/logout?redirect_to=/admin/login`.
- **`admin/settings/llm/+page.server.ts`** — `load` calls `GET /api/admin/settings/llm` via `apiFetch`. `actions.default` reads the form, posts to `PUT /api/admin/settings/llm`, surfaces `400`/`403` via `fail()`.
- **`admin/settings/llm/+page.svelte`** — form: provider `<select>` (anthropic/openai/gemini/fake), `model_id` text input, `base_url` text input, `api_key` password input (placeholder shows the masked value, left blank = unchanged per §5), save button, inline success/error message.

`frontend/src/routes/logout/+server.ts` gains an optional `redirect_to` query param (default `/login`), read via `url.searchParams.get('redirect_to')`, so the admin logout button lands back on `/admin/login` instead of `/login`.

Embedding settings page (`admin/settings/embedding`) is not built now — same pattern, deferred until needed.

## 8. Error Handling Summary

| Case | Behavior |
|---|---|
| `SETTINGS_ENCRYPTION_KEY` unset at startup | panic (fail fast) |
| Decrypt failure at startup | panic with clear message |
| `PUT` with unknown provider string | `400` |
| `PUT` missing `model_id`/`api_key` for new non-fake provider config | `400` |
| Caller lacks `system_config` permission | `403` |
| Frontend: no access token / lacks admin permission | redirect to `/admin/login` |

## 9. Testing

- Unit: `crypto::encrypt`/`decrypt` round trip, including a tampered-ciphertext case (decrypt must fail, not silently return garbage).
- Backend integration: `PUT` then `GET` reflects the (masked) update; `PUT` with an invalid provider → `400`; non-admin caller → `403`; omitting `api_key` on `PUT` preserves the prior key (verified indirectly — the provider still authenticates); saving `fake` as the LLM provider and immediately sending a chat message proves the live swap took effect without restart (reuses the existing fake-provider sentinel pattern).
- Frontend e2e (Playwright, matching the existing `e2e/` specs): unauthenticated visit to `/admin/settings/llm` redirects to `/admin/login`; a non-platform-admin login redirects to `/admin/login?error=forbidden`; a platform-admin can log in, view current settings, and submit an update.

## Out of Scope

- Per-organization provider configuration (explicit decision — global only for now; see Purpose).
- `SETTINGS_ENCRYPTION_KEY` rotation tooling (re-encrypting existing rows under a new key) — not needed until key rotation is actually required.
- Embedding settings frontend page (backend endpoint exists; UI deferred, same pattern as LLM).
- Audit log of who changed settings when beyond the single `updated_by`/`updated_at` columns already on the row.
