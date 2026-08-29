# Multi-Provider Embeddings — Design

## Goal

The app currently supports exactly one embedding provider (OpenAI) for the personality/chitchat memory feature's semantic search. Add Gemini and Cohere as additional provider options, all producing 1536-dimension vectors so the existing `memory_items.embedding VECTOR(1536)` column and its ivfflat index need no schema migration. Tag stored memories with the provider/model that embedded them so a provider switch can't silently compare incompatible vector spaces. Ship a small admin UI page so this is configurable without touching env vars or the database directly.

Voyage AI was considered and rejected: its current models don't support truncating output to exactly 1536 dimensions (only 256/512/1024/2048, or a fixed native size), so including it would force either dropping the dimension target to 1024 (a real schema migration, discarding all existing `memory_items`) or excluding it. Given only one existing provider needs to be preserved without migration, Voyage is out of scope for this pass.

Anthropic is not an option — it has no public embeddings API.

## Current State (verified against the code, not assumed)

- `EmbeddingProvider` trait: `nomi-embedding/src/lib.rs:12-14` — `async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>`.
- `EmbeddingProviderKind`: `nomi-embedding/src/config.rs:6-9` — `{OpenAi, Fake}`. `EmbeddingConfig`: `config.rs:11-17` — flat `{provider, model_id, api_key, base_url: Option<String>}`, one shape for every provider (no per-provider struct needed).
- `OpenAiEmbeddingProvider`: `nomi-embedding/src/openai.rs` — `POST {base_url}/v1/embeddings`, `Authorization: Bearer`, body `{model, input}`, response `data[0].embedding`.
- `FakeEmbeddingProvider`: `nomi-embedding/src/fake.rs` — returns `vec![0.0; 1536]`, confirming 1536 is already the implicit contract everywhere in tests.
- Backend routes for embedding settings **already exist and are wired up**: `nomi-server/src/routes/settings.rs` (`get_embedding_settings`/`put_embedding_settings`), registered at `GET`/`PUT /api/admin/settings/embedding` in `app.rs:71-72`. They call the generic `nomi-settings` crate (`get_settings`/`upsert_settings`/`crypto::encrypt`/`crypto::decrypt`/`mask_api_key`) against the single-row `provider_settings` table (`setting_type = 'embedding'`, DB-enforced unique via `migrations/0010_provider_settings.sql:3`).
- The only gap on the routes side: `settings.rs:104`'s allowlist `!["openai", "fake"].contains(&req.provider.as_str())` rejects any provider name that isn't `openai`/`fake`.
- `bootstrap/providers.rs:20-24` (`embedding_provider_kind_from_str`) panics on any unrecognized provider string. `bootstrap/providers.rs:127-161` (`build_embedding_provider_from_settings_or_env`) also panics via `.expect(...)` on a decrypt failure — no fallback tier, unlike the LLM chat sibling (`resolve_llm_model_config`, `bootstrap/providers.rs:63-100`), which already degrades gracefully (`tracing::error!` + fall through) on the same failure mode.
- `nomi-agent-core/src/memory.rs` has three call sites that need the provider/model identity threaded through: `try_retrieve_memories` (reads), `extract_and_store_memory` (writes), and the raw SQL in `retrieve_relevant_memories`/the `INSERT` in `extract_and_store_memory`.
- `memory_items` schema (`migrations/0006_memory_and_events.sql`): `VECTOR(1536) NOT NULL`, `ivfflat` index on `vector_cosine_ops`. No provider/model tagging column exists.
- No frontend page exists for embedding settings (`frontend/src/routes/admin/(protected)/settings/` has only `llm/`). The LLM settings page (`llm/+page.svelte` + `+page.server.ts`) is a *list* (multiple `admin_llm_models` rows); embedding settings is a single row, so the new page is a simpler single-form mirror of that pattern, not a literal copy.
- Existing Gemini auth convention in this codebase (`nomi-llm/src/gemini.rs:82-84`): `?key={api_key}` query-string auth, not the `x-goog-api-key` header — the new `GeminiEmbeddingProvider` follows the same convention for consistency within the codebase, even though Google's API accepts either.

