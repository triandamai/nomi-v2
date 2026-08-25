# User-Selectable LLM Models (with BYOK) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let an admin curate a list of ready-to-use LLM models, let any user pick one or bring their own provider+model+API key, and have that choice be a per-user preference the worker resolves on every turn — surfaced via a picker in the chat input area.

**Architecture:** Two new Postgres tables (`admin_llm_models`, a reference-based `user_llm_selections`), a new `routes/llm_models.rs` with admin CRUD + user selection endpoints, a resolution function the worker calls per-turn instead of the old single global row, and SvelteKit picker UI in the chat page plus a rewritten admin settings page.

**Tech Stack:** Same stack as the rest of this backend (axum, sqlx, Postgres) and frontend (SvelteKit, form actions) — no new dependencies.

**Spec:** `docs/superpowers/specs/2026-08-25-user-selectable-llm-models-design.md`

## Global Constraints

- Embeddings are untouched — still the single global `provider_settings` row, `setting_type = 'embedding'`. Nothing in this plan touches embedding code paths.
- `user_llm_selections` stores a *reference* to `admin_llm_models` (not a copy of its key) — rotating a shared admin key updates every user pointing at it automatically.
- A user has at most one active custom (BYOK) config at a time — switching overwrites it, no saved-multiple-keys catalog.
- BYOK validation happens synchronously on save: build the candidate provider, run one real `complete()` call with a minimal probe request; reject with the provider's actual error text on failure; only encrypt-and-store on success.
- Resolution order the worker must follow, in this exact sequence: (1) user's `admin_model_id` reference if set → (2) user's `custom_*` fields if set → (3) the admin model flagged `is_default` → (4) the existing `LLM_PROVIDER`/`LLM_MODEL_ID`/`LLM_API_KEY`/`LLM_BASE_URL` env-var fallback, unchanged, for when no admin models exist at all yet.
- Allowed providers for both admin models and BYOK custom entries: `anthropic`, `openai`, `gemini`, `fake` — the same list already used by the existing admin settings endpoints.
- Model selection is a standing per-user preference, not a per-message control — no plumbing through `send_message`/`turn_jobs` is needed anywhere in this plan.

---

## File Structure

