# Multi-Provider Embeddings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Gemini and Cohere as embedding providers alongside the existing OpenAI one, tag stored memories with their provenance so a provider switch can't compare incompatible vector spaces, and ship a small admin page to configure the active provider.

**Architecture:** `nomi-embedding`'s `EmbeddingProvider` trait grows two identity methods (`provider_name`/`model_id`) plus one behavioral method with a default (`embed_for_query`, for providers whose storage/query embeddings differ). Two new provider structs are added following the existing `OpenAiEmbeddingProvider` shape exactly. The backend routes for configuring this already exist and are already wired up — only their provider allowlist needs the two new names. A migration adds provenance columns to `memory_items`; the memory retrieval query filters on them. A new frontend page fills the one real gap (no UI existed at all for embedding config).

**Tech Stack:** Rust (`nomi-embedding`, `nomi-server`, `nomi-agent-core`, `nomi-test-support` crates), sqlx/Postgres/pgvector, SvelteKit/Svelte 5 (frontend admin page), wiremock (provider HTTP tests).

**Spec:** docs/superpowers/specs/2026-08-29-multi-provider-embeddings-design.md

## Global Constraints

- Target embedding dimension is fixed at 1536 for every provider (Gemini via `output_dimensionality`, Cohere via `output_dimension`) — no schema/column-type change to `memory_items.embedding`.
- `EmbeddingProvider::embed` stays the storage-time method for every provider; only Cohere overrides `embed_for_query` (its models are asymmetric — a different `input_type` for storing vs. querying is a real API requirement, not a style choice).
- Every new provider struct mirrors `OpenAiEmbeddingProvider`'s existing shape exactly: `{client, api_key, model, base_url}` fields, a `new()` constructor, a `default_base_url()` associated function, and the same `EmbeddingError::ProviderError`/`EmbeddingError::ParseError` error mapping on non-2xx / malformed responses.
- Gemini auth follows this codebase's existing convention (`nomi-llm/src/gemini.rs`) of `?key={api_key}` as a query parameter, not the `x-goog-api-key` header, even though Google's API accepts either.
- No schema migration touches `memory_items.embedding`'s type or its ivfflat index — only two new columns are added.
- Existing `memory_items` rows backfill to `embedding_provider = 'openai', embedding_model = ''` — the empty model value is intentional (never matches a real configured `model_id`, which correctly makes pre-migration rows unretrievable rather than wrongly compared).
- Panics already present in `bootstrap/providers.rs` for the DB-row path (as opposed to the env-var fallback path, which may still panic on boot-time misconfiguration, matching existing precedent for the LLM sibling function) must become graceful `tracing::error!` + fallback, not panics — every new provider variant is one more way to mistype the `provider` column at runtime.
- Frontend: reuse `Select`/`TextField`/`Card`/`Button` from `$lib/components/m3/` exactly as the existing LLM settings page uses them — no new components.

---

### Task 1: Extend `EmbeddingProvider` with identity + `embed_for_query`

**Files:**
- Modify: `backend/crates/nomi-embedding/src/lib.rs`
- Modify: `backend/crates/nomi-embedding/src/openai.rs`
- Modify: `backend/crates/nomi-embedding/src/fake.rs`
- Modify: `backend/crates/nomi-test-support/src/lib.rs`
- Modify: `backend/crates/nomi-embedding/tests/embedding_openai.rs`

**Interfaces:**
- Produces (used by every later task): `EmbeddingProvider::provider_name(&self) -> &'static str`, `EmbeddingProvider::model_id(&self) -> &str`, `EmbeddingProvider::embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>` (default impl calls `self.embed(text)`).

- [ ] **Step 1: Add the three trait members**

Replace `backend/crates/nomi-embedding/src/lib.rs` in full:

```rust
pub mod config;
pub mod openai;
pub mod types;
pub mod fake;

pub use config::{build_embedding_provider, EmbeddingConfig, EmbeddingProviderKind};
pub use types::EmbeddingError;

use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed text for storage — this is the "document" side of an asymmetric model.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Embed text for a similarity query. Providers whose models are asymmetric (a
    /// different request shape for text you're storing vs. text you're searching with)
    /// override this; everyone else inherits the default, which just calls `embed`.
    async fn embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed(text).await
    }

    /// A short, fixed identifier for which provider this is (e.g. "openai", "gemini") —
    /// used to tag stored memories so a provider switch doesn't compare embeddings from
    /// incompatible vector spaces.
    fn provider_name(&self) -> &'static str;

    /// The specific model this instance is configured with.
    fn model_id(&self) -> &str;
}
```

Note: `pub mod gemini;`/`pub mod cohere;` are NOT added here — Task 2 adds the former when it creates `gemini.rs`, Task 3 adds the latter when it creates `cohere.rs`. This keeps this task's own build green on its own, rather than depending on Tasks 2–3 landing first.

- [ ] **Step 2: Implement the two new methods for `OpenAiEmbeddingProvider`**