## Provider API Contracts (verified against current docs 2026-08-29, not memory)

**Gemini** — `POST {base_url}/v1beta/models/{model}:embedContent?key={api_key}` (mirrors the existing chat provider's auth style). Body: `{"content": {"parts": [{"text": "..."}]}, "output_dimensionality": 1536}`. Response: embedding vector lives in `embedding.values` (singular `embedContent` → singular `embedding`, not the batch `embedContents` shape). Current models: `gemini-embedding-001`/`gemini-embedding-2`, native 3072-dim, valid output range 128–3072, 1536 explicitly documented as a supported/recommended value. Default `base_url`: `https://generativelanguage.googleapis.com`.

**Cohere** — `POST {base_url}/v2/embed`, `Authorization: Bearer {api_key}`. Body: `{"model": "embed-v4.0", "texts": [text], "input_type": "search_document" | "search_query", "output_dimension": 1536, "embedding_types": ["float"]}`. Response: `embeddings.float[0]` (array of arrays, one per input text). `input_type` must differ between storing (`search_document`) and querying (`search_query`) — Cohere's models are asymmetric, this is a real API requirement, not a style choice, and is a small retrieval-quality win over OpenAI's symmetric embeddings. `output_dimension` valid values for `embed-v4.0`: 256, 512, 1024, 1536 (1536 is the max/default). Default `base_url`: `https://api.cohere.com`.

## Design

### 1. `nomi-embedding` crate

- `EmbeddingProviderKind` grows to `{OpenAi, Gemini, Cohere, Fake}`.
- New `nomi-embedding/src/gemini.rs`: `GeminiEmbeddingProvider { client, api_key, model, base_url }`, `default_base_url() -> "https://generativelanguage.googleapis.com"`, `embed()` per the contract above. Same error-handling shape as `openai.rs` (non-2xx → `EmbeddingError::ProviderError`, parse failure → `EmbeddingError::ParseError`).
- New `nomi-embedding/src/cohere.rs`: `CohereEmbeddingProvider { client, api_key, model, base_url }`, `default_base_url() -> "https://api.cohere.com"`. Because Cohere requires different `input_type` for storage vs. query, `EmbeddingProvider::embed(&self, text: &str)`'s single-purpose signature isn't quite enough — see "Trait change" below.
- **Trait change**: `EmbeddingProvider` gains two more required methods:
  ```rust
  fn provider_name(&self) -> &'static str;
  fn model_id(&self) -> &str;
  ```
  This makes every provider self-describing so `memory.rs` can tag rows without threading a separate `(provider, model)` pair through every call site — each provider already stores its own `model` field, this just exposes it. `provider_name()` returns a fixed string per struct (`"openai"`, `"gemini"`, `"cohere"`, `"fake"`); `model_id()` returns `&self.model`.
  For the asymmetric-embedding requirement, `embed` stays single-purpose (used for both storing and querying today) but `CohereEmbeddingProvider` needs to know which mode it's in. Simplest fix consistent with the trait's existing minimalism: add one more trait method with a default implementation so `OpenAi`/`Gemini`/`Fake` don't need to change:
  ```rust
  async fn embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
      self.embed(text).await // default: same as storage-time embedding
  }
  ```
  `CohereEmbeddingProvider` overrides `embed_for_query` to send `input_type: "search_query"` while its `embed` sends `input_type: "search_document"`. `memory.rs`'s `try_retrieve_memories` (the read path) calls `embed_for_query` instead of `embed`; `extract_and_store_memory` (the write path) keeps calling `embed`.
- `build_embedding_provider` (`config.rs:19-27`) gains two match arms constructing the new providers, same shape as the existing `OpenAi` arm.

### 2. `bootstrap/providers.rs` — remove the two panics this change would otherwise make worse

- `embedding_provider_kind_from_str` (`providers.rs:20-24`) changes from `-> EmbeddingProviderKind` (panicking) to `-> Result<EmbeddingProviderKind, String>`, adding `"gemini" => Ok(EmbeddingProviderKind::Gemini)` and `"cohere" => Ok(EmbeddingProviderKind::Cohere)` arms.
- `build_embedding_provider_from_settings_or_env` (`providers.rs:127-161`): the `Some(row)` branch's `.expect("failed to decrypt stored embedding api key")` and the new `Result`-returning `embedding_provider_kind_from_str` call both get the same `tracing::error!` + fall-through-to-env-var treatment already established for the LLM chat sibling (`model_config_from_admin_model`, `providers.rs:38-56`) — not a new pattern, applying an existing one. This is directly necessitated by this change: every new provider variant is one more way to mistype the `provider` column, and that must not panic a request path.

### 3. `nomi-server/src/routes/settings.rs`

- Line 104's allowlist becomes `!["openai", "gemini", "cohere", "fake"].contains(&req.provider.as_str())`, error message updated to match.
- No other change needed — `resolve_api_key`, `load_settings_response`, encryption, and masking are already provider-agnostic.

### 4. Migration `0015_memory_embedding_provenance.sql`

```sql
ALTER TABLE memory_items
    ADD COLUMN embedding_provider TEXT NOT NULL DEFAULT 'openai',
    ADD COLUMN embedding_model TEXT NOT NULL DEFAULT '';
```
Existing rows backfill to `'openai'`/empty model (the app has only ever run against OpenAI so far; an empty `embedding_model` default is acceptable since the filter compares against whatever the *currently configured* model actually is — an empty stored value simply won't match a real configured model_id, which is the correct "don't retrieve this" behavior for genuinely unknown provenance). No `DEFAULT` needed going forward — the application always supplies both on insert.

### 5. `nomi-agent-core/src/memory.rs`

- `retrieve_relevant_memories` gains `current_provider: &str, current_model: &str` parameters and an added `AND embedding_provider = $4 AND embedding_model = $5` clause.
- `try_retrieve_memories` calls `embedding_provider.embed_for_query(text)` (not `embed`) and passes `embedding_provider.provider_name()`/`embedding_provider.model_id()` through to `retrieve_relevant_memories`.
- `extract_and_store_memory`'s `INSERT` gains `embedding_provider`/`embedding_model` columns, populated from `embedding_provider.provider_name()`/`.model_id()`.

### 6. Frontend — new admin page `admin/(protected)/settings/embedding/`

Single-form page mirroring the LLM settings page's server-load/action pattern (`+page.server.ts`'s `apiFetch`/`fail()` conventions), simplified for one resource instead of a list:

- `load`: `GET /api/admin/settings/embedding` → the current `{provider, model_id, base_url, api_key_masked}`, or a "not configured yet" empty state on 404.
- One `update` action: `PUT /api/admin/settings/embedding` with `{provider, model_id, api_key, base_url}` (blank `api_key` in the form means "keep the existing key," matching `resolve_api_key`'s existing semantics in `settings.rs:67-87`).
- `+page.svelte`: uses the `Select` component (already built) for the provider dropdown (`openai`/`gemini`/`cohere`/`fake`), `TextField` for model ID/API key/base URL — same components already used on the LLM settings page.
- Added to the admin sidebar nav alongside the existing "LLM Settings" link.

## Out of Scope

- Voyage AI (dimension mismatch, see Goal section).
- Backfilling/re-embedding existing `memory_items` rows when the provider changes — they simply stop being retrieved (per the "tag + filter" decision) until naturally superseded by new memories. A backfill job is a separate, later task if it turns out to matter in practice.
- Any change to `agent_events`/`agent_sessions`/anything related to the multi-agent supervisor system (separate, later sub-project).
- Streaming/batch embedding endpoints — all three providers support batch requests (multiple texts in one call), but every current call site in `memory.rs` embeds exactly one string at a time. Not adding batch support since nothing consumes it.