- Create `backend/migrations/0012_admin_llm_models.sql` — the two new tables, seeded from any existing `provider_settings` "llm" row.
- Create `backend/src/settings/llm_models.rs` — DB-layer CRUD for both new tables (mirrors `backend/src/settings/mod.rs`'s existing `provider_settings` pattern).
- Modify `backend/src/settings/mod.rs` — register the new submodule.
- Modify `backend/src/llm/mod.rs` — add `validate_model_config`, the BYOK save-time probe call.
- Modify `backend/src/bootstrap/providers.rs` — replace `build_llm_provider_from_settings_or_env` with `build_llm_provider_for_user` (and the testable `resolve_llm_model_config` it wraps).
- Modify `backend/src/bootstrap/mod.rs` — update the re-export.
- Modify `backend/src/worker.rs` — call the new per-user resolution instead of the old global one.
- Create `backend/src/routes/llm_models.rs` — admin CRUD endpoints + user selection endpoints.
- Modify `backend/src/routes/mod.rs` — register the new route module.
- Modify `backend/src/routes/settings.rs` — remove the retired `get_llm_settings`/`put_llm_settings` handlers; make `require_system_config_permission` `pub(crate)` so the new file can reuse it.
- Modify `backend/src/app.rs` — remove the old singular `/api/admin/settings/llm` route, add the new list-based admin routes and the two user-facing routes.
- Create `backend/tests/llm_models_db.rs`, `backend/tests/llm_provider_resolution.rs`, `backend/tests/admin_llm_models_routes.rs`, `backend/tests/user_llm_selection_routes.rs` — new test files matching this codebase's per-file `#[sqlx::test]` convention.
- Modify `frontend/src/lib/types.ts` — new types for the picker/admin list.
- Modify `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` and `+page.svelte` — the input-area picker.
- Modify `frontend/src/routes/admin/(protected)/settings/llm/+page.server.ts` and `+page.svelte` — rewritten as a list view.
- Modify `frontend/e2e/conversation.e2e.ts` or create `frontend/e2e/model-picker.e2e.ts` (created — this is a distinct enough flow to warrant its own file rather than growing the existing one further).

---

### Task 1: Database — `admin_llm_models` and `user_llm_selections`

**Files:**
- Create: `backend/migrations/0012_admin_llm_models.sql`
- Create: `backend/src/settings/llm_models.rs`
- Modify: `backend/src/settings/mod.rs`
- Test: `backend/tests/llm_models_db.rs`

**Interfaces:**
- Produces: `AdminLlmModel { id: Uuid, label: String, provider: String, model_id: String, api_key_encrypted: Vec<u8>, base_url: Option<String>, is_default: bool }` and `UserLlmSelectionRow { admin_model_id: Option<Uuid>, custom_label: Option<String>, custom_provider: Option<String>, custom_model_id: Option<String>, custom_api_key_encrypted: Option<Vec<u8>>, custom_base_url: Option<String> }`, plus the functions listed in Step 3 below — Tasks 3, 4, and 5 call these directly.

- [ ] **Step 1: Write the migration**

Create `backend/migrations/0012_admin_llm_models.sql`:

```sql
CREATE TABLE admin_llm_models (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label              TEXT NOT NULL,
    provider           TEXT NOT NULL CHECK (provider IN ('anthropic', 'openai', 'gemini', 'fake')),
    model_id           TEXT NOT NULL,
    api_key_encrypted  BYTEA NOT NULL,
    base_url           TEXT,
    is_default         BOOLEAN NOT NULL DEFAULT false,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by         UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE user_llm_selections (
    user_id                    UUID PRIMARY KEY REFERENCES users(id),
    admin_model_id             UUID REFERENCES admin_llm_models(id) ON DELETE SET NULL,
    custom_label               TEXT,
    custom_provider            TEXT CHECK (custom_provider IN ('anthropic', 'openai', 'gemini', 'fake')),
    custom_model_id            TEXT,
    custom_api_key_encrypted   BYTEA,
    custom_base_url            TEXT,
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT exactly_one_source CHECK (
        (admin_model_id IS NOT NULL AND custom_provider IS NULL) OR
        (admin_model_id IS NULL AND custom_provider IS NOT NULL) OR
        (admin_model_id IS NULL AND custom_provider IS NULL)
    )
);

-- Seed from the existing single-row LLM config, if one was ever configured, so upgrading
-- doesn't strand every user without a working model. Fresh installs get no seed row; the
-- env-var fallback in resolve_llm_model_config covers that until an admin adds one.
INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, base_url, is_default, updated_by)
SELECT 'Default', provider, model_id, api_key_encrypted, base_url, true, updated_by
FROM provider_settings
WHERE setting_type = 'llm';
```

- [ ] **Step 2: Run it to confirm it applies cleanly**

Run: `cd backend && export DATABASE_URL="postgres://nomi:nomi@localhost:5432/nomi" && sqlx migrate run`
Expected: `Applied 12/migrate admin_llm_models` (or similar), no errors.

- [ ] **Step 3: Write the failing tests**

Create `backend/tests/llm_models_db.rs`:

```rust
use nomi_orchestrator::settings::llm_models::{
    create_admin_llm_model, delete_admin_llm_model, get_admin_llm_model, get_default_admin_llm_model,
    get_user_llm_selection, list_admin_llm_models, set_default_admin_llm_model, set_user_llm_selection_admin,
    set_user_llm_selection_custom, update_admin_llm_model, CustomLlmSelection, DeleteAdminLlmModelError,
    NewAdminLlmModel, SetDefaultAdminLlmModelError, UpdateAdminLlmModel,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn new_model_input(label: &str, updated_by: Uuid) -> NewAdminLlmModel<'_> {
    NewAdminLlmModel {
        label,
        provider: "anthropic",
        model_id: "claude-haiku-4-5",
        api_key_encrypted: vec![1, 2, 3],
        base_url: None,
        updated_by,
    }
}

#[sqlx::test]
async fn the_first_model_created_becomes_the_default(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    assert!(model.is_default);
}

#[sqlx::test]
async fn a_second_model_created_is_not_the_default(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();
    assert!(!second.is_default);
}

#[sqlx::test]
async fn list_returns_all_models_in_creation_order(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    let models = list_admin_llm_models(&pool).await.unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].label, "First");
    assert_eq!(models[1].label, "Second");
}

#[sqlx::test]
async fn get_default_admin_llm_model_returns_none_when_no_models_exist(pool: PgPool) {
    assert!(get_default_admin_llm_model(&pool).await.unwrap().is_none());
}

#[sqlx::test]
async fn update_changes_fields_and_keeps_existing_key_when_none_given(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    let updated = update_admin_llm_model(
        &pool,
        model.id,
        UpdateAdminLlmModel {
            label: "Renamed",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: None,
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.label, "Renamed");
    assert_eq!(updated.provider, "openai");
    assert_eq!(updated.model_id, "gpt-4o");
    assert_eq!(updated.api_key_encrypted, vec![1, 2, 3]);
}

#[sqlx::test]
async fn update_replaces_the_key_when_one_is_given(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    let updated = update_admin_llm_model(
        &pool,
        model.id,
        UpdateAdminLlmModel {
            label: "First",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: Some(vec![9, 9, 9]),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.api_key_encrypted, vec![9, 9, 9]);
}

#[sqlx::test]
async fn update_of_a_nonexistent_model_returns_none(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let result = update_admin_llm_model(
        &pool,
        Uuid::new_v4(),
        UpdateAdminLlmModel {
            label: "X",
            provider: "anthropic",
            model_id: "x",
            api_key_encrypted: None,
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    assert!(result.is_none());
}

#[sqlx::test]
async fn delete_rejects_the_only_model_since_it_is_always_the_default(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("Only", user_id)).await.unwrap();

    let result = delete_admin_llm_model(&pool, model.id).await;
    assert!(matches!(result, Err(DeleteAdminLlmModelError::IsDefault)));
}

#[sqlx::test]
async fn delete_rejects_the_current_default_even_with_others_present(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let first = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    let result = delete_admin_llm_model(&pool, first.id).await;
    assert!(matches!(result, Err(DeleteAdminLlmModelError::IsDefault)));
}

#[sqlx::test]
async fn delete_succeeds_for_a_non_default_model_when_others_remain(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    delete_admin_llm_model(&pool, second.id).await.unwrap();
    assert!(get_admin_llm_model(&pool, second.id).await.unwrap().is_none());
}

#[sqlx::test]
async fn set_default_swaps_the_default_flag_to_exactly_the_new_model(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let first = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    set_default_admin_llm_model(&pool, second.id).await.unwrap();

    let first = get_admin_llm_model(&pool, first.id).await.unwrap().unwrap();
    let second = get_admin_llm_model(&pool, second.id).await.unwrap().unwrap();
    assert!(!first.is_default);
    assert!(second.is_default);
}

#[sqlx::test]
async fn set_default_of_a_nonexistent_model_returns_not_found(pool: PgPool) {
    let result = set_default_admin_llm_model(&pool, Uuid::new_v4()).await;
    assert!(matches!(result, Err(SetDefaultAdminLlmModelError::NotFound)));
}

#[sqlx::test]
async fn a_user_with_no_selection_row_resolves_to_none(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    assert!(get_user_llm_selection(&pool, user_id).await.unwrap().is_none());
}

#[sqlx::test]
async fn setting_an_admin_selection_then_a_custom_one_clears_the_admin_reference(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    set_user_llm_selection_admin(&pool, user_id, model.id).await.unwrap();
    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, Some(model.id));

    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "My key",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: vec![4, 5, 6],
            base_url: None,
        },
    )
    .await
    .unwrap();

    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, None);
    assert_eq!(row.custom_provider, Some("openai".to_string()));
    assert_eq!(row.custom_model_id, Some("gpt-4o".to_string()));
}

#[sqlx::test]
async fn setting_a_custom_selection_then_an_admin_one_clears_the_custom_fields(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "My key",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: vec![4, 5, 6],
            base_url: None,
        },
    )
    .await
    .unwrap();

    set_user_llm_selection_admin(&pool, user_id, model.id).await.unwrap();

    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, Some(model.id));
    assert_eq!(row.custom_provider, None);
    assert_eq!(row.custom_model_id, None);
}

#[sqlx::test]
async fn deleting_a_referenced_admin_model_clears_the_users_reference(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("Default", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    set_user_llm_selection_admin(&pool, user_id, second.id).await.unwrap();
    delete_admin_llm_model(&pool, second.id).await.unwrap();

    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, None);
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cd backend && cargo test --test llm_models_db`
Expected: fails to compile — `nomi_orchestrator::settings::llm_models` doesn't exist yet.

- [ ] **Step 5: Write the implementation**

Create `backend/src/settings/llm_models.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminLlmModel {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<String>,
    pub is_default: bool,
}

pub async fn list_admin_llm_models(pool: &PgPool) -> Result<Vec<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "SELECT id, label, provider, model_id, api_key_encrypted, base_url, is_default \
         FROM admin_llm_models ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_admin_llm_model(pool: &PgPool, id: Uuid) -> Result<Option<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "SELECT id, label, provider, model_id, api_key_encrypted, base_url, is_default \
         FROM admin_llm_models WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_default_admin_llm_model(pool: &PgPool) -> Result<Option<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "SELECT id, label, provider, model_id, api_key_encrypted, base_url, is_default \
         FROM admin_llm_models WHERE is_default = true",
    )
    .fetch_optional(pool)
    .await
}

pub struct NewAdminLlmModel<'a> {
    pub label: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<&'a str>,
    pub updated_by: Uuid,
}

/// The first model ever created becomes the default automatically (there must always be one
/// once at least one exists); later ones are created non-default until an admin explicitly
/// promotes them via `set_default_admin_llm_model`.
pub async fn create_admin_llm_model(pool: &PgPool, input: NewAdminLlmModel<'_>) -> Result<AdminLlmModel, sqlx::Error> {
    let existing_count: i64 = sqlx::query_scalar("SELECT count(*) FROM admin_llm_models").fetch_one(pool).await?;
    let is_default = existing_count == 0;

    sqlx::query_as::<_, AdminLlmModel>(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, base_url, is_default, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, label, provider, model_id, api_key_encrypted, base_url, is_default",
    )
    .bind(input.label)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .bind(is_default)
    .bind(input.updated_by)
    .fetch_one(pool)
    .await
}

pub struct UpdateAdminLlmModel<'a> {
    pub label: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Option<Vec<u8>>,
    pub base_url: Option<&'a str>,
    pub updated_by: Uuid,
}

pub async fn update_admin_llm_model(
    pool: &PgPool,
    id: Uuid,
    input: UpdateAdminLlmModel<'_>,
) -> Result<Option<AdminLlmModel>, sqlx::Error> {
    sqlx::query_as::<_, AdminLlmModel>(
        "UPDATE admin_llm_models SET \
         label = $2, provider = $3, model_id = $4, \
         api_key_encrypted = COALESCE($5, api_key_encrypted), \
         base_url = $6, updated_by = $7, updated_at = now() \
         WHERE id = $1 \
         RETURNING id, label, provider, model_id, api_key_encrypted, base_url, is_default",
    )
    .bind(id)
    .bind(input.label)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .bind(input.updated_by)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteAdminLlmModelError {
    #[error("model not found")]
    NotFound,
    #[error("cannot delete the current default; set a different default first")]
    IsDefault,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// There's no separate "last remaining model" guard: the first model ever created is always
/// the default (see `create_admin_llm_model`), and defaults can never be deleted here — so the
/// `IsDefault` check alone already guarantees at least one model always survives. A dedicated
/// row-count check would be dead code, since a lone remaining row is always the default by
/// construction.
pub async fn delete_admin_llm_model(pool: &PgPool, id: Uuid) -> Result<(), DeleteAdminLlmModelError> {
    let model = get_admin_llm_model(pool, id).await?.ok_or(DeleteAdminLlmModelError::NotFound)?;
    if model.is_default {
        return Err(DeleteAdminLlmModelError::IsDefault);
    }
    sqlx::query("DELETE FROM admin_llm_models WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SetDefaultAdminLlmModelError {
    #[error("model not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub async fn set_default_admin_llm_model(pool: &PgPool, id: Uuid) -> Result<(), SetDefaultAdminLlmModelError> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query("UPDATE admin_llm_models SET is_default = true WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if updated.rows_affected() == 0 {
        return Err(SetDefaultAdminLlmModelError::NotFound);
    }
    sqlx::query("UPDATE admin_llm_models SET is_default = false WHERE id != $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserLlmSelectionRow {
    pub admin_model_id: Option<Uuid>,
    pub custom_label: Option<String>,
    pub custom_provider: Option<String>,
    pub custom_model_id: Option<String>,
    pub custom_api_key_encrypted: Option<Vec<u8>>,
    pub custom_base_url: Option<String>,
}

pub async fn get_user_llm_selection(pool: &PgPool, user_id: Uuid) -> Result<Option<UserLlmSelectionRow>, sqlx::Error> {
    sqlx::query_as::<_, UserLlmSelectionRow>(
        "SELECT admin_model_id, custom_label, custom_provider, custom_model_id, \
         custom_api_key_encrypted, custom_base_url \
         FROM user_llm_selections WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn set_user_llm_selection_admin(pool: &PgPool, user_id: Uuid, admin_model_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, admin_model_id) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET \
         admin_model_id = $2, custom_label = NULL, custom_provider = NULL, custom_model_id = NULL, \
         custom_api_key_encrypted = NULL, custom_base_url = NULL, updated_at = now()",
    )
    .bind(user_id)
    .bind(admin_model_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub struct CustomLlmSelection<'a> {
    pub label: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<&'a str>,
}

pub async fn set_user_llm_selection_custom(
    pool: &PgPool,
    user_id: Uuid,
    input: CustomLlmSelection<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, custom_label, custom_provider, custom_model_id, \
         custom_api_key_encrypted, custom_base_url) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET \
         admin_model_id = NULL, custom_label = $2, custom_provider = $3, custom_model_id = $4, \
         custom_api_key_encrypted = $5, custom_base_url = $6, updated_at = now()",
    )
    .bind(user_id)
    .bind(input.label)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .execute(pool)
    .await?;
    Ok(())
}
```

Edit `backend/src/settings/mod.rs`, add at the top:

```rust
pub mod crypto;
pub mod llm_models;
```

(replaces the existing `pub mod crypto;` line — adds the new line right after it)

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cd backend && cargo test --test llm_models_db`
Expected: all 15 tests pass.

- [ ] **Step 7: Commit**

```bash
git add backend/migrations/0012_admin_llm_models.sql backend/src/settings/llm_models.rs backend/src/settings/mod.rs backend/tests/llm_models_db.rs
git commit -m "feat: add admin_llm_models and user_llm_selections tables"
```

---

### Task 2: BYOK validation call

**Files:**
- Modify: `backend/src/llm/mod.rs`

**Interfaces:**
- Consumes: `ModelConfig`, `build_provider`, `complete` (all already in this file).
- Produces: `pub async fn validate_model_config(config: ModelConfig, http_client: reqwest::Client) -> Result<(), LlmError>` — Task 4's `put_user_selection` handler calls this for the BYOK save path.

- [ ] **Step 1: Write the failing test**

Edit `backend/src/llm/mod.rs`, add at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate_model_config_succeeds_against_the_fake_provider() {
        let config = ModelConfig {
            provider: ProviderKind::Fake,
            model_id: String::new(),
            api_key: String::new(),
            base_url: None,
        };
        let result = validate_model_config(config, reqwest::Client::new()).await;
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cd backend && cargo test --lib llm::tests::validate_model_config_succeeds_against_the_fake_provider`
Expected: fails to compile — `validate_model_config` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

Edit `backend/src/llm/mod.rs`, add right after the existing `complete` function (before `enum PendingBlock`):

```rust
/// Runs a minimal real call through the given config to prove it actually works, without
/// persisting anything — used to validate a bring-your-own-key submission before saving it.
pub async fn validate_model_config(config: ModelConfig, http_client: reqwest::Client) -> Result<(), LlmError> {
    let provider = build_provider(config, http_client);
    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "Hi".to_string() }] }],
        tools: vec![],
        max_tokens: 8,
    };
    complete(provider.as_ref(), request).await?;
    Ok(())
}
```

- [ ] **Step 4: Run it to verify it passes**

Run: `cd backend && cargo test --lib llm::tests::validate_model_config_succeeds_against_the_fake_provider`
Expected: passes.

**Note on test scope:** this function is exactly `build_provider` (infallible, a plain match) plus `complete()` (whose error-propagation is already exercised throughout this codebase's turn-processing tests) plus `.map(|_| ())`. A genuine "the provider rejects a bad key" failure is a live-network scenario, not something worth mocking here — the success path above is the only behavior this specific function adds, so it's the only thing tested at this level. Task 4's HTTP-level tests cover the input-validation `400`s (bad provider name, missing fields) that don't require a real network call.

- [ ] **Step 5: Commit**

```bash
git add backend/src/llm/mod.rs
git commit -m "feat: add validate_model_config for BYOK save-time validation"
```

---

### Task 3: Admin CRUD endpoints

**Files:**
- Create: `backend/src/routes/llm_models.rs`
- Modify: `backend/src/routes/mod.rs`
- Modify: `backend/src/routes/settings.rs`
- Modify: `backend/src/app.rs`
- Test: `backend/tests/admin_llm_models_routes.rs`

**Interfaces:**
- Consumes: everything from Task 1 (`settings::llm_models::*`), `require_system_config_permission` (widened to `pub(crate)` in this task).
- Produces: `list_admin_models`, `create_admin_model`, `update_admin_model`, `delete_admin_model`, `set_default_admin_model` handler functions, and the constant `ALLOWED_PROVIDERS: [&str; 4]` — Task 4 adds user-facing handlers to this same file and reuses `ALLOWED_PROVIDERS`.

- [ ] **Step 1: Write the failing tests**

Create `backend/tests/admin_llm_models_routes.rs`:

```rust
mod support;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: support::TEST_SETTINGS_KEY,
        mqtt_broker_host: support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: support::TEST_MQTT_BROKER_PORT,
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

#[sqlx::test]
async fn non_admin_is_forbidden_from_listing_models(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/settings/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn admin_can_create_then_list_a_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin1@example.com").await;

    let (status, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Claude Sonnet", "provider": "anthropic", "model_id": "claude-sonnet-5", "api_key": "sk-abcdefgh1234", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(create_body["label"], "Claude Sonnet");
    assert_eq!(create_body["is_default"], true);
    assert_eq!(create_body["api_key_masked"], "...1234");

    let (status, list_body) = json_request(router, "GET", "/api/admin/settings/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_body.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn create_rejects_an_unknown_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let (status, _) = json_request(
        router,
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "X", "provider": "not-real", "model_id": "x", "api_key": "key", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn update_keeps_the_existing_key_when_none_given(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin3@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "First", "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-original-key9", "base_url": null }),
        Some(&token),
    )
    .await;
    let id = create_body["id"].as_str().unwrap();

    let (status, update_body) = json_request(
        router,
        "PUT",
        &format!("/api/admin/settings/llm/models/{id}"),
        json!({ "label": "Renamed", "provider": "anthropic", "model_id": "claude-sonnet-5", "api_key": null, "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(update_body["label"], "Renamed");
    assert_eq!(update_body["api_key_masked"], "...key9");
}

#[sqlx::test]
async fn delete_rejects_the_last_remaining_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin4@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Only", "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-key", "base_url": null }),
        Some(&token),
    )
    .await;
    let id = create_body["id"].as_str().unwrap();

    let (status, _) = json_request(router, "DELETE", &format!("/api/admin/settings/llm/models/{id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn set_default_then_delete_the_old_default_succeeds(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin5@example.com").await;

    let (_, first) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "First", "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-key1", "base_url": null }),
        Some(&token),
    )
    .await;
    let first_id = first["id"].as_str().unwrap();

    let (_, second) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Second", "provider": "openai", "model_id": "gpt-4o", "api_key": "sk-key2", "base_url": null }),
        Some(&token),
    )
    .await;
    let second_id = second["id"].as_str().unwrap();

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        &format!("/api/admin/settings/llm/models/{second_id}/default"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = json_request(router, "DELETE", &format!("/api/admin/settings/llm/models/{first_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test admin_llm_models_routes`
Expected: fails to compile — `/api/admin/settings/llm/models` routes don't exist yet.

- [ ] **Step 3: Write the implementation**

Edit `backend/src/routes/settings.rs` — remove the `get_llm_settings` and `put_llm_settings` functions entirely (everything else in the file — `ProviderSettingsResponse`, `UpdateProviderSettingsRequest`, `load_settings_response`, `resolve_api_key`, `get_embedding_settings`, `put_embedding_settings` — stays, still used by embeddings), and change:

```rust
fn require_system_config_permission(claims: &Claims) -> Result<(), (StatusCode, &'static str)> {
```

to:

```rust
pub(crate) fn require_system_config_permission(claims: &Claims) -> Result<(), (StatusCode, &'static str)> {
```

Create `backend/src/routes/llm_models.rs`:

```rust
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::extractor::AuthClaims;
use crate::routes::settings::require_system_config_permission;
use crate::settings::{self, llm_models};

pub(crate) const ALLOWED_PROVIDERS: [&str; 4] = ["anthropic", "openai", "gemini", "fake"];

#[derive(Serialize)]
pub struct AdminLlmModelResponse {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub is_default: bool,
    pub api_key_masked: String,
}

fn to_response(
    model: llm_models::AdminLlmModel,
    settings_key: &[u8; 32],
) -> Result<AdminLlmModelResponse, (StatusCode, &'static str)> {
    let api_key = settings::crypto::decrypt(settings_key, &model.api_key_encrypted)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key"))?;
    Ok(AdminLlmModelResponse {
        id: model.id,
        label: model.label,
        provider: model.provider,
        model_id: model.model_id,
        base_url: model.base_url,
        is_default: model.is_default,
        api_key_masked: settings::mask_api_key(&api_key),
    })
}

pub async fn list_admin_models(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<AdminLlmModelResponse>>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    let models = llm_models::list_admin_llm_models(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load models"))?;
    let responses = models.into_iter().map(|m| to_response(m, &state.settings_key)).collect::<Result<Vec<_>, _>>()?;
    Ok(Json(responses))
}

#[derive(Deserialize)]
pub struct CreateAdminModelRequest {
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub async fn create_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<CreateAdminModelRequest>,
) -> Result<(StatusCode, Json<AdminLlmModelResponse>), (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    if !ALLOWED_PROVIDERS.contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected anthropic, openai, gemini, or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }
    if !is_fake && req.api_key.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider"));
    }
    if req.label.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "label is required"));
    }

    let model = llm_models::create_admin_llm_model(
        &state.pool,
        llm_models::NewAdminLlmModel {
            label: &req.label,
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &req.api_key),
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create model"))?;

    Ok((StatusCode::CREATED, Json(to_response(model, &state.settings_key)?)))
}

#[derive(Deserialize)]
pub struct UpdateAdminModelRequest {
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub async fn update_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAdminModelRequest>,
) -> Result<Json<AdminLlmModelResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    if !ALLOWED_PROVIDERS.contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected anthropic, openai, gemini, or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }
    if req.label.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "label is required"));
    }

    let api_key_encrypted = match req.api_key.as_deref() {
        Some(key) if !key.is_empty() => Some(settings::crypto::encrypt(&state.settings_key, key)),
        _ => None,
    };

    let model = llm_models::update_admin_llm_model(
        &state.pool,
        id,
        llm_models::UpdateAdminLlmModel {
            label: &req.label,
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted,
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to update model"))?
    .ok_or((StatusCode::NOT_FOUND, "model not found"))?;

    Ok(Json(to_response(model, &state.settings_key)?))
}

pub async fn delete_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    llm_models::delete_admin_llm_model(&state.pool, id).await.map_err(|e| match e {
        llm_models::DeleteAdminLlmModelError::NotFound => (StatusCode::NOT_FOUND, "model not found"),
        llm_models::DeleteAdminLlmModelError::IsDefault => {
            (StatusCode::BAD_REQUEST, "cannot delete the current default; set a different default first")
        }
        llm_models::DeleteAdminLlmModelError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete model"),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    llm_models::set_default_admin_llm_model(&state.pool, id).await.map_err(|e| match e {
        llm_models::SetDefaultAdminLlmModelError::NotFound => (StatusCode::NOT_FOUND, "model not found"),
        llm_models::SetDefaultAdminLlmModelError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to set default"),
    })?;
    Ok(StatusCode::NO_CONTENT)
}
```

Edit `backend/src/routes/mod.rs`:

```rust
pub mod auth;
pub mod llm_models;
pub mod sessions;
pub mod settings;
```

Edit `backend/src/app.rs` — add the import and replace the old LLM settings route:

```rust
use crate::routes::llm_models as llm_models_routes;
```

(add alongside the other `use crate::routes::...` lines)

Replace:
```rust
        .route(
            "/api/admin/settings/llm",
            get(settings_routes::get_llm_settings).put(settings_routes::put_llm_settings),
        )
```
with:
```rust
        .route(
            "/api/admin/settings/llm/models",
            get(llm_models_routes::list_admin_models).post(llm_models_routes::create_admin_model),
        )
        .route(
            "/api/admin/settings/llm/models/:id",
            put(llm_models_routes::update_admin_model).delete(llm_models_routes::delete_admin_model),
        )
        .route(
            "/api/admin/settings/llm/models/:id/default",
            put(llm_models_routes::set_default_admin_model),
        )
```

And update the top import line:
```rust
use axum::{routing::{delete, get, post}, Router};
```
to:
```rust
use axum::{routing::{delete, get, post, put}, Router};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test admin_llm_models_routes`
Expected: all 6 tests pass.

- [ ] **Step 5: Run the previously-passing settings tests to confirm nothing broke**

Run: `cd backend && cargo test --test settings_routes`
Expected: the two `llm`-specific tests that referenced the now-removed `/api/admin/settings/llm` endpoint (`get_llm_settings_returns_404_when_unconfigured`, `non_admin_is_forbidden_from_reading_settings`, `admin_can_save_and_then_read_back_masked_llm_settings`, `put_rejects_an_unknown_provider`, `put_without_api_key_keeps_the_existing_key`) will now fail to compile/fail at runtime since that route no longer exists — remove those five test functions from `backend/tests/settings_routes.rs` (the embedding-only tests — `non_admin_is_forbidden_from_reading_embedding_settings`, `admin_can_save_and_then_read_back_masked_embedding_settings` — stay, unaffected). Re-run: `cargo test --test settings_routes` — expected: the remaining 2 tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/routes/llm_models.rs backend/src/routes/mod.rs backend/src/routes/settings.rs backend/src/app.rs backend/tests/admin_llm_models_routes.rs backend/tests/settings_routes.rs
git commit -m "feat: add admin CRUD endpoints for LLM model catalog"
```

---

### Task 4: User selection endpoints

**Files:**
- Modify: `backend/src/routes/llm_models.rs`
- Modify: `backend/src/app.rs`
- Test: `backend/tests/user_llm_selection_routes.rs`

**Interfaces:**
- Consumes: `ALLOWED_PROVIDERS` and `settings::llm_models::*` from Task 3/1; `validate_model_config` from Task 2.
- Produces: `get_user_models`, `put_user_selection` handlers — nothing downstream in this plan depends on these directly (Task 6 calls them over HTTP, not as Rust functions).

- [ ] **Step 1: Write the failing tests**

Create `backend/tests/user_llm_selection_routes.rs`:

```rust
mod support;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: support::TEST_SETTINGS_KEY,
        mqtt_broker_host: support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: support::TEST_MQTT_BROKER_PORT,
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

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
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

/// Registers, promotes to platform admin, then logs in — login must happen *after* promotion,
/// since permissions are baked into the JWT at login time (same helper as
/// `tests/admin_llm_models_routes.rs` and `tests/settings_routes.rs`).
async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    make_platform_admin(pool, email).await;
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

#[sqlx::test]
async fn get_models_returns_the_admin_list_and_no_selection_initially(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user1@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["admin_models"].as_array().unwrap().len(), 0);
    assert!(body["selection"].is_null());
}

#[sqlx::test]
async fn a_user_can_select_an_admin_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Claude Sonnet", "provider": "fake", "model_id": "", "api_key": "", "base_url": null }),
        Some(&admin_token),
    )
    .await;
    let model_id = create_body["id"].as_str().unwrap();

    let user_token = register_and_login(router.clone(), "user2@example.com").await;
    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "admin", "admin_model_id": model_id }),
        Some(&user_token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, models_body) = json_request(router, "GET", "/api/llm/models", Value::Null, Some(&user_token)).await;
    assert_eq!(models_body["selection"]["kind"], "admin");
    assert_eq!(models_body["selection"]["admin_model_id"], model_id);
}

