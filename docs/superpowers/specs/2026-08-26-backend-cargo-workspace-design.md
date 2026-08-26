# Backend Cargo Workspace + Agent Registry

Date: 2026-08-26
Status: Approved (pending user review of this doc)

## Purpose

The backend (`backend/`) is a single 4,700-line Cargo crate. Two problems compound as the number of LLM agents grows: (1) everything compiles as one unit with no enforced boundaries between domains (auth, settings, LLM providers, agent orchestration), and (2) adding a new agent today requires editing the orchestrator's hardcoded dispatch (`turn/mod.rs`'s `run_locked_turn` match) and the intent classifier's closed `Intent` enum (`turn/routing.rs`) — every new agent touches shared control-flow code, which is exactly the shape of change that turns into spaghetti as the agent count grows past a handful.

This design splits `backend/` into a Cargo workspace with one crate per bounded domain, and — the change that actually solves the stated problem — replaces the hardcoded `Intent` enum and match-based dispatch with a `SubAgent` trait + runtime `AgentRegistry`, so a new agent is a new crate plus one registration line, with zero edits to orchestration code.

## 1. Current State (what exists today, for reference)

- `turn/subagent.rs` already defines a `SubAgent` trait (`agent_type`, `system_prompt`, `tools`, `execute_tool`), implemented once by `turn/money_agent.rs`.
- `turn/tools.rs`'s `run_tool_calling_loop` is already agent-agnostic — it takes `&dyn SubAgent` and runs the tool-calling loop against whatever's passed in. This part doesn't need to change.
- `turn/chitchat.rs` is NOT a `SubAgent` — it's hand-written: its own history fetch, its own memory retrieval (`turn/memory.rs`) folded into the system prompt, its own streaming via `provider.complete_stream()` + MQTT delta publishing, its own memory extraction after the reply. `run_subagent_turn` (which drives money_agent) never streams and never touches memory.
- `turn/routing.rs`'s `classify_intent` sends a fixed prompt ("Classify... as exactly one of: chitchat, money") and parses the result into a closed `enum Intent { Chitchat, Money }`.
- `turn/mod.rs`'s `run_locked_turn` matches on `Intent` and hardcodes construction of `money_agent::MoneyAgent` by name in two places (continuing an active session, and starting a fresh one after classification).

## 2. Crate Layout

`backend/`'s existing top-level files (`migrations/`, `docker-compose.yml`, `.env.example`, `tests/`, `.cargo/config.toml`) stay exactly where they are. `backend/Cargo.toml` becomes a workspace manifest; a new `backend/crates/` holds the members:

| Crate | Owns | Depends on (workspace-internal) |
|---|---|---|
| `nomi-llm` | `LlmProvider` trait, anthropic/openai/gemini/fake impls, `ToolDefinition`/`LlmMessage`/`LlmRequest`/`LlmResponse`/`ContentBlock`/`StreamEvent` types, `build_provider`, `complete`/`complete_stream`/`collect_stream` | — |
| `nomi-embedding` | `EmbeddingProvider` trait + openai/fake impls, `build_embedding_provider` | — |
| `nomi-realtime` | `MqttPublisher`, `MqttError`, `StreamEnvelope` | `nomi-llm` (for `StreamEvent` inside `StreamEnvelope::Delta`) |
| `nomi-auth` | JWT `Claims` + encode/decode, `AuthClaims` extractor (generic over state, see §4), password hashing, login/registration, permissions/authorize | — |
| `nomi-settings` | `admin_llm_models`/`user_llm_selections`/`provider_settings` CRUD, `crypto::{encrypt,decrypt,mask_api_key}` | — |
| **`nomi-agent-core`** | `SubAgent` trait (extended, §3), `AgentRegistry`, the generic turn-running engine (tool-calling loop + always-on streaming + opt-in memory), `TurnError`/`LoopOutcome`, memory retrieval/extraction (moved from `turn/memory.rs`) | `nomi-llm`, `nomi-embedding`, `nomi-realtime` |
| **`nomi-agent-money`** | `MoneyAgent` (today's `turn/money_agent.rs`, unchanged behavior) | `nomi-agent-core`, `nomi-llm` |
| **`nomi-agent-chitchat`** | `ChitchatAgent` (chitchat reframed as a `SubAgent`, §3) | `nomi-agent-core`, `nomi-llm` |
| `nomi-turn` | Routing/classification (now registry-driven), session lock/queue/ingest/bootstrap, `handle_inbound_message`/`process_turn`, `TurnOutcome` | `nomi-agent-core`, `nomi-llm`, `nomi-embedding`, `nomi-realtime` — **never** a specific `nomi-agent-*` crate |
| `nomi-server` (bin) | `app.rs`/`routes/*` (axum), `bootstrap/providers.rs` (provider resolution), `web_identity.rs`, `worker.rs`, `main.rs`, `bin/worker.rs` | every crate above, plus `nomi-agent-money` and `nomi-agent-chitchat` — this is the **composition root** |

Binary names are unchanged: `nomi-server`'s `src/main.rs` declares `[[bin]] name = "nomi-orchestrator"` explicitly (package name no longer auto-matches), and `src/bin/worker.rs` keeps Cargo's automatic binary discovery, so `cargo run --bin nomi-orchestrator` / `cargo run --bin worker` keep working unchanged from `backend/` (Cargo runs any workspace member's binary by name from the workspace root) — nothing in `playwright.config.ts`, `scripts/dev.sh`, or CI needs to change.

Total: 9 library crates + 1 binary crate. Given the "many (7+) agents" expectation, a dedicated crate per agent is the right trade-off — each agent's blast radius is its own `Cargo.toml`, its own compile unit, and reviewers can see exactly what it depends on.

## 3. The `SubAgent` Trait and Registry

```rust
// nomi-agent-core
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

    /// Fed into the classifier's prompt verbatim. Not called for the default agent.
    fn intent_label(&self) -> &'static str;
    fn intent_description(&self) -> &'static str;

    /// Exactly one registered agent returns true. That agent is used when classification
    /// fails to parse, the LLM call errors, or no other agent's intent_label matches.
    fn is_default(&self) -> bool { false }

    /// When true, the orchestrator retrieves relevant memories before the first LLM call
    /// (folded into the system prompt) and extracts+stores new memories after the reply.
    fn uses_memory(&self) -> bool { false }
}

pub struct AgentRegistry {
    agents: Vec<Box<dyn SubAgent>>,
}

impl AgentRegistry {
    pub fn new(agents: Vec<Box<dyn SubAgent>>) -> Self {
        // Panics at startup if zero or more-than-one agent claims is_default() — a
        // configuration error, not a runtime condition to handle gracefully.
        ...
    }
    pub fn default_agent(&self) -> &dyn SubAgent { ... }
    pub fn find(&self, intent_label: &str) -> Option<&dyn SubAgent> { ... }
    pub fn classification_prompt(&self) -> String {
        // "Classify the user's message as exactly one of: {labels joined}. Reply with only
        // that single word, nothing else." built from every non-default agent's intent_label
        // (the default agent is the fallback, not a classification target — same behavior
        // as today, where "chitchat" only wins via explicit classification OR fallback).
    }
}
```

`nomi-turn::routing::classify_intent` changes from returning a closed `Intent` enum to returning `&'static str` (the winning agent's `intent_label()`, or the registry's default), built and parsed generically against whatever's registered — no `match` arm to add per agent.

`nomi-turn::run_locked_turn` changes from matching on `Intent` and constructing `money_agent::MoneyAgent` by name, to: look up the active agent session's `agent_type` in the registry (`registry.find(&details.agent_type)`) to continue it, or `registry.find(&classified_label).unwrap_or_else(|| registry.default_agent())` to start fresh. **This function does not change when an agent is added or removed** — it only changes if the orchestration *shape* itself changes (e.g. a new kind of session lifecycle).

**Unifying chitchat**: `nomi-agent-core`'s engine (the renamed/generalized `run_tool_calling_loop`) always uses `complete_stream()` and publishes `Delta` events when an `MqttPublisher` handle is available — not conditionally for chitchat only. Before the first LLM call, if `agent.uses_memory()`, the engine calls the (moved-from-`turn/memory.rs`) retrieval function and folds the result into the system prompt exactly as chitchat does today; after a `Reply` outcome, if `uses_memory()`, it calls extraction/storage. With both capabilities generalized, `ChitchatAgent` becomes:

```rust
// nomi-agent-chitchat
pub struct ChitchatAgent;

#[async_trait]
impl SubAgent for ChitchatAgent {
    fn agent_type(&self) -> &'static str { "chitchat" }
    fn system_prompt(&self) -> &'static str { CHITCHAT_SYSTEM_PROMPT }
    fn tools(&self) -> Vec<ToolDefinition> { vec![] }
    async fn execute_tool(&self, ..) -> Result<String, String> {
        Err("chitchat has no tools".to_string()) // unreachable: tools() is empty
    }
    fn intent_label(&self) -> &'static str { "chitchat" }
    fn intent_description(&self) -> &'static str { "General conversation, questions, or anything not covered by another agent" }
    fn is_default(&self) -> bool { true }
    fn uses_memory(&self) -> bool { true }
}
```

MoneyAgent is unchanged except it gains `intent_label() -> "money"`, `intent_description() -> "Questions about the user's transactions, spending, or budget"`, and inherits the trait's default `is_default() -> false` / `uses_memory() -> false`.

**Behavior changes from this unification** (called out explicitly, not hidden): money_agent turns now stream live deltas over MQTT (previously they didn't — a strict improvement, every future agent gets it for free) and the tool-calling loop's LLM calls move from `complete()` to `complete_stream()` end-to-end.

## 4. Resolving the one real circular-dependency risk

`auth::extractor::AuthClaims` (an axum `FromRequestParts` extractor) currently takes `&AppState` directly — but `AppState` is defined in `app.rs`, which is server/composition-root territory, and `nomi-auth` must not depend on `nomi-server` (that would be backwards — `nomi-server` depends on `nomi-auth`, not the other way around).

Resolved by making the extractor generic over any state exposing what it needs:

```rust
// nomi-auth
pub trait HasJwtSecret {
    fn jwt_secret(&self) -> &str;
}

pub struct AuthClaims(pub Claims);

#[axum::async_trait]
impl<S: HasJwtSecret + Send + Sync> FromRequestParts<S> for AuthClaims {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> { ... }
}
```

`nomi-server::AppState` implements `HasJwtSecret` (one line: `fn jwt_secret(&self) -> &str { &self.jwt_secret }`). `nomi-auth` has zero knowledge of `AppState`'s existence.

## 5. Migration Mechanics

- **`sqlx::migrate!()`**: currently resolves `./migrations` relative to `CARGO_MANIFEST_DIR`. Once `main.rs` moves to `crates/nomi-server/`, this becomes `sqlx::migrate!("../../migrations")` (two levels up: `crates/nomi-server/` → `crates/` → `backend/`).
- **`backend/tests/*.rs`**: every existing integration test file currently does `use nomi_orchestrator::...`. Each moves to import from whichever crate now owns what it tests — e.g. `llm_models_db.rs`/`admin_llm_models_routes.rs`/`user_llm_selection_routes.rs` → `nomi_settings`/`nomi_server`, `llm_provider_resolution.rs` → `nomi_server` (provider resolution stays in `bootstrap/providers.rs`, composition-root territory), `sessions_routes.rs`/`web_identity.rs` test → `nomi_server`, `settings_routes.rs` → `nomi_settings`. Route-level integration tests (anything hitting `AppState`/`build_router`) necessarily stay in `nomi-server`'s own test suite since that's the only crate with a router; pure-logic tests (DB CRUD, crypto, agent behavior) move to their owning crate.
- **Workspace-level `Cargo.lock`**: one lockfile at `backend/Cargo.lock` for the whole workspace, as today.
- **`.cargo/config.toml`**'s `DATABASE_URL` env default: unaffected — it's workspace-wide already (`[env]` in `.cargo/config.toml` applies to every crate built under `backend/`).

## 6. Testing

Every new library crate gets its own `#[cfg(test)]` unit tests where logic warrants it (e.g. `nomi-agent-core`'s registry construction/lookup, `AgentRegistry::new`'s panic-on-misconfiguration). `sqlx::test`-based integration tests redistribute per §5. The full workspace `cargo test` (run from `backend/`, exercises every member) and the frontend's Playwright e2e suite (which only observes HTTP behavior, not crate boundaries) must both stay green throughout — this is a pure internal restructuring with two called-out, intentional behavior changes (money_agent gains streaming; chitchat's memory/streaming code moves but behavior is unchanged), not a feature change.

## Out of Scope

- No new agents are added by this work — `nomi-agent-money` and `nomi-agent-chitchat` are the only two, ported with the trait extension described above.
- No change to any HTTP endpoint, request/response shape, or frontend-visible behavior, except the incidental streaming improvement for money_agent-style turns (deltas were previously silent for tool-calling agents; they now stream like chitchat always has).
- No change to the `admin_llm_models`/`user_llm_selections` schema or the per-user provider-resolution logic (`bootstrap/providers.rs`) — that logic stays exactly where it is, in `nomi-server`.
- No dynamic plugin loading (`.so`/`dlopen`) — "adding an agent" means adding a Rust crate to the workspace and one line in `nomi-server`'s composition root, then rebuilding. True runtime plugin loading is a different, much larger design and isn't what "many agents" requires here.