In `backend/crates/nomi-embedding/src/openai.rs`, inside the existing `impl EmbeddingProvider for OpenAiEmbeddingProvider` block, add after the closing `}` of the existing `embed` method (still inside the `impl` block):

```rust
    fn provider_name(&self) -> &'static str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
```

- [ ] **Step 3: Implement the two new methods for `FakeEmbeddingProvider`**

Replace `backend/crates/nomi-embedding/src/fake.rs` in full:

```rust
use async_trait::async_trait;

use super::{EmbeddingError, EmbeddingProvider};

pub struct FakeEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(vec![0.0; 1536])
    }

    fn provider_name(&self) -> &'static str {
        "fake"
    }

    fn model_id(&self) -> &str {
        ""
    }
}
```

- [ ] **Step 4: Implement the two new methods for the test-double `FakeEmbeddingProvider`**

In `backend/crates/nomi-test-support/src/lib.rs`, inside the existing `impl EmbeddingProvider for FakeEmbeddingProvider` block, add after the closing `}` of the existing `embed` method (still inside the `impl` block):

```rust
    fn provider_name(&self) -> &'static str {
        "fake"
    }

    fn model_id(&self) -> &str {
        "fake-model"
    }
```

This test double now reports identity `("fake", "fake-model")` — Task 5's writing test will assert against these exact values.

- [ ] **Step 5: Add tests proving the new trait members work on `OpenAiEmbeddingProvider`**