#[sqlx::test]
async fn selecting_an_unknown_admin_model_returns_404(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user3@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "admin", "admin_model_id": "00000000-0000-0000-0000-000000000000" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_user_can_save_a_valid_custom_fake_selection(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user4@example.com").await;

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "custom", "label": "My key", "provider": "fake", "model_id": "", "api_key": "", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, models_body) = json_request(router, "GET", "/api/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(models_body["selection"]["kind"], "custom");
    assert_eq!(models_body["selection"]["label"], "My key");
}

#[sqlx::test]
async fn custom_selection_rejects_an_unknown_provider(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user5@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "custom", "label": "X", "provider": "not-real", "model_id": "x", "api_key": "key", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test user_llm_selection_routes`
Expected: fails to compile — `/api/llm/models` and `/api/llm/selection` don't exist yet.

- [ ] **Step 3: Write the implementation**

Edit `backend/src/routes/llm_models.rs`, append at the end of the file:

```rust
#[derive(Serialize)]
pub struct UserModelOption {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub model_id: String,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UserSelectionResponse {
    Admin { admin_model_id: Uuid },
    Custom { label: String, provider: String, model_id: String, api_key_masked: String, base_url: Option<String> },
}

#[derive(Serialize)]
pub struct UserModelsResponse {
    pub admin_models: Vec<UserModelOption>,
    pub selection: Option<UserSelectionResponse>,
}

pub async fn get_user_models(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<UserModelsResponse>, (StatusCode, &'static str)> {
    let admin_models = llm_models::list_admin_llm_models(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load models"))?
        .into_iter()
        .map(|m| UserModelOption { id: m.id, label: m.label, provider: m.provider, model_id: m.model_id })
        .collect();

    let selection_row = llm_models::get_user_llm_selection(&state.pool, claims.sub)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load selection"))?;

    let selection = match selection_row {
        Some(row) => {
            if let Some(admin_model_id) = row.admin_model_id {
                Some(UserSelectionResponse::Admin { admin_model_id })
            } else if let Some(provider) = row.custom_provider {
                let api_key = settings::crypto::decrypt(
                    &state.settings_key,
                    row.custom_api_key_encrypted.as_deref().unwrap_or_default(),
                )
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key"))?;
                Some(UserSelectionResponse::Custom {
                    label: row.custom_label.unwrap_or_default(),
                    provider,
                    model_id: row.custom_model_id.unwrap_or_default(),
                    api_key_masked: settings::mask_api_key(&api_key),
                    base_url: row.custom_base_url,
                })
            } else {
                None
            }
        }
        None => None,
    };

    Ok(Json(UserModelsResponse { admin_models, selection }))
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectionRequest {
    Admin { admin_model_id: Uuid },
    Custom { label: String, provider: String, model_id: String, api_key: String, base_url: Option<String> },
}

fn provider_kind_from_str(s: &str) -> Option<crate::llm::ProviderKind> {
    match s {
        "anthropic" => Some(crate::llm::ProviderKind::Anthropic),
        "openai" => Some(crate::llm::ProviderKind::OpenAi),
        "gemini" => Some(crate::llm::ProviderKind::Gemini),
        "fake" => Some(crate::llm::ProviderKind::Fake),
        _ => None,
    }
}

/// Unlike every other handler in this file, errors here carry a dynamic `String` rather than
/// `&'static str` — the BYOK validation failure needs to surface the provider's actual error
/// text (spec requirement: "the provider's actual error message... surfaced inline on the
/// form"), which structurally cannot be a `&'static str`.
pub async fn put_user_selection(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<SelectionRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    match req {
        SelectionRequest::Admin { admin_model_id } => {
            let exists = llm_models::get_admin_llm_model(&state.pool, admin_model_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to look up model".to_string()))?
                .is_some();
            if !exists {
                return Err((StatusCode::NOT_FOUND, "model not found".to_string()));
            }
            llm_models::set_user_llm_selection_admin(&state.pool, claims.sub, admin_model_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save selection".to_string()))?;
        }
        SelectionRequest::Custom { label, provider, model_id, api_key, base_url } => {
            if !ALLOWED_PROVIDERS.contains(&provider.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "unknown provider (expected anthropic, openai, gemini, or fake)".to_string(),
                ));
            }
            let is_fake = provider == "fake";
            if !is_fake && model_id.trim().is_empty() {
                return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider".to_string()));
            }
            if !is_fake && api_key.trim().is_empty() {
                return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider".to_string()));
            }

            let provider_kind = provider_kind_from_str(&provider)
                .ok_or((StatusCode::BAD_REQUEST, "unknown provider".to_string()))?;
            let model_config = crate::llm::ModelConfig {
                provider: provider_kind,
                model_id: model_id.clone(),
                api_key: api_key.clone(),
                base_url: base_url.clone(),
            };
            crate::llm::validate_model_config(model_config, state.http_client.clone())
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

            llm_models::set_user_llm_selection_custom(
                &state.pool,
                claims.sub,
                llm_models::CustomLlmSelection {
                    label: &label,
                    provider: &provider,
                    model_id: &model_id,
                    api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &api_key),
                    base_url: base_url.as_deref(),
                },
            )
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save selection".to_string()))?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
```

Edit `backend/src/app.rs`, add two routes after the admin LLM model routes:

```rust
        .route("/api/llm/models", get(llm_models_routes::get_user_models))
        .route("/api/llm/selection", put(llm_models_routes::put_user_selection))
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test user_llm_selection_routes`
Expected: all 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add backend/src/routes/llm_models.rs backend/src/app.rs backend/tests/user_llm_selection_routes.rs
git commit -m "feat: add user-facing LLM model selection endpoints with BYOK validation"
```

---

### Task 5: Worker resolution

**Files:**
- Modify: `backend/src/bootstrap/providers.rs`
- Modify: `backend/src/bootstrap/mod.rs`
- Modify: `backend/src/worker.rs`
- Test: `backend/tests/llm_provider_resolution.rs`

**Interfaces:**
- Consumes: `settings::llm_models::*` from Task 1.
- Produces: `pub async fn resolve_llm_model_config(pool: &PgPool, user_id: Uuid, settings_key: &[u8; 32]) -> ModelConfig` and `pub async fn build_llm_provider_for_user(pool: &PgPool, user_id: Uuid, settings_key: &[u8; 32], http_client: reqwest::Client) -> Arc<dyn LlmProvider>` — the worker's own call site is the only consumer within this plan, but `resolve_llm_model_config` is `pub` specifically so tests can assert on the resolved config's fields without going through the opaque trait object `build_llm_provider_for_user` returns.

- [ ] **Step 1: Write the failing tests**

Create `backend/tests/llm_provider_resolution.rs`:

```rust
use nomi_orchestrator::bootstrap::resolve_llm_model_config;
use nomi_orchestrator::llm::ProviderKind;
use nomi_orchestrator::settings::crypto;
use nomi_orchestrator::settings::llm_models::{
    create_admin_llm_model, delete_admin_llm_model, set_user_llm_selection_admin, set_user_llm_selection_custom,
    CustomLlmSelection, NewAdminLlmModel,
};
use sqlx::PgPool;
use uuid::Uuid;

const SETTINGS_KEY: [u8; 32] = [7u8; 32];

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn resolves_to_the_users_admin_selection_when_set(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Picked",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-picked"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    set_user_llm_selection_admin(&pool, user_id, model.id).await.unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Anthropic);
    assert_eq!(config.model_id, "claude-haiku-4-5");
    assert_eq!(config.api_key, "sk-picked");
}

#[sqlx::test]
async fn resolves_to_the_users_custom_selection_when_set(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "Mine",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-mine"),
            base_url: Some("https://api.openai.com/v1"),
        },
    )
    .await
    .unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::OpenAi);
    assert_eq!(config.model_id, "gpt-4o");
    assert_eq!(config.api_key, "sk-mine");
    assert_eq!(config.base_url, Some("https://api.openai.com/v1".to_string()));
}

#[sqlx::test]
async fn falls_back_to_the_admin_default_when_the_user_has_no_selection(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Gemini);
    assert_eq!(config.api_key, "sk-default");
}

#[sqlx::test]
async fn falls_back_to_the_admin_default_when_the_referenced_model_was_deleted(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    let picked = create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Picked",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-picked"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    set_user_llm_selection_admin(&pool, user_id, picked.id).await.unwrap();
    delete_admin_llm_model(&pool, picked.id).await.unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Gemini);
    assert_eq!(config.api_key, "sk-default");
}

#[sqlx::test]
async fn falls_back_to_the_env_var_config_when_no_admin_models_exist_at_all(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    std::env::set_var("LLM_PROVIDER", "fake");

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Fake);

    std::env::remove_var("LLM_PROVIDER");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test llm_provider_resolution`
Expected: fails to compile — `resolve_llm_model_config` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

Edit `backend/src/bootstrap/providers.rs` — replace the entire `build_llm_provider_from_settings_or_env` function with:

```rust
pub async fn build_llm_provider_for_user(
    pool: &PgPool,
    user_id: uuid::Uuid,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn LlmProvider> {
    let model_config = resolve_llm_model_config(pool, user_id, settings_key).await;
    Arc::from(build_provider(model_config, http_client))
}

fn model_config_from_admin_model(settings_key: &[u8; 32], model: crate::settings::llm_models::AdminLlmModel) -> ModelConfig {
    let api_key = settings::crypto::decrypt(settings_key, &model.api_key_encrypted)
        .expect("failed to decrypt stored admin llm model api key");
    ModelConfig {
        provider: llm_provider_kind_from_str(&model.provider),
        model_id: model.model_id,
        api_key,
        base_url: model.base_url,
    }
}

pub async fn resolve_llm_model_config(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> ModelConfig {
    let selection = crate::settings::llm_models::get_user_llm_selection(pool, user_id)
        .await
        .expect("failed to query user_llm_selections");

    if let Some(row) = selection {
        if let Some(admin_model_id) = row.admin_model_id {
            if let Some(admin_model) = crate::settings::llm_models::get_admin_llm_model(pool, admin_model_id)
                .await
                .expect("failed to query admin_llm_models")
            {
                return model_config_from_admin_model(settings_key, admin_model);
            }
            // Referenced admin model no longer exists — fall through to the default below.
        } else if let Some(provider) = row.custom_provider {
            let api_key = settings::crypto::decrypt(
                settings_key,
                row.custom_api_key_encrypted.as_ref().expect("custom selection always carries a key"),
            )
            .expect("failed to decrypt stored custom api key");
            return ModelConfig {
                provider: llm_provider_kind_from_str(&provider),
                model_id: row.custom_model_id.expect("custom selection always carries a model_id"),
                api_key,
                base_url: row.custom_base_url,
            };
        }
    }

    if let Some(default_model) = crate::settings::llm_models::get_default_admin_llm_model(pool)
        .await
        .expect("failed to query admin_llm_models")
    {
        return model_config_from_admin_model(settings_key, default_model);
    }

    // No admin models configured at all yet (fresh install) — same env-var fallback the
    // function this replaced always used when nothing was configured.
    let provider = llm_provider_kind_from_str(&var("LLM_PROVIDER").expect("LLM_PROVIDER must be set"));
    let (model_id, api_key) = match provider {
        ProviderKind::Fake => (String::new(), String::new()),
        _ => (
            var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
            var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
        ),
    };
    ModelConfig { provider, model_id, api_key, base_url: var("LLM_BASE_URL").ok() }
}
```

Edit `backend/src/bootstrap/mod.rs`, change:
```rust
pub use providers::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
```
to:
```rust
pub use providers::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user, resolve_llm_model_config};
```

Edit `backend/src/worker.rs`:

Change the import line:
```rust
use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
```
to:
```rust
use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};
```

Reorder the claimed-job body so `user_id` resolves before the LLM provider is built (currently the provider build comes first):

```rust
            let claimed = match queue::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "worker: failed to claim next turn job");
                    break;
                }
            };

            let user_id: Result<Uuid, sqlx::Error> =
                sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
                    .bind(claimed.sender_channel_identity_id)
                    .fetch_one(&pool)
                    .await;
            let user_id = match user_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, job_id = %claimed.id, "worker: failed to resolve user_id for claimed job");
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };

            let provider = build_llm_provider_for_user(&pool, user_id, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
```

(this replaces both the old `let provider = ...` / `let embedding_provider = ...` block AND the old `let user_id = ...` block that used to come after it — the rest of the loop body, from `let result = crate::turn::process_turn(...)` onward, is unchanged)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test llm_provider_resolution`
Expected: all 5 tests pass.

- [ ] **Step 5: Confirm the backend still builds and the full suite passes**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass (this catches any other reference to the now-removed `build_llm_provider_from_settings_or_env` name).

- [ ] **Step 6: Commit**

```bash
git add backend/src/bootstrap/providers.rs backend/src/bootstrap/mod.rs backend/src/worker.rs backend/tests/llm_provider_resolution.rs
git commit -m "feat: resolve the LLM provider per-user instead of one global row"
```

---

### Task 6: Chat page picker

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`

**Interfaces:**
- Consumes: `GET /api/llm/models`, `PUT /api/llm/selection` from Task 4.
- Produces: nothing further downstream in this plan.

No new automated test in this task — Svelte page behavior in this codebase is proven through Playwright (Task 8), not a component-test framework. Verified by `npm run check`.

- [ ] **Step 1: Add the new types**

Edit `frontend/src/lib/types.ts`, add at the end:

```ts
export interface LlmAdminModelOption {
	id: string;
	label: string;
	provider: string;
	model_id: string;
}

export type LlmUserSelection =
	| { kind: 'admin'; admin_model_id: string }
	| { kind: 'custom'; label: string; provider: string; model_id: string; api_key_masked: string; base_url: string | null };

export interface LlmModelsResponse {
	admin_models: LlmAdminModelOption[];
	selection: LlmUserSelection | null;
}
```

- [ ] **Step 2: Load the models and add the selection actions**

Edit `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` — replace the whole file:

```ts
import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LlmModelsResponse, MessageItem } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);

	if (response.status === 404) {
		throw redirect(303, '/');
	}
	if (!response.ok) {
		throw error(response.status, 'Could not load this chat.');
	}

	const { messages } = (await response.json()) as { messages: MessageItem[] };

	const modelsResponse = await apiFetch(fetch, cookies, '/api/llm/models');
	const models: LlmModelsResponse = modelsResponse.ok
		? ((await modelsResponse.json()) as LlmModelsResponse)
		: { admin_models: [], selection: null };

	return { messages, models };
};

export const actions: Actions = {
	default: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: 'Message cannot be empty.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to send message.' });
		}

		const { user_message } = (await response.json()) as { user_message: MessageItem };

		return { user_message };
	},

	selectAdminModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const adminModelId = data.get('admin_model_id');

		if (typeof adminModelId !== 'string') {
			return fail(400, { modelError: 'Invalid model selection.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({ kind: 'admin', admin_model_id: adminModelId }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { modelError: message || 'Failed to select model.' });
		}

		return { modelSelected: true };
	},

	selectCustomModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const label = data.get('label');
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');

		if (
			typeof label !== 'string' ||
			typeof provider !== 'string' ||
			typeof modelId !== 'string' ||
			typeof apiKey !== 'string' ||
			!label.trim() ||
			!provider.trim()
		) {
			return fail(400, { modelError: 'Label and provider are required.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({
				kind: 'custom',
				label,
				provider,
				model_id: modelId,
				api_key: apiKey,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { modelError: message || 'Failed to validate and save this model.' });
		}

		return { modelSelected: true };
	},
};
```

- [ ] **Step 3: Add the picker UI**

Edit `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte` — replace the whole file:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let pendingReply = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);
	let modelPickerOpen = $state(false);
	let showCustomForm = $state(false);

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	const activeModelLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return 'Default model';
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? 'Default model';
		}
		return selection.label;
	});

	onMount(() => {
		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = new WebSocket(`/chat/${page.params.sessionId}/ws`);

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
				connectionLost = false;
				if (hasConnectedBefore) {
					invalidateAll();
				}
				hasConnectedBefore = true;
			});

			socket.addEventListener('message', (event) => {
				let envelope: { kind: string };
				try {
					envelope = JSON.parse(event.data);
				} catch {
					return;
				}
				if (envelope.kind === 'Delta') {
					pendingReply = true;
				} else if (envelope.kind === 'TurnCompleted') {
					pendingReply = false;
					turnError = false;
					invalidateAll();
				} else if (envelope.kind === 'TurnFailed') {
					pendingReply = false;
					turnError = true;
				}
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) {
					connectionLost = true;
					return;
				}
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});
</script>

<div class="flex h-full flex-col">
	<div class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each data.messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if pendingReply}
			<div class="flex justify-start">
				<div class="max-w-md rounded-2xl bg-neutral-100 px-4 py-2 text-neutral-400">Typing…</div>
			</div>
		{/if}
		{#if turnError}
			<p class="text-center text-sm text-red-600">Something went wrong — try sending again.</p>
		{/if}
		{#if form?.error}
			<p class="text-center text-sm text-red-600">{form.error}</p>
		{/if}
		{#if connectionLost}
			<p class="text-center text-sm text-red-600">Couldn't connect to this chat — try reloading the page.</p>
		{/if}
	</div>

	<form
		method="POST"
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: true });
			};
		}}
		class="border-t border-neutral-200 bg-white px-6 py-4"
	>
		<div class="mb-2 flex justify-end">
			<div class="relative">
				<button
					type="button"
					onclick={() => (modelPickerOpen = !modelPickerOpen)}
					class="rounded-full border border-neutral-300 px-3 py-1 text-xs text-neutral-600 hover:bg-neutral-50"
				>
					{activeModelLabel}
				</button>
				{#if modelPickerOpen}
					<div class="absolute bottom-full right-0 mb-2 w-72 rounded-xl border border-neutral-200 bg-white p-3 shadow-lg">
						{#if form?.modelError}
							<p class="mb-2 text-xs text-red-600">{form.modelError}</p>
						{/if}
						<p class="mb-1 text-xs font-medium text-neutral-500">Available models</p>
						{#each data.models.admin_models as model (model.id)}
							<form
								method="POST"
								action="?/selectAdminModel"
								use:enhance={() => {
									return async ({ update }) => {
										await update();
										modelPickerOpen = false;
									};
								}}
							>
								<input type="hidden" name="admin_model_id" value={model.id} />
								<button
									type="submit"
									class="block w-full rounded-lg px-2 py-1 text-left text-sm hover:bg-neutral-100 {data.models
										.selection?.kind === 'admin' && data.models.selection.admin_model_id === model.id
										? 'font-semibold'
										: ''}"
								>
									{model.label}
								</button>
							</form>
						{/each}

						<p class="mt-3 mb-1 text-xs font-medium text-neutral-500">Your own key</p>
						{#if data.models.selection?.kind === 'custom'}
							<p class="px-2 py-1 text-sm font-semibold">
								{data.models.selection.label} ({data.models.selection.api_key_masked})
							</p>
						{/if}
						{#if showCustomForm}
							<form
								method="POST"
								action="?/selectCustomModel"
								use:enhance={() => {
									return async ({ update }) => {
										await update();
										modelPickerOpen = true;
										showCustomForm = false;
									};
								}}
								class="mt-1 space-y-1"
							>
								<input name="label" type="text" placeholder="Label" required class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<select name="provider" required class="w-full rounded border border-neutral-300 px-2 py-1 text-sm">
									<option value="anthropic">Anthropic</option>
									<option value="openai">OpenAI</option>
									<option value="gemini">Gemini</option>
									<option value="fake">Fake (testing)</option>
								</select>
								<input name="model_id" type="text" placeholder="Model ID" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<input name="api_key" type="password" placeholder="API key" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<input name="base_url" type="text" placeholder="Base URL (optional)" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<button type="submit" class="w-full rounded bg-neutral-900 px-2 py-1 text-sm text-white">Save & validate</button>
							</form>
						{:else}
							<button
								type="button"
								onclick={() => (showCustomForm = true)}
								class="mt-1 block w-full rounded-lg px-2 py-1 text-left text-sm text-neutral-600 hover:bg-neutral-100"
							>
								+ Use your own API key
							</button>
						{/if}
					</div>
				{/if}
			</div>
		</div>
		<div class="flex items-center gap-2 rounded-full border border-neutral-300 px-4 py-2">
			<input
				name="text"
				type="text"
				placeholder="Ask me anything..."
				required
				class="flex-1 border-none bg-transparent outline-none"
			/>
			<button
				type="submit"
				class="rounded-full bg-neutral-900 px-4 py-1.5 text-sm font-medium text-white hover:bg-neutral-800"
			>
				Send
			</button>
		</div>
	</form>
</div>
```

- [ ] **Step 4: Verify types check cleanly**

Run: `cd frontend && npm run check`
Expected: no new errors.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/lib/types.ts "frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts" "frontend/src/routes/(app)/chat/[sessionId]/+page.svelte"
git commit -m "feat: add the model picker to the chat input area"
```

---

### Task 7: Admin model list page

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/routes/admin/(protected)/settings/llm/+page.server.ts`
- Modify: `frontend/src/routes/admin/(protected)/settings/llm/+page.svelte`

**Interfaces:**
- Consumes: the admin CRUD endpoints from Task 3.
- Produces: nothing further downstream in this plan.

No new automated test in this task, same reasoning as Task 6 — verified by `npm run check`; Task 8's e2e test exercises the create flow.

- [ ] **Step 1: Add the admin model type**

Edit `frontend/src/lib/types.ts`, add at the end:

```ts
export interface AdminLlmModel {
	id: string;
	label: string;
	provider: string;
	model_id: string;
	base_url: string | null;
	is_default: boolean;
	api_key_masked: string;
}
```

- [ ] **Step 2: Rewrite the page as a list with create/update/delete/set-default actions**

Edit `frontend/src/routes/admin/(protected)/settings/llm/+page.server.ts` — replace the whole file:

```ts
import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminLlmModel } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/models');
	if (!response.ok) {
		return { models: [] as AdminLlmModel[] };
	}
	const models = (await response.json()) as AdminLlmModel[];
	return { models };
};

function readForm(data: FormData) {
	const label = data.get('label');
	const provider = data.get('provider');
	const modelId = data.get('model_id');
	const apiKey = data.get('api_key');
	const baseUrl = data.get('base_url');
	return {
		label: typeof label === 'string' ? label : '',
		provider: typeof provider === 'string' ? provider : '',
		modelId: typeof modelId === 'string' ? modelId : '',
		apiKey: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
		baseUrl: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
	};
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const { label, provider, modelId, apiKey, baseUrl } = readForm(await request.formData());
		if (!label || !provider) {
			return fail(400, { error: 'Label and provider are required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/models', {
			method: 'POST',
			body: JSON.stringify({ label, provider, model_id: modelId, api_key: apiKey ?? '', base_url: baseUrl }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to create model.' });
		}
		return { success: true };
	},

	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const { label, provider, modelId, apiKey, baseUrl } = readForm(data);
		if (typeof id !== 'string' || !label || !provider) {
			return fail(400, { error: 'Label and provider are required.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}`, {
			method: 'PUT',
			body: JSON.stringify({ label, provider, model_id: modelId, api_key: apiKey, base_url: baseUrl }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update model.' });
		}
		return { success: true };
	},

	delete: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: 'Invalid model.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}`, { method: 'DELETE' });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to delete model.' });
		}
		return { success: true };
	},

	setDefault: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: 'Invalid model.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}/default`, { method: 'PUT' });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to set default.' });
		}
		return { success: true };
	},
};
```

- [ ] **Step 3: Rewrite the page markup as a list with an add form and an inline edit toggle per row**

Edit `frontend/src/routes/admin/(protected)/settings/llm/+page.svelte` — replace the whole file:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	let showCreateForm = $state(false);
	let editingId = $state<string | null>(null);
</script>

<h1 class="text-2xl font-semibold">LLM models</h1>

{#if form?.error}
	<p class="mt-2 text-sm text-red-600">{form.error}</p>
{/if}

<div class="mt-6 space-y-3">
	{#each data.models as model (model.id)}
		<div class="rounded-2xl border border-neutral-200 bg-white p-4">
			{#if editingId === model.id}
				<form
					method="POST"
					action="?/update"
					use:enhance={() => {
						return async ({ update }) => {
							await update();
							editingId = null;
						};
					}}
					class="space-y-2"
				>
					<input type="hidden" name="id" value={model.id} />
					<input name="label" type="text" value={model.label} required class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<select name="provider" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm">
						<option value="anthropic" selected={model.provider === 'anthropic'}>Anthropic</option>
						<option value="openai" selected={model.provider === 'openai'}>OpenAI</option>
						<option value="gemini" selected={model.provider === 'gemini'}>Gemini</option>
						<option value="fake" selected={model.provider === 'fake'}>Fake (testing)</option>
					</select>
					<input name="model_id" type="text" value={model.model_id} class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<input name="base_url" type="text" value={model.base_url ?? ''} placeholder="Base URL (optional)" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<input name="api_key" type="password" placeholder={`Leave blank to keep ${model.api_key_masked}`} class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<div class="flex gap-2">
						<button type="submit" class="rounded bg-neutral-900 px-3 py-1 text-sm text-white">Save</button>
						<button type="button" onclick={() => (editingId = null)} class="rounded border border-neutral-300 px-3 py-1 text-sm">Cancel</button>
					</div>
				</form>
			{:else}
				<div class="flex items-center justify-between">
					<div>
						<p class="font-medium">
							{model.label}
							{#if model.is_default}
								<span class="ml-2 rounded-full bg-neutral-900 px-2 py-0.5 text-xs text-white">Default</span>
							{/if}
						</p>
						<p class="text-sm text-neutral-500">{model.provider} · {model.model_id} · {model.api_key_masked}</p>
					</div>
					<div class="flex gap-2">
						<button type="button" onclick={() => (editingId = model.id)} class="text-sm text-neutral-600 hover:underline">Edit</button>
						{#if !model.is_default}
							<form method="POST" action="?/setDefault" use:enhance>
								<input type="hidden" name="id" value={model.id} />
								<button type="submit" class="text-sm text-neutral-600 hover:underline">Set default</button>
							</form>
							<form method="POST" action="?/delete" use:enhance>
								<input type="hidden" name="id" value={model.id} />
								<button type="submit" class="text-sm text-red-600 hover:underline">Delete</button>
							</form>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	{/each}
</div>

<div class="mt-6">
	{#if showCreateForm}
		<form
			method="POST"
			action="?/create"
			use:enhance={() => {
				return async ({ update }) => {
					await update({ reset: true });
					showCreateForm = false;
				};
			}}
			class="max-w-lg space-y-4 rounded-2xl border border-neutral-200 bg-white p-6"
		>
			<div>
				<label for="label" class="block text-sm font-medium text-neutral-700">Label</label>
				<input id="label" name="label" type="text" required class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<div>
				<label for="provider" class="block text-sm font-medium text-neutral-700">Provider</label>
				<select id="provider" name="provider" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2">
					<option value="anthropic">Anthropic</option>
					<option value="openai">OpenAI</option>
					<option value="gemini">Gemini</option>
					<option value="fake">Fake (testing)</option>
				</select>
			</div>
			<div>
				<label for="model_id" class="block text-sm font-medium text-neutral-700">Model ID</label>
				<input id="model_id" name="model_id" type="text" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<div>
				<label for="base_url" class="block text-sm font-medium text-neutral-700">Base URL (optional)</label>
				<input id="base_url" name="base_url" type="text" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<div>
				<label for="api_key" class="block text-sm font-medium text-neutral-700">API key</label>
				<input id="api_key" name="api_key" type="password" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<button type="submit" class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800">
				Add model
			</button>
		</form>
	{:else}
		<button
			onclick={() => (showCreateForm = true)}
			class="rounded-lg border border-neutral-300 px-4 py-2 text-sm font-medium hover:bg-neutral-50"
		>
			+ Add model
		</button>
	{/if}
</div>
```

