# Backend Cargo Workspace + Agent Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the single-crate `backend/` into a Cargo workspace of ~10 crates, and replace the orchestrator's hardcoded agent dispatch with a `SubAgent` trait + runtime `AgentRegistry`, so a new agent is a new crate plus one registration line — zero edits to shared orchestration code.

**Architecture:** Strangler-fig migration. Each task physically moves one domain's source files (and its integration tests) into a new `crates/nomi-X/` member crate, then replaces the old monolith's `pub mod x;` with `pub use nomi_x as x;` in its `lib.rs` — every not-yet-migrated file elsewhere in the monolith keeps compiling against `crate::x::...` completely unchanged, because the alias makes `crate::x` transparently point at the new crate's public API. The old monolith (package name `nomi-orchestrator`, still `backend/Cargo.toml`'s `[package]`) and the new workspace members (`backend/Cargo.toml`'s `[workspace] members`) coexist in the same manifest throughout — Cargo supports a package that is simultaneously its own workspace root. Only the final task deletes the old monolith outright.

**Tech Stack:** Rust 2021, Cargo workspaces, axum, sqlx, tokio — same as today, no new external dependencies.

**Spec:** `docs/superpowers/specs/2026-08-26-backend-cargo-workspace-design.md`

## Global Constraints

- Binary names do not change: `cargo run --bin nomi-orchestrator` and `cargo run --bin worker`, run from `backend/`, must work identically before, during, and after this plan (verified at the end of every task via `cargo build` from `backend/`).
- A Cargo package named `nomi-llm` is referenced from Rust code as `nomi_llm` (Cargo's standard hyphen→underscore mangling) — every new crate follows this; note it once here, not per-task.
- Every new crate uses `edition = "2021"`, matching the existing crate.
- No HTTP-visible behavior changes, with exactly one called-out exception: money_agent-style (tool-calling) turns start streaming live deltas over MQTT, matching what chitchat has always done — this is a direct consequence of generalizing the streaming code that used to be chitchat-only (spec §3).
- After every task: `cargo build` (whole workspace, from `backend/`) and `cargo test` (whole workspace) must both be green before moving to the next task. Never leave the workspace in a broken state between tasks.
- The migration alias (`pub use nomi_x as x;` in the old monolith's `lib.rs`) is a temporary scaffold, not a permanent public API — it is deleted along with the rest of the old monolith in Task 10, and no new code should ever be written against it.
- `AgentRegistry::new` panics at construction (startup) if zero or more than one registered agent returns `true` from `is_default()` — a wrong startup wiring is a programmer error, not a runtime condition to degrade gracefully from.

---

### Task 1: Workspace scaffold + `nomi-llm`

**Files:**
- Modify: `backend/Cargo.toml` (add `[workspace]`, add `nomi-llm` path dependency, keep everything else)
- Modify: `backend/src/lib.rs` (replace `pub mod llm;` with `pub use nomi_llm as llm;`)
- Create: `backend/crates/nomi-llm/Cargo.toml`
- Move: `backend/src/llm/*.rs` → `backend/crates/nomi-llm/src/*.rs` (verbatim, same relative structure: `mod.rs`, `types.rs`, `config.rs`, `anthropic.rs`, `openai.rs`, `gemini.rs`, `fake.rs` — rename `mod.rs` to `lib.rs` since it's now a crate root, keep every other file's name and content byte-for-byte identical)
- Move: `backend/tests/llm_anthropic.rs`, `llm_config.rs`, `llm_gemini.rs`, `llm_openai.rs`, `llm_streaming.rs` → `backend/crates/nomi-llm/tests/*.rs`
- Delete: `backend/src/llm/` (after the move)

**Interfaces:**
- Produces: crate `nomi-llm`, public API identical to today's `crate::llm` module (verbatim move — `LlmProvider`, `LlmError`, `LlmMessage`, `LlmRequest`, `LlmResponse`, `ContentBlock`, `LlmRole`, `StopReason`, `ToolDefinition`, `StreamEvent`, `PartialBlock`, `ModelConfig`, `ProviderKind`, `build_provider`, `complete`, `complete_stream`, `collect_stream`, `response_to_stream`, provider structs `AnthropicProvider`/`OpenAiProvider`/`GeminiProvider`/`FakeLlmProvider`).
- Consumes: nothing workspace-internal.

- [ ] **Step 1: Create the workspace scaffold**

Edit `backend/Cargo.toml` — append at the very end of the file (keep the existing `[package]`/`[dependencies]`/`[dev-dependencies]` sections exactly as they are):

```toml

[workspace]
resolver = "2"
members = ["crates/*"]
```

- [ ] **Step 2: Create `nomi-llm`'s manifest**

Create `backend/crates/nomi-llm/Cargo.toml`:

```toml
[package]
name = "nomi-llm"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
thiserror = "1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
futures-core = "0.3.33"
futures-util = "0.3.33"
eventsource-stream = "0.2.3"
async-stream = "0.3.6"
tracing = "0.1.44"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

- [ ] **Step 3: Move the source files**

```bash
mkdir -p backend/crates/nomi-llm/src
git -C backend mv src/llm/types.rs crates/nomi-llm/src/types.rs
git -C backend mv src/llm/config.rs crates/nomi-llm/src/config.rs
git -C backend mv src/llm/anthropic.rs crates/nomi-llm/src/anthropic.rs
git -C backend mv src/llm/openai.rs crates/nomi-llm/src/openai.rs
git -C backend mv src/llm/gemini.rs crates/nomi-llm/src/gemini.rs
git -C backend mv src/llm/fake.rs crates/nomi-llm/src/fake.rs
git -C backend mv src/llm/mod.rs crates/nomi-llm/src/lib.rs
```

Do not edit the moved files' contents — the module tree (`pub mod types;` etc.) and every `use super::...`/`use crate::...` line inside them is already correct as a crate root, since `mod.rs` becoming `lib.rs` preserves the exact same relative module structure (`crate::` inside these files meant "the `llm` module" before, and now correctly means "this new crate" — no change needed).

- [ ] **Step 4: Alias the old module**

Edit `backend/src/lib.rs`, change:
```rust
pub mod llm;
```
to:
```rust
pub use nomi_llm as llm;
```

Edit `backend/Cargo.toml`'s `[dependencies]` section (the existing one, for package `nomi-orchestrator`), add:
```toml
nomi-llm = { path = "crates/nomi-llm" }
```

- [ ] **Step 5: Move the tests**

```bash
mkdir -p backend/crates/nomi-llm/tests
git -C backend mv tests/llm_anthropic.rs crates/nomi-llm/tests/llm_anthropic.rs
git -C backend mv tests/llm_config.rs crates/nomi-llm/tests/llm_config.rs
git -C backend mv tests/llm_gemini.rs crates/nomi-llm/tests/llm_gemini.rs
git -C backend mv tests/llm_openai.rs crates/nomi-llm/tests/llm_openai.rs
git -C backend mv tests/llm_streaming.rs crates/nomi-llm/tests/llm_streaming.rs
```

In each moved test file, change every `use nomi_orchestrator::llm::` to `use nomi_llm::`. These are the only import lines that need changing (e.g. `use nomi_orchestrator::llm::anthropic::AnthropicProvider;` → `use nomi_llm::anthropic::AnthropicProvider;`); nothing else in these files references anything outside `nomi_llm`.

- [ ] **Step 6: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors. All tests pass, including the 5 moved files now running as `nomi-llm`'s own integration tests and the rest of the old monolith's tests (unaffected, since `crate::llm` still resolves via the alias).

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: extract nomi-llm crate from the backend monolith"
```

---

### Task 2: `nomi-embedding`

Same mechanics as Task 1, applied to the embedding module.

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/lib.rs`
- Create: `backend/crates/nomi-embedding/Cargo.toml`
- Move: `backend/src/embedding/*.rs` → `backend/crates/nomi-embedding/src/*.rs` (`mod.rs` → `lib.rs`)
- Move: `backend/tests/embedding_config.rs`, `embedding_openai.rs` → `backend/crates/nomi-embedding/tests/`
- Delete: `backend/src/embedding/`

**Interfaces:**
- Produces: crate `nomi-embedding` — `EmbeddingProvider` trait, `EmbeddingError`, `EmbeddingConfig`, `EmbeddingProviderKind`, `build_embedding_provider`, provider structs `OpenAiEmbeddingProvider`/`FakeEmbeddingProvider`.
- Consumes: nothing workspace-internal.

- [ ] **Step 1: Create `nomi-embedding`'s manifest**

Create `backend/crates/nomi-embedding/Cargo.toml`:

```toml
[package]
name = "nomi-embedding"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
serde_json = "1"
thiserror = "1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

- [ ] **Step 2: Move the source files**

```bash
mkdir -p backend/crates/nomi-embedding/src
git -C backend mv src/embedding/types.rs crates/nomi-embedding/src/types.rs
git -C backend mv src/embedding/config.rs crates/nomi-embedding/src/config.rs
git -C backend mv src/embedding/openai.rs crates/nomi-embedding/src/openai.rs
git -C backend mv src/embedding/fake.rs crates/nomi-embedding/src/fake.rs
git -C backend mv src/embedding/mod.rs crates/nomi-embedding/src/lib.rs
```

Contents unchanged, same reasoning as Task 1 Step 3.

- [ ] **Step 3: Alias the old module**

Edit `backend/src/lib.rs`, change `pub mod embedding;` to `pub use nomi_embedding as embedding;`.

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-embedding = { path = "crates/nomi-embedding" }`.

- [ ] **Step 4: Move the tests**

```bash
mkdir -p backend/crates/nomi-embedding/tests
git -C backend mv tests/embedding_config.rs crates/nomi-embedding/tests/embedding_config.rs
git -C backend mv tests/embedding_openai.rs crates/nomi-embedding/tests/embedding_openai.rs
```

Change every `use nomi_orchestrator::embedding::` to `use nomi_embedding::` in both files.

- [ ] **Step 5: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: extract nomi-embedding crate from the backend monolith"
```

---

### Task 3: `nomi-realtime`

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/lib.rs`
- Create: `backend/crates/nomi-realtime/Cargo.toml`
- Move: `backend/src/realtime/*.rs` → `backend/crates/nomi-realtime/src/*.rs` (`mod.rs` → `lib.rs`)
- Move: `backend/tests/realtime_mqtt.rs`, `realtime_envelope.rs` → `backend/crates/nomi-realtime/tests/`
- Delete: `backend/src/realtime/`

**Interfaces:**
- Consumes: `nomi_llm::StreamEvent` (used inside `StreamEnvelope::Delta`).
- Produces: crate `nomi-realtime` — `MqttPublisher`, `MqttError`, `StreamEnvelope`.

- [ ] **Step 1: Create `nomi-realtime`'s manifest**

Create `backend/crates/nomi-realtime/Cargo.toml`:

```toml
[package]
name = "nomi-realtime"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-llm = { path = "../nomi-llm" }
rumqttc = "0.25.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"
tracing = "0.1.44"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Move the source files**

```bash
mkdir -p backend/crates/nomi-realtime/src
git -C backend mv src/realtime/mqtt.rs crates/nomi-realtime/src/mqtt.rs
git -C backend mv src/realtime/mod.rs crates/nomi-realtime/src/lib.rs
```

Inside the moved `lib.rs`, change `use crate::llm::StreamEvent;` to `use nomi_llm::StreamEvent;`. Nothing else changes.

- [ ] **Step 3: Alias the old module**

Edit `backend/src/lib.rs`, change `pub mod realtime;` to `pub use nomi_realtime as realtime;`.

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-realtime = { path = "crates/nomi-realtime" }`.

- [ ] **Step 4: Move the tests**

```bash
mkdir -p backend/crates/nomi-realtime/tests
git -C backend mv tests/realtime_mqtt.rs crates/nomi-realtime/tests/realtime_mqtt.rs
git -C backend mv tests/realtime_envelope.rs crates/nomi-realtime/tests/realtime_envelope.rs
```

In both: change `use nomi_orchestrator::realtime::` to `use nomi_realtime::`. In `realtime_envelope.rs` specifically, also change `use nomi_orchestrator::llm::{PartialBlock, StopReason, StreamEvent};` to `use nomi_llm::{PartialBlock, StopReason, StreamEvent};`.

- [ ] **Step 5: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: extract nomi-realtime crate from the backend monolith"
```

---

### Task 4: `nomi-auth`

This task also resolves the one real circular-dependency risk in the codebase (spec §4): `AuthClaims`'s extractor becomes generic over a small trait instead of the concrete `AppState`, so `nomi-auth` never depends on the old monolith (or, later, on `nomi-server`).

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/lib.rs`
- Modify: `backend/src/app.rs` (implement the new trait for `AppState` — the only edit this task makes to a file it doesn't otherwise own)
- Create: `backend/crates/nomi-auth/Cargo.toml`
- Move: `backend/src/auth/*.rs` → `backend/crates/nomi-auth/src/*.rs` (`mod.rs` → `lib.rs`)
- Move: `backend/tests/auth_authorize.rs`, `auth_extractor.rs`, `auth_login.rs`, `auth_permissions.rs`, `auth_refresh.rs`, `auth_registration.rs`, `auth_schema.rs` → `backend/crates/nomi-auth/tests/`
- Delete: `backend/src/auth/`

**Interfaces:**
- Produces: crate `nomi-auth` — `claims::Claims`, `extractor::AuthClaims`, `extractor::HasJwtSecret` (new), `login::{login, LoginError}`, `password::*`, `permissions::compute_permissions`, `registration::{register_user, OrgMode, RegistrationError}`, `authorize::{authorize_org_action, AuthorizeError}`, `refresh_token::*`.
- Consumes: nothing workspace-internal.

- [ ] **Step 1: Create `nomi-auth`'s manifest**

Create `backend/crates/nomi-auth/Cargo.toml`:

```toml
[package]
name = "nomi-auth"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["ws"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
jsonwebtoken = "9"
argon2 = "0.5"
rand = "0.8"
sha2 = "0.10"
hex = "0.4"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
```

- [ ] **Step 2: Move the source files**

```bash
mkdir -p backend/crates/nomi-auth/src
git -C backend mv src/auth/authorize.rs crates/nomi-auth/src/authorize.rs
git -C backend mv src/auth/claims.rs crates/nomi-auth/src/claims.rs
git -C backend mv src/auth/extractor.rs crates/nomi-auth/src/extractor.rs
git -C backend mv src/auth/login.rs crates/nomi-auth/src/login.rs
git -C backend mv src/auth/password.rs crates/nomi-auth/src/password.rs
git -C backend mv src/auth/permissions.rs crates/nomi-auth/src/permissions.rs
git -C backend mv src/auth/refresh_token.rs crates/nomi-auth/src/refresh_token.rs
git -C backend mv src/auth/registration.rs crates/nomi-auth/src/registration.rs
git -C backend mv src/auth/mod.rs crates/nomi-auth/src/lib.rs
```

- [ ] **Step 3: Decouple the extractor from `AppState`**

Replace the full contents of the moved `backend/crates/nomi-auth/src/extractor.rs` with:

```rust
use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};

use super::claims::Claims;

/// Implemented by whatever axum state type wires up JWT auth (today, `AppState` in the
/// server crate) — lets `AuthClaims` work as an extractor for any state without nomi-auth
/// depending on the server crate that defines it.
pub trait HasJwtSecret {
    fn jwt_secret(&self) -> &str;
}

pub struct AuthClaims(pub Claims);

#[axum::async_trait]
impl<S: HasJwtSecret + Send + Sync> FromRequestParts<S> for AuthClaims {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "malformed authorization header"))?;

        let claims = Claims::decode(token, state.jwt_secret())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

        Ok(AuthClaims(claims))
    }
}
```

- [ ] **Step 4: Implement the trait for the (still old-monolith-owned) `AppState`**

Edit `backend/src/app.rs`. Add, right after the `AppState` struct definition:

```rust
impl crate::auth::extractor::HasJwtSecret for AppState {
    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}
```

(`crate::auth` still resolves correctly here — it's the alias set up in Step 5 below. This is the one edit this task makes outside `nomi-auth` itself; `app.rs` is not otherwise touched until Task 10.)

- [ ] **Step 5: Alias the old module**

Edit `backend/src/lib.rs`, change `pub mod auth;` to `pub use nomi_auth as auth;`.

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-auth = { path = "crates/nomi-auth" }`.

- [ ] **Step 6: Move the tests**

```bash
mkdir -p backend/crates/nomi-auth/tests
git -C backend mv tests/auth_authorize.rs crates/nomi-auth/tests/auth_authorize.rs
git -C backend mv tests/auth_extractor.rs crates/nomi-auth/tests/auth_extractor.rs
git -C backend mv tests/auth_login.rs crates/nomi-auth/tests/auth_login.rs
git -C backend mv tests/auth_permissions.rs crates/nomi-auth/tests/auth_permissions.rs
git -C backend mv tests/auth_refresh.rs crates/nomi-auth/tests/auth_refresh.rs
git -C backend mv tests/auth_registration.rs crates/nomi-auth/tests/auth_registration.rs
git -C backend mv tests/auth_schema.rs crates/nomi-auth/tests/auth_schema.rs
```

In every file except `auth_extractor.rs`: change `use nomi_orchestrator::auth::` to `use nomi_auth::`.

`auth_extractor.rs` needs a real edit, not just an import rename: read its current contents first (it currently builds a real `AppState` to exercise the extractor). Replace its `AppState` construction with a minimal local test type implementing `HasJwtSecret`, e.g.:

```rust
struct TestState { jwt_secret: String }

impl nomi_auth::extractor::HasJwtSecret for TestState {
    fn jwt_secret(&self) -> &str { &self.jwt_secret }
}
```

...and use `TestState` wherever the test previously constructed `AppState`, keeping every assertion (valid token accepted, missing/malformed header rejected, expired/invalid token rejected) unchanged — only the state type under test changes. Change `use nomi_orchestrator::app::AppState;` / `use nomi_orchestrator::auth::{claims::Claims, extractor::AuthClaims};` to `use nomi_auth::{claims::Claims, extractor::{AuthClaims, HasJwtSecret}};`.

- [ ] **Step 7: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor: extract nomi-auth crate, decouple AuthClaims from AppState"
```

---

### Task 5: `nomi-settings`

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/lib.rs`
- Create: `backend/crates/nomi-settings/Cargo.toml`
- Move: `backend/src/settings/*.rs` → `backend/crates/nomi-settings/src/*.rs` (`mod.rs` → `lib.rs`)
- Move: `backend/tests/llm_models_db.rs`, `provider_settings.rs` → `backend/crates/nomi-settings/tests/`
- Delete: `backend/src/settings/`

**Interfaces:**
- Produces: crate `nomi-settings` — `ProviderSettingsRow`, `UpsertInput`, `get_settings`, `upsert_settings`, `mask_api_key`, `crypto::{encrypt, decrypt, CryptoError}`, `llm_models::*` (all the `AdminLlmModel`/`UserLlmSelectionRow`/CRUD functions).
- Consumes: nothing workspace-internal.

- [ ] **Step 1: Create `nomi-settings`'s manifest**

Create `backend/crates/nomi-settings/Cargo.toml`:

```toml
[package]
name = "nomi-settings"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
aes-gcm = "0.10"
rand = "0.8"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Move the source files**

```bash
mkdir -p backend/crates/nomi-settings/src
git -C backend mv src/settings/crypto.rs crates/nomi-settings/src/crypto.rs
git -C backend mv src/settings/llm_models.rs crates/nomi-settings/src/llm_models.rs
git -C backend mv src/settings/mod.rs crates/nomi-settings/src/lib.rs
```

- [ ] **Step 3: Alias the old module**

Edit `backend/src/lib.rs`, change `pub mod settings;` to `pub use nomi_settings as settings;`.

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-settings = { path = "crates/nomi-settings" }`.

- [ ] **Step 4: Move the tests**

```bash
mkdir -p backend/crates/nomi-settings/tests
git -C backend mv tests/llm_models_db.rs crates/nomi-settings/tests/llm_models_db.rs
git -C backend mv tests/provider_settings.rs crates/nomi-settings/tests/provider_settings.rs
```

In both: change `use nomi_orchestrator::settings::` to `use nomi_settings::`.

- [ ] **Step 5: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: extract nomi-settings crate from the backend monolith"
```

---

### Task 6: `nomi-agent-core`

The first task with real new logic, not just a move: the `SubAgent` trait gains four methods, the tool-calling loop generalizes to always stream and to optionally retrieve/extract memory, and `turn/memory.rs` moves here as a capability any agent can opt into. This crate is the stable extension point every future agent crate depends on.

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/lib.rs`, `backend/src/turn/mod.rs` (its `run_subagent_turn` caller — see Step 5)
- Create: `backend/crates/nomi-agent-core/Cargo.toml`
- Move: `backend/src/turn/subagent.rs` → `backend/crates/nomi-agent-core/src/subagent.rs` (extended, see Step 2)
- Move: `backend/src/turn/tools.rs` → `backend/crates/nomi-agent-core/src/engine.rs` (generalized, see Step 3)
- Move: `backend/src/turn/memory.rs` → `backend/crates/nomi-agent-core/src/memory.rs` (verbatim)
- Move: `backend/src/turn/types.rs`'s `TurnError` → `backend/crates/nomi-agent-core/src/error.rs` (see Step 4; `TurnOutcome` stays behind, it moves to `nomi-turn` in Task 9)
- Create: `backend/crates/nomi-agent-core/src/registry.rs` (new — `AgentRegistry`)
- Create: `backend/crates/nomi-agent-core/src/lib.rs` (new crate root)
- Move: `backend/tests/turn_tools.rs` → `backend/crates/nomi-agent-core/tests/engine.rs` (adapted, see Step 6)
- Move: `backend/tests/turn_memory_retrieval.rs`, `turn_memory_writing.rs`, `turn_reinforcement.rs` → `backend/crates/nomi-agent-core/tests/`
- Create: `backend/crates/nomi-agent-core/tests/registry.rs` (new tests for `AgentRegistry`)

**Interfaces:**
- Consumes: `nomi_llm::*`, `nomi_embedding::EmbeddingProvider`, `nomi_realtime::{MqttPublisher, StreamEnvelope}`.
- Produces: `SubAgent` trait (extended), `AgentRegistry`, `TurnError`, `run_agent_turn` (the generalized engine, replacing `run_tool_calling_loop`), `LoopOutcome`, `COMPLETE_TASK_TOOL_NAME`, `memory::{retrieve_relevant_memories, extract_and_store_memory, reinforce, RetrievedMemory, ReinforcementSignal}`.

- [ ] **Step 1: Create `nomi-agent-core`'s manifest**

Create `backend/crates/nomi-agent-core/Cargo.toml`:

```toml
[package]
name = "nomi-agent-core"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-llm = { path = "../nomi-llm" }
nomi-embedding = { path = "../nomi-embedding" }
nomi-realtime = { path = "../nomi-realtime" }
async-trait = "0.1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }
serde_json = "1"
thiserror = "1"
futures-util = "0.3.33"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Extend the `SubAgent` trait**

```bash
mkdir -p backend/crates/nomi-agent-core/src
git -C backend mv src/turn/subagent.rs crates/nomi-agent-core/src/subagent.rs
```

Replace the full contents of the moved `subagent.rs` with:

```rust
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String>;

    /// Fed verbatim into the intent classifier's prompt. Never called for the agent that
    /// returns `true` from `is_default()` — that agent is the fallback, not something the
    /// classifier picks between.
    fn intent_label(&self) -> &'static str;
    fn intent_description(&self) -> &'static str;

    /// Exactly one registered agent must return `true`. See `AgentRegistry::new`.
    fn is_default(&self) -> bool {
        false
    }

    /// When `true`, `run_agent_turn` retrieves relevant memories before the first LLM call
    /// (folded into the system prompt) and extracts+stores new memories after a `Reply`
    /// outcome (never after `Completed` — a completed task summary isn't a conversational
    /// reply worth remembering facts from).
    fn uses_memory(&self) -> bool {
        false
    }
}
```

- [ ] **Step 3: Generalize the engine**

```bash
git -C backend mv src/turn/tools.rs crates/nomi-agent-core/src/engine.rs
```

Replace the full contents of the moved `engine.rs` with:

```rust
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use nomi_embedding::EmbeddingProvider;
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::error::TurnError;
use crate::memory;
use crate::subagent::SubAgent;

const MAX_TOOL_TURNS: u32 = 10;
pub const COMPLETE_TASK_TOOL_NAME: &str = "complete_task";

fn complete_task_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: COMPLETE_TASK_TOOL_NAME.to_string(),
        description: "Call this when you are done helping with this task, whether it succeeded or the user wants to stop.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "status": {"type": "string", "enum": ["completed", "cancelled"]},
                "summary": {"type": "string"}
            },
            "required": ["status", "summary"]
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply(String),
    Completed { status: String, summary: String },
}

/// Runs one agent turn to completion: retrieves memory first if `agent.uses_memory()`,
/// then drives the tool-calling loop with every LLM call streamed (deltas published over
/// `mqtt` when provided — this now happens for every agent, not just a hardcoded chitchat
/// case), then extracts+stores memory after a `Reply` outcome if `agent.uses_memory()`.
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    mut messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
    let mut tools = agent.tools();
    tools.push(complete_task_tool_definition());

    let memories = if agent.uses_memory() {
        let last_user_text = messages
            .iter()
            .rev()
            .find(|m| m.role == LlmRole::User)
            .and_then(|m| m.content.iter().find_map(|b| match b {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            }))
            .unwrap_or_default();
        memory::try_retrieve_memories(conn, embedding_provider, user_id, &last_user_text).await
    } else {
        Vec::new()
    };

    let system_prompt = if memories.is_empty() {
        agent.system_prompt().to_string()
    } else {
        let mut prompt = format!("{}\n\nRelevant things you know about this user:\n", agent.system_prompt());
        for m in &memories {
            prompt.push_str(&format!("- {}\n", m.content));
        }
        prompt
    };

    for _ in 0..MAX_TOOL_TURNS {
        let request = LlmRequest {
            system: Some(system_prompt.clone()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
        };

        // The LLM call happens outside any DB transaction: holding a transaction open across
        // a slow network round trip would needlessly extend how long this connection's locks
        // are held.
        let stream = provider.complete_stream(request).await.map_err(TurnError::LlmCallFailed)?;
        let response = match mqtt {
            Some((publisher, turn_job_id)) => {
                use futures_util::StreamExt;
                // LlmEventStream requires 'static (boxed as `dyn Stream + Send`, no lifetime),
                // so this clones the publisher handle (cheap — wraps rumqttc's AsyncClient,
                // itself a cheap handle clone) rather than capturing the `&MqttPublisher`
                // borrow.
                let publisher = publisher.clone();
                let published = stream.then(move |event_result| {
                    let publisher = publisher.clone();
                    async move {
                        if let Ok(event) = &event_result {
                            let envelope = StreamEnvelope::Delta { turn_job_id, event: event.clone() };
                            // Best-effort: an MQTT publish failure never fails the turn.
                            let _ = publisher.publish(session_id, &envelope).await;
                        }
                        event_result
                    }
                });
                nomi_llm::collect_stream(Box::pin(published)).await.map_err(TurnError::LlmCallFailed)?
            }
            None => nomi_llm::collect_stream(stream).await.map_err(TurnError::LlmCallFailed)?,
        };

        messages.push(LlmMessage { role: LlmRole::Assistant, content: response.content.clone() });

        if response.stop_reason != StopReason::ToolUse {
            let reply_text = response
                .content
                .into_iter()
                .find_map(|block| match block {
                    ContentBlock::Text { text } => Some(text),
                    _ => None,
                })
                .unwrap_or_default();

            if agent.uses_memory() {
                let last_user_text = messages
                    .iter()
                    .rev()
                    .find(|m| m.role == LlmRole::User)
                    .and_then(|m| m.content.iter().find_map(|b| match b {
                        ContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    }))
                    .unwrap_or_default();
                // Best-effort: never changes the turn's outcome. See the equivalent
                // note that used to live in turn/chitchat.rs before this generalization.
                memory::extract_and_store_memory(conn, provider, embedding_provider, user_id, &last_user_text, &reply_text).await;
            }

            return Ok(LoopOutcome::Reply(reply_text));
        }

        let mut tool_results = Vec::new();
        for block in &response.content {
            if let ContentBlock::ToolUse { id, name, input } = block {
                let (result_text, is_error) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false)
                } else {
                    match agent.execute_tool(conn, user_id, name, input.clone()).await {
                        Ok(text) => (text, false),
                        Err(err) => (err, true),
                    }
                };

                log_tool_call(conn, session_id, agent_session_id, agent.agent_type(), name, input, &result_text, is_error).await;

                if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
                    let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    return Ok(LoopOutcome::Completed { status, summary });
                }

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: id.clone(),
                    content: result_text,
                    is_error,
                });
            }
        }

        messages.push(LlmMessage { role: LlmRole::User, content: tool_results });
    }

    Err(TurnError::ToolLoopExceeded)
}

async fn log_tool_call(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_type: &str,
    tool_name: &str,
    input: &serde_json::Value,
    result: &str,
    is_error: bool,
) {
    let _ = sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'ToolCalled', $4)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(agent_type)
    .bind(serde_json::json!({"tool_name": tool_name, "input": input, "result": result, "is_error": is_error}))
    .execute(&mut **conn)
    .await;
}
```

Note the two differences from the original `run_tool_calling_loop`: it takes `mqtt`/`embedding_provider`/`session_id` now (needed for streaming and memory), and it always streams via `complete_stream()` instead of `complete()`.

- [ ] **Step 4: Move memory and the error type**

```bash
git -C backend mv src/turn/memory.rs crates/nomi-agent-core/src/memory.rs
```

Inside the moved `memory.rs`: change `use crate::embedding::EmbeddingProvider;` to `use nomi_embedding::EmbeddingProvider;`, and `use crate::llm::{...}` to `use nomi_llm::{...}`. Also change `use super::types::TurnError;` to `use crate::error::TurnError;`.

Add one new function to `memory.rs` — the memory-retrieval helper the engine calls, lifted verbatim out of what used to be `turn/chitchat.rs`'s private `try_retrieve_memories`:

```rust
pub async fn try_retrieve_memories(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    embedding_provider: &dyn nomi_embedding::EmbeddingProvider,
    user_id: uuid::Uuid,
    text: &str,
) -> Vec<RetrievedMemory> {
    const MEMORY_RETRIEVAL_LIMIT: i64 = 5;
    let embedding = match embedding_provider.embed(text).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    retrieve_relevant_memories(conn, user_id, &embedding, MEMORY_RETRIEVAL_LIMIT)
        .await
        .unwrap_or_default()
}
```

(Place it anywhere in the file; it calls the file's own pre-existing `retrieve_relevant_memories`.)

Create `backend/crates/nomi-agent-core/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("llm call failed: {0}")]
    LlmCallFailed(#[from] nomi_llm::LlmError),
    #[error("tool-calling loop exceeded its turn limit without completing")]
    ToolLoopExceeded,
}
```

- [ ] **Step 5: Write the `AgentRegistry`**

Create `backend/crates/nomi-agent-core/src/registry.rs`:

```rust
use crate::subagent::SubAgent;

pub struct AgentRegistry {
    agents: Vec<Box<dyn SubAgent>>,
    default_index: usize,
}

impl AgentRegistry {
    /// Panics if zero or more than one agent returns `true` from `is_default()` — a
    /// misconfigured registry is a startup-time programmer error, not something to degrade
    /// gracefully from at runtime.
    pub fn new(agents: Vec<Box<dyn SubAgent>>) -> Self {
        let default_indices: Vec<usize> = agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.is_default())
            .map(|(i, _)| i)
            .collect();

        match default_indices.as_slice() {
            [index] => Self { agents, default_index: *index },
            [] => panic!("AgentRegistry::new: no registered agent returns true from is_default()"),
            _ => panic!(
                "AgentRegistry::new: more than one registered agent returns true from is_default(): {:?}",
                default_indices.iter().map(|&i| agents[i].agent_type()).collect::<Vec<_>>()
            ),
        }
    }

    pub fn default_agent(&self) -> &dyn SubAgent {
        self.agents[self.default_index].as_ref()
    }

    pub fn find(&self, agent_type: &str) -> Option<&dyn SubAgent> {
        self.agents.iter().find(|a| a.agent_type() == agent_type).map(|a| a.as_ref())
    }

    /// Finds the agent whose `intent_label()` matches, ignoring case/whitespace (matching
    /// how the classifier's raw LLM text response is normalized before lookup).
    pub fn find_by_intent_label(&self, label: &str) -> Option<&dyn SubAgent> {
        let normalized = label.trim().to_lowercase();
        self.agents.iter().find(|a| a.intent_label() == normalized).map(|a| a.as_ref())
    }

    /// Builds the classifier's prompt from every non-default registered agent's
    /// intent_label/intent_description. The default agent is the fallback, not a
    /// classification target (matching today's behavior, where chitchat only wins via
    /// explicit classification OR fallback, never by having its own enum variant picked
    /// exclusively for it).
    pub fn classification_prompt(&self) -> String {
        let options: Vec<String> = self
            .agents
            .iter()
            .filter(|a| !a.is_default())
            .map(|a| format!("{}: {}", a.intent_label(), a.intent_description()))
            .collect();

        let labels: Vec<&str> = self
            .agents
            .iter()
            .filter(|a| !a.is_default())
            .map(|a| a.intent_label())
            .collect();

        format!(
            "Classify the user's message as exactly one of: {}, or \"{}\" if none of the specific \
             categories apply. Reply with only that single word, nothing else.\n\nCategories:\n{}",
            labels.join(", "),
            self.default_agent().intent_label(),
            options.join("\n"),
        )
    }
}
```

- [ ] **Step 6: Create the crate root**

Create `backend/crates/nomi-agent-core/src/lib.rs`:

```rust
pub mod engine;
pub mod error;
pub mod memory;
pub mod registry;
pub mod subagent;

pub use engine::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME};
pub use error::TurnError;
pub use registry::AgentRegistry;
pub use subagent::SubAgent;
```

- [ ] **Step 7: Update `turn/mod.rs`'s call site (temporary — full rewrite comes in Task 9)**

`turn/mod.rs`'s `run_subagent_turn` (which calls `tools::run_tool_calling_loop`) still lives in the old monolith at this point, and its own home (`turn::run_locked_turn`'s registry-driven rewrite) is Task 9's job, not this one. For now, only fix the compile break this task causes: edit `backend/src/turn/mod.rs`, change the `run_subagent_turn` function's body from calling `tools::run_tool_calling_loop(conn, provider, agent, session_id, agent_session_id, user_id, messages, SUBAGENT_MAX_TOKENS)` to calling the new signature:

```rust
crate::agent_core::run_agent_turn(
    conn,
    None, // mqtt: money_agent-style turns don't stream yet at this point in the migration —
          // Task 9 threads `mqtt` through from `process_turn` once run_locked_turn itself
          // is rewritten. Passing None here preserves today's exact behavior (no streaming
          // for subagent turns) until that task deliberately changes it.
    provider,
    embedding_provider,
    agent,
    session_id,
    agent_session_id,
    user_id,
    messages,
    SUBAGENT_MAX_TOKENS,
)
```

This requires `run_subagent_turn` to also receive `embedding_provider` as a parameter now (it didn't before) — thread it through from its one call site in `run_locked_turn` (which already has `embedding_provider` in scope). Also add, near the top of `turn/mod.rs`: `use super::subagent;` becomes unnecessary (the module moved) — remove the now-dead `pub mod subagent;` and `pub mod tools;` and `pub mod memory;` lines from `turn/mod.rs`'s module list, and change `mod.rs`'s own `use crate::llm::...`-derived references to `money_agent::MoneyAgent` etc. to keep compiling by adding, near its other `use` lines:

```rust
use crate::agent_core::{self, SubAgent};
```

and change every unqualified `subagent::SubAgent` / `tools::LoopOutcome` reference in `turn/mod.rs` to `agent_core::SubAgent` / `agent_core::LoopOutcome`.

Edit `backend/src/lib.rs`: add `pub use nomi_agent_core as agent_core;` (this is a new alias — `agent_core` didn't exist as an old module name, so this is purely additive, not a replacement of an existing `pub mod` line).

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-agent-core = { path = "crates/nomi-agent-core" }`.

- [ ] **Step 8: Move the tests**

```bash
mkdir -p backend/crates/nomi-agent-core/tests
git -C backend mv tests/turn_tools.rs crates/nomi-agent-core/tests/engine.rs
git -C backend mv tests/turn_memory_retrieval.rs crates/nomi-agent-core/tests/turn_memory_retrieval.rs
git -C backend mv tests/turn_memory_writing.rs crates/nomi-agent-core/tests/turn_memory_writing.rs
git -C backend mv tests/turn_reinforcement.rs crates/nomi-agent-core/tests/turn_reinforcement.rs
```

In `turn_memory_retrieval.rs`, `turn_memory_writing.rs`, `turn_reinforcement.rs`: change `use nomi_orchestrator::turn::memory::` to `use nomi_agent_core::memory::`, and any `use nomi_orchestrator::llm::` to `use nomi_llm::`.

`engine.rs` (formerly `turn_tools.rs`) needs real adaptation, not just an import rename, because `run_tool_calling_loop` became `run_agent_turn` with a wider signature. Read the moved file's current contents first, then for every call site: rename `run_tool_calling_loop` to `run_agent_turn`, and add two new arguments matching the new signature — a fake `embedding_provider` (this crate's tests already construct fake `LlmProvider`s; do the same for `EmbeddingProvider`, e.g. reuse `nomi_embedding`'s test-only fake if it's `pub`, or a local test-only stub returning `Ok(vec![])`/an error, matching whatever pattern the existing test file already uses for its fake `LlmProvider`) and `mqtt: None` — every existing assertion in this file should keep passing unchanged, since `uses_memory()` defaults to `false` on whatever test `SubAgent` impl this file defines (unless the file's fake agent needs a new `intent_label`/`intent_description`/`is_default`/`uses_memory` impl added to satisfy the extended trait — add minimal ones: `intent_label() -> "test"`, `intent_description() -> "test agent"`, leave `is_default`/`uses_memory` at their trait defaults). Change `use nomi_orchestrator::llm::` to `use nomi_llm::`, `use nomi_orchestrator::turn::subagent::SubAgent` to `use nomi_agent_core::SubAgent`, `use nomi_orchestrator::turn::tools::{run_tool_calling_loop, LoopOutcome, COMPLETE_TASK_TOOL_NAME}` to `use nomi_agent_core::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME}`.

- [ ] **Step 9: Write `AgentRegistry` tests**

Create `backend/crates/nomi-agent-core/tests/registry.rs`:

```rust
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, SubAgent};
use nomi_llm::ToolDefinition;

struct StubAgent {
    agent_type: &'static str,
    intent_label: &'static str,
    is_default: bool,
}

#[async_trait]
impl SubAgent for StubAgent {
    fn agent_type(&self) -> &'static str {
        self.agent_type
    }
    fn system_prompt(&self) -> &'static str {
        "stub"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        self.intent_label
    }
    fn intent_description(&self) -> &'static str {
        "a stub agent"
    }
    fn is_default(&self) -> bool {
        self.is_default
    }
}

fn stub(agent_type: &'static str, intent_label: &'static str, is_default: bool) -> Box<dyn SubAgent> {
    Box::new(StubAgent { agent_type, intent_label, is_default })
}

#[test]
fn finds_a_registered_agent_by_agent_type() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    assert_eq!(registry.find("money").unwrap().agent_type(), "money");
    assert!(registry.find("nonexistent").is_none());
}

#[test]
fn finds_the_default_agent() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    assert_eq!(registry.default_agent().agent_type(), "chitchat");
}

#[test]
fn finds_an_agent_by_intent_label_case_insensitively() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    assert_eq!(registry.find_by_intent_label("Money").unwrap().agent_type(), "money");
    assert_eq!(registry.find_by_intent_label("  money  ").unwrap().agent_type(), "money");
}

#[test]
fn classification_prompt_excludes_the_default_agent_as_a_target_but_names_it_as_the_fallback() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    let prompt = registry.classification_prompt();
    assert!(prompt.contains("money"));
    assert!(prompt.contains("chitchat")); // named as the fallback, not filtered out of the text entirely
}

#[test]
#[should_panic(expected = "no registered agent returns true from is_default")]
fn panics_with_zero_default_agents() {
    AgentRegistry::new(vec![stub("money", "money", false)]);
}

#[test]
#[should_panic(expected = "more than one registered agent returns true from is_default")]
fn panics_with_more_than_one_default_agent() {
    AgentRegistry::new(vec![stub("a", "a", true), stub("b", "b", true)]);
}
```

- [ ] **Step 10: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass, including the new `registry.rs` tests (6 tests).

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: extract nomi-agent-core with a SubAgent registry and a unified streaming+memory engine"
```

---

### Task 7: `nomi-agent-money`

A straight port — `MoneyAgent`'s behavior is unchanged, it just gains the four new trait methods and moves to its own crate.

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/turn/mod.rs` (the `money_agent::MoneyAgent` construction sites)
- Create: `backend/crates/nomi-agent-money/Cargo.toml`
- Move: `backend/src/turn/money_agent.rs` → `backend/crates/nomi-agent-money/src/lib.rs`
- Move: `backend/tests/turn_money_agent.rs` → `backend/crates/nomi-agent-money/tests/`
- Delete: `backend/src/turn/money_agent.rs` (via the move)

**Interfaces:**
- Consumes: `nomi_agent_core::SubAgent`, `nomi_llm::ToolDefinition`.
- Produces: `MoneyAgent`, `MONEY_AGENT_TYPE`.

- [ ] **Step 1: Create `nomi-agent-money`'s manifest**

Create `backend/crates/nomi-agent-money/Cargo.toml`:

```toml
[package]
name = "nomi-agent-money"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-llm = { path = "../nomi-llm" }
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Move and extend `MoneyAgent`**

```bash
mkdir -p backend/crates/nomi-agent-money/src
git -C backend mv src/turn/money_agent.rs crates/nomi-agent-money/src/lib.rs
```

In the moved `lib.rs`: change `use crate::llm::ToolDefinition;` to `use nomi_llm::ToolDefinition;`, and `use super::subagent::SubAgent;` to `use nomi_agent_core::SubAgent;`.

Inside the `impl SubAgent for MoneyAgent` block, add these four methods (the trait's other four — `agent_type`, `system_prompt`, `tools`, `execute_tool` — are unchanged):

```rust
    fn intent_label(&self) -> &'static str {
        "money"
    }

    fn intent_description(&self) -> &'static str {
        "Questions about the user's transactions, spending, or budget"
    }
```

(`is_default()` and `uses_memory()` are left at the trait's defaults — `false` — since `MoneyAgent` doesn't override them.)

- [ ] **Step 3: Fix the old monolith's construction sites**

Edit `backend/src/turn/mod.rs`. Every place that currently does `&money_agent::MoneyAgent` or references `money_agent::MONEY_AGENT_TYPE`: change `money_agent::MoneyAgent` to `nomi_agent_money::MoneyAgent` and `money_agent::MONEY_AGENT_TYPE` to `nomi_agent_money::MONEY_AGENT_TYPE`. Remove the now-dead `pub mod money_agent;` line from `turn/mod.rs`'s module list (the file no longer exists at that path).

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-agent-money = { path = "crates/nomi-agent-money" }`.

- [ ] **Step 4: Move the tests**

```bash
mkdir -p backend/crates/nomi-agent-money/tests
git -C backend mv tests/turn_money_agent.rs crates/nomi-agent-money/tests/turn_money_agent.rs
```

Change `use nomi_orchestrator::turn::money_agent::MoneyAgent;` to `use nomi_agent_money::MoneyAgent;`, and `use nomi_orchestrator::turn::subagent::SubAgent;` to `use nomi_agent_core::SubAgent;`.

- [ ] **Step 5: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: extract nomi-agent-money crate"
```

---

### Task 8: `nomi-agent-chitchat`

This is where chitchat becomes "just another agent." Its bespoke history-fetch/memory/streaming code is deleted outright — that behavior now lives generically in `nomi-agent-core`'s engine (Task 6). What's left is small: a system prompt and four trait methods.

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/turn/mod.rs` (the chitchat call site — temporary, full rewrite in Task 9)
- Create: `backend/crates/nomi-agent-chitchat/Cargo.toml`
- Create: `backend/crates/nomi-agent-chitchat/src/lib.rs` (new — the slimmed-down `ChitchatAgent`)
- Delete: `backend/src/turn/chitchat.rs` (its logic is now `nomi-agent-core`'s engine + memory, ported in Task 6; nothing here is a move)
- Create: `backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs` (rewritten from `backend/tests/turn_chitchat.rs`, see Step 4)
- Delete: `backend/tests/turn_chitchat.rs` (via the rewrite)

**Interfaces:**
- Consumes: `nomi_agent_core::SubAgent`.
- Produces: `ChitchatAgent`, `CHITCHAT_AGENT_TYPE`.

- [ ] **Step 1: Create `nomi-agent-chitchat`'s manifest**

Create `backend/crates/nomi-agent-chitchat/Cargo.toml`:

```toml
[package]
name = "nomi-agent-chitchat"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-llm = { path = "../nomi-llm" }
async-trait = "0.1"
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Write `ChitchatAgent`**

Create `backend/crates/nomi-agent-chitchat/src/lib.rs`:

```rust
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const CHITCHAT_AGENT_TYPE: &str = "chitchat";

const CHITCHAT_SYSTEM_PROMPT: &str =
    "You are a helpful, friendly assistant chatting with the user. Keep replies concise.";

pub struct ChitchatAgent;

#[async_trait]
impl SubAgent for ChitchatAgent {
    fn agent_type(&self) -> &'static str {
        CHITCHAT_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        CHITCHAT_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }

    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _user_id: Uuid,
        _name: &str,
        _input: Value,
    ) -> Result<String, String> {
        // Unreachable: tools() returns an empty list, so the engine never calls this for
        // chitchat (the only tool it could ever see is complete_task, which the engine
        // handles itself before reaching an agent's execute_tool).
        Err("chitchat has no tools".to_string())
    }

    fn intent_label(&self) -> &'static str {
        CHITCHAT_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "General conversation, questions, or anything not covered by another agent"
    }

    fn is_default(&self) -> bool {
        true
    }

    fn uses_memory(&self) -> bool {
        true
    }
}
```

- [ ] **Step 3: Remove the old chitchat module and fix the old monolith's call site (temporary)**

```bash
git -C backend rm src/turn/chitchat.rs
```

Edit `backend/src/turn/mod.rs`: remove the `pub mod chitchat;` line. Every place that currently calls `chitchat::run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id, text)`: replace with a call into the generalized engine, matching how `run_subagent_turn` already calls it (Task 6 Step 7) but using `ChitchatAgent` and fetching history the same way `run_subagent_turn` does today (via `fetch_recent_messages`, already defined lower in this file) instead of chitchat's own now-deleted inline history fetch:

```rust
{
    let messages = fetch_recent_messages(conn, session_id).await?;
    // chitchat has no agent_session_id (it's not a stateful multi-turn agent session like
    // money_agent) — pass session_id itself; agent_events rows keyed to a "session" rather
    // than an "agent session" for chitchat's tool-call logging is harmless since chitchat
    // has no tools, so log_tool_call is never actually invoked for it.
    agent_core::run_agent_turn(
        conn,
        mqtt,
        provider,
        embedding_provider,
        &nomi_agent_chitchat::ChitchatAgent,
        session_id,
        session_id,
        user_id,
        messages,
        CHITCHAT_MAX_TOKENS,
    )
    .await
    .map(|outcome| match outcome {
        agent_core::LoopOutcome::Reply(text) => text,
        agent_core::LoopOutcome::Completed { summary, .. } => summary, // chitchat never completes; unreachable in practice
    })
}
```

Add `const CHITCHAT_MAX_TOKENS: u32 = 1024;` near `turn/mod.rs`'s other constants (matching the value chitchat.rs used to define). This is intentionally rough — it's a bridge to keep the monolith compiling for one more task; Task 9 replaces all of `run_locked_turn` (including this exact call site) with the real registry-driven version, so precision here matters less than "it compiles and existing tests still pass."

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-agent-chitchat = { path = "crates/nomi-agent-chitchat" }`.

- [ ] **Step 4: Rewrite the test**

Read `backend/tests/turn_chitchat.rs` first to see its exact current assertions (memory retrieval affecting the system prompt, streaming deltas published to a fake MQTT publisher, the reply persisted, memory extraction triggered afterward). Then create `backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs`, porting every one of those assertions to call `nomi_agent_core::run_agent_turn(..., &nomi_agent_chitchat::ChitchatAgent, ...)` directly instead of the deleted `run_chitchat_turn` — same fixtures (fake `LlmProvider`, fake `EmbeddingProvider`, a real Postgres pool via `#[sqlx::test]`), same behavior being verified, just through the new call shape. Delete the old file:

```bash
git -C backend rm tests/turn_chitchat.rs
```

- [ ] **Step 5: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: extract nomi-agent-chitchat — chitchat is now a SubAgent, not bespoke code"
```

---

### Task 9: `nomi-turn`

The orchestrator becomes registry-driven: `routing.rs`'s closed `Intent` enum is deleted, and `mod.rs`'s hardcoded dispatch is replaced with `AgentRegistry` lookups. This crate depends on `nomi-agent-core` only — never on `nomi-agent-money` or `nomi-agent-chitchat` directly, which is what makes it agent-count-agnostic (dependency inversion: the registry is passed in by the caller, wired up in Task 10's composition root).

**Files:**
- Modify: `backend/Cargo.toml`, `backend/src/lib.rs`
- Create: `backend/crates/nomi-turn/Cargo.toml`
- Move: `backend/src/turn/bootstrap.rs`, `lock.rs`, `queue.rs`, `ingest.rs` → `backend/crates/nomi-turn/src/*.rs` (verbatim)
- Move + rewrite: `backend/src/turn/routing.rs` → `backend/crates/nomi-turn/src/routing.rs` (registry-driven, see Step 3)
- Move + rewrite: `backend/src/turn/mod.rs` → `backend/crates/nomi-turn/src/lib.rs` (registry-driven, see Step 4)
- Move: `backend/src/turn/types.rs`'s `TurnOutcome` → `backend/crates/nomi-turn/src/lib.rs` (`TurnError` stays re-exported from `nomi-agent-core`, not duplicated)
- Move: `backend/tests/turn_bootstrap.rs`, `turn_ingest.rs`, `turn_lock.rs`, `turn_queue.rs`, `turn_routing.rs`, `turn_intent_classification.rs`, `turn_handle_inbound_message.rs`, `turn_process.rs`, `turn_subagent_state_machine.rs` → `backend/crates/nomi-turn/tests/` (see Step 5)
- Delete: `backend/src/turn/`

**Interfaces:**
- Consumes: `nomi_agent_core::{AgentRegistry, SubAgent, TurnError, LoopOutcome, run_agent_turn}`, `nomi_llm::*`, `nomi_embedding::EmbeddingProvider`, `nomi_realtime::{MqttPublisher, StreamEnvelope}`.
- Produces: `TurnOutcome`, `handle_inbound_message`, `process_turn` (both now take `&AgentRegistry` as a new parameter — see below), `routing::*`, `queue::*`, `lock::*`, `ingest::*`, `bootstrap::*`.

- [ ] **Step 1: Create `nomi-turn`'s manifest**

Create `backend/crates/nomi-turn/Cargo.toml`:

```toml
[package]
name = "nomi-turn"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-llm = { path = "../nomi-llm" }
nomi-embedding = { path = "../nomi-embedding" }
nomi-realtime = { path = "../nomi-realtime" }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1"

[dev-dependencies]
nomi-agent-money = { path = "../nomi-agent-money" }
nomi-agent-chitchat = { path = "../nomi-agent-chitchat" }
tokio = { version = "1", features = ["full"] }
```

(`nomi-agent-money`/`nomi-agent-chitchat` are dev-dependencies only — needed to build a real `AgentRegistry` in this crate's own integration tests, without the *library* ever depending on a concrete agent. This is the dependency-inversion property that makes new agents pluggable.)

- [ ] **Step 2: Move the plumbing files verbatim**

```bash
mkdir -p backend/crates/nomi-turn/src
git -C backend mv src/turn/bootstrap.rs crates/nomi-turn/src/bootstrap.rs
git -C backend mv src/turn/lock.rs crates/nomi-turn/src/lock.rs
git -C backend mv src/turn/queue.rs crates/nomi-turn/src/queue.rs
git -C backend mv src/turn/ingest.rs crates/nomi-turn/src/ingest.rs
```

In each of the four: change `use super::types::TurnError;` to `use nomi_agent_core::TurnError;`. Nothing else changes in these four files.

- [ ] **Step 3: Rewrite routing to be registry-driven**

```bash
git -C backend mv src/turn/routing.rs crates/nomi-turn/src/routing.rs
```

Replace the full contents with:

```rust
use chrono::{DateTime, Utc};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, SubAgent, TurnError};
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

const INTENT_CLASSIFICATION_MAX_TOKENS: u32 = 10;

/// Classifies `text` against whatever's registered in `registry`, returning the matching
/// agent (or the registry's default agent on no match, a parse failure, or an LLM error).
/// Replaces the old closed `Intent` enum — there is nothing here to edit when a new agent
/// is registered; the classifier prompt and the matching logic are both built from
/// `registry` at call time.
pub async fn classify_intent<'a>(provider: &dyn LlmProvider, registry: &'a AgentRegistry, text: &str) -> &'a dyn SubAgent {
    let request = LlmRequest {
        system: Some(registry.classification_prompt()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }],
        tools: vec![],
        max_tokens: INTENT_CLASSIFICATION_MAX_TOKENS,
    };

    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(_) => return registry.default_agent(),
    };

    let text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    registry.find_by_intent_label(&text).unwrap_or_else(|| registry.default_agent())
}

pub async fn find_active_agent_session(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
) -> Result<Option<Uuid>, TurnError> {
    let id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM agent_sessions WHERE session_id = $1 AND sender_channel_identity_id = $2 AND status = 'active'",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .fetch_optional(&mut **conn)
    .await?;
    Ok(id)
}

const AGENT_EXPIRY_HOURS: i64 = 24;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveAgentSessionDetails {
    pub agent_type: String,
    pub last_activity_at: DateTime<Utc>,
}

pub async fn load_active_agent_session_details(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
) -> Result<ActiveAgentSessionDetails, TurnError> {
    let (agent_type, last_activity_at): (String, DateTime<Utc>) =
        sqlx::query_as("SELECT agent_type, last_activity_at FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&mut **conn)
            .await?;
    Ok(ActiveAgentSessionDetails { agent_type, last_activity_at })
}

pub fn is_stale(last_activity_at: DateTime<Utc>) -> bool {
    Utc::now() - last_activity_at > chrono::Duration::hours(AGENT_EXPIRY_HOURS)
}

pub async fn mark_expired(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = 'expired', ended_at = now() WHERE id = $1")
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentExpired')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    Ok(())
}

pub async fn spawn_agent_session(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    agent_type: &str,
) -> Result<Uuid, TurnError> {
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(agent_type)
    .fetch_one(&mut **conn)
    .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentSpawned')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    Ok(agent_session_id)
}

pub async fn complete_agent_session(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
    status: &str,
    summary: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = $1, ended_at = now() WHERE id = $2")
        .bind(status)
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    let event_type = if status == "cancelled" { "AgentCancelled" } else { "AgentCompleted" };

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .bind(event_type)
        .bind(serde_json::json!({"summary": summary}))
        .execute(&mut **conn)
        .await?;

    Ok(())
}
```

(Everything below `classify_intent` is verbatim from today's `routing.rs` — only `classify_intent` itself changes shape, from returning a closed `Intent` enum to returning `&dyn SubAgent` looked up through the registry.)

- [ ] **Step 4: Rewrite the orchestrator**

```bash
git -C backend mv src/turn/mod.rs crates/nomi-turn/src/lib.rs
```

Replace the full contents with:

```rust
pub mod bootstrap;
pub mod ingest;
pub mod lock;
pub mod queue;
pub mod routing;

pub use nomi_agent_core::TurnError;

use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use nomi_agent_core::AgentRegistry;
use nomi_embedding::EmbeddingProvider;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

const SUBAGENT_HISTORY_LIMIT: i64 = 20;
const SUBAGENT_MAX_TOKENS: u32 = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
}

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    let result = run_locked_turn(&mut conn, None, provider, embedding_provider, registry, session_id, sender_channel_identity_id, user_id, text).await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// The worker's entry point (see backend/src/bin/worker.rs): processes an already-ingested
/// message (see nomi_turn::ingest::ingest_inbound_message) — bootstrap and the inbound
/// message insert have already happened, so this only acquires the session lock and runs
/// routing/dispatch, threading `mqtt`/`turn_job_id` through for live delta publishing.
#[allow(clippy::too_many_arguments)]
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    turn_job_id: Uuid,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = run_locked_turn(
        &mut conn,
        Some((mqtt, turn_job_id)),
        provider,
        embedding_provider,
        registry,
        session_id,
        sender_channel_identity_id,
        user_id,
        text,
    )
    .await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            // Best-effort: an MQTT publish failure never changes the turn's outcome.
            let _ = mqtt
                .publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id, error: err.to_string() })
                .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// Shared by handle_inbound_message and process_turn: routing/classification and agent
/// dispatch through `registry`, assuming the session lock is already held by the caller and
/// the inbound message has already been persisted (by the caller, before this runs).
#[allow(clippy::too_many_arguments)]
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
    let active = routing::find_active_agent_session(conn, session_id, sender_channel_identity_id).await?;

    enum RoutingOutcome<'a> {
        Continue { agent: &'a dyn nomi_agent_core::SubAgent, agent_session_id: Uuid },
        NeedsClassification,
    }

    let routing_outcome = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(conn, agent_session_id, session_id, &details.agent_type).await?;
                RoutingOutcome::NeedsClassification
            } else {
                match registry.find(&details.agent_type) {
                    Some(agent) => RoutingOutcome::Continue { agent, agent_session_id },
                    // The active session's agent_type isn't registered anymore (e.g. an
                    // agent crate was removed) — fall back to classifying fresh, same as
                    // an unrecognized/stale session.
                    None => RoutingOutcome::NeedsClassification,
                }
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue { agent, agent_session_id } => {
            run_subagent_turn(conn, mqtt, provider, embedding_provider, agent, session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::NeedsClassification => {
            let agent = routing::classify_intent(provider, registry, text).await;

            if agent.agent_type() == registry.default_agent().agent_type() {
                // The default agent (chitchat) never gets a persistent agent_sessions row —
                // matching today's behavior, where chitchat has no agent_session_id at all.
                // agent_session_id == session_id here purely as a stand-in for logging
                // (see nomi-agent-chitchat's own comment on this at its call site's origin).
                run_subagent_turn(conn, mqtt, provider, embedding_provider, agent, session_id, session_id, user_id).await
            } else {
                let agent_session_id =
                    routing::spawn_agent_session(conn, session_id, sender_channel_identity_id, agent.agent_type()).await?;
                run_subagent_turn(conn, mqtt, provider, embedding_provider, agent, session_id, agent_session_id, user_id).await
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    agent: &dyn nomi_agent_core::SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
) -> Result<String, TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = nomi_agent_core::run_agent_turn(
        conn,
        mqtt,
        provider,
        embedding_provider,
        agent,
        session_id,
        agent_session_id,
        user_id,
        messages,
        SUBAGENT_MAX_TOKENS,
    )
    .await?;

    match outcome {
        nomi_agent_core::LoopOutcome::Reply(reply_text) => {
            let mut tx = conn.begin().await?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                .bind(session_id)
                .bind(&reply_text)
                .execute(&mut *tx)
                .await?;
            sqlx::query("UPDATE agent_sessions SET last_activity_at = now() WHERE id = $1")
                .bind(agent_session_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(reply_text)
        }
        nomi_agent_core::LoopOutcome::Completed { status, summary } => {
            let mut tx = conn.begin().await?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                .bind(session_id)
                .bind(&summary)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;

            routing::complete_agent_session(conn, agent_session_id, session_id, agent.agent_type(), &status, &summary).await?;

            Ok(summary)
        }
    }
}

async fn fetch_recent_messages(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<Vec<LlmMessage>, TurnError> {
    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM ( \
             SELECT sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC",
    )
    .bind(session_id)
    .bind(SUBAGENT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(sender, content)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect())
}

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
```

Note `handle_inbound_message` and `process_turn` both gained a new `registry: &AgentRegistry` parameter — every caller (the old monolith's `worker.rs`, and any test) must pass one now. `run_subagent_turn` now handles chitchat too (via the default-agent branch in `run_locked_turn`), so the bridge code from Task 8 Step 3 is fully superseded and gone — chitchat and money_agent (and every future agent) go through the exact same code path from here on, which is the point of this whole plan.

- [ ] **Step 5: Move and adapt the tests**

```bash
mkdir -p backend/crates/nomi-turn/tests
git -C backend mv tests/turn_bootstrap.rs crates/nomi-turn/tests/turn_bootstrap.rs
git -C backend mv tests/turn_ingest.rs crates/nomi-turn/tests/turn_ingest.rs
git -C backend mv tests/turn_lock.rs crates/nomi-turn/tests/turn_lock.rs
git -C backend mv tests/turn_queue.rs crates/nomi-turn/tests/turn_queue.rs
git -C backend mv tests/turn_routing.rs crates/nomi-turn/tests/turn_routing.rs
git -C backend mv tests/turn_intent_classification.rs crates/nomi-turn/tests/turn_intent_classification.rs
git -C backend mv tests/turn_handle_inbound_message.rs crates/nomi-turn/tests/turn_handle_inbound_message.rs
git -C backend mv tests/turn_process.rs crates/nomi-turn/tests/turn_process.rs
git -C backend mv tests/turn_subagent_state_machine.rs crates/nomi-turn/tests/turn_subagent_state_machine.rs
```

For `turn_bootstrap.rs`, `turn_ingest.rs`, `turn_lock.rs`, `turn_queue.rs`: change `use nomi_orchestrator::turn::X::` to `use nomi_turn::X::`. No other changes needed.

For `turn_routing.rs`: change `use nomi_orchestrator::turn::routing::find_active_agent_session;` to `use nomi_turn::routing::find_active_agent_session;`. No other changes (this file doesn't touch `classify_intent`/`Intent`).

For `turn_intent_classification.rs`: read its current contents first — it tests `classify_intent`/`Intent` directly, both of which changed shape. Rewrite it to build a small `AgentRegistry` (using `nomi-agent-money`/`nomi-agent-chitchat` as dev-dependencies — real agents, not stubs, since this test is specifically about classification against the real registered set) and assert `classify_intent(provider, &registry, text).agent_type()` returns `"money"` or `"chitchat"` for the same fixture inputs the old test used to assert `Intent::Money`/`Intent::Chitchat` for. Change `use nomi_orchestrator::turn::routing::{classify_intent, Intent};` to `use nomi_turn::routing::classify_intent;` plus `use nomi_agent_core::AgentRegistry;` plus `use nomi_agent_money::MoneyAgent;` plus `use nomi_agent_chitchat::ChitchatAgent;`.

For `turn_handle_inbound_message.rs`, `turn_process.rs`, `turn_subagent_state_machine.rs`: read each file's current contents first. Every call to `handle_inbound_message(...)` or `process_turn(...)` needs a new `&registry` argument inserted at the position shown in Step 4's new signatures (right after `embedding_provider`); build the registry once per test the same way as `turn_intent_classification.rs` (`AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)])`). Change `use nomi_orchestrator::turn::` to `use nomi_turn::`, `use nomi_orchestrator::llm::` to `use nomi_llm::`, `use nomi_orchestrator::realtime::` to `use nomi_realtime::`. Every existing assertion (which agent responds to which fixture input, session persistence, streaming, tool-call sequencing) should keep passing unchanged — only the call sites gain the registry argument.

- [ ] **Step 6: Alias the old module and remove the dead `turn/types.rs`**

```bash
git -C backend rm src/turn/types.rs
```

Edit `backend/src/lib.rs`, change `pub mod turn;` to `pub use nomi_turn as turn;`. Remove the now-superseded `pub use nomi_agent_core as agent_core;` line added in Task 6 Step 7 (nothing outside `nomi-turn` needs to reach `agent_core` directly anymore — anything that did now goes through `crate::turn` instead, e.g. `crate::turn::TurnError`, matching `nomi-turn`'s own `pub use nomi_agent_core::TurnError;` re-export).

Edit `backend/Cargo.toml`'s `[dependencies]`, add: `nomi-turn = { path = "crates/nomi-turn" }`.

`backend/src/worker.rs` (still in the old monolith, untouched until Task 10) calls `crate::turn::queue` and, indirectly through `crate::turn::process_turn` inside its own body — check its current call to `process_turn` and add a registry argument there too, minimally: construct `nomi_agent_core::AgentRegistry::new(vec![Box::new(nomi_agent_money::MoneyAgent), Box::new(nomi_agent_chitchat::ChitchatAgent)])` once (e.g. at the top of the worker loop or wherever `process_turn` is called from) and pass `&registry`. `nomi-agent-core`, `nomi-agent-money`, and `nomi-agent-chitchat` are already present in `backend/Cargo.toml`'s `[dependencies]` from Tasks 6-8 — no manifest change needed here. This is the one other file (besides `app.rs` in Task 4) that this task touches outside `nomi-turn` itself, for the same reason: something in the not-yet-migrated monolith needs one precise edit to keep building against this task's changed signature.

- [ ] **Step 7: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, all tests pass.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: extract nomi-turn — registry-driven routing, zero hardcoded agent dispatch"
```

---

### Task 10: `nomi-server` — composition root, final cutover

This is where the old monolith stops existing. Everything remaining (`app.rs`, `routes/*`, `bootstrap/*`, `web_identity.rs`, `worker.rs`, `main.rs`, `bin/worker.rs`) moves into `crates/nomi-server/`, the `AgentRegistry` gets built for real (this is the one place a new agent gets wired in), and `backend/Cargo.toml`'s `[package]` section — along with all of `backend/src/` — is deleted.

**Files:**
- Modify: `backend/Cargo.toml` (remove `[package]`/`[dependencies]`/`[dev-dependencies]` entirely — becomes a pure workspace manifest)
- Delete: `backend/src/` (everything remaining: `app.rs`, `bootstrap/`, `web_identity.rs`, `worker.rs`, `main.rs`, `bin/worker.rs`, `lib.rs`, `routes/`)
- Create: `backend/crates/nomi-server/Cargo.toml`
- Move: `backend/src/app.rs` → `backend/crates/nomi-server/src/app.rs`
- Move: `backend/src/routes/*.rs` → `backend/crates/nomi-server/src/routes/*.rs`
- Move: `backend/src/bootstrap/*.rs` → `backend/crates/nomi-server/src/bootstrap/*.rs`
- Move: `backend/src/web_identity.rs` → `backend/crates/nomi-server/src/web_identity.rs`
- Move: `backend/src/worker.rs` → `backend/crates/nomi-server/src/worker.rs`
- Move: `backend/src/main.rs` → `backend/crates/nomi-server/src/main.rs`
- Move: `backend/src/bin/worker.rs` → `backend/crates/nomi-server/src/bin/worker.rs`
- Create: `backend/crates/nomi-server/src/lib.rs` (new — the crate root `main.rs`/`bin/worker.rs` both use)
- Move: every remaining `backend/tests/*.rs` (`session_ws.rs`, `sessions_routes.rs`, `settings_routes.rs`, `admin_llm_models_routes.rs`, `user_llm_selection_routes.rs`, `auth_routes.rs`, `web_identity.rs`, `llm_provider_resolution.rs`, `agent_sessions.rs`, `organizations.rs`, `identity.rs`, `extensions.rs`, `memory_and_events.rs`, `sessions.rs`) → `backend/crates/nomi-server/tests/`
- Delete: `backend/tests/` (now empty)

**Interfaces:**
- Consumes: every other crate in the workspace.
- Produces: binaries `nomi-orchestrator` and `worker`.

- [ ] **Step 1: Create `nomi-server`'s manifest**

Create `backend/crates/nomi-server/Cargo.toml`:

```toml
[package]
name = "nomi-server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "nomi-orchestrator"
path = "src/main.rs"

[dependencies]
nomi-llm = { path = "../nomi-llm" }
nomi-embedding = { path = "../nomi-embedding" }
nomi-realtime = { path = "../nomi-realtime" }
nomi-auth = { path = "../nomi-auth" }
nomi-settings = { path = "../nomi-settings" }
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-agent-money = { path = "../nomi-agent-money" }
nomi-agent-chitchat = { path = "../nomi-agent-chitchat" }
nomi-turn = { path = "../nomi-turn" }
axum = { version = "0.7", features = ["ws"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1"
serde = { version = "1", features = ["derive"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }
tower-http = { version = "0.7.0", features = ["trace"] }

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
wiremock = "0.6"
tokio-tungstenite = "0.24"
```

(`src/bin/worker.rs` is picked up by Cargo's automatic `src/bin/*.rs` discovery — no `[[bin]]` entry needed for it, exactly as today. The `nomi-orchestrator` binary needs an explicit `[[bin]]` because the package name (`nomi-server`) no longer matches the binary name it must produce.)

- [ ] **Step 2: Move every remaining source file**

```bash
mkdir -p backend/crates/nomi-server/src/routes backend/crates/nomi-server/src/bootstrap backend/crates/nomi-server/src/bin
git -C backend mv src/app.rs crates/nomi-server/src/app.rs
git -C backend mv src/routes/auth.rs crates/nomi-server/src/routes/auth.rs
git -C backend mv src/routes/llm_models.rs crates/nomi-server/src/routes/llm_models.rs
git -C backend mv src/routes/sessions.rs crates/nomi-server/src/routes/sessions.rs
git -C backend mv src/routes/settings.rs crates/nomi-server/src/routes/settings.rs
git -C backend mv src/routes/mod.rs crates/nomi-server/src/routes/mod.rs
git -C backend mv src/bootstrap/providers.rs crates/nomi-server/src/bootstrap/providers.rs
git -C backend mv src/bootstrap/mod.rs crates/nomi-server/src/bootstrap/mod.rs
git -C backend mv src/web_identity.rs crates/nomi-server/src/web_identity.rs
git -C backend mv src/worker.rs crates/nomi-server/src/worker.rs
git -C backend mv src/main.rs crates/nomi-server/src/main.rs
git -C backend mv src/bin/worker.rs crates/nomi-server/src/bin/worker.rs
```

- [ ] **Step 3: Create the crate root**

Read `backend/src/lib.rs`'s current contents first (by this point it should be almost entirely `pub use nomi_x as x;` aliases from Tasks 1-9, plus the still-real `pub mod app; pub mod bootstrap; pub mod routes; pub mod web_identity; pub mod worker;`). Create `backend/crates/nomi-server/src/lib.rs`:

```rust
pub mod app;
pub mod bootstrap;
pub mod routes;
pub mod web_identity;
pub mod worker;
```

Delete the old `backend/src/lib.rs` entirely (it's fully superseded — every alias it contained pointed at a crate that `nomi-server` now depends on directly by its real name).

- [ ] **Step 4: Fix every moved file's imports**

In `app.rs`, `routes/*.rs`, `bootstrap/*.rs`, `web_identity.rs`, `worker.rs`, `main.rs`, `bin/worker.rs`: every `use crate::llm::` → `use nomi_llm::`; every `use crate::embedding::` → `use nomi_embedding::`; every `use crate::realtime::` → `use nomi_realtime::`; every `use crate::auth::` → `use nomi_auth::`; every `use crate::settings::` → `use nomi_settings::`; every `use crate::turn::` → `use nomi_turn::`; every `use crate::web_identity::` (from `routes/sessions.rs`) → `use nomi_server::web_identity::` (this crate's own root, referenced the normal way now that it's a real crate rather than implicitly in scope — `bin/worker.rs` and `main.rs` need `use nomi_server::{app, bootstrap, worker};` style imports for the same reason, since binaries in `src/bin/` don't automatically see the package's own `lib.rs` items without an explicit `use nomi_server::...`).

In `app.rs` specifically: keep the `impl nomi_auth::extractor::HasJwtSecret for AppState` block added in Task 4 Step 4 — just update its `crate::auth::extractor::HasJwtSecret` reference to `nomi_auth::extractor::HasJwtSecret` (the alias it relied on no longer exists; the real crate name works the same way).

- [ ] **Step 5: Wire up the `AgentRegistry` — the composition root**

Read `worker.rs`'s current contents (as moved) — it should already contain the temporary registry-construction added in Task 9 Step 6. Replace that inline construction with a small reusable function so both `main.rs` (if it ever needs one — check whether any HTTP route path constructs a registry; if not, skip this for `main.rs`) and `worker.rs` build the identical set. Add to `backend/crates/nomi-server/src/lib.rs`:

```rust
pub mod app;
pub mod bootstrap;
pub mod routes;
pub mod web_identity;
pub mod worker;

/// The one place a new agent gets wired in. Adding an agent: implement `SubAgent` in its
/// own crate (see nomi-agent-money or nomi-agent-chitchat for the shape), add it as a
/// dependency of this crate's Cargo.toml, and add one line here.
pub fn build_agent_registry() -> nomi_agent_core::AgentRegistry {
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
    ])
}
```

In `worker.rs`, replace whatever inline `AgentRegistry::new(...)` Task 9 left behind with a single call to `nomi_server::build_agent_registry()` (constructed once, outside the poll loop, and passed by reference into every `process_turn` call — registries are cheap to hold for the worker's whole lifetime and there's no reason to rebuild one per job).

- [ ] **Step 6: Fix the migration path**

In `main.rs` (and `bin/worker.rs`, if it separately calls `sqlx::migrate!` — check; if only `main.rs` runs migrations at boot, `bin/worker.rs` needs no change here), find the `sqlx::migrate!(...)` call and change its path argument from `"./migrations"` (or an implicit default) to `"../../migrations"` — two directories up from `crates/nomi-server/` reaches `backend/migrations/`.

- [ ] **Step 7: Move the remaining tests**

```bash
mkdir -p backend/crates/nomi-server/tests
git -C backend mv tests/session_ws.rs crates/nomi-server/tests/session_ws.rs
git -C backend mv tests/sessions_routes.rs crates/nomi-server/tests/sessions_routes.rs
git -C backend mv tests/settings_routes.rs crates/nomi-server/tests/settings_routes.rs
git -C backend mv tests/admin_llm_models_routes.rs crates/nomi-server/tests/admin_llm_models_routes.rs
git -C backend mv tests/user_llm_selection_routes.rs crates/nomi-server/tests/user_llm_selection_routes.rs
git -C backend mv tests/auth_routes.rs crates/nomi-server/tests/auth_routes.rs
git -C backend mv tests/web_identity.rs crates/nomi-server/tests/web_identity.rs
git -C backend mv tests/llm_provider_resolution.rs crates/nomi-server/tests/llm_provider_resolution.rs
git -C backend mv tests/agent_sessions.rs crates/nomi-server/tests/agent_sessions.rs
git -C backend mv tests/organizations.rs crates/nomi-server/tests/organizations.rs
git -C backend mv tests/identity.rs crates/nomi-server/tests/identity.rs
git -C backend mv tests/extensions.rs crates/nomi-server/tests/extensions.rs
git -C backend mv tests/memory_and_events.rs crates/nomi-server/tests/memory_and_events.rs
git -C backend mv tests/sessions.rs crates/nomi-server/tests/sessions.rs
```

In every moved file that imports from `nomi_orchestrator`: change `use nomi_orchestrator::app::` → `use nomi_server::app::`, `use nomi_orchestrator::auth::` → `use nomi_auth::`, `use nomi_orchestrator::llm::` → `use nomi_llm::`, `use nomi_orchestrator::realtime::` → `use nomi_realtime::`, `use nomi_orchestrator::settings::` → `use nomi_settings::`, `use nomi_orchestrator::bootstrap::` → `use nomi_server::bootstrap::`, `use nomi_orchestrator::web_identity::` → `use nomi_server::web_identity::`. `agent_sessions.rs`, `organizations.rs`, `identity.rs`, `extensions.rs`, `memory_and_events.rs`, `sessions.rs` have no `nomi_orchestrator` imports at all (pure schema/migration tests against a raw `sqlx::PgPool`) — these move with zero content changes.

- [ ] **Step 8: Delete the old monolith**

```bash
rm -rf backend/src
rmdir backend/tests 2>/dev/null || true  # only removes it if now empty; confirm it is before running
```

Edit `backend/Cargo.toml`: delete everything except the `[workspace]` section added in Task 1 Step 1. Final contents:

```toml
[workspace]
resolver = "2"
members = ["crates/*"]
```

- [ ] **Step 9: Verify**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, every crate's tests pass.

Run: `cd backend && cargo run --bin nomi-orchestrator` (with the usual `DATABASE_URL`/`JWT_SECRET`/`SETTINGS_ENCRYPTION_KEY`/`LLM_PROVIDER=fake` env vars) — confirm it starts, logs `listening on 0.0.0.0:8080`, and the embedded worker logs `worker: listening for new turn jobs`, then stop it.

Run: `cd backend && cargo run --bin worker` (same env) — confirm it starts and logs the same worker-listening line, then stop it.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: extract nomi-server, wire up the AgentRegistry composition root, delete the old monolith"
```

---

### Task 11: End-to-end proof

**Files:**
- None (verification only).

- [ ] **Step 1: Full workspace build and test**

Run: `cd backend && cargo build && cargo test`
Expected: 0 build errors, 0 warnings introduced by this plan (pre-existing warnings, e.g. the `sqlx-postgres` future-incompatibility notice, are not this plan's concern), every crate's test suite passes.

- [ ] **Step 2: Confirm the binaries still work exactly as before**

Start the full stack the normal way (`scripts/dev.sh`, or manually: Postgres+EMQX via `docker compose up -d` in `backend/`, then `cargo run --bin nomi-orchestrator` from `backend/`). Confirm:
- The backend logs the same startup sequence as before this plan (`connected to database`, `migrations up to date`, `embedded worker enabled`, `listening on 0.0.0.0:8080`, `worker: listening for new turn jobs`).
- `curl http://localhost:8080/api/admin/settings/llm/models` (no auth header) returns `401`, same as before.

- [ ] **Step 3: Full frontend e2e suite**

Run: `cd frontend && npm run test:e2e` (from the repo root, with Postgres/EMQX up and `DATABASE_URL` set as usual).
Expected: all 17 tests pass, unchanged — this plan makes zero frontend-visible behavior changes other than the called-out money_agent streaming improvement, which nothing in the e2e suite asserts against either way.

- [ ] **Step 4: Confirm a new agent really is just a new crate**

As a design proof, not a permanent addition: temporarily create a minimal throwaway crate implementing `SubAgent` (a few lines, e.g. an agent with `intent_label() -> "weather"` and one trivial tool), add it as a `nomi-server` dependency, add one line to `build_agent_registry()`, and confirm `cargo build` succeeds with **zero edits to `nomi-turn`, `nomi-agent-core`, or any existing agent crate**. Then revert this step entirely (`git checkout` the throwaway changes) — it's a verification exercise for this plan, not a feature to ship.

- [ ] **Step 5: Report**

Summarize: crate count, lines moved per crate (rough), confirmation that Global Constraints all held (binary names unchanged, no HTTP behavior change besides the flagged one, workspace green throughout). No commit needed for this task (verification-only; Step 4's throwaway changes are reverted, not committed).