Append to `backend/crates/nomi-embedding/tests/embedding_openai.rs` (the file already imports `EmbeddingProvider`, `EmbeddingError`, `wiremock`'s `Mock`/`MockServer`/`ResponseTemplate`, and `method`/`path`/`body_partial_json` — no new imports needed):

```rust
#[tokio::test]
async fn provider_name_and_model_id_are_exposed() {
    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        "http://example.invalid".to_string(),
    );
    assert_eq!(provider.provider_name(), "openai");
    assert_eq!(provider.model_id(), "text-embedding-3-small");
}

#[tokio::test]
async fn embed_for_query_defaults_to_calling_embed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"embedding": [0.5, 0.6], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 1, "total_tokens": 1}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let embedding = provider.embed_for_query("hello").await.unwrap();
    assert_eq!(embedding, vec![0.5f32, 0.6]);
}
```

- [ ] **Step 6: Verify**

```bash
cd backend && cargo test -p nomi-embedding -p nomi-test-support 2>&1 | tail -60
```

Expected: clean build, all tests pass (including the two new tests from Step 5).

- [ ] **Step 7: Commit**

```bash
git add backend/crates/nomi-embedding/src/lib.rs backend/crates/nomi-embedding/src/openai.rs \
        backend/crates/nomi-embedding/src/fake.rs backend/crates/nomi-test-support/src/lib.rs \
        backend/crates/nomi-embedding/tests/embedding_openai.rs
git commit -m "feat: add provider identity and embed_for_query to EmbeddingProvider"
```

---

### Task 2: Gemini embedding provider

**Files:**
- Create: `backend/crates/nomi-embedding/src/gemini.rs`
- Modify: `backend/crates/nomi-embedding/src/config.rs`
- Create: `backend/crates/nomi-embedding/tests/embedding_gemini.rs`

**Interfaces:**
- Consumes: `EmbeddingProvider` trait, `EmbeddingError` (Task 1).
- Produces (used by Task 4): `GeminiEmbeddingProvider::new(client, api_key, model, base_url) -> Self`, `GeminiEmbeddingProvider::default_base_url() -> String`, `EmbeddingProviderKind::Gemini` variant.

- [ ] **Step 1: Declare the module**

In `backend/crates/nomi-embedding/src/lib.rs`, add `pub mod gemini;` alongside the existing `pub mod openai;` line (alphabetically after it):

```rust
pub mod config;
pub mod openai;
pub mod gemini;
pub mod types;
pub mod fake;
```

- [ ] **Step 2: Create the provider**

Create `backend/crates/nomi-embedding/src/gemini.rs`:

```rust
use async_trait::async_trait;
use serde_json::json;

use super::types::EmbeddingError;
use super::EmbeddingProvider;

const OUTPUT_DIMENSIONALITY: u32 = 1536;

pub struct GeminiEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiEmbeddingProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://generativelanguage.googleapis.com".to_string()
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = json!({
            "content": { "parts": [{ "text": text }] },
            "output_dimensionality": OUTPUT_DIMENSIONALITY,
        });

        let url = format!("{}/v1beta/models/{}:embedContent?key={}", self.base_url, self.model, self.api_key);

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body
            .get("embedding")
            .and_then(|e| e.get("values"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| EmbeddingError::ParseError("missing embedding.values".to_string()))?;

        embedding
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| EmbeddingError::ParseError("embedding element is not a number".to_string()))
            })
            .collect()
    }

    fn provider_name(&self) -> &'static str {
        "gemini"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}
```

- [ ] **Step 3: Wire the new variant into `config.rs`**

Replace `backend/crates/nomi-embedding/src/config.rs` in full:

```rust
use super::fake::FakeEmbeddingProvider;
use super::gemini::GeminiEmbeddingProvider;
use super::openai::OpenAiEmbeddingProvider;
use super::EmbeddingProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingProviderKind {
    OpenAi,
    Gemini,
    Fake,
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_embedding_provider(config: EmbeddingConfig, http_client: reqwest::Client) -> Box<dyn EmbeddingProvider> {
    match config.provider {
        EmbeddingProviderKind::OpenAi => {
            let base_url = config.base_url.unwrap_or_else(OpenAiEmbeddingProvider::default_base_url);
            Box::new(OpenAiEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        EmbeddingProviderKind::Gemini => {
            let base_url = config.base_url.unwrap_or_else(GeminiEmbeddingProvider::default_base_url);
            Box::new(GeminiEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        EmbeddingProviderKind::Fake => Box::new(FakeEmbeddingProvider),
    }
}
```

(Task 3 adds the `Cohere` variant and arm on top of this — expect a small conflict-free addition there, not a rewrite of what you just wrote.)

- [ ] **Step 4: Write the provider's tests**

Create `backend/crates/nomi-embedding/tests/embedding_gemini.rs`:

```rust
use nomi_embedding::gemini::GeminiEmbeddingProvider;
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn embed_returns_the_vector_from_the_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .and(body_partial_json(json!({
            "content": { "parts": [{ "text": "hello world" }] },
            "output_dimensionality": 1536
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embedding": { "values": [0.1, 0.2, 0.3] }
        })))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    let embedding = provider.embed("hello world").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn api_key_is_sent_as_a_query_parameter() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .and(query_param("key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embedding": { "values": [0.1] }
        })))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    provider.embed("hello").await.unwrap();
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ProviderError(_))));
}

#[tokio::test]
async fn malformed_response_becomes_a_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"embedding": {}})))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ParseError(_))));
}

#[tokio::test]
async fn provider_name_and_model_id_are_exposed() {
    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        "http://example.invalid".to_string(),
    );
    assert_eq!(provider.provider_name(), "gemini");
    assert_eq!(provider.model_id(), "gemini-embedding-001");
}
```

- [ ] **Step 5: Verify**

```bash
cd backend && cargo test -p nomi-embedding 2>&1 | tail -60
```

Expected: clean build, all tests pass (the whole crate, not just the new `gemini` tests — `config.rs` changed too).

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-embedding/src/lib.rs backend/crates/nomi-embedding/src/gemini.rs \
        backend/crates/nomi-embedding/src/config.rs backend/crates/nomi-embedding/tests/embedding_gemini.rs
git commit -m "feat: add Gemini embedding provider"
```

---

### Task 3: Cohere embedding provider

**Files:**
- Create: `backend/crates/nomi-embedding/src/cohere.rs`
- Modify: `backend/crates/nomi-embedding/src/config.rs`
- Create: `backend/crates/nomi-embedding/tests/embedding_cohere.rs`

**Interfaces:**
- Consumes: `EmbeddingProvider` trait, `EmbeddingError` (Task 1); the `config.rs` shape Task 2 left behind.
- Produces (used by Task 4): `CohereEmbeddingProvider::new(client, api_key, model, base_url) -> Self`, `CohereEmbeddingProvider::default_base_url() -> String`, `EmbeddingProviderKind::Cohere` variant.

- [ ] **Step 1: Declare the module**

In `backend/crates/nomi-embedding/src/lib.rs` (as left by Task 2), add `pub mod cohere;` alongside the existing `pub mod gemini;` line (alphabetically before it):

```rust
pub mod config;
pub mod openai;
pub mod cohere;
pub mod gemini;
pub mod types;
pub mod fake;
```

- [ ] **Step 2: Create the provider**

Create `backend/crates/nomi-embedding/src/cohere.rs`:

```rust
use async_trait::async_trait;
use serde_json::json;

use super::types::EmbeddingError;
use super::EmbeddingProvider;

const OUTPUT_DIMENSION: u32 = 1536;

pub struct CohereEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl CohereEmbeddingProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.cohere.com".to_string()
    }

    async fn embed_with_input_type(&self, text: &str, input_type: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = json!({
            "model": self.model,
            "texts": [text],
            "input_type": input_type,
            "output_dimension": OUTPUT_DIMENSION,
            "embedding_types": ["float"],
        });

        let response = self
            .client
            .post(format!("{}/v2/embed", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ProviderError(format!("cohere returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body
            .get("embeddings")
            .and_then(|e| e.get("float"))
            .and_then(|f| f.as_array())
            .and_then(|f| f.first())
            .and_then(|e| e.as_array())
            .ok_or_else(|| EmbeddingError::ParseError("missing embeddings.float[0]".to_string()))?;

        embedding
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| EmbeddingError::ParseError("embedding element is not a number".to_string()))
            })
            .collect()
    }
}

#[async_trait]
impl EmbeddingProvider for CohereEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_with_input_type(text, "search_document").await
    }

    async fn embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_with_input_type(text, "search_query").await
    }

    fn provider_name(&self) -> &'static str {
        "cohere"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}