- [ ] **Step 4: Verify types check cleanly**

Run: `cd frontend && npm run check`
Expected: no new errors.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/lib/types.ts "frontend/src/routes/admin/(protected)/settings/llm/+page.server.ts" "frontend/src/routes/admin/(protected)/settings/llm/+page.svelte"
git commit -m "feat: rewrite the admin LLM settings page as a model list"
```

---

### Task 8: End-to-end proof

**Files:**
- Create: `frontend/e2e/model-picker.e2e.ts`

**Interfaces:**
- Consumes: everything from Tasks 1-7, exercised through a real browser against the real backend/worker/EMQX/Postgres (same setup as the existing `conversation.e2e.ts`).
- Produces: nothing further downstream — this is the plan's last task.

**Before running this task's tests:** same precondition as `conversation.e2e.ts` — `cargo run --bin worker` (or rely on the now-default embedded worker inside `cargo run --bin nomi-orchestrator`, per the earlier `RUN_WORKER_INLINE` change already on `main`) must be processing turns, and EMQX/Postgres must be up.

**Testing note:** the fake provider returns identical canned text regardless of which model is selected, so this test cannot prove "a different model produced a different reply." It proves the parts that are actually verifiable: the admin can create a model and it appears in the picker; selecting it updates the picker's displayed label; a BYOK submission using the fake provider validates and saves successfully and the picker reflects it.

- [ ] **Step 1: Write the test**

Create `frontend/e2e/model-picker.e2e.ts`:

```ts
import { expect, test } from '@playwright/test';

