# App-Configurable LLM & Embedding Providers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move LLM/embedding provider configuration from env vars into a DB-backed admin API with live runtime reload, plus a frontend admin area (separate login, guarded shell, LLM settings page) to manage it.

**Architecture:** A new `provider_settings` Postgres table stores one row each for `llm`/`embedding` config, with the API key AES-256-GCM-encrypted at rest under a master key from `SETTINGS_ENCRYPTION_KEY`. `AppState` holds each provider behind `Arc<RwLock<Arc<dyn Provider>>>` so an admin API call can swap the live provider without a restart. A new SvelteKit `admin/` route tree (separate login, permission-gated layout) calls that API.

**Tech Stack:** Rust/axum/sqlx backend (existing), `aes-gcm` crate (new), SvelteKit frontend (existing), `postgres` npm package (new, e2e-only).

## Global Constraints

- Provider config scope is global (one config for the whole app), not per-organization.
- `GET` settings endpoints never return the real API key — only a masked form (last 4 chars, e.g. `...ab12`).
- `PUT` with an omitted/empty `api_key` keeps the existing stored key; only a non-empty `api_key` replaces it.
- On boot, if no `provider_settings` row exists for a type, fall back to today's env-var construction — the DB does not need to be pre-seeded.
- `SETTINGS_ENCRYPTION_KEY` is a required env var: 64 hex characters (32 bytes). (Deviation from the design doc's "base64" wording — hex reuses the `hex` crate already in `Cargo.toml`, avoiding a new dependency; functionally equivalent.)
- Admin settings routes require the `nomi:admin:system_config:[...]` permission, granted to users with `is_platform_admin = true`.
- Provider swap on save must be visible to the very next request with no process restart.

---

## Task 1: AES-GCM crypto module for stored API keys

**Files:**
- Modify: `backend/Cargo.toml`
- Create: `backend/src/settings/mod.rs` (module stub for now — `pub mod crypto;`)
- Create: `backend/src/settings/crypto.rs`
- Modify: `backend/src/lib.rs`
- Test: inline `#[cfg(test)]` module in `backend/src/settings/crypto.rs`

**Interfaces:**
- Produces: `pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Vec<u8>` — returns 12-byte nonce prepended to ciphertext.
- Produces: `pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<String, CryptoError>`.
- Produces: `pub fn parse_key(hex_str: &str) -> Option<[u8; 32]>` — decodes a 64-hex-char string into a 32-byte key.
- Produces: `pub enum CryptoError { DecryptionFailed }` (implements `std::error::Error` via `thiserror`).

- [ ] **Step 1: Add the `aes-gcm` dependency**

In `backend/Cargo.toml`, add this line under `[dependencies]` (alongside the other crypto-ish deps like `argon2`/`sha2`):

```toml
aes-gcm = "0.10"
```

- [ ] **Step 2: Write the failing tests**

Create `backend/src/settings/crypto.rs`:

```rust
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("decryption failed")]
    DecryptionFailed,
}

pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("aes-gcm encryption does not fail for valid inputs");
    let mut out = nonce_bytes.to_vec();
    out.extend_from_slice(&ciphertext);
    out
}

pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<String, CryptoError> {
    if data.len() < 12 {
        return Err(CryptoError::DecryptionFailed);
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| CryptoError::DecryptionFailed)?;
    String::from_utf8(plaintext).map_err(|_| CryptoError::DecryptionFailed)
}

pub fn parse_key(hex_str: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(hex_str).ok()?;
    bytes.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7u8; 32];

    #[test]
    fn encrypt_decrypt_roundtrip_recovers_the_plaintext() {
        let ciphertext = encrypt(&KEY, "sk-super-secret-key");
        let plaintext = decrypt(&KEY, &ciphertext).unwrap();
        assert_eq!(plaintext, "sk-super-secret-key");
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext() {
        let mut ciphertext = encrypt(&KEY, "sk-super-secret-key");
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        assert!(matches!(decrypt(&KEY, &ciphertext), Err(CryptoError::DecryptionFailed)));
    }

    #[test]
    fn decrypt_rejects_the_wrong_key() {
        let ciphertext = encrypt(&KEY, "sk-super-secret-key");
        let wrong_key = [9u8; 32];
        assert!(matches!(decrypt(&wrong_key, &ciphertext), Err(CryptoError::DecryptionFailed)));
    }

    #[test]
    fn parse_key_accepts_64_hex_chars() {
        let hex_str = "07".repeat(32);
        assert_eq!(parse_key(&hex_str), Some(KEY));
    }

    #[test]
    fn parse_key_rejects_wrong_length() {
        assert_eq!(parse_key("07"), None);
    }

    #[test]
    fn parse_key_rejects_non_hex() {
        assert_eq!(parse_key(&"zz".repeat(32)), None);
    }
}
```

Create `backend/src/settings/mod.rs`:

```rust
pub mod crypto;
```

In `backend/src/lib.rs`, add `pub mod settings;` alongside the other `pub mod` lines.

- [ ] **Step 2b: Run tests to verify they fail (crate doesn't compile yet without the dependency added)**

Run: `cd backend && cargo test --lib settings::crypto`
Expected: compiles and passes once Step 1's dependency is in place — if it fails to compile, confirm `aes-gcm = "0.10"` was added correctly.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd backend && cargo test --lib settings::crypto`
Expected: 6 tests pass (`encrypt_decrypt_roundtrip_recovers_the_plaintext`, `decrypt_rejects_tampered_ciphertext`, `decrypt_rejects_the_wrong_key`, `parse_key_accepts_64_hex_chars`, `parse_key_rejects_wrong_length`, `parse_key_rejects_non_hex`).

- [ ] **Step 4: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/settings/mod.rs backend/src/settings/crypto.rs backend/src/lib.rs
git commit -m "feat: add AES-256-GCM crypto module for encrypting stored provider API keys"
```

---

## Task 2: `provider_settings` table and settings data-access module

**Files:**
- Create: `backend/migrations/0010_provider_settings.sql`
- Modify: `backend/src/settings/mod.rs`
- Test: `backend/tests/provider_settings.rs`

**Interfaces:**
- Consumes: nothing new (uses `sqlx::PgPool`, `uuid::Uuid` — both existing deps).
- Produces: `pub struct ProviderSettingsRow { pub setting_type: String, pub provider: String, pub model_id: String, pub api_key_encrypted: Vec<u8>, pub base_url: Option<String> }` (derives `sqlx::FromRow`, `Debug`, `Clone`).
- Produces: `pub async fn get_settings(pool: &PgPool, setting_type: &str) -> Result<Option<ProviderSettingsRow>, sqlx::Error>`.
- Produces: `pub struct UpsertInput<'a> { pub setting_type: &'a str, pub provider: &'a str, pub model_id: &'a str, pub api_key_encrypted: Vec<u8>, pub base_url: Option<&'a str>, pub updated_by: Uuid }`.
- Produces: `pub async fn upsert_settings(pool: &PgPool, input: UpsertInput<'_>) -> Result<(), sqlx::Error>`.
- Produces: `pub fn mask_api_key(key: &str) -> String` — `""` for an empty key, otherwise `"...{last 4 chars}"`.

- [ ] **Step 1: Write the migration**

Create `backend/migrations/0010_provider_settings.sql`:

```sql
CREATE TABLE provider_settings (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_type       TEXT NOT NULL UNIQUE CHECK (setting_type IN ('llm', 'embedding')),
    provider           TEXT NOT NULL,
    model_id           TEXT NOT NULL,
    api_key_encrypted  BYTEA NOT NULL,
    base_url           TEXT,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by         UUID NOT NULL REFERENCES users(id)
);
```

- [ ] **Step 2: Write the failing tests**

Create `backend/tests/provider_settings.rs`:

```rust
use nomi_orchestrator::settings::{get_settings, mask_api_key, upsert_settings, UpsertInput};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn get_settings_returns_none_when_no_row_exists(pool: PgPool) {
    let row = get_settings(&pool, "llm").await.unwrap();
    assert!(row.is_none());
}

#[sqlx::test]
async fn upsert_then_get_roundtrips_the_row(pool: PgPool) {
    let user_id = insert_user(&pool).await;

    upsert_settings(
        &pool,
        UpsertInput {
            setting_type: "llm",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: vec![1, 2, 3],
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();

    let row = get_settings(&pool, "llm").await.unwrap().unwrap();
    assert_eq!(row.provider, "anthropic");
    assert_eq!(row.model_id, "claude-haiku-4-5");
    assert_eq!(row.api_key_encrypted, vec![1, 2, 3]);
    assert_eq!(row.base_url, None);
}

#[sqlx::test]
async fn upsert_twice_updates_the_same_row_instead_of_inserting_a_second_one(pool: PgPool) {
    let user_id = insert_user(&pool).await;

    for model_id in ["claude-haiku-4-5", "claude-sonnet-5"] {
        upsert_settings(
            &pool,
            UpsertInput {
                setting_type: "llm",
                provider: "anthropic",
                model_id,
                api_key_encrypted: vec![1, 2, 3],
                base_url: None,
                updated_by: user_id,
            },
        )
        .await
        .unwrap();
    }

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM provider_settings WHERE setting_type = 'llm'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    let row = get_settings(&pool, "llm").await.unwrap().unwrap();
    assert_eq!(row.model_id, "claude-sonnet-5");
}

#[sqlx::test]
async fn llm_and_embedding_rows_are_independent(pool: PgPool) {
    let user_id = insert_user(&pool).await;

    upsert_settings(
        &pool,
        UpsertInput {
            setting_type: "llm",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: vec![1],
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    upsert_settings(
        &pool,
        UpsertInput {
            setting_type: "embedding",
            provider: "openai",
            model_id: "text-embedding-3-small",
            api_key_encrypted: vec![2],
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();

    assert_eq!(get_settings(&pool, "llm").await.unwrap().unwrap().provider, "anthropic");
    assert_eq!(get_settings(&pool, "embedding").await.unwrap().unwrap().provider, "openai");
}

#[test]
fn mask_api_key_shows_only_the_last_four_characters() {
    assert_eq!(mask_api_key("sk-abcdefgh1234"), "...1234");
}

#[test]
fn mask_api_key_of_an_empty_key_is_empty() {
    assert_eq!(mask_api_key(""), "");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && cargo test --test provider_settings`
Expected: FAIL — `get_settings`, `upsert_settings`, `UpsertInput`, `mask_api_key` not found.

- [ ] **Step 4: Implement**

Replace `backend/src/settings/mod.rs` with:

```rust
pub mod crypto;

use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderSettingsRow {
    pub setting_type: String,
    pub provider: String,
    pub model_id: String,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<String>,
}

pub async fn get_settings(pool: &PgPool, setting_type: &str) -> Result<Option<ProviderSettingsRow>, sqlx::Error> {
    sqlx::query_as::<_, ProviderSettingsRow>(
        "SELECT setting_type, provider, model_id, api_key_encrypted, base_url \
         FROM provider_settings WHERE setting_type = $1",
    )
    .bind(setting_type)
    .fetch_optional(pool)
    .await
}

pub struct UpsertInput<'a> {
    pub setting_type: &'a str,
    pub provider: &'a str,
    pub model_id: &'a str,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<&'a str>,
    pub updated_by: Uuid,
}

pub async fn upsert_settings(pool: &PgPool, input: UpsertInput<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO provider_settings (setting_type, provider, model_id, api_key_encrypted, base_url, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (setting_type) DO UPDATE SET \
         provider = $2, model_id = $3, api_key_encrypted = $4, base_url = $5, updated_by = $6, updated_at = now()",
    )
    .bind(input.setting_type)
    .bind(input.provider)
    .bind(input.model_id)
    .bind(input.api_key_encrypted)
    .bind(input.base_url)
    .bind(input.updated_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    let tail: String = key.chars().rev().take(4).collect::<Vec<char>>().into_iter().rev().collect();
    format!("...{tail}")
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && cargo test --test provider_settings`
Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/migrations/0010_provider_settings.sql backend/src/settings/mod.rs backend/tests/provider_settings.rs
git commit -m "feat: add provider_settings table and get/upsert/mask data-access functions"
```

---

## Task 3: Live-reloadable providers in `AppState`

**Files:**
- Modify: `backend/src/app.rs`
- Modify: `backend/src/main.rs`
- Modify: `backend/src/routes/sessions.rs` (the `state.provider.as_ref()` call site)
- Modify: `backend/tests/support/mod.rs` (add a shared test settings key)
- Modify: `backend/tests/sessions_routes.rs`, `backend/tests/auth_extractor.rs`, `backend/tests/auth_routes.rs` (update `AppState` construction)
- Test: existing test suite (no new test file — this task is a refactor validated by the existing suite continuing to pass, plus one new test proving the swap works)
- Test: `backend/tests/app_state_reload.rs` (new)

**Interfaces:**
- Consumes: `settings::crypto::{encrypt, decrypt, parse_key}` (Task 1), `settings::get_settings` (Task 2).
- Produces: `AppState.provider: Arc<tokio::sync::RwLock<Arc<dyn LlmProvider>>>`, `AppState.embedding_provider: Arc<tokio::sync::RwLock<Arc<dyn EmbeddingProvider>>>`, `AppState.http_client: reqwest::Client`, `AppState.settings_key: [u8; 32]` — later tasks (routes) read/write these fields directly.

- [ ] **Step 1: Write the failing test**

Create `backend/tests/app_state_reload.rs`:

```rust
mod support;

use nomi_orchestrator::app::AppState;
use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use sqlx::PgPool;
use std::sync::Arc;
use support::FakeLlmProvider;
use tokio::sync::RwLock;

#[sqlx::test]
async fn swapping_the_provider_lock_is_visible_to_the_next_reader(pool: PgPool) {
    let provider = Arc::new(FakeLlmProvider::success(LlmResponse {
        content: vec![ContentBlock::Text { text: "first".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 0,
        output_tokens: 0,
    }));
    let lock: Arc<RwLock<Arc<dyn nomi_orchestrator::llm::LlmProvider>>> = Arc::new(RwLock::new(provider));

    let new_provider = Arc::new(FakeLlmProvider::success(LlmResponse {
        content: vec![ContentBlock::Text { text: "second".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 0,
        output_tokens: 0,
    }));
    *lock.write().await = new_provider;

    let current = lock.read().await.clone();
    let response = current
        .complete(nomi_orchestrator::llm::LlmRequest {
            system: None,
            messages: vec![],
            tools: vec![],
            max_tokens: 10,
        })
        .await
        .unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "second".to_string() }]);

    // pool is unused directly but #[sqlx::test] requires the parameter to provision a test DB
    let _ = &pool;
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test --test app_state_reload`
Expected: FAIL to compile — `AppState` fields haven't changed yet, but this test doesn't touch `AppState` directly, so it should actually compile and pass already since it only exercises the `RwLock<Arc<dyn LlmProvider>>` pattern in isolation. Run it anyway to confirm the pattern itself works before wiring it into `AppState`.
Expected: PASS (this step validates the swap mechanism before Step 4 wires it into real code).

- [ ] **Step 3: Update the three test files' `AppState` construction (will not compile until Step 4)**

In `backend/tests/support/mod.rs`, add near the top-level items (after imports):

```rust
pub const TEST_SETTINGS_KEY: [u8; 32] = [7u8; 32];
```

In `backend/tests/sessions_routes.rs`, replace the `test_state` function body:

```rust
fn test_state(pool: PgPool, provider: FakeLlmProvider) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        provider: Arc::new(tokio::sync::RwLock::new(Arc::new(provider) as Arc<dyn nomi_orchestrator::llm::LlmProvider>)),
        embedding_provider: Arc::new(tokio::sync::RwLock::new(
            Arc::new(FakeEmbeddingProvider::success(dummy_embedding())) as Arc<dyn nomi_orchestrator::embedding::EmbeddingProvider>,
        )),
        http_client: reqwest::Client::new(),
        settings_key: support::TEST_SETTINGS_KEY,
    }
}
```

In `backend/tests/auth_extractor.rs`, apply the same shape change to its `AppState { ... }` literal (change `provider: Arc::new(FakeLlmProvider::success(...))` to `provider: Arc::new(tokio::sync::RwLock::new(Arc::new(FakeLlmProvider::success(...)) as Arc<dyn nomi_orchestrator::llm::LlmProvider>))`, same for `embedding_provider`, and add the `http_client`/`settings_key` fields as above).

In `backend/tests/auth_routes.rs`, apply the same change to its `test_state` function.

- [ ] **Step 4: Implement the `AppState` and `main.rs` changes**

Replace `backend/src/app.rs`:

```rust
use std::sync::Arc;

use axum::{routing::{delete, get, post, put}, Router};
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::auth::extractor::AuthClaims;
use crate::embedding::EmbeddingProvider;
use crate::llm::LlmProvider;
use crate::routes::auth as auth_routes;
use crate::routes::sessions as sessions_routes;
use crate::routes::settings as settings_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub provider: Arc<RwLock<Arc<dyn LlmProvider>>>,
    pub embedding_provider: Arc<RwLock<Arc<dyn EmbeddingProvider>>>,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login_handler))
        .route("/api/auth/refresh", post(auth_routes::refresh_handler))
        .route("/api/auth/logout", post(auth_routes::logout_handler))
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .route("/api/orgs/:org_id/members/:user_id", delete(auth_routes::remove_member))
        .route(
            "/api/sessions",
            post(sessions_routes::create_session).get(sessions_routes::list_sessions),
        )
        .route(
            "/api/sessions/:id/messages",
            get(sessions_routes::list_messages).post(sessions_routes::send_message),
        )
        .route(
            "/api/admin/settings/llm",
            get(settings_routes::get_llm_settings).put(settings_routes::put_llm_settings),
        )
        .route(
            "/api/admin/settings/embedding",
            get(settings_routes::get_embedding_settings).put(settings_routes::put_embedding_settings),
        )
        .with_state(state)
}
```

Note: this references `crate::routes::settings`, which Task 5 creates. Add a temporary empty stub now so `app.rs` compiles: create `backend/src/routes/settings.rs` with:

```rust
// Populated in Task 5 of docs/superpowers/plans/2026-08-02-app-configurable-providers.md
```

and add `pub mod settings;` to `backend/src/routes/mod.rs`. (Task 5 replaces this stub with real handlers — the two `.route(...)` calls above will not compile against the stub, so also comment out the two `/api/admin/settings/*` `.route(...)` calls in this task, with a `// TODO(Task 5)` marker, and Task 5 uncomments them once the handlers exist. This keeps every task's build green.)

Replace `backend/src/main.rs`:

```rust
use std::env::var;
use std::sync::Arc;

use nomi_orchestrator::embedding::{
    build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind,
};
use nomi_orchestrator::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use nomi_orchestrator::settings;

fn llm_provider_kind_from_str(s: &str) -> ProviderKind {
    match s {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        "fake" => ProviderKind::Fake,
        other => panic!("unknown LLM provider: {other} (expected anthropic, openai, gemini, or fake)"),
    }
}

fn embedding_provider_kind_from_str(s: &str) -> EmbeddingProviderKind {
    match s {
        "openai" => EmbeddingProviderKind::OpenAi,
        "fake" => EmbeddingProviderKind::Fake,
        other => panic!("unknown embedding provider: {other} (expected openai or fake)"),
    }
}

async fn build_llm_provider_from_settings_or_env(
    pool: &sqlx::PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn LlmProvider> {
    let row = settings::get_settings(pool, "llm").await.expect("failed to query provider_settings");

    let model_config = match row {
        Some(row) => {
            let api_key = settings::crypto::decrypt(settings_key, &row.api_key_encrypted)
                .expect("failed to decrypt stored llm api key");
            ModelConfig {
                provider: llm_provider_kind_from_str(&row.provider),
                model_id: row.model_id,
                api_key,
                base_url: row.base_url,
            }
        }
        None => {
            let provider =
                llm_provider_kind_from_str(&std::env::var("LLM_PROVIDER").expect("LLM_PROVIDER must be set"));
            let (model_id, api_key) = match provider {
                ProviderKind::Fake => (String::new(), String::new()),
                _ => (
                    std::env::var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
                    std::env::var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
                ),
            };
            ModelConfig { provider, model_id, api_key, base_url: std::env::var("LLM_BASE_URL").ok() }
        }
    };

    Arc::from(build_provider(model_config, http_client))
}

async fn build_embedding_provider_from_settings_or_env(
    pool: &sqlx::PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn EmbeddingProvider> {
    let row = settings::get_settings(pool, "embedding").await.expect("failed to query provider_settings");

    let embedding_config = match row {
        Some(row) => {
            let api_key = settings::crypto::decrypt(settings_key, &row.api_key_encrypted)
                .expect("failed to decrypt stored embedding api key");
            EmbeddingConfig {
                provider: embedding_provider_kind_from_str(&row.provider),
                model_id: row.model_id,
                api_key,
                base_url: row.base_url,
            }
        }
        None => {
            let provider = embedding_provider_kind_from_str(
                &std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            );
            let (model_id, api_key) = match provider {
                EmbeddingProviderKind::Fake => (String::new(), String::new()),
                EmbeddingProviderKind::OpenAi => (
                    std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
                    std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
                ),
            };
            EmbeddingConfig { provider, model_id, api_key, base_url: std::env::var("EMBEDDING_BASE_URL").ok() }
        }
    };

    Arc::from(build_embedding_provider(embedding_config, http_client))
}

#[tokio::main]
async fn main() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set");
    let settings_key = settings::crypto::parse_key(
        &var("SETTINGS_ENCRYPTION_KEY").expect("SETTINGS_ENCRYPTION_KEY must be set"),
    )
    .expect("SETTINGS_ENCRYPTION_KEY must be 64 hex characters (32 bytes)");

    let pool = sqlx::PgPool::connect(&database_url).await.expect("failed to connect to database");
    sqlx::migrate!().run(&pool).await.expect("failed to run migrations");

    let http_client = reqwest::Client::new();

    let provider = build_llm_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
    let embedding_provider =
        build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

    let state = nomi_orchestrator::app::AppState {
        pool,
        jwt_secret,
        provider: Arc::new(tokio::sync::RwLock::new(provider)),
        embedding_provider: Arc::new(tokio::sync::RwLock::new(embedding_provider)),
        http_client,
        settings_key,
    };
    let app = nomi_orchestrator::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.expect("failed to bind to port 8080");
    axum::serve(listener, app).await.expect("server error");
}
```

In `backend/src/routes/sessions.rs`, replace the `send_message` body's provider-passing lines:

```rust
    let provider = state.provider.read().await.clone();
    let embedding_provider = state.embedding_provider.read().await.clone();

    crate::turn::handle_inbound_message(
        &state.pool,
        provider.as_ref(),
        embedding_provider.as_ref(),
        &channel,
        &chat_type,
        &chat_id,
        &claims.sub.to_string(),
        &req.text,
        Some(claims.active_org_id),
    )
```

(replacing the previous `state.provider.as_ref()`, `state.embedding_provider.as_ref()` arguments).

- [ ] **Step 5: Run the full backend test suite to verify it passes**

Run: `cd backend && cargo test`
Expected: all tests compile and pass, including the new `app_state_reload` test and the three updated test files. (`SETTINGS_ENCRYPTION_KEY` is not required for `cargo test` since `main.rs`'s `main()` isn't exercised by tests — only `cargo run` needs it.)

- [ ] **Step 6: Commit**

```bash
git add backend/src/app.rs backend/src/main.rs backend/src/routes/sessions.rs backend/src/routes/settings.rs backend/src/routes/mod.rs backend/tests/support/mod.rs backend/tests/sessions_routes.rs backend/tests/auth_extractor.rs backend/tests/auth_routes.rs backend/tests/app_state_reload.rs
git commit -m "refactor: make AppState providers live-swappable behind RwLock, load from DB with env fallback"
```

---

## Task 4: `system_config` permission for platform admins

**Files:**
- Modify: `backend/src/auth/permissions.rs`
- Test: `backend/tests/auth_permissions.rs`

**Interfaces:**
- Produces: platform admins' `compute_permissions` result now includes `"nomi:admin:system_config:[view,manage]"` in addition to the existing `"nomi:admin:user:[view,manage]"`.

- [ ] **Step 1: Write the failing test**

Open `backend/tests/auth_permissions.rs`, find the existing platform-admin test (it currently asserts on the `admin:user` permission), and add a new test in the same file:

```rust
#[sqlx::test]
async fn platform_admins_get_the_system_config_permission(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:system_config:[view,manage]".to_string()));
}

#[sqlx::test]
async fn non_platform_admins_do_not_get_the_system_config_permission(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(!permissions.iter().any(|p| p.contains("system_config")));
}
```

(Match the file's existing imports — it already imports `compute_permissions`, `PgPool`, `Uuid` for the pre-existing tests in this file.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test auth_permissions platform_admins_get_the_system_config_permission`
Expected: FAIL — assertion fails, permission not present.

- [ ] **Step 3: Implement**

In `backend/src/auth/permissions.rs`, change:

```rust
    if is_platform_admin {
        permissions.push(permission_string("admin", "user", &["view", "manage"]));
    }
```

to:

```rust
    if is_platform_admin {
        permissions.push(permission_string("admin", "user", &["view", "manage"]));
        permissions.push(permission_string("admin", "system_config", &["view", "manage"]));
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test auth_permissions`
Expected: all tests in the file pass.

- [ ] **Step 5: Commit**

```bash
git add backend/src/auth/permissions.rs backend/tests/auth_permissions.rs
git commit -m "feat: grant platform admins the system_config permission"
```

---

## Task 5: Admin settings routes (GET/PUT for LLM and embedding)

**Files:**
- Modify: `backend/src/routes/settings.rs` (replace the Task 3 stub)
- Modify: `backend/src/app.rs` (uncomment the two `/api/admin/settings/*` routes from Task 3)
- Test: `backend/tests/settings_routes.rs`

**Interfaces:**
- Consumes: `settings::{get_settings, upsert_settings, mask_api_key, UpsertInput}` (Task 2), `settings::crypto::{encrypt, decrypt}` (Task 1), `AppState.provider`/`embedding_provider`/`http_client`/`settings_key` (Task 3), `Claims::has_permission` (existing), `llm::{build_provider, ModelConfig, ProviderKind}` (existing), `embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProviderKind}` (existing).
- Produces: `GET /api/admin/settings/llm`, `PUT /api/admin/settings/llm`, `GET /api/admin/settings/embedding`, `PUT /api/admin/settings/embedding` — all requiring `AuthClaims` with the `system_config` `manage` permission.

- [ ] **Step 1: Write the failing tests**

Create `backend/tests/settings_routes.rs`:

```rust
mod support;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use nomi_orchestrator::llm::{LlmResponse, StopReason};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider, TEST_SETTINGS_KEY};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        provider: Arc::new(tokio::sync::RwLock::new(Arc::new(FakeLlmProvider::success(LlmResponse {
            content: vec![],
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        })) as Arc<dyn nomi_orchestrator::llm::LlmProvider>)),
        embedding_provider: Arc::new(tokio::sync::RwLock::new(
            Arc::new(FakeEmbeddingProvider::success(dummy_embedding())) as Arc<dyn nomi_orchestrator::embedding::EmbeddingProvider>,
        )),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
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

/// Registers, promotes to platform admin, then logs in — login must happen
/// *after* promotion, since permissions are baked into the JWT at login time.
async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

#[sqlx::test]
async fn get_llm_settings_returns_404_when_unconfigured(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin1@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/settings/llm", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn non_admin_is_forbidden_from_reading_settings(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/settings/llm", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn admin_can_save_and_then_read_back_masked_llm_settings(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let (status, put_body) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-abcdefgh1234", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["api_key_masked"], "...1234");

    let (status, get_body) = json_request(router, "GET", "/api/admin/settings/llm", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(get_body["provider"], "anthropic");
    assert_eq!(get_body["model_id"], "claude-haiku-4-5");
    assert_eq!(get_body["api_key_masked"], "...1234");
}

#[sqlx::test]
async fn put_rejects_an_unknown_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin3@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "not-a-real-provider", "model_id": "x", "api_key": "key", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_without_api_key_keeps_the_existing_key(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin4@example.com").await;

    json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-original-key9", "base_url": null }),
        Some(&token),
    )
    .await;

    let (status, put_body) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "anthropic", "model_id": "claude-sonnet-5", "api_key": null, "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["model_id"], "claude-sonnet-5");
    assert_eq!(put_body["api_key_masked"], "...key9");
}

#[sqlx::test]
async fn saving_the_fake_llm_provider_takes_effect_immediately_without_restart(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin5@example.com").await;

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "fake", "model_id": "", "api_key": null, "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, session_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = session_body["session_id"].as_str().unwrap();

    let (status, message_body) = json_request(
        router,
        "POST",
        &format!("/api/sessions/{session_id}/messages"),
        json!({ "text": "hello" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        message_body["assistant_message"]["content"],
        "This is a fake response for local development and testing."
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test settings_routes`
Expected: FAIL — routes don't exist yet (404s where 200/403/400 expected), or compile error since `app.rs`'s admin routes are still commented out from Task 3.

- [ ] **Step 3: Implement**

Replace `backend/src/routes/settings.rs`:

```rust
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app::AppState;
use crate::auth::claims::Claims;
use crate::auth::extractor::AuthClaims;
use crate::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use crate::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use crate::settings;

#[derive(Serialize)]
pub struct ProviderSettingsResponse {
    pub provider: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key_masked: String,
}

#[derive(Deserialize)]
pub struct UpdateProviderSettingsRequest {
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

fn require_system_config_permission(claims: &Claims) -> Result<(), (StatusCode, &'static str)> {
    if claims.has_permission("admin", "system_config", "manage") {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to manage system settings"))
    }
}

async fn load_settings_response(
    state: &AppState,
    setting_type: &str,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    let row = settings::get_settings(&state.pool, setting_type)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load settings"))?
        .ok_or((StatusCode::NOT_FOUND, "settings not configured"))?;

    let api_key = settings::crypto::decrypt(&state.settings_key, &row.api_key_encrypted)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key"))?;

    Ok(Json(ProviderSettingsResponse {
        provider: row.provider,
        model_id: row.model_id,
        base_url: row.base_url,
        api_key_masked: settings::mask_api_key(&api_key),
    }))
}

pub async fn get_llm_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    load_settings_response(&state, "llm").await
}

pub async fn get_embedding_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    load_settings_response(&state, "embedding").await
}

async fn resolve_api_key(
    state: &AppState,
    setting_type: &str,
    submitted: Option<&str>,
    is_fake: bool,
) -> Result<String, (StatusCode, &'static str)> {
    if let Some(key) = submitted {
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }
    let existing = settings::get_settings(&state.pool, setting_type)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load existing settings"))?;
    match existing {
        Some(row) => settings::crypto::decrypt(&state.settings_key, &row.api_key_encrypted)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key")),
        None if is_fake => Ok(String::new()),
        None => Err((StatusCode::BAD_REQUEST, "api_key is required for a new non-fake provider configuration")),
    }
}

pub async fn put_llm_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdateProviderSettingsRequest>,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    if !["anthropic", "openai", "gemini", "fake"].contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected anthropic, openai, gemini, or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }

    let api_key = resolve_api_key(&state, "llm", req.api_key.as_deref(), is_fake).await?;
    if !is_fake && api_key.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider"));
    }

    settings::upsert_settings(
        &state.pool,
        settings::UpsertInput {
            setting_type: "llm",
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &api_key),
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;

    let provider_kind = match req.provider.as_str() {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        _ => ProviderKind::Fake,
    };
    let new_provider: Arc<dyn LlmProvider> = Arc::from(build_provider(
        ModelConfig {
            provider: provider_kind,
            model_id: req.model_id.clone(),
            api_key: api_key.clone(),
            base_url: req.base_url.clone(),
        },
        state.http_client.clone(),
    ));
    *state.provider.write().await = new_provider;

    Ok(Json(ProviderSettingsResponse {
        provider: req.provider,
        model_id: req.model_id,
        base_url: req.base_url,
        api_key_masked: settings::mask_api_key(&api_key),
    }))
}

pub async fn put_embedding_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdateProviderSettingsRequest>,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    if !["openai", "fake"].contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected openai or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }

    let api_key = resolve_api_key(&state, "embedding", req.api_key.as_deref(), is_fake).await?;
    if !is_fake && api_key.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider"));
    }

    settings::upsert_settings(
        &state.pool,
        settings::UpsertInput {
            setting_type: "embedding",
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &api_key),
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;

    let provider_kind = if is_fake { EmbeddingProviderKind::Fake } else { EmbeddingProviderKind::OpenAi };
    let new_provider: Arc<dyn EmbeddingProvider> = Arc::from(build_embedding_provider(
        EmbeddingConfig {
            provider: provider_kind,
            model_id: req.model_id.clone(),
            api_key: api_key.clone(),
            base_url: req.base_url.clone(),
        },
        state.http_client.clone(),
    ));
    *state.embedding_provider.write().await = new_provider;

    Ok(Json(ProviderSettingsResponse {
        provider: req.provider,
        model_id: req.model_id,
        base_url: req.base_url,
        api_key_masked: settings::mask_api_key(&api_key),
    }))
}
```

In `backend/src/app.rs`, uncomment the two `/api/admin/settings/*` `.route(...)` calls added (as comments) in Task 3.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test`
Expected: full suite passes, including all 6 new tests in `settings_routes.rs`.

- [ ] **Step 5: Commit**

```bash
git add backend/src/routes/settings.rs backend/src/app.rs backend/tests/settings_routes.rs
git commit -m "feat: add admin GET/PUT routes for live LLM and embedding provider settings"
```

---

## Task 6: Frontend admin auth area (separate login, permission-gated shell)

**Files:**
- Create: `frontend/src/routes/admin/login/+page.svelte`
- Create: `frontend/src/routes/admin/login/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/+layout.server.ts`
- Create: `frontend/src/routes/admin/(protected)/+layout.svelte`
- Create: `frontend/src/routes/admin/(protected)/+page.svelte`
- Modify: `frontend/src/routes/logout/+server.ts`
- Create: `frontend/e2e/support/db.ts`
- Modify: `frontend/package.json` (new devDependency)
- Test: `frontend/e2e/admin.e2e.ts`

**Interfaces:**
- Consumes: `apiFetch`, `apiUrl` from `$lib/server/api` (existing), backend `/api/auth/login`, `/api/whoami` (existing).
- Produces: `promoteToPlatformAdmin(email: string): Promise<void>` in `frontend/e2e/support/db.ts`, used by Task 7's e2e tests too. `/admin` as the guarded landing route; `/admin/login` as the public entry.

- [ ] **Step 1: Add the `postgres` devDependency**

```bash
cd frontend && pnpm add -D postgres
```

- [ ] **Step 2: Write the failing e2e tests**

Create `frontend/e2e/support/db.ts`:

```ts
import postgres from 'postgres';

const DATABASE_URL = process.env.DATABASE_URL ?? 'postgres://postgres:postgres@localhost:5432/nomi_dev';

export async function promoteToPlatformAdmin(email: string): Promise<void> {
	const sql = postgres(DATABASE_URL);
	try {
		await sql`UPDATE users SET is_platform_admin = true
			WHERE id = (SELECT user_id FROM web_credentials WHERE email = ${email})`;
	} finally {
		await sql.end();
	}
}
```

Create `frontend/e2e/admin.e2e.ts`:

```ts
import { expect, test, type Page } from '@playwright/test';
import { promoteToPlatformAdmin } from './support/db';

function uniqueEmail(prefix: string): string {
	return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;
}

async function registerViaUi(page: Page, email: string, orgName: string): Promise<void> {
	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill(orgName);
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');
}

async function loginViaAdminUi(page: Page, email: string): Promise<void> {
	await page.goto('/admin/login');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByRole('button', { name: 'Log in' }).click();
}

test('visiting the admin area while logged out redirects to admin login', async ({ page }) => {
	await page.goto('/admin');
	await expect(page).toHaveURL('/admin/login');
});

test('a non-admin account is redirected to admin login as forbidden', async ({ page }) => {
	const email = uniqueEmail('nonadmin');
	await registerViaUi(page, email, 'Acme');
	await page.context().clearCookies();

	await loginViaAdminUi(page, email);

	await expect(page).toHaveURL('/admin/login?error=forbidden');
	await expect(page.getByText(/does not have admin access/i)).toBeVisible();
});

test('a platform admin can log in and reach the admin dashboard', async ({ page }) => {
	const email = uniqueEmail('admin');
	await registerViaUi(page, email, 'Acme');
	await promoteToPlatformAdmin(email);
	await page.context().clearCookies();

	await loginViaAdminUi(page, email);

	await expect(page).toHaveURL('/admin');
	await expect(page.getByRole('heading', { name: 'Admin dashboard' })).toBeVisible();
});
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd frontend && npx playwright test admin.e2e.ts`
Expected: FAIL — `/admin` and `/admin/login` don't exist yet (404s).

- [ ] **Step 4: Implement**

Create `frontend/src/routes/admin/login/+page.server.ts`:

```ts
import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const email = data.get('email');
		const password = data.get('password');

		if (typeof email !== 'string' || typeof password !== 'string' || !email || !password) {
			return fail(400, { error: 'Email and password are required.' });
		}

		const response = await fetch(apiUrl('/api/auth/login'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password }),
		});

		if (response.status === 401) {
			return fail(401, { error: 'Invalid email or password.' });
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Login failed. Please try again.' });
		}

		const { access_token, refresh_token } = (await response.json()) as {
			access_token: string;
			refresh_token: string;
		};

		cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('refresh_token', refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('user_email', email, { httpOnly: false, path: '/', sameSite: 'lax' });

		throw redirect(303, '/admin');
	},
};
```

Create `frontend/src/routes/admin/login/+page.svelte`:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-50">
	<form
		method="POST"
		use:enhance
		class="w-full max-w-sm space-y-4 rounded-2xl border border-neutral-200 bg-white p-8 shadow-sm"
	>
		<h1 class="text-2xl font-semibold">Admin sign in</h1>
		{#if form?.error}
			<p class="text-sm text-red-600">{form.error}</p>
		{:else if $page.url.searchParams.get('error') === 'forbidden'}
			<p class="text-sm text-red-600">That account does not have admin access.</p>
		{/if}
		<div>
			<label for="email" class="block text-sm font-medium text-neutral-700">Email</label>
			<input
				id="email"
				name="email"
				type="email"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<div>
			<label for="password" class="block text-sm font-medium text-neutral-700">Password</label>
			<input
				id="password"
				name="password"
				type="password"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<button
			type="submit"
			class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800"
		>
			Log in
		</button>
	</form>
</div>
```

Create `frontend/src/routes/admin/(protected)/+layout.server.ts`:

```ts
import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/admin/login');
	}

	const response = await apiFetch(fetch, cookies, '/api/whoami');
	if (!response.ok) {
		throw redirect(303, '/admin/login');
	}

	const claims = (await response.json()) as { permissions: string[] };
	const isSystemAdmin = claims.permissions.some((permission) =>
		permission.startsWith('nomi:admin:system_config:'),
	);
	if (!isSystemAdmin) {
		throw redirect(303, '/admin/login?error=forbidden');
	}

	return {};
};
```

Create `frontend/src/routes/admin/(protected)/+layout.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LayoutData } from './$types';

	let { children }: { data: LayoutData; children: Snippet } = $props();
</script>

<div class="flex h-screen bg-neutral-50">
	<aside class="w-56 border-r border-neutral-200 bg-white p-4">
		<h2 class="mb-4 text-lg font-semibold">Admin</h2>
		<nav class="space-y-2 text-sm">
			<a href="/admin" class="block text-neutral-700 hover:underline">Dashboard</a>
			<a href="/admin/settings/llm" class="block text-neutral-700 hover:underline">LLM Settings</a>
		</nav>
		<form method="POST" action="/logout?redirect_to=/admin/login" class="mt-6">
			<button type="submit" class="text-sm text-neutral-500 underline">Log out</button>
		</form>
	</aside>
	<main class="flex-1 overflow-y-auto p-8">
		{@render children()}
	</main>
</div>
```

Create `frontend/src/routes/admin/(protected)/+page.svelte`:

```svelte
<h1 class="text-2xl font-semibold">Admin dashboard</h1>
<p class="mt-2 text-neutral-600">Manage app-wide configuration.</p>
```

Replace `frontend/src/routes/logout/+server.ts`:

```ts
import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ cookies, fetch, url }) => {
	const refreshToken = cookies.get('refresh_token');
	if (refreshToken) {
		await fetch(apiUrl('/api/auth/logout'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ refresh_token: refreshToken }),
		});
	}
	cookies.delete('access_token', { path: '/' });
	cookies.delete('refresh_token', { path: '/' });
	cookies.delete('user_email', { path: '/' });

	const requested = url.searchParams.get('redirect_to');
	const redirectTo = requested && requested.startsWith('/') && !requested.startsWith('//') ? requested : '/login';
	throw redirect(303, redirectTo);
};
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd frontend && npx playwright test admin.e2e.ts`
Expected: all 3 tests pass. (This starts the real backend via `cargo run` per `playwright.config.ts` — ensure `SETTINGS_ENCRYPTION_KEY` is set in that `webServer.env` block first; see Step 5b.)

- [ ] **Step 5b: Wire `SETTINGS_ENCRYPTION_KEY` into the e2e backend process**

In `frontend/playwright.config.ts`, add to the `env` object of the `cargo run` webServer entry:

```ts
SETTINGS_ENCRYPTION_KEY: '0'.repeat(64),
```

(a fixed all-zero key is fine for e2e — it only needs to be 64 hex chars; never reuse this for a real deployment).

- [ ] **Step 6: Commit**

```bash
git add frontend/src/routes/admin frontend/src/routes/logout/+server.ts frontend/e2e/support/db.ts frontend/e2e/admin.e2e.ts frontend/playwright.config.ts frontend/package.json frontend/pnpm-lock.yaml
git commit -m "feat: add admin login, permission-gated admin shell, and logout redirect_to support"
```

---

## Task 7: Frontend LLM provider settings page

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Create: `frontend/src/routes/admin/(protected)/settings/llm/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/settings/llm/+page.svelte`
- Modify: `frontend/e2e/admin.e2e.ts` (append one more test)

**Interfaces:**
- Consumes: `ProviderSettings` type (new), backend `GET/PUT /api/admin/settings/llm` (Task 5), `promoteToPlatformAdmin` (Task 6).

- [ ] **Step 1: Add the type**

In `frontend/src/lib/types.ts`, append:

```ts
export interface ProviderSettings {
	provider: string;
	model_id: string;
	base_url: string | null;
	api_key_masked: string;
}
```

- [ ] **Step 2: Write the failing e2e test**

Append to `frontend/e2e/admin.e2e.ts`:

```ts
test('a platform admin can configure the fake LLM provider', async ({ page }) => {
	const email = uniqueEmail('admin-settings');
	await registerViaUi(page, email, 'Acme');
	await promoteToPlatformAdmin(email);
	await page.context().clearCookies();

	await loginViaAdminUi(page, email);
	await expect(page).toHaveURL('/admin');

	await page.goto('/admin/settings/llm');
	await page.getByLabel('Provider').selectOption('fake');
	await page.getByRole('button', { name: 'Save' }).click();

	await expect(page.getByText('Settings saved.')).toBeVisible();
	await page.reload();
	await expect(page.getByLabel('Provider')).toHaveValue('fake');
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd frontend && npx playwright test admin.e2e.ts -g "configure the fake LLM provider"`
Expected: FAIL — `/admin/settings/llm` doesn't exist yet (404).

- [ ] **Step 4: Implement**

Create `frontend/src/routes/admin/(protected)/settings/llm/+page.server.ts`:

```ts
import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProviderSettings } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm');
	if (response.status === 404) {
		return { settings: null as ProviderSettings | null };
	}
	if (!response.ok) {
		return { settings: null as ProviderSettings | null };
	}
	const settings = (await response.json()) as ProviderSettings;
	return { settings };
};

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');

		if (typeof provider !== 'string' || typeof modelId !== 'string') {
			return fail(400, { error: 'Provider and model are required.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm', {
			method: 'PUT',
			body: JSON.stringify({
				provider,
				model_id: modelId,
				api_key: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to save settings.' });
		}

		const settings = (await response.json()) as ProviderSettings;
		return { settings, success: true };
	},
};
```

Create `frontend/src/routes/admin/(protected)/settings/llm/+page.svelte`:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const settings = $derived(form?.settings ?? data.settings);
</script>

<h1 class="text-2xl font-semibold">LLM provider settings</h1>

<form method="POST" use:enhance class="mt-6 max-w-lg space-y-4 rounded-2xl border border-neutral-200 bg-white p-6">
	{#if form?.error}
		<p class="text-sm text-red-600">{form.error}</p>
	{/if}
	{#if form?.success}
		<p class="text-sm text-green-600">Settings saved.</p>
	{/if}

	<div>
		<label for="provider" class="block text-sm font-medium text-neutral-700">Provider</label>
		<select
			id="provider"
			name="provider"
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			value={settings?.provider ?? 'anthropic'}
		>
			<option value="anthropic">Anthropic</option>
			<option value="openai">OpenAI</option>
			<option value="gemini">Gemini</option>
			<option value="fake">Fake (testing)</option>
		</select>
	</div>
	<div>
		<label for="model_id" class="block text-sm font-medium text-neutral-700">Model ID</label>
		<input
			id="model_id"
			name="model_id"
			type="text"
			value={settings?.model_id ?? ''}
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
		/>
	</div>
	<div>
		<label for="base_url" class="block text-sm font-medium text-neutral-700">Base URL (optional)</label>
		<input
			id="base_url"
			name="base_url"
			type="text"
			value={settings?.base_url ?? ''}
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
		/>
	</div>
	<div>
		<label for="api_key" class="block text-sm font-medium text-neutral-700">
			API key
			{#if settings?.api_key_masked}
				<span class="text-neutral-400">(current: {settings.api_key_masked})</span>
			{/if}
		</label>
		<input
			id="api_key"
			name="api_key"
			type="password"
			placeholder={settings?.api_key_masked ? 'Leave blank to keep current key' : ''}
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
		/>
	</div>
	<button
		type="submit"
		class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800"
	>
		Save
	</button>
</form>
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd frontend && npx playwright test admin.e2e.ts`
Expected: all 4 tests in the file pass.

- [ ] **Step 6: Run the full test suite (backend + frontend) one more time**

Run: `cd backend && cargo test && cd ../frontend && npx playwright test`
Expected: everything passes — this is the final integration checkpoint for the whole plan.

- [ ] **Step 7: Commit**

```bash
git add frontend/src/lib/types.ts frontend/src/routes/admin/\(protected\)/settings frontend/e2e/admin.e2e.ts
git commit -m "feat: add admin LLM provider settings page"
```