```

- [ ] **Step 3: Add the `Cohere` variant to `config.rs`**

Modify `backend/crates/nomi-embedding/src/config.rs` (as left by Task 2) — add one import line, one enum variant, one match arm:

```rust
use super::cohere::CohereEmbeddingProvider;
```
(alongside the existing `use super::fake::...`/`use super::gemini::...`/`use super::openai::...` lines, alphabetically between `super::cohere` and `super::fake` — i.e. first)

```rust
pub enum EmbeddingProviderKind {
    OpenAi,
    Gemini,
    Cohere,
    Fake,
}
```

```rust
        EmbeddingProviderKind::Cohere => {
            let base_url = config.base_url.unwrap_or_else(CohereEmbeddingProvider::default_base_url);
            Box::new(CohereEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
```
(inserted into the `match config.provider { ... }` block in `build_embedding_provider`, after the `Gemini` arm and before the `Fake` arm)

- [ ] **Step 4: Write the provider's tests**

Create `backend/crates/nomi-embedding/tests/embedding_cohere.rs`:

```rust
use nomi_embedding::cohere::CohereEmbeddingProvider;
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn embed_sends_search_document_input_type_and_returns_the_vector() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .and(body_partial_json(json!({
            "model": "embed-v4.0",
            "texts": ["hello world"],
            "input_type": "search_document",
            "output_dimension": 1536
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test",
            "embeddings": { "float": [[0.1, 0.2, 0.3]] }
        })))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let embedding = provider.embed("hello world").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn embed_for_query_sends_search_query_input_type() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .and(body_partial_json(json!({ "input_type": "search_query" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test",
            "embeddings": { "float": [[0.4, 0.5]] }
        })))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let embedding = provider.embed_for_query("what did I eat").await.unwrap();
    assert_eq!(embedding, vec![0.4f32, 0.5]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ProviderError(_))));
}

#[tokio::test]
async fn malformed_response_becomes_a_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"embeddings": {}})))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ParseError(_))));
}

#[tokio::test]
async fn provider_name_and_model_id_are_exposed() {
    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        "http://example.invalid".to_string(),
    );
    assert_eq!(provider.provider_name(), "cohere");
    assert_eq!(provider.model_id(), "embed-v4.0");
}
```

- [ ] **Step 5: Verify**

```bash
cd backend && cargo test -p nomi-embedding 2>&1 | tail -60
```

Expected: the whole `nomi-embedding` crate compiles and all tests pass, including every provider added across Tasks 1–3.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-embedding/src/lib.rs backend/crates/nomi-embedding/src/cohere.rs \
        backend/crates/nomi-embedding/src/config.rs backend/crates/nomi-embedding/tests/embedding_cohere.rs
git commit -m "feat: add Cohere embedding provider"
```

---

### Task 4: Remove panics, expand the routes allowlist

**Files:**
- Modify: `backend/crates/nomi-server/src/bootstrap/providers.rs`
- Modify: `backend/crates/nomi-server/src/routes/settings.rs`
- Modify: `backend/crates/nomi-server/tests/settings_routes.rs`

**Interfaces:**
- Consumes: `EmbeddingProviderKind::{Gemini, Cohere}` (Tasks 2–3).

- [ ] **Step 1: Make `embedding_provider_kind_from_str` fallible instead of panicking**

In `backend/crates/nomi-server/src/bootstrap/providers.rs`, replace the existing function (lines 20-26):

```rust
fn embedding_provider_kind_from_str(s: &str) -> EmbeddingProviderKind {
    match s {
        "openai" => EmbeddingProviderKind::OpenAi,
        "fake" => EmbeddingProviderKind::Fake,
        other => panic!("unknown embedding provider: {other} (expected openai or fake)"),
    }
}
```

with:

```rust
fn embedding_provider_kind_from_str(s: &str) -> Result<EmbeddingProviderKind, String> {
    match s {
        "openai" => Ok(EmbeddingProviderKind::OpenAi),
        "gemini" => Ok(EmbeddingProviderKind::Gemini),
        "cohere" => Ok(EmbeddingProviderKind::Cohere),
        "fake" => Ok(EmbeddingProviderKind::Fake),
        other => Err(format!("unknown embedding provider: {other} (expected openai, gemini, cohere, or fake)")),
    }
}
```

- [ ] **Step 2: Make `build_embedding_provider_from_settings_or_env` degrade gracefully on a bad DB row**

Replace the existing function (lines 127-161) in full:

```rust
pub async fn build_embedding_provider_from_settings_or_env(
    pool: &PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn EmbeddingProvider> {
    let row = settings::get_settings(pool, "embedding").await.expect("failed to query provider_settings");

    let from_row = row.and_then(|row| {
        let api_key = match settings::crypto::decrypt(settings_key, &row.api_key_encrypted) {
            Ok(key) => key,
            Err(e) => {
                tracing::error!(error = %e, "failed to decrypt stored embedding api key; falling back to env vars");
                return None;
            }
        };
        let provider = match embedding_provider_kind_from_str(&row.provider) {
            Ok(provider) => provider,
            Err(e) => {
                tracing::error!(error = %e, "stored embedding provider settings are invalid; falling back to env vars");
                return None;
            }
        };
        Some(EmbeddingConfig { provider, model_id: row.model_id, api_key, base_url: row.base_url })
    });

    let embedding_config = match from_row {
        Some(config) => config,
        None => {
            let provider = embedding_provider_kind_from_str(
                &std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            )
            .expect("EMBEDDING_PROVIDER env var must be a known provider");
            let (model_id, api_key) = match provider {
                EmbeddingProviderKind::Fake => (String::new(), String::new()),
                _ => (
                    std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
                    std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
                ),
            };
            EmbeddingConfig { provider, model_id, api_key, base_url: std::env::var("EMBEDDING_BASE_URL").ok() }
        }
    };

    Arc::from(build_embedding_provider(embedding_config, http_client))
}
```

Note the `.expect()` on the env-var fallback path is intentionally kept — this matches the existing precedent in `resolve_llm_model_config`'s own env-var fallback (`providers.rs:116`), which still `.expect()`s on `LLM_PROVIDER`/`LLM_MODEL_ID`/`LLM_API_KEY`. Boot-time env misconfiguration is a different failure class from a corrupted/stale DB row and is allowed to fail loudly; a DB row that fails to decrypt or parse at request time is not.

- [ ] **Step 3: Add unit tests for the now-fallible parser**

Append to the end of `backend/crates/nomi-server/src/bootstrap/providers.rs` (this file has no existing `#[cfg(test)]` block):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_provider_kind_from_str_accepts_all_known_providers() {
        assert_eq!(embedding_provider_kind_from_str("openai"), Ok(EmbeddingProviderKind::OpenAi));
        assert_eq!(embedding_provider_kind_from_str("gemini"), Ok(EmbeddingProviderKind::Gemini));
        assert_eq!(embedding_provider_kind_from_str("cohere"), Ok(EmbeddingProviderKind::Cohere));
        assert_eq!(embedding_provider_kind_from_str("fake"), Ok(EmbeddingProviderKind::Fake));
    }

    #[test]
    fn embedding_provider_kind_from_str_rejects_unknown_providers() {
        assert!(embedding_provider_kind_from_str("not-a-real-provider").is_err());
    }
}
```

- [ ] **Step 4: Expand the routes allowlist**

In `backend/crates/nomi-server/src/routes/settings.rs`, replace (around line 104-106):

```rust
    if !["openai", "fake"].contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected openai or fake)"));
    }
```

with:

```rust
    if !["openai", "gemini", "cohere", "fake"].contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected openai, gemini, cohere, or fake)"));
    }
```

- [ ] **Step 5: Add route tests for the expanded allowlist**

Append to `backend/crates/nomi-server/tests/settings_routes.rs` (the file already has `test_state`, `json_request`, `register_admin_and_login` helpers — no new imports needed):

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn admin_can_configure_a_non_openai_embedding_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-embed-gemini@example.com").await;

    let (status, put_body) = json_request(
        router,
        "PUT",
        "/api/admin/settings/embedding",
        json!({ "provider": "gemini", "model_id": "gemini-embedding-001", "api_key": "test-gemini-key1", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["provider"], "gemini");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_provider_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-embed-bad@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/admin/settings/embedding",
        json!({ "provider": "not-a-real-provider", "model_id": "x", "api_key": "test-key12345678", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
```

- [ ] **Step 6: Verify**

```bash
cd backend && cargo test -p nomi-server settings_routes 2>&1 | tail -60
cd backend && cargo test -p nomi-server --lib bootstrap 2>&1 | tail -30
```

Expected: all pass, including the two new route tests and the two new unit tests.

- [ ] **Step 7: Commit**

```bash
git add backend/crates/nomi-server/src/bootstrap/providers.rs backend/crates/nomi-server/src/routes/settings.rs \
        backend/crates/nomi-server/tests/settings_routes.rs
git commit -m "fix: stop panicking on embedding provider misconfiguration; allow gemini and cohere"
```

---

### Task 5: Tag memories with embedding provenance and filter retrieval by it

**Files:**
- Create: `backend/migrations/0015_memory_embedding_provenance.sql`
- Modify: `backend/crates/nomi-agent-core/src/memory.rs`
- Modify: `backend/crates/nomi-agent-core/tests/turn_memory_retrieval.rs`
- Modify: `backend/crates/nomi-agent-core/tests/turn_memory_writing.rs`

**Interfaces:**
- Consumes: `EmbeddingProvider::provider_name()`/`model_id()`/`embed_for_query()` (Task 1), the test-double `FakeEmbeddingProvider`'s identity `("fake", "fake-model")` (Task 1, Step 4).
- Produces: `retrieve_relevant_memories`'s new signature (2 extra required params before `limit`) — no other crate calls this function directly outside this crate's own tests, so no other call site needs updating.