async function registerAdminAndLogin(page: import('@playwright/test').Page): Promise<void> {
	const email = `admin-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;
	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');
}

async function startChat(page: import('@playwright/test').Page): Promise<void> {
	await page.getByRole('button', { name: 'New Chat', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);
}

test('an admin-created model appears in the picker and can be selected', async ({ page }) => {
	await registerAdminAndLogin(page);

	await page.goto('/admin/settings/llm');
	await page.getByRole('button', { name: '+ Add model' }).click();
	await page.getByLabel('Label').fill('Second Model');
	await page.getByLabel('Provider').selectOption('fake');
	await page.getByRole('button', { name: 'Add model' }).click();
	await expect(page.getByText('Second Model')).toBeVisible();

	await page.goto('/');
	await startChat(page);

	await page.getByRole('button', { name: 'Default model' }).click();
	await page.getByRole('button', { name: 'Second Model' }).click();
	await expect(page.getByRole('button', { name: 'Second Model' })).toBeVisible();
});

test('a user can bring their own key using the fake provider and see it become active', async ({ page }) => {
	await registerAdminAndLogin(page);
	await startChat(page);

	await page.getByRole('button', { name: 'Default model' }).click();
	await page.getByRole('button', { name: '+ Use your own API key' }).click();
	await page.getByPlaceholder('Label').fill('My fake key');
	await page.getByPlaceholder('Model ID').fill('');
	await page.getByRole('button', { name: 'Save & validate' }).click();

	await expect(page.getByRole('button', { name: 'My fake key' })).toBeVisible();
});
```

- [ ] **Step 2: Run it**

Ensure the same preconditions as `conversation.e2e.ts` (Postgres/EMQX up, backend running with the worker processing — either standalone or embedded), then run: `cd frontend && npm run test:e2e`
Expected: both new tests pass, plus the full pre-existing suite still passes.

- [ ] **Step 3: Commit**

```bash
git add frontend/e2e/model-picker.e2e.ts
git commit -m "test: prove the LLM model picker and BYOK flow end to end"
```

---

## Out of Scope (unchanged from the design spec)

- Per-message model override.
- Multiple saved BYOK configs per user.
- Per-user embedding model selection.
- Notifying a user when the admin model they were using gets deleted.
- Rate limiting, usage tracking, or cost accounting for BYOK usage.