- [ ] **Step 1: Add the migration**

Create `backend/migrations/0015_memory_embedding_provenance.sql`:

```sql
ALTER TABLE memory_items
    ADD COLUMN embedding_provider TEXT NOT NULL DEFAULT 'openai',
    ADD COLUMN embedding_model TEXT NOT NULL DEFAULT '';
```

- [ ] **Step 2: Thread provenance through `memory.rs`**

In `backend/crates/nomi-agent-core/src/memory.rs`, replace the three functions `retrieve_relevant_memories`, `try_retrieve_memories`, and the body of `extract_and_store_memory` (lines 19-106 in the current file — everything from `retrieve_relevant_memories`'s signature through the end of `extract_and_store_memory`, leaving the `to_vector_literal` helper above and the `ReinforcementSignal`/`reinforce` code below untouched):

```rust
pub async fn retrieve_relevant_memories(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    query_embedding: &[f32],
    current_provider: &str,
    current_model: &str,
    limit: i64,
) -> Result<Vec<RetrievedMemory>, TurnError> {
    let literal = to_vector_literal(query_embedding);
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, content FROM memory_items \
         WHERE user_id = $1 AND embedding_provider = $2 AND embedding_model = $3 \
         ORDER BY weight * (1 - (embedding <=> $4::vector)) DESC \
         LIMIT $5",
    )
    .bind(user_id)
    .bind(current_provider)
    .bind(current_model)
    .bind(&literal)
    .bind(limit)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows.into_iter().map(|(id, content)| RetrievedMemory { id, content }).collect())
}

pub async fn try_retrieve_memories(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    embedding_provider: &dyn nomi_embedding::EmbeddingProvider,
    user_id: uuid::Uuid,
    text: &str,
) -> Vec<RetrievedMemory> {
    const MEMORY_RETRIEVAL_LIMIT: i64 = 5;
    let embedding = match embedding_provider.embed_for_query(text).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    retrieve_relevant_memories(
        conn,
        user_id,
        &embedding,
        embedding_provider.provider_name(),
        embedding_provider.model_id(),
        MEMORY_RETRIEVAL_LIMIT,
    )
    .await
    .unwrap_or_default()
}

const EXTRACTION_SYSTEM_PROMPT: &str =
    "Extract at most one durable fact worth remembering long-term from this exchange, or say NONE if nothing is worth storing.";
const EXTRACTION_MAX_TOKENS: u32 = 128;

pub async fn extract_and_store_memory(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    user_text: &str,
    assistant_text: &str,
) {
    let request = LlmRequest {
        system: Some(EXTRACTION_SYSTEM_PROMPT.to_string()),
        messages: vec![
            LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: user_text.to_string() }] },
            LlmMessage { role: LlmRole::Assistant, content: vec![ContentBlock::Text { text: assistant_text.to_string() }] },
        ],
        tools: vec![],
        max_tokens: EXTRACTION_MAX_TOKENS,
    };

    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(_) => return,
    };

    let extracted = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });

    let fact = match extracted {
        Some(f) if f.trim() != "NONE" && !f.trim().is_empty() => f.trim().to_string(),
        _ => return,
    };

    let embedding = match embedding_provider.embed(&fact).await {
        Ok(e) => e,
        Err(_) => return,
    };

    let literal = to_vector_literal(&embedding);
    let _ = sqlx::query(
        "INSERT INTO memory_items (user_id, content, embedding, embedding_provider, embedding_model) \
         VALUES ($1, $2, $3::vector, $4, $5)",
    )
    .bind(user_id)
    .bind(&fact)
    .bind(&literal)
    .bind(embedding_provider.provider_name())
    .bind(embedding_provider.model_id())
    .execute(&mut **conn)
    .await;
}
```

- [ ] **Step 3: Update `turn_memory_retrieval.rs` for the new signature and add provenance-filtering tests**

Replace `backend/crates/nomi-agent-core/tests/turn_memory_retrieval.rs` in full:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::memory::retrieve_relevant_memories;

const PROVIDER: &str = "openai";
const MODEL: &str = "text-embedding-3-small";

fn make_embedding(first: f32, second: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v[1] = second;
    v
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn seed_memory(pool: &PgPool, user_id: Uuid, content: &str, embedding: &[f32], weight: f64) {
    seed_memory_with_provider(pool, user_id, content, embedding, weight, PROVIDER, MODEL).await
}

async fn seed_memory_with_provider(
    pool: &PgPool,
    user_id: Uuid,
    content: &str,
    embedding: &[f32],
    weight: f64,
    provider: &str,
    model: &str,
) {
    sqlx::query(
        "INSERT INTO memory_items (user_id, content, embedding, weight, embedding_provider, embedding_model) \
         VALUES ($1, $2, $3::vector, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(content)
    .bind(to_vector_literal(embedding))
    .bind(weight)
    .bind(provider)
    .bind(model)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn retrieves_memories_ordered_by_weighted_similarity(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory(&pool, user_id, "close match", &make_embedding(1.0, 0.0), 1.0).await;
    seed_memory(&pool, user_id, "far match", &make_embedding(0.0, 1.0), 1.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(0.9, 0.1);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, PROVIDER, MODEL, 5).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].content, "close match");
    assert_eq!(results[1].content, "far match");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_high_weight_can_outrank_a_higher_raw_similarity(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory(&pool, user_id, "closer but low weight", &make_embedding(1.0, 0.0), 0.5).await;
    seed_memory(&pool, user_id, "farther but high weight", &make_embedding(0.7, 0.3), 5.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, PROVIDER, MODEL, 5).await.unwrap();

    assert_eq!(results[0].content, "farther but high weight");
}

#[sqlx::test(migrations = "../../migrations")]
async fn respects_the_limit_parameter(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    for i in 0..3 {
        seed_memory(&pool, user_id, &format!("memory-{i}"), &make_embedding(1.0 - (i as f32 * 0.1), 0.0), 1.0).await;
    }

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, PROVIDER, MODEL, 2).await.unwrap();

    assert_eq!(results.len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_returns_memories_for_the_queried_user(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    seed_memory(&pool, user_a, "user a's memory", &make_embedding(1.0, 0.0), 1.0).await;
    seed_memory(&pool, user_b, "user b's memory", &make_embedding(1.0, 0.0), 1.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_a, &query, PROVIDER, MODEL, 5).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "user a's memory");
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_empty_when_the_user_has_no_memories(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, PROVIDER, MODEL, 5).await.unwrap();

    assert!(results.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_memory_from_a_different_embedding_provider_is_not_retrieved(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory_with_provider(
        &pool,
        user_id,
        "embedded by a different provider",
        &make_embedding(1.0, 0.0),
        1.0,
        "gemini",
        "gemini-embedding-001",
    )
    .await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, PROVIDER, MODEL, 5).await.unwrap();

    assert!(results.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_memory_from_the_same_provider_but_a_different_model_is_not_retrieved(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory_with_provider(
        &pool,
        user_id,
        "embedded by an older model",
        &make_embedding(1.0, 0.0),
        1.0,
        PROVIDER,
        "text-embedding-ada-002",
    )
    .await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, PROVIDER, MODEL, 5).await.unwrap();

    assert!(results.is_empty());
}
```

- [ ] **Step 4: Update the one `turn_memory_writing.rs` test that checks stored content**

In `backend/crates/nomi-agent-core/tests/turn_memory_writing.rs`, replace the `a_real_extracted_fact_is_embedded_and_stored` test:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn a_real_extracted_fact_is_embedded_and_stored(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("User is vegetarian"));
    let embedder = FakeEmbeddingProvider::success(vec![0.2; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I don't eat meat", "Noted!").await;

    let (content, provider, model): (String, String, String) = sqlx::query_as(
        "SELECT content, embedding_provider, embedding_model FROM memory_items WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "User is vegetarian");
    assert_eq!(provider, "fake");
    assert_eq!(model, "fake-model");
}
```

The other three tests in this file (`none_extraction_stores_nothing`, `extraction_llm_failure_stores_nothing`, `embedding_failure_after_a_good_extraction_stores_nothing`) only assert `count == 0` and need no change.

- [ ] **Step 5: Verify**

```bash
cd backend && cargo test -p nomi-agent-core turn_memory 2>&1 | tail -80
```

Expected: all pass, including the two new provenance-filtering tests.

- [ ] **Step 6: Commit**

```bash
git add backend/migrations/0015_memory_embedding_provenance.sql backend/crates/nomi-agent-core/src/memory.rs \
        backend/crates/nomi-agent-core/tests/turn_memory_retrieval.rs backend/crates/nomi-agent-core/tests/turn_memory_writing.rs
git commit -m "feat: tag stored memories with embedding provenance and filter retrieval by it"
```

---

### Task 6: Admin frontend page for embedding settings

**Files:**
- Create: `frontend/src/routes/admin/(protected)/settings/embedding/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/settings/embedding/+page.svelte`
- Modify: `frontend/src/routes/admin/(protected)/+layout.svelte`

**Interfaces:**
- Consumes: `GET`/`PUT /api/admin/settings/embedding` (already exist, unchanged shape — Task 4 only widened the accepted `provider` values), `Select`/`TextField`/`Card`/`Button`/`Icon`/`IconButton` from `$lib/components/m3/` (all pre-existing).

- [ ] **Step 1: Create the server load/action**

Create `frontend/src/routes/admin/(protected)/settings/embedding/+page.server.ts`:

```typescript
import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export type EmbeddingSettings = {
	provider: string;
	model_id: string;
	base_url: string | null;
	api_key_masked: string;
};

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/settings/embedding');
	if (!response.ok) {
		return { settings: null as EmbeddingSettings | null };
	}
	const settings = (await response.json()) as EmbeddingSettings;
	return { settings };
};

export const actions: Actions = {
	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');
		if (typeof provider !== 'string' || !provider) {
			return fail(400, { error: 'Provider is required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/embedding', {
			method: 'PUT',
			body: JSON.stringify({
				provider,
				model_id: typeof modelId === 'string' ? modelId : '',
				api_key: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update embedding settings.' });
		}
		return { success: true };
	},
};
```

- [ ] **Step 2: Create the page**

Create `frontend/src/routes/admin/(protected)/settings/embedding/+page.svelte`:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const PROVIDER_OPTIONS = [
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'cohere', label: 'Cohere' },
		{ value: 'fake', label: 'Fake (testing)' },
	];
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Embedding provider</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Used by the personality memory feature's semantic search. Changing the provider stops older
	memories from being retrieved until new ones are stored under the new provider — they aren't
	deleted, just no longer comparable to new queries.
</p>

{#if form?.error}
	<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
{/if}

<div class="mt-6 max-w-lg">
	<Card variant="outlined" class="p-6">
		<form method="POST" action="?/update" class="space-y-4" use:enhance>
			<Select
				label="Provider"
				name="provider"
				options={PROVIDER_OPTIONS}
				value={data.settings?.provider ?? 'openai'}
			/>
			<TextField id="model_id" name="model_id" label="Model ID" value={data.settings?.model_id ?? ''} />
			<TextField
				id="api_key"
				name="api_key"
				type="password"
				label="API key"
				placeholder={data.settings ? `Leave blank to keep ${data.settings.api_key_masked}` : 'Required'}
			/>
			<TextField id="base_url" name="base_url" label="Base URL (optional)" value={data.settings?.base_url ?? ''} />
			<Button type="submit" variant="filled" class="w-full">Save</Button>
		</form>
	</Card>
</div>
```

- [ ] **Step 3: Add the sidebar nav link**

In `frontend/src/routes/admin/(protected)/+layout.svelte`, add an "Embedding Settings" link next to the existing "LLM Settings" one in both the collapsed (`IconButton`) and expanded (`<a>`) branches of the `<nav>` block:

Replace:

```svelte
			{#if collapsed}
				<IconButton href="/admin" aria-label="Dashboard">
					<Icon name="dashboard" />
				</IconButton>
				<IconButton href="/admin/settings/llm" aria-label="LLM Settings">
					<Icon name="settings" />
				</IconButton>
				<IconButton href="/admin/agents" aria-label="Agents">
					<Icon name="agents" />
				</IconButton>
			{:else}
				<a href="/admin" class="m3-nav-link">Dashboard</a>
				<a href="/admin/settings/llm" class="m3-nav-link">LLM Settings</a>
				<a href="/admin/agents" class="m3-nav-link">Agents</a>
			{/if}
```

with:

```svelte
			{#if collapsed}
				<IconButton href="/admin" aria-label="Dashboard">
					<Icon name="dashboard" />
				</IconButton>
				<IconButton href="/admin/settings/llm" aria-label="LLM Settings">
					<Icon name="settings" />
				</IconButton>
				<IconButton href="/admin/settings/embedding" aria-label="Embedding Settings">
					<Icon name="settings" />
				</IconButton>
				<IconButton href="/admin/agents" aria-label="Agents">
					<Icon name="agents" />
				</IconButton>
			{:else}
				<a href="/admin" class="m3-nav-link">Dashboard</a>
				<a href="/admin/settings/llm" class="m3-nav-link">LLM Settings</a>
				<a href="/admin/settings/embedding" class="m3-nav-link">Embedding Settings</a>
				<a href="/admin/agents" class="m3-nav-link">Agents</a>
			{/if}
```

- [ ] **Step 4: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 5: Self-review**

Confirm: submitting the form with a blank API key on an already-configured provider keeps the existing key (matches `resolve_api_key`'s existing "blank means keep" semantics in `settings.rs`); switching the `Select` to a different provider and saving actually updates `data.settings.provider` after the page reloads; the "not configured yet" state (no row exists — fresh install) renders the form with empty defaults and a "Required" placeholder on the API key field rather than crashing on a null `data.settings`.

- [ ] **Step 6: Commit**

```bash
git add "frontend/src/routes/admin/(protected)/settings/embedding" "frontend/src/routes/admin/(protected)/+layout.svelte"
git commit -m "feat: add the admin embedding settings page"
```

---

## Task Ordering Note

Tasks 1, 2, and 3 are strictly sequential (each ends with a green build and passing tests on its own): Task 1 extends the trait without referencing Gemini/Cohere at all; Task 2 adds its own `pub mod gemini;` line alongside creating `gemini.rs`; Task 3 adds its own `pub mod cohere;` line alongside creating `cohere.rs` on top of Task 2's version of `config.rs`. Tasks 4, 5, and 6 each depend on Tasks 1–3 being complete (they need `EmbeddingProviderKind::{Gemini, Cohere}` and the trait's new methods to exist) but are otherwise independent of each other — Task 4 touches `nomi-server`, Task 5 touches `nomi-agent-core` + a migration, Task 6 touches only the frontend, so they could be parallelized across different reviewers if desired.
